//! Provide capsule driver for controlling buttons on a board.  This allows for much more cross
//! platform controlling of buttons without having to know which of the GPIO pins exposed across
//! the syscall interface are buttons.

use kernel::{AppId, Container, Callback, Driver, ReturnCode};
use kernel::hil;
use kernel::hil::gpio::{Client, InterruptMode};
use kernel::process::Error;

pub type SubscribeMap = u32;

pub struct Button<'a, G: hil::gpio::Pin + 'a> {
    pins: &'a [&'a G],
    callback: Container<(Option<Callback>, SubscribeMap)>,
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Button<'a, G> {
    pub fn new(pins: &'a [&'a G],
               container: Container<(Option<Callback>, SubscribeMap)>)
               -> Button<'a, G> {
        // Make all pins output and off
        for (i, pin) in pins.iter().enumerate() {
            pin.make_input();
            pin.enable_interrupt(i, InterruptMode::EitherEdge);
        }

        Button {
            pins: pins,
            callback: container,
        }
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for Button<'a, G> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // set callback for pin interrupts (no affect or reliance on individual pins being
            // configured as interrupts)
            0 => {
                self.callback
                    .enter(callback.app_id(), |cntr, _| {
                        cntr.0 = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, appid: AppId) -> ReturnCode {
        let pins = self.pins.as_ref();
        match command_num {
            // return pin count
            0 => ReturnCode::SuccessWithValue { value: pins.len() as usize },
            // enable interrupts on pin
            1 => {
                if data < pins.len() {
                    self.callback
                        .enter(appid, |cntr, _| {
                            cntr.1 |= 1 << data;
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        })
                } else {
                    ReturnCode::EINVAL /* impossible pin */
                }
            }

            // disable interrupts on pin
            // (no affect or reliance on registered callback)
            2 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    self.callback
                        .enter(appid, |cntr, _| {
                            cntr.1 &= !(1 << data);
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        })
                }
            }

            // read input
            3 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let pin_state = pins[data].read();
                    ReturnCode::SuccessWithValue { value: pin_state as usize }
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, G: hil::gpio::Pin> Client for Button<'a, G> {
    fn fired(&self, pin_num: usize) {
        // read the value of the pin
        let pins = self.pins.as_ref();
        let pin_state = pins[pin_num].read();

        // schedule callback with the pin number and value
        self.callback.each(|cntr| {
            cntr.0.map(|mut callback| if cntr.1 & (1 << pin_num) != 0 {
                callback.schedule(pin_num, pin_state as usize, 0);
            });
        });
    }
}
