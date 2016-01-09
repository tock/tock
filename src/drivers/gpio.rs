use core::cell::RefCell;
use hil::{Driver, Callback, NUM_PROCS, AppPtr, Shared};
use hil::gpio::{self, GPIOPin};

#[derive(Clone, Copy)]
struct PinSubscription {
    callback: Option<Callback>,
    pin_mask: usize
}

pub struct GPIO<S: AsRef<[&'static GPIOPin]>> {
    pins: S,
    callbacks: [RefCell<Option<AppPtr<Shared, PinSubscription>>>; NUM_PROCS]
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
    fn fired(&self, pin_idx: usize) {
        for mcb in self.callbacks.iter() {
            mcb.borrow_mut().as_mut().map(|subscription| {
                if subscription.pin_mask & (1 << pin_idx) != 0 {
                    subscription.callback.as_mut().map(|cb| cb.schedule(0, 0, 0));
                }
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
            let subscription = PinSubscription {
                callback: Some(callback),
                pin_mask: 0
            };
            let mut mcb = self.callbacks[callback.app_id().idx()].borrow_mut();
            *mcb = AppPtr::alloc(subscription, callback.app_id());
            0
        }
    }

    fn command(&self, cmd_num: usize, r0: usize, _: usize) -> isize {
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

