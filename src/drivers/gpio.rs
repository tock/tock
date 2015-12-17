use hil::{Driver};
use hil::gpio::GPIOPin;

pub struct GPIO<S: AsRef<[&'static GPIOPin]>> {
    pins: S,
}

impl<S: AsRef<[&'static GPIOPin]>> GPIO<S> {
    pub fn new(pins: S) -> GPIO<S> {
        GPIO {
            pins: pins
        }
    }
}

impl<S: AsRef<[&'static GPIOPin]>> Driver for GPIO<S> {
    fn command(&self, cmd_num: usize, pin_num: usize, _: usize) -> isize {
        let pins = self.pins.as_ref();
        match cmd_num {
            0 /* output/input */ => {
                if pin_num >= pins.len() {
                    -1
                } else {
                    pins[pin_num].enable_output();
                    0
                }
            },
            2 /* set */ => {
                if pin_num >= pins.len() {
                    -1
                } else {
                    pins[pin_num].set();
                    0
                }
            },
            3 /* clear */ => {
                if pin_num >= pins.len() {
                    -1
                } else {
                    pins[pin_num].clear();
                    0
                }
            },
            4 /* toggle */ => {
                if pin_num >= pins.len() {
                    -1
                } else {
                    pins[pin_num].toggle();
                    0
                }
            },
            _ => -1
        }
    }
}

