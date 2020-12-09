//! Provides userspace access to LEDs on an LED matrix.
//!
//! This allows for much more cross platform controlling of LEDs without having
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let led_pins = static_init!(
//!     [(&'static sam4l::gpio::GPIOPin, kernel::hil::gpio::ActivationMode); 3],
//!     [(&sam4l::gpio::PA[13], kernel::hil::gpio::ActivationMode::ActiveLow),   // Red
//!      (&sam4l::gpio::PA[15], kernel::hil::gpio::ActivationMode::ActiveLow),   // Green
//!      (&sam4l::gpio::PA[14], kernel::hil::gpio::ActivationMode::ActiveLow)]); // Blue
//! let led = static_init!(
//!     capsules::gpio::Pin<'static, sam4l::gpio::GPIOPin>,
//!     capsules::gpio::Pin::new(led_pins));
//! ```
//!
//! Syscall Interface
//! -----------------
//!
//! - Stability: 2 - Stable
//!
//! ### Command
//!
//! All LED operations are synchronous, so this capsule only uses the `command`
//! syscall.
//!
//! #### `command_num`
//!
//! - `0`: Return the number of LEDs on this platform.
//!   - `data`: Unused.
//!   - Return: Number of LEDs.
//! - `1`: Turn the LED on.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `2`: Turn the LED off.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `3`: Toggle the on/off state of the LED.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.

use kernel::hil::gpio;
use kernel::{AppId, Driver, ReturnCode};

use core::cell::Cell;
use kernel::common::cells::TakeCell;

use kernel::hil::time::{Alarm, AlarmClient};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of LEDs and implements a `Driver` interface to
/// control them.
pub struct LedMatrixDriver<'a, L: gpio::Pin, A: Alarm<'a>> {
    cols: &'a [&'a L],
    rows: &'a [&'a L],
    buffer: TakeCell<'a, [u8]>,
    alarm: &'a A,
    current_row: Cell<usize>,
}

impl<'a, L: gpio::Pin, A: Alarm<'a>> LedMatrixDriver<'a, L, A> {
    pub fn new(cols: &'a [&'a L], rows: &'a [&'a L], buffer: &'a mut [u8], alarm: &'a A) -> Self {
        // Initialize all LEDs and turn them off
        for led in cols {
            led.make_output();
            led.set();
        }

        for led in rows {
            led.make_output();
            led.clear();
        }

        if (buffer.len() * 8) < cols.len() * rows.len() {
            panic!("Matrix LED Driver: provided buffer is too small");
        }

        Self {
            cols,
            rows,
            buffer: TakeCell::new(buffer),
            alarm,
            current_row: Cell::new(0),
        }
    }

    pub fn init(&self) {
        self.next_row();
    }

    fn next_row(&self) {
        self.rows[self.current_row.get()].clear();
        self.current_row
            .set((self.current_row.get() + 1) % self.rows.len());
        self.buffer.map(|bits| {
            for led in 0..self.cols.len() {
                let pos = self.current_row.get() * self.cols.len() + led;
                if (bits[pos / 8] >> (pos % 8)) & 0x1 == 1 {
                    self.cols[led].clear();
                } else {
                    self.cols[led].set();
                }
            }
        });
        self.rows[self.current_row.get()].set();
        let interval = A::ticks_from_ms(1);
        self.alarm.set_alarm(self.alarm.now(), interval);
    }
}

impl<'a, L: gpio::Pin, A: Alarm<'a>> AlarmClient for LedMatrixDriver<'a, L, A> {
    fn alarm(&self) {
        self.next_row();
    }
}

impl<'a, L: gpio::Pin, A: Alarm<'a>> Driver for LedMatrixDriver<'a, L, A> {
    /// Control the LEDs.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Returns the number of LEDs on the board. This will always be 0 or
    ///        greater, and therefore also allows for checking for this driver.
    /// - `1`: Turn the LED at index specified by `data` on. Returns `EINVAL` if
    ///        the LED index is not valid.
    /// - `2`: Turn the LED at index specified by `data` off. Returns `EINVAL`
    ///        if the LED index is not valid.
    /// - `3`: Toggle the LED at index specified by `data` on or off. Returns
    ///        `EINVAL` if the LED index is not valid.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // get number of LEDs
            0 => ReturnCode::SuccessWithValue {
                value: self.cols.len() as usize * self.rows.len() as usize,
            },

            // on
            1 => {
                if data >= self.cols.len() as usize * self.rows.len() as usize {
                    ReturnCode::EINVAL /* led out of range */
                } else {
                    self.buffer
                        .map(|bits| bits[data / 8] = bits[data / 8] | (1 << (data % 8)));
                    ReturnCode::SUCCESS
                }
            }

            // off
            2 => {
                if data >= self.cols.len() as usize * self.rows.len() as usize {
                    ReturnCode::EINVAL /* led out of range */
                } else {
                    self.buffer
                        .map(|bits| bits[data / 8] = bits[data / 8] & !(1 << data % 8));
                    ReturnCode::SUCCESS
                }
            }

            // toggle
            3 => {
                if data >= self.cols.len() as usize * self.rows.len() as usize {
                    ReturnCode::EINVAL /* led out of range */
                } else {
                    self.buffer
                        .map(|bits| bits[data / 8] = bits[data % 8] ^ (1 << (data % 8)));
                    ReturnCode::SUCCESS
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
