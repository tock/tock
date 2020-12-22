//! Provides userspace access to LEDs on an LED matrix.
//!
//! Usage
//! -----
//!
//! ```rust
//! let buffer = static_init!([u8; 5], [0; 5]);
//!
//! let cols = static_init!(
//!     [&nrf52833::gpio::GPIOPin; 5],
//!     [
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[4]]
//!     ]
//! );
//!
//! let rows = static_init!(
//!     [&nrf52833::gpio::GPIOPin; 5],
//!     [
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[4]]
//!     ]
//! );
//!
//! let led = static_init!(
//!     capsules::led_matrix::LedMatrixDriver<
//!         'static,
//!         nrf52::gpio::GPIOPin<'static>,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!     >,
//!     capsules::led_matrix::LedMatrixDriver::new(cols, rows, buffer, led_alarm, kernel::hil::gpio::ActivationMode::ActiveLow, kernel::hil::gpio::ActivationMode::ActiveHigh, 60)
//! );
//!
//! led_alarm.set_alarm_client(led);
//!
//! led.init();
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

use kernel::hil::gpio::ActivationMode;
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
    timing: u8,
    row_activation: ActivationMode,
    col_activation: ActivationMode,
}

impl<'a, L: gpio::Pin, A: Alarm<'a>> LedMatrixDriver<'a, L, A> {
    pub fn new(
        cols: &'a [&'a L],
        rows: &'a [&'a L],
        buffer: &'a mut [u8],
        alarm: &'a A,
        col_activation: ActivationMode,
        row_activation: ActivationMode,
        refresh_rate: usize,
    ) -> Self {
        // Initialize all LEDs and turn them off
        if (buffer.len() * 8) < cols.len() * rows.len() {
            panic!("Matrix LED Driver: provided buffer is too small");
        }

        Self {
            cols,
            rows,
            buffer: TakeCell::new(buffer),
            alarm,
            col_activation: col_activation,
            row_activation: row_activation,
            current_row: Cell::new(0),
            timing: (1000 / (refresh_rate * rows.len())) as u8,
        }
    }

    pub fn init(&self) {
        for led in self.cols {
            led.make_output();
            self.col_clear(led);
        }

        for led in self.rows {
            led.make_output();
            self.row_clear(led);
        }
        self.next_row();
    }

    fn next_row(&self) {
        self.row_clear(self.rows[self.current_row.get()]);
        self.current_row
            .set((self.current_row.get() + 1) % self.rows.len());
        self.buffer.map(|bits| {
            for led in 0..self.cols.len() {
                let pos = self.current_row.get() * self.cols.len() + led;
                if (bits[pos / 8] >> (pos % 8)) & 0x1 == 1 {
                    self.col_set(self.cols[led]);
                } else {
                    self.col_clear(self.cols[led]);
                }
            }
        });
        self.row_set(self.rows[self.current_row.get()]);
        let interval = A::ticks_from_ms(self.timing as u32);
        self.alarm.set_alarm(self.alarm.now(), interval);
    }

    fn col_set(&self, l: &L) {
        match self.col_activation {
            ActivationMode::ActiveHigh => l.set(),
            ActivationMode::ActiveLow => l.clear(),
        }
    }

    fn col_clear(&self, l: &L) {
        match self.col_activation {
            ActivationMode::ActiveHigh => l.clear(),
            ActivationMode::ActiveLow => l.set(),
        }
    }

    fn row_set(&self, l: &L) {
        match self.row_activation {
            ActivationMode::ActiveHigh => l.set(),
            ActivationMode::ActiveLow => l.clear(),
        }
    }

    fn row_clear(&self, l: &L) {
        match self.row_activation {
            ActivationMode::ActiveHigh => l.clear(),
            ActivationMode::ActiveLow => l.set(),
        }
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
