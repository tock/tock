use hil::{Driver, Callback};
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
    fn subscribe(&self, _: usize, _: Callback) -> isize {
        -1
    }

    fn command(&self, cmd_num: usize, r0: usize) -> isize {
        let pins = self.pins.as_ref();
        match cmd_num {
            0 /* enable output */ => {
                if r0 >= pins.len() {
                    -1
                } else {
                    pins[r0].enable_output();
                    0
                }
            },
            2 /* set */ => {
                if r0 >= pins.len() {
                    -1
                } else {
                    pins[r0].set();
                    0
                }
            },
            3 /* clear */ => {
                if r0 >= pins.len() {
                    -1
                } else {
                    pins[r0].clear();
                    0
                }
            },
            4 /* toggle */ => {
                if r0 >= pins.len() {
                    -1
                } else {
                    pins[r0].toggle();
                    0
                }
            },
            _ => -1
        }
    }
}

