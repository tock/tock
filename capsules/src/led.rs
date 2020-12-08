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
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `2`: Turn the LED off.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `3`: Toggle the on/off state of the LED.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.

use kernel::common::cells::TakeCell;
use kernel::hil::led;
use kernel::{AppId, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of LEDs and implements a `Driver` interface to
/// control them.
pub struct LedDriver<'a, L: led::Led> {
    leds: TakeCell<'a, [&'a mut L]>,
}

impl<'a, L: led::Led> LedDriver<'a, L> {
    pub fn new(leds: &'a mut [&'a mut L]) -> Self {
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

impl<L: led::Led> Driver for LedDriver<'_, L> {
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
        self.leds
            .map(|leds| {
                match command_num {
                    // get number of LEDs
                    0 => ReturnCode::SuccessWithValue {
                        value: leds.len() as usize,
                    },

                    // on
                    1 => {
                        if data >= leds.len() {
                            ReturnCode::EINVAL /* led out of range */
                        } else {
                            leds[data].on();
                            ReturnCode::SUCCESS
                        }
                    }

                    // off
                    2 => {
                        if data >= leds.len() {
                            ReturnCode::EINVAL /* led out of range */
                        } else {
                            leds[data].off();
                            ReturnCode::SUCCESS
                        }
                    }

                    // toggle
                    3 => {
                        if data >= leds.len() {
                            ReturnCode::EINVAL /* led out of range */
                        } else {
                            leds[data].toggle();
                            ReturnCode::SUCCESS
                        }
                    }

                    // default
                    _ => ReturnCode::ENOSUPPORT,
                }
            })
            .expect("LEDs slice taken")
    }
}
