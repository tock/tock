use core::prelude::*;
use hil::{Driver, Callback};
use hil::gpio::GPIOPin;

pub struct GPIO<S: AsMut<[&'static mut GPIOPin]>> {
    pins: S,
}

impl<S: AsMut<[&'static mut GPIOPin]>> GPIO<S> {
    pub fn new(pins: S) -> GPIO<S> {
        GPIO {
            pins: pins
        }
    }
}

impl<S: AsMut<[&'static mut GPIOPin]>> Driver for GPIO<S> {
    fn subscribe(&mut self, _: usize, _: Callback) -> isize {
        -1
    }

    fn command(&mut self, cmd_num: usize, r0: usize) -> isize {
        let pins = self.pins.as_mut();
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

