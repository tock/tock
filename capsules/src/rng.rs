//! Provides a simple driver for userspace applications to request randomness.
//!
//! The RNG accepts a user-defined callback and buffer to hold received
//! randomness. A single command starts the RNG, the callback is called when the
//! requested amount of randomness is received, or the buffer is filled.
//!
//! Usage
//! -----
//!
//! ```rust
//! let rng = static_init!(
//!         capsules::rng::SimpleRng<'static, sam4l::trng::Trng>,
//!         capsules::rng::SimpleRng::new(&sam4l::trng::TRNG, kernel::Grant::create()));
//! sam4l::trng::TRNG.set_client(rng);
//! ```

use core::cell::Cell;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};
use kernel::hil::rng;
use kernel::process::Error;

/// Syscall number
pub const DRIVER_NUM: usize = 0x40001;

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

pub struct SimpleRng<'a, RNG: rng::RNG<'a> + 'a> {
    rng: &'a RNG,
    apps: Grant<App>,
    getting_randomness: Cell<bool>,
}

impl<'a, RNG: rng::RNG<'a>> SimpleRng<'a, RNG> {
    pub fn new(rng: &'a RNG, grant: Grant<App>) -> SimpleRng<'a, RNG> {
        SimpleRng {
            rng: rng,
            apps: grant,
            getting_randomness: Cell::new(false),
        }
    }
}

impl<'a, RNG: rng::RNG<'a>> rng::Client for SimpleRng<'a, RNG> {
    fn randomness_available(&self, randomness: &mut Iterator<Item = u32>) -> rng::Continue {
        let mut done = true;
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                // Check if this app needs random values.
                if app.remaining > 0 && app.callback.is_some() && app.buffer.is_some() {
                    app.buffer.take().map(|mut buffer| {
                        // Check that the app is not asking for more than can
                        // fit in the provided buffer.

                        if buffer.len() < app.idx + app.remaining {
                            app.remaining = buffer.len() - app.idx;
                        }

                        {
                            // Add all available and requested randomness to the app buffer.

                            // 1. Slice buffer to start from current idx
                            let buf = &mut buffer.as_mut()[app.idx..(app.idx + app.remaining)];
                            // 2. Take at most as many random samples as needed to fill the buffer
                            //    (if app.remaining is not word-sized, take an extra one).
                            let remaining_ints = if app.remaining % 4 == 0 {
                                app.remaining / 4
                            } else {
                                app.remaining / 4 + 1
                            };

                            // 3. Zip over the randomness iterator and chunks
                            //    of up to 4 bytes from the buffer.
                            for (inp, outs) in
                                randomness.take(remaining_ints).zip(buf.chunks_mut(4))
                            {
                                // 4. For each word of randomness input, update
                                //    the remaining and idx and add to buffer.
                                for (i, b) in outs.iter_mut().enumerate() {
                                    *b = ((inp >> i * 8) & 0xff) as u8;
                                    app.remaining -= 1;
                                    app.idx += 1;
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
                            cb.schedule(0, app.idx, 0);
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

impl<'a, RNG: rng::RNG<'a>> Driver for SimpleRng<'a, RNG> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        // pass buffer in from application
        match allow_num {
            0 => self.apps
                .enter(appid, |app, _| {
                    app.buffer = Some(slice);
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 => self.apps
                .enter(callback.app_id(), |app, _| {
                    app.callback = Some(callback);
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => /* Check if exists */ ReturnCode::SUCCESS,

            // Ask for a given number of random bytes.
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.remaining = data;
                        app.idx = 0;

                        if app.callback.is_some() && app.buffer.is_some() {
                            if !self.getting_randomness.get() {
                                self.getting_randomness.set(true);
                                self.rng.get();
                            }
                            ReturnCode::SUCCESS
                        }
                        else {
                            ReturnCode::ERESERVE
                        }
                    })
                    .unwrap_or_else(|err| {
                        match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        }
                    })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
