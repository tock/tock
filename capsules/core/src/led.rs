// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let led_pins = static_init!(
//!     [(&'static sam4l::gpio::GPIOPin, kernel::hil::gpio::ActivationMode); 3],
//!     [(&sam4l::gpio::PA[13], kernel::hil::gpio::ActivationMode::ActiveLow),   // Red
//!      (&sam4l::gpio::PA[15], kernel::hil::gpio::ActivationMode::ActiveLow),   // Green
//!      (&sam4l::gpio::PA[14], kernel::hil::gpio::ActivationMode::ActiveLow)]); // Blue
//! let led = static_init!(
//!     capsules_core::led::LED<'static, sam4l::gpio::GPIOPin>,
//!     capsules_core::led::LED::new(led_pins));
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
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Led as usize;

/// Holds the array of LEDs and implements a `Driver` interface to
/// control them.
pub struct LedDriver<'a, L: led::Led, const NUM_LEDS: usize> {
    leds: &'a [&'a L; NUM_LEDS],
}

impl<'a, L: led::Led, const NUM_LEDS: usize> LedDriver<'a, L, NUM_LEDS> {
    pub fn new(leds: &'a [&'a L; NUM_LEDS]) -> Self {
        // Initialize all LEDs and turn them off
        for led in leds.iter() {
            led.init();
            led.off();
        }

        Self { leds }
    }
}

impl<L: led::Led, const NUM_LEDS: usize> SyscallDriver for LedDriver<'_, L, NUM_LEDS> {
    /// Control the LEDs.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Returns the number of LEDs on the board. This will always be 0 or
    ///   greater, and therefore also allows for checking for this driver.
    /// - `1`: Turn the LED at index specified by `data` on. Returns `INVAL` if
    ///   the LED index is not valid.
    /// - `2`: Turn the LED at index specified by `data` off. Returns `INVAL` if
    ///   the LED index is not valid.
    /// - `3`: Toggle the LED at index specified by `data` on or off. Returns
    ///   `INVAL` if the LED index is not valid.
    fn command(&self, command_num: usize, data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            // get number of LEDs
            // TODO(Tock 3.0): TRD104 specifies that Command 0 should return Success, not SuccessU32,
            // but this driver is unchanged since it has been stabilized. It will be brought into
            // compliance as part of the next major release of Tock. See #3375.
            0 => CommandReturn::success_u32(NUM_LEDS as u32),

            // on
            1 => {
                if data >= NUM_LEDS {
                    CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                } else {
                    self.leds[data].on();
                    CommandReturn::success()
                }
            }

            // off
            2 => {
                if data >= NUM_LEDS {
                    CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                } else {
                    self.leds[data].off();
                    CommandReturn::success()
                }
            }

            // toggle
            3 => {
                if data >= NUM_LEDS {
                    CommandReturn::failure(ErrorCode::INVAL) /* led out of range */
                } else {
                    self.leds[data].toggle();
                    CommandReturn::success()
                }
            }

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
