//! Provide capsule driver for controlling LEDs on a board.
//! This allows for much more cross platform controlling of LEDs
//! without having to know which of the GPIO pins exposed across
//! the syscall interface are LEDs.

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

/// Whether the LEDs are active high or active low on this platform.
#[derive(Clone,Copy)]
pub enum ActivationMode {
    ActiveHigh,
    ActiveLow,
}

pub struct LED<'a, G: hil::gpio::Pin + 'a> {
    pins_init: &'a [(&'a G, ActivationMode)],
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> LED<'a, G> {
    pub fn new(pins_init: &'a [(&'a G, ActivationMode)]) -> LED<'a, G> {
        // Make all pins output and off
        for &(pin, mode) in pins_init.as_ref().iter() {
            pin.make_output();
            match mode {
                ActivationMode::ActiveHigh => pin.clear(),
                ActivationMode::ActiveLow => pin.set(),
            }
        }

        LED { pins_init: pins_init }
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for LED<'a, G> {
    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        let pins_init = self.pins_init.as_ref();
        match command_num {
            // get number of LEDs
            0 => ReturnCode::SuccessWithValue { value: pins_init.len() as usize },

            // on
            1 => {
                if data >= pins_init.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let (pin, mode) = pins_init[data];
                    match mode {
                        ActivationMode::ActiveHigh => pin.set(),
                        ActivationMode::ActiveLow => pin.clear(),
                    }
                    ReturnCode::SUCCESS
                }
            }

            // off
            2 => {
                if data >= pins_init.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let (pin, mode) = pins_init[data];
                    match mode {
                        ActivationMode::ActiveHigh => pin.clear(),
                        ActivationMode::ActiveLow => pin.set(),
                    }
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
