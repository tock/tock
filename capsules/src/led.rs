//! Provide capsule driver for controlling LEDs on a board.
//! This allows for much more cross platform controlling of LEDs
//! without having to know which of the GPIO pins exposed across
//! the syscall interface are LEDs.

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

/// Whether the LEDs are active high or active low on this platform.
pub enum ActivationMode {
    ActiveHigh,
    ActiveLow,
}

pub struct LED<'a, G: hil::gpio::Pin + 'a> {
    pins: &'a [&'a G],
    mode: ActivationMode,
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> LED<'a, G> {
    pub fn new(pins: &'a [&'a G], mode: ActivationMode) -> LED<'a, G> {
        // Make all pins output and off
        for pin in pins.iter() {
            pin.make_output();
            match mode {
                ActivationMode::ActiveHigh => pin.clear(),
                ActivationMode::ActiveLow => pin.set(),
            }
        }

        LED {
            pins: pins,
            mode: mode,
        }
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for LED<'a, G> {
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        let pins = self.pins.as_ref();
        match command_num {
            // get number of LEDs
            0 => ReturnCode::SuccessWithValue { value: pins.len() as usize },

            // on
            1 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    match self.mode {
                        ActivationMode::ActiveHigh => pins[data].set(),
                        ActivationMode::ActiveLow => pins[data].clear(),
                    }
                    ReturnCode::SUCCESS
                }
            }

            // off
            2 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    match self.mode {
                        ActivationMode::ActiveHigh => pins[data].clear(),
                        ActivationMode::ActiveLow => pins[data].set(),
                    }
                    ReturnCode::SUCCESS
                }
            }

            // toggle
            3 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].toggle();
                    ReturnCode::SUCCESS
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
