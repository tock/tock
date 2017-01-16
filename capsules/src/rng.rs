
use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Shared, Driver};
use kernel::hil::rng;

pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
    remaining: usize,
    idx: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            buffer: None,
            remaining: 0,
            idx: 0,
        }
    }
}

pub struct SimpleRng<'a, RNG: rng::RNG + 'a> {
    rng: &'a RNG,
    apps: Container<App>,
    getting_randomness: Cell<bool>,
}

impl<'a, RNG: rng::RNG> SimpleRng<'a, RNG> {
    pub fn new(rng: &'a RNG, container: Container<App>) -> SimpleRng<'a, RNG> {
        SimpleRng {
            rng: rng,
            apps: container,
            getting_randomness: Cell::new(false),
        }
    }
}

impl<'a, RNG: rng::RNG> rng::Client for SimpleRng<'a, RNG> {
    fn randomness_available(&self, randomness: &mut Iterator<Item = u32>) -> rng::Continue {
        let mut done = true;

        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                // Check if this app needs and random values.
                if app.remaining > 0 && app.callback.is_some() && app.buffer.is_some() {
                    app.buffer.take().map(|mut buffer| {

                        // Check that the app is not asking for more than can
                        // fit in the provided buffer.
                        if buffer.len() < app.idx + app.remaining {
                            app.remaining = buffer.len() - app.idx;
                        }

                        {
                            // Add all available and requested randomness to the app buffer.
                            let buf_len = buffer.len();
                            let d = &mut buffer.as_mut()[0..buf_len];
                            for (_, a) in randomness.enumerate() {
                                for j in 0..4 {
                                    if app.remaining == 0 {
                                        break;
                                    }
                                    d[app.idx] = ((a >> j * 8) & 0xff) as u8;
                                    app.idx += 1;
                                    app.remaining -= 1;
                                }
                                if app.remaining == 0 {
                                    break;
                                }
                            }
                        }
                        // Replace taken buffer
                        app.buffer = Some(buffer);

                    });

                    if app.remaining > 0 {
                        done = false;
                    } else {
                        app.callback.map(|mut cb| {
                            cb.schedule(0, 0, 0);
                        });
                    }
                }
            });

            // Check if done switched to false. If it did, then that app
            // didn't get enough random, so there's no way there is more for
            // other apps.
            if done == false {
                break;
            }
        }

        if done {
            self.getting_randomness.set(false);
            rng::Continue::Done
        } else {
            rng::Continue::More
        }
    }
}

impl<'a, RNG: rng::RNG> Driver for SimpleRng<'a, RNG> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        // pass buffer in from application
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.buffer = Some(slice);
                        0
                    })
                    .unwrap_or(-1)
            }
            _ => -1,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.apps
                    .enter(callback.app_id(), |app, _| {
                        app.callback = Some(callback);
                        0
                    })
                    .unwrap_or(-1)
            }

            // default
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, appid: AppId) -> isize {
        match command_num {
            0 => /* Check if exists */ 0,

            // Ask for a given number of random bytes.
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.remaining = data;
                        app.idx = 0;

                        if !self.getting_randomness.get() {
                            self.getting_randomness.set(true);
                            self.rng.get();
                        }
                        0
                    })
                    .unwrap_or(-1)
            }
            _ => -1,
        }
    }
}
