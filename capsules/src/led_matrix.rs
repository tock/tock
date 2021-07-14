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
//! let single_led = static_init!(
//!     capsules::led_matrix::LedMatrixLed<
//!         'static,
//!         nrf52::gpio::GPIOPin<'static>,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!     >,
//!     led,
//!     1,
//!     2
//! );
//!
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
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.
//! - `2`: Turn the LED off.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.
//! - `3`: Toggle the on/off state of the LED.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.

use core::cell::Cell;

use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::TakeCell;
use kernel::{ErrorCode, ProcessId};

use kernel::hil::gpio::{ActivationMode, Pin};
use kernel::hil::led::Led;
use kernel::hil::time::{Alarm, AlarmClient};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of LEDs and implements a `Driver` interface to
/// control them.
pub struct LedMatrixDriver<'a, L: Pin, A: Alarm<'a>> {
    cols: &'a [&'a L],
    rows: &'a [&'a L],
    buffer: TakeCell<'a, [u8]>,
    alarm: &'a A,
    current_row: Cell<usize>,
    timing: u8,
    row_activation: ActivationMode,
    col_activation: ActivationMode,
}

impl<'a, L: Pin, A: Alarm<'a>> LedMatrixDriver<'a, L, A> {
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

    pub fn cols_len(&self) -> usize {
        self.cols.len()
    }

    pub fn rows_len(&self) -> usize {
        self.rows.len()
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

    pub fn on(&self, col: usize, row: usize) -> Result<(), ErrorCode> {
        self.on_index(row * self.rows.len() + col)
    }

    fn on_index(&self, led_index: usize) -> Result<(), ErrorCode> {
        if led_index < self.rows.len() * self.cols.len() {
            self.buffer
                .map(|bits| bits[led_index / 8] = bits[led_index / 8] | (1 << (led_index % 8)));
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    pub fn off(&self, col: usize, row: usize) -> Result<(), ErrorCode> {
        self.off_index(row * self.rows.len() + col)
    }

    fn off_index(&self, led_index: usize) -> Result<(), ErrorCode> {
        if led_index < self.rows.len() * self.cols.len() {
            self.buffer
                .map(|bits| bits[led_index / 8] = bits[led_index / 8] & !(1 << led_index % 8));
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    pub fn toggle(&self, col: usize, row: usize) -> Result<(), ErrorCode> {
        self.toggle_index(row * self.rows.len() + col)
    }

    fn toggle_index(&self, led_index: usize) -> Result<(), ErrorCode> {
        if led_index < self.rows.len() * self.cols.len() {
            self.buffer
                .map(|bits| bits[led_index / 8] = bits[led_index % 8] ^ (1 << (led_index % 8)));
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    fn read(&self, col: usize, row: usize) -> Result<bool, ErrorCode> {
        if row < self.rows.len() && col < self.cols.len() {
            let pos = row * self.rows.len() + col;
            self.buffer.map_or(Err(ErrorCode::FAIL), |bits| {
                match bits[pos / 8] & (1 << (pos % 8)) {
                    0 => Ok(false),
                    _ => Ok(true),
                }
            })
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

impl<'a, L: Pin, A: Alarm<'a>> AlarmClient for LedMatrixDriver<'a, L, A> {
    fn alarm(&self) {
        self.next_row();
    }
}

impl<'a, L: Pin, A: Alarm<'a>> SyscallDriver for LedMatrixDriver<'a, L, A> {
    /// Control the LEDs.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Returns the number of LEDs on the board. This will always be 0 or
    ///        greater, and therefore also allows for checking for this driver.
    /// - `1`: Turn the LED at index specified by `data` on. Returns `INVAL` if
    ///        the LED index is not valid.
    /// - `2`: Turn the LED at index specified by `data` off. Returns `INVAL`
    ///        if the LED index is not valid.
    /// - `3`: Toggle the LED at index specified by `data` on or off. Returns
    ///        `INVAL` if the LED index is not valid.
    fn command(&self, command_num: usize, data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            // get number of LEDs
            0 => CommandReturn::success_u32((self.cols.len() * self.rows.len()) as u32),

            // on
            1 => CommandReturn::from(self.on_index(data)),

            // off
            2 => CommandReturn::from(self.off_index(data)),

            // toggle
            3 => CommandReturn::from(self.toggle_index(data)),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
// one Led from the matrix
pub struct LedMatrixLed<'a, L: Pin, A: Alarm<'a>> {
    matrix: &'a LedMatrixDriver<'a, L, A>,
    row: usize,
    col: usize,
}

impl<'a, L: Pin, A: Alarm<'a>> LedMatrixLed<'a, L, A> {
    pub fn new(matrix: &'a LedMatrixDriver<'a, L, A>, col: usize, row: usize) -> Self {
        if col >= matrix.cols_len() || row >= matrix.rows_len() {
            panic!("LET at position ({}, {}) does not exist", col, row);
        }
        LedMatrixLed { matrix, col, row }
    }
}

impl<'a, L: Pin, A: Alarm<'a>> Led for LedMatrixLed<'a, L, A> {
    fn init(&self) {}

    fn on(&self) {
        let _ = self.matrix.on(self.col, self.row);
    }

    fn off(&self) {
        let _ = self.matrix.off(self.col, self.row);
    }

    fn toggle(&self) {
        let _ = self.matrix.toggle(self.col, self.row);
    }

    fn read(&self) -> bool {
        match self.matrix.read(self.col, self.row) {
            Ok(v) => v,
            Err(_) => false,
        }
    }
}
