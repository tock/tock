//! Provides userspace access to LEDs on a board.
//!
//! This allows for much more cross platform controlling of LEDs without having
//! to know which of the GPIO pins exposed across the syscall interface are
//! LEDs.
//!
//! This capsule takes an array of pins and the polarity of the LED (active high
//! or active low). This allows the board to configure how the underlying GPIO
//! must be controlled to turn on and off LEDs, such that the syscall driver
//! interface can be agnostic to the LED polarity.
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
//!     capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
//!     capsules::led::LED::new(led_pins));
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
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.
//! - `2`: Turn the LED off.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.
//! - `3`: Toggle the on/off state of the LED.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `Ok(())` if the LED index was valid, `INVAL` otherwise.

use kernel::hil::led;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::TakeCell;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of LEDs and implements a `Driver` interface to
/// control them.
pub struct LedDriver<'a, L: led::Led> {
    leds: TakeCell<'a, [&'a L]>,
}

impl<'a, L: led::Led> LedDriver<'a, L> {
    pub fn new(leds: &'a mut [&'a L]) -> Self {
        // Initialize all LEDs and turn them off
        for led in leds.iter_mut() {
            led.init();
            led.off();
        }

        Self {
            leds: TakeCell::new(leds),
        }
    }
}

impl<L: led::Led> SyscallDriver for LedDriver<'_, L> {
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
        self.leds
            .map(|leds| {
                match command_num {
                    // get number of LEDs
                    0 => CommandReturn::success_u32(leds.len() as u32),
                    // on
                    1 => {
                        if data >= leds.len() {
                            CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                        } else {
                            leds[data].on();
                            CommandReturn::success()
                        }
                    }

                    // off
                    2 => {
                        if data >= leds.len() {
                            CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                        } else {
                            leds[data].off();
                            CommandReturn::success()
                        }
                    }

                    // toggle
                    3 => {
                        if data >= leds.len() {
                            CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                        } else {
                            leds[data].toggle();
                            CommandReturn::success()
                        }
                    }

                    // default
                    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                }
            })
            .expect("LEDs slice taken")
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
