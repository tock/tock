//! Provides userspace access to 7 segment digit displays.
//!
//! This capsule was developed using the following components:
//! - Microbit_v2
//! - Edge Connector Breakout Board for Microbit (PPMB00126)
//! - 7 segment display with 4 digits (3461BS-1)
//! - breadboard, 220 ohms resistances and jump wires
//!
//! Usage
//! -----
//!
//! Example of use for a display with 4 digits and the Microbit:
//! Microbit Pins: <https://tech.microbit.org/hardware/schematic/>
//! 4 digit 7 segment display pinout: <https://www.dotnetlovers.com/images/4digit7segmentdisplay85202024001AM.jpg>
//!
//! ```rust
//! const NUM_DIGITS: usize = 4;
//! const DIGITS: [Pin; 4] = [Pin::P1_02, Pin::P0_12, Pin::P0_30, Pin::P0_09]; // [D1, D2, D3, D4]
//! const SEGMENTS: [Pin; 7] = [
//!     Pin::P0_02, // A
//!     Pin::P0_03, // B
//!     Pin::P0_04, // C
//!     Pin::P0_31, // D
//!     Pin::P0_28, // E
//!     Pin::P0_10, // F
//!     Pin::P1_05, // G
//! ];
//! const DOT: Pin = Pin::P0_11;
//!
//! let segment_array = static_init!(
//!     [&'static nrf52::gpio::GPIOPin<'static>; 8],
//!     [
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[0]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[1]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[2]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[3]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[4]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[5]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[SEGMENTS[6]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[DOT]
//!         ),
//!     ]
//! );
//!
//! let digit_array = static_init!(
//!     [&'static nrf52::gpio::GPIOPin<'static>; 4],
//!     [
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[DIGITS[0]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[DIGITS[1]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[DIGITS[2]]
//!         ),
//!         static_init!(
//!             &'static nrf52::gpio::GPIOPin<'static>,
//!             &nrf52833_peripherals.gpio_port[DIGITS[3]]
//!         ),
//!     ]
//! );
//!
//! let buffer = static_init!([u8; 4], [0; 4]);
//!
//! let digit_display = static_init!(
//!     capsules::digits::DigitsDriver<
//!         'static,
//!         nrf52::gpio::GPIOPin<'static>,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!     >,
//!     capsules::digits::DigitsDriver::new(
//!         segment_array,
//!         digit_array,
//!         buffer,
//!         virtual_alarm_digit,
//!         kernel::hil::gpio::ActivationMode::ActiveLow,
//!         kernel::hil::gpio::ActivationMode::ActiveHigh,
//!         60
//!     ),
//! );
//! ```
//!
//! virtual_alarm_digit.set_alarm_client(digit_display);
//!
//! digit_display.init();
//!
//!
//! Syscall Interface
//! -----------------
//!
//! ### Command
//!
//! All operations are synchronous, so this capsule only uses the `command`
//! syscall.
//!
//! #### `command_num`
//!
//! - `0`: Return the number of digits on the display being used.
//!   - `data1`: Unused.
//!   - `data2`: Unused.
//!   - Return: Number of digits.
//! - `1`: Prints one digit at the requested position.
//!   - `data1`: The position of the digit. Starts at 1.
//!   - `data2`: The digit to be represented, from 0 to 9.
//!   - Return: `Ok(())` if the digit index was valid, `INVAL` otherwise.
//! - `2`: Clears all digits currently being displayed.
//!   - `data1`: Unused.
//!   - `data2`: Unused.
//! - `3`: Print a dot at the requested digit position.
//!   - `data1`: The position of the dot. Starts at 1.
//!   - Return: `Ok(())` if the index was valid, `INVAL` otherwise.
//! - `4`: Print a custom pattern for a digit on a certain position.
//!   - `data1`: The position of the digit. Starts at 1.
//!   - `data2`: The custom pattern to be represented.
//!   - Return: `Ok(())` if the index was valid, `INVAL` otherwise.

use core::cell::Cell;

use kernel::hil::gpio::{ActivationMode, Pin};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;
use kernel::ProcessId;

/// Syscall driver number.
use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::SevenSegment as usize;

/// Digit patterns
//
//      A
//      _
//   F |_| B       center = G
//   E |_| C . Dp
//      D
//
const DIGITS: [u8; 10] = [
    // pattern: 0bDpGFEDCBA
    0b00111111, // 0
    0b00000110, // 1
    0b01011011, // 2
    0b01001111, // 3
    0b01100110, // 4
    0b01101101, // 5
    0b01111101, // 6
    0b00100111, // 7
    0b01111111, // 8
    0b01101111, // 9
];

/// Holds an array of digits and an array of segments for each digit.

pub struct SevenSegmentDriver<'a, P: Pin, A: Alarm<'a>, const NUM_DIGITS: usize> {
    /// An array of 8 segment pins (7 for digit segments and one dot segment)
    segments: &'a [&'a P; 8],
    /// An array of `NUM_DIGITS` digit pins, each one corresponding to one digit on the display
    /// For each digit selected, a pattern of lit and unlit segments will be represented
    digits: &'a [&'a P; NUM_DIGITS],
    /// A buffer which contains the patterns displayed for each digit
    /// Each element of the buffer array represents the pattern for one digit, and
    /// is a sequence of bits that have the value 1 for a lit segment and the value 0 for
    /// an unlit segment.
    buffer: TakeCell<'a, [u8; NUM_DIGITS]>,
    alarm: &'a A,
    current_digit: Cell<usize>,
    /// How fast the driver should switch between digits (ms)
    timing: u8,
    segment_activation: ActivationMode,
    digit_activation: ActivationMode,
}

impl<'a, P: Pin, A: Alarm<'a>, const NUM_DIGITS: usize> SevenSegmentDriver<'a, P, A, NUM_DIGITS> {
    pub fn new(
        segments: &'a [&'a P; 8],
        digits: &'a [&'a P; NUM_DIGITS],
        buffer: &'a mut [u8; NUM_DIGITS],
        alarm: &'a A,
        segment_activation: ActivationMode,
        digit_activation: ActivationMode,
        refresh_rate: usize,
    ) -> Self {
        // Check if the buffer has enough space to hold patterns for all digits
        if (buffer.len() * 8) < segments.len() * digits.len() {
            panic!("Digits Driver: provided buffer is too small");
        }

        Self {
            segments,
            digits,
            buffer: TakeCell::new(buffer),
            alarm,
            segment_activation: segment_activation,
            digit_activation: digit_activation,
            current_digit: Cell::new(0),
            timing: (1000 / (refresh_rate * digits.len())) as u8,
        }
    }

    /// Initialize the digit and segment pins.
    /// Does not override pins if they have already been initialized for another driver.
    pub fn init(&self) {
        for segment in self.segments {
            segment.make_output();
            self.segment_clear(segment);
        }

        for digit in self.digits {
            digit.make_output();
            self.digit_clear(digit);
        }

        self.next_digit();
    }

    /// Returns the number of digits on the display.
    pub fn digits_len(&self) -> usize {
        self.digits.len()
    }

    /// Represents each digit with its corresponding pattern.
    fn next_digit(&self) {
        self.digit_clear(self.digits[self.current_digit.get()]);
        self.current_digit
            .set((self.current_digit.get() + 1) % self.digits.len());
        self.buffer.map(|bits| {
            for segment in 0..self.segments.len() {
                let location = self.current_digit.get() * self.segments.len() + segment;
                if (bits[location / 8] >> (location % 8)) & 0x1 == 1 {
                    self.segment_set(self.segments[segment]);
                } else {
                    self.segment_clear(self.segments[segment]);
                }
            }
        });
        self.digit_set(self.digits[self.current_digit.get()]);
        let interval = self.alarm.ticks_from_ms(self.timing as u32);
        self.alarm.set_alarm(self.alarm.now(), interval);
    }

    fn segment_set(&self, p: &P) {
        match self.segment_activation {
            ActivationMode::ActiveHigh => p.set(),
            ActivationMode::ActiveLow => p.clear(),
        }
    }

    fn segment_clear(&self, p: &P) {
        match self.segment_activation {
            ActivationMode::ActiveHigh => p.clear(),
            ActivationMode::ActiveLow => p.set(),
        }
    }

    fn digit_set(&self, p: &P) {
        match self.digit_activation {
            ActivationMode::ActiveHigh => p.set(),
            ActivationMode::ActiveLow => p.clear(),
        }
    }

    fn digit_clear(&self, p: &P) {
        match self.digit_activation {
            ActivationMode::ActiveHigh => p.clear(),
            ActivationMode::ActiveLow => p.set(),
        }
    }

    /// Sets the pattern for the digit on the requested position.
    fn print_digit(&self, position: usize, digit: usize) -> Result<(), ErrorCode> {
        if position <= self.digits.len() {
            self.buffer.map(|bits| bits[position - 1] = DIGITS[digit]);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    /// Clears all digits currently being displayed.
    fn clear_digits(&self) -> Result<(), ErrorCode> {
        self.buffer.map(|bits| {
            for index in 0..self.digits.len() {
                bits[index] = 0;
            }
        });
        Ok(())
    }

    /// Prints a dot at the requested digit position.
    fn print_dot(&self, position: usize) -> Result<(), ErrorCode> {
        if position <= self.digits.len() {
            self.buffer.map(|bits| {
                // set the first bit of the digit on this position
                bits[position - 1] |= 1 << (self.segments.len() - 1);
            });
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    /// Prints a custom pattern at a requested position.
    fn print(&self, position: usize, pattern: u8) -> Result<(), ErrorCode> {
        if position <= self.digits.len() {
            self.buffer.map(|bits| {
                bits[position - 1] = pattern;
            });
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

impl<'a, P: Pin, A: Alarm<'a>, const NUM_DIGITS: usize> AlarmClient
    for SevenSegmentDriver<'a, P, A, NUM_DIGITS>
{
    fn alarm(&self) {
        self.next_digit();
    }
}

impl<'a, P: Pin, A: Alarm<'a>, const NUM_DIGITS: usize> SyscallDriver
    for SevenSegmentDriver<'a, P, A, NUM_DIGITS>
{
    /// Control the digit display.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Returns the number of digits on the display. This will always be 0 or
    ///        greater, and therefore also allows for checking for this driver.
    /// - `1`: Prints one digit at the requested position. Returns `INVAL` if the
    ///        position is not valid.
    /// - `2`: Clears all digits currently being displayed.
    /// - `3`: Print a dot at the requested digit position. Returns
    ///        `INVAL` if the position is not valid.
    /// - `4`: Print a custom pattern for a certain digit. Returns
    ///        `INVAL` if the position is not valid.
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        _: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Return number of digits
            0 => CommandReturn::success_u32(self.digits.len() as u32),

            // Print one digit
            1 => CommandReturn::from(self.print_digit(data1, data2)),

            // Clear all digits
            2 => CommandReturn::from(self.clear_digits()),

            // Print dot
            3 => CommandReturn::from(self.print_dot(data1)),

            // Print a custom pattern
            4 => CommandReturn::from(self.print(data1, data2 as u8)),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
