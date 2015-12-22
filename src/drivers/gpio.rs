use core::cell::RefCell;
use hil::{Driver, Callback, NUM_PROCS, AppPtr, Shared};
use hil::gpio::{self, GPIOPin};

pub struct GPIO<S: AsRef<[&'static GPIOPin]>> {
    pins: S,
    callbacks: [RefCell<Option<AppPtr<Shared, Callback>>>; NUM_PROCS]
}

impl<S: AsRef<[&'static GPIOPin]>> GPIO<S> {
    pub fn new(pins: S) -> GPIO<S> {
        GPIO {
            pins: pins,
            callbacks: [RefCell::new(None); NUM_PROCS]
        }
    }
}

impl<S: AsRef<[&'static GPIOPin]>> gpio::Client for GPIO<S> {
    fn fired(&self, pin_index: usize) {
        for mcb in self.callbacks.iter() {
            mcb.borrow_mut().as_mut().map(|callback| {
                callback.schedule(0, 0, 0);
            });
        }
    }
}

impl<S: AsRef<[&'static GPIOPin]>> Driver for GPIO<S> {
    fn subscribe(&self, pin_num: usize, callback: Callback) -> isize {
        let pins = self.pins.as_ref();
        if pin_num >= pins.len() {
            -1
        } else {
            let mut mcb = self.callbacks[callback.app_id().idx()].borrow_mut();
            *mcb = AppPtr::alloc(callback, callback.app_id());
            0
        }
    }

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

