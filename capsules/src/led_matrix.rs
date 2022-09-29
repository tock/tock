//! Service capsule for access to LEDs on a LED matrix.
//!
//! Usage
//! -----
//!
//! ```rust
//! let led_matrix = components::led_matrix_component_helper!(
//!     nrf52833::gpio::GPIOPin,
//!     nrf52::rtc::Rtc<'static>,
//!     mux_alarm,
//!     @fps => 60,
//!     @cols => kernel::hil::gpio::ActivationMode::ActiveLow,
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[0]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[1]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[2]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[3]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[4]],
//!     @rows => kernel::hil::gpio::ActivationMode::ActiveHigh,
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[4]]
//!
//! )
//! .finalize(components::led_matrix_component_buf!(
//!     nrf52833::gpio::GPIOPin,
//!     nrf52::rtc::Rtc<'static>
//! ));
//!
//! let led = static_init!(
//!     capsules::led::LedDriver<
//!         'static,
//!         capsules::led_matrix::LedMatrixLed<
//!             'static,
//!             nrf52::gpio::GPIOPin<'static>,
//!             capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!         >,
//!         25,
//!     >,
//!     capsules::led::LedDriver::new(components::led_matrix_leds!(
//!         nrf52::gpio::GPIOPin<'static>,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!         led_matrix,
//!         (0, 0),
//!         (1, 0),
//!         (2, 0),
//!         (3, 0),
//!         (4, 0),
//!         (0, 1),
//!         (1, 1),
//!         (2, 1),
//!         (3, 1),
//!         (4, 1),
//!         (0, 2),
//!         (1, 2),
//!         (2, 2),
//!         (3, 2),
//!         (4, 2),
//!         (0, 3),
//!         (1, 3),
//!         (2, 3),
//!         (3, 3),
//!         (4, 3),
//!         (0, 4),
//!         (1, 4),
//!         (2, 4),
//!         (3, 4),
//!         (4, 4)
//!     )),
//! );

use core::cell::Cell;

use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

use kernel::hil::gpio::{ActivationMode, Pin};
use kernel::hil::led::Led;
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};

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
        let interval = self.alarm.ticks_from_ms(self.timing as u32);
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
                .map(|bits| bits[led_index / 8] = bits[led_index / 8] ^ (1 << (led_index % 8)));
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

// one Led from the matrix
pub struct LedMatrixLed<'a, L: Pin, A: Alarm<'a>> {
    matrix: &'a LedMatrixDriver<'a, L, A>,
    row: usize,
    col: usize,
}

impl<'a, L: Pin, A: Alarm<'a>> LedMatrixLed<'a, L, A> {
    pub fn new(matrix: &'a LedMatrixDriver<'a, L, A>, col: usize, row: usize) -> Self {
        if col >= matrix.cols_len() || row >= matrix.rows_len() {
            panic!("LED at position ({}, {}) does not exist", col, row);
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
