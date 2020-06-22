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

use kernel::hil::gpio;
use kernel::{AppId, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of GPIO pins attached to the LEDs and implements a `Driver`
/// interface to control them.
pub struct LED<'a, P: gpio::Pin> {
    pins_init: &'a [(&'a P, gpio::ActivationMode)],
}

impl<'a, P: gpio::Pin> LED<'a, P> {
    pub fn new(pins_init: &'a [(&'a P, gpio::ActivationMode)]) -> Self {
        // Make all pins output and off
        for &(pin, mode) in pins_init.as_ref().iter() {
            pin.make_output();
            pin.write_activation(gpio::ActivationState::Inactive, mode);
        }

        Self {
            pins_init: pins_init,
        }
    }
}

impl<P: gpio::Pin> Driver for LED<'_, P> {
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
        let pins_init = self.pins_init.as_ref();
        match command_num {
            // get number of LEDs
            0 => ReturnCode::SuccessWithValue {
                value: pins_init.len() as usize,
            },

            // on
            1 => {
                if data >= pins_init.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let (pin, mode) = pins_init[data];
                    pin.write_activation(gpio::ActivationState::Active, mode);
                    ReturnCode::SUCCESS
                }
            }

            // off
            2 => {
                if data >= pins_init.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let (pin, mode) = pins_init[data];
                    pin.write_activation(gpio::ActivationState::Inactive, mode);
                    ReturnCode::SUCCESS
                }
            }

            // toggle
            3 => {
                if data >= pins_init.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let (pin, _) = pins_init[data];
                    pin.toggle();
                    ReturnCode::SUCCESS
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
