use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::hil::gpio::{Pin, PinCtl, InputMode, InterruptMode, Client};

pub struct GPIO<'a, G: Pin + 'a> {
    pins: &'a [&'a G],
    callback: Cell<Option<Callback>>,
}

impl<'a, G: Pin + PinCtl> GPIO<'a, G> {
    pub fn new(pins: &'a [&'a G]) -> GPIO<'a, G> {
        GPIO {
            pins: pins,
            callback: Cell::new(None),
        }
    }

    fn configure_input_pin(&self, pin_num: usize, config: usize) -> isize {
        let pin = self.pins[pin_num];
        pin.make_input();
        match config {
            0 => {
                pin.set_input_mode(InputMode::PullUp);
                0
            }

            1 => {
                pin.set_input_mode(InputMode::PullDown);
                0
            }

            2 => {
                pin.set_input_mode(InputMode::PullNone);
                0
            }

            _ => -1,
        }
    }

    fn configure_interrupt(&self, pin_num: usize, config: usize) -> isize {
        let pins = self.pins.as_ref();
        match config {
            0 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::EitherEdge);
                0
            }

            1 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::RisingEdge);
                0
            }

            2 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::FallingEdge);
                0
            }

            _ => -1,
        }
    }
}

impl<'a, G: Pin> Client for GPIO<'a, G> {
    fn fired(&self, pin_num: usize) {
        // read the value of the pin
        let pins = self.pins.as_ref();
        let pin_state = pins[pin_num].read();

        // schedule callback with the pin number and value
        if self.callback.get().is_some() {
            self.callback.get().unwrap().schedule(pin_num, pin_state as usize, 0);
        }
    }
}

impl<'a, G: Pin + PinCtl> Driver for GPIO<'a, G> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            // subscribe to all pin interrupts
            // (no affect or reliance on individual pins being configured as interrupts)
            0 => {
                self.callback.set(Some(callback));
                0
            }

            // default
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        let pins = self.pins.as_ref();
        match command_num {
            // enable output
            0 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].make_output();
                    0
                }
            }

            // set pin
            1 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].set();
                    0
                }
            }

            // clear pin
            2 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].clear();
                    0
                }
            }

            // toggle pin
            3 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].toggle();
                    0
                }
            }

            // enable and configure input
            4 => {
                // XXX: this is clunky
                // data == ((pin_config << 8) | pin)
                // this allows two values to be passed into a command interface
                let pin_num = data & 0xFF;
                let pin_config = (data >> 8) & 0xFF;
                if pin_num >= pins.len() {
                    -1
                } else {
                    let err_code = self.configure_input_pin(pin_num, pin_config);
                    err_code
                }
            }

            // read input
            5 => {
                if data >= pins.len() {
                    -1
                } else {
                    let pin_state = pins[data].read();
                    pin_state as isize
                }
            }

            // enable and configure interrupts on pin, also sets pin as input
            // (no affect or reliance on registered callback)
            6 => {
                // TODO(brghena): this is clunky
                // data == ((irq_config << 16) | (pin_config << 8) | pin)
                // this allows three values to be passed into a command interface
                let pin_num = data & 0xFF;
                let pin_config = (data >> 8) & 0xFF;
                let irq_config = (data >> 16) & 0xFF;
                if pin_num >= pins.len() {
                    -1
                } else {
                    let mut err_code = self.configure_input_pin(pin_num, pin_config);
                    if err_code == 0 {
                        err_code = self.configure_interrupt(pin_num, irq_config);
                    }
                    err_code
                }
            }

            // disable interrupts on pin, also disables pin
            // (no affect or reliance on registered callback)
            7 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].disable_interrupt();
                    pins[data].disable();
                    0
                }
            }

            // disable pin
            8 => {
                if data >= pins.len() {
                    -1
                } else {
                    pins[data].disable();
                    0
                }
            }

            // default
            _ => -1,
        }
    }
}
