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
use kernel::common::cells::OptionalCell;
use kernel::hil::rng::{self, Client32, Client8, Rng32, Rng8};
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall number
pub const DRIVER_NUM: usize = 0x40001;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
    remaining: usize,
    idx: usize,
}

pub struct SimpleRng<'a> {
    rng: &'a Rng32<'a>,
    apps: Grant<App>,
    getting_randomness: Cell<bool>,
}

impl<'a> SimpleRng<'a> {
    pub fn new(rng: &'a Rng32<'a>, grant: Grant<App>) -> SimpleRng<'a> {
        SimpleRng {
            rng: rng,
            apps: grant,
            getting_randomness: Cell::new(false),
        }
    }
}

impl<'a> Client32 for SimpleRng<'a> {
    fn randomness_available(&self,
                            randomness: &mut Iterator<Item = u32>,
                            _error: ReturnCode) -> rng::Continue {
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

impl<'a> Driver for SimpleRng<'a> {
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        // pass buffer in from application
        match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    app.buffer = slice;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    app.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into()),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 =>
            /* Check if exists */
            {
                ReturnCode::SUCCESS
            }

            // Ask for a given number of random bytes.
            1 => self
                .apps
                .enter(appid, |app, _| {
                    app.remaining = data;
                    app.idx = 0;

                    if app.callback.is_some() && app.buffer.is_some() {
                        if !self.getting_randomness.get() {
                            self.getting_randomness.set(true);
                            self.rng.get();
                        }
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ERESERVE
                    }
                }).unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}


pub struct Rng8To32<'a> {
    rng: &'a Rng8<'a>,
    client: OptionalCell<&'a rng::Client32>,
    count: Cell<usize>,
    bytes: Cell<u32>,

}

impl Rng8To32<'a> {
    pub fn new(rng: &'a Rng8<'a>) -> Rng8To32<'a> {
        Rng8To32 {
            rng: rng,
            client: OptionalCell::empty(),
            count: Cell::new(0),
            bytes: Cell::new(0),
        }
    }
}

impl Rng32<'a> for Rng8To32<'a> {
    fn get(&self) -> ReturnCode {
        self.rng.get()
    }

    /// Cancel acquisition of random numbers.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no oustanding request. No `randomness_available`
    ///     callback will be issued.
    ///   - FAIL: There will be a randomness_available callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode {
        self.rng.cancel()
    }

    fn set_client(&'a self, client: &'a Client32) {
        self.rng.set_client(self);
        self.client.set(client);
    }
}

impl<'a> Client8 for Rng8To32<'a> {
    fn randomness_available(&self,
                            randomness: &mut Iterator<Item = u8>,
                            error: ReturnCode) -> rng::Continue {
        self.client.map_or(rng::Continue::Done, |client|
            {
                if error != ReturnCode::SUCCESS {
                    client.randomness_available(&mut Rng8To32Iter(self), error)
                } else {
                    let mut count = self.count.get();
                    // Read in one byte at a time until we have 4;
                    // return More if we need more, else return the value
                    // of the upper randomness_available, as if it needs more
                    // we'll need more from the underlying Rng8.
                    while count < 4 {
                        let byte = randomness.next();
                        match byte {
                            None => {
                                return rng::Continue::More;
                            },
                            Some(val) => {
                                let current = self.bytes.get();
                                let bits = val as u32;
                                let result = current | (bits << (8 * count));
                                count = count + 1;
                                //debug!("Count: {}, current: {:08x}, bits: {:08x}, result: {:08x}", count, current, bits, result);
                                self.count.set(count);
                                self.bytes.set(result)
                            }
                        }
                    }
                    let rval = client.randomness_available(&mut Rng8To32Iter(self),
                                                           ReturnCode::SUCCESS);
                    self.bytes.set(0);
                    rval
                }
            }
        )
    }
}

struct Rng8To32Iter<'a, 'b: 'a>(&'a Rng8To32<'b>);

impl Iterator for Rng8To32Iter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let count = self.0.count.get();
        if count == 4 {
            self.0.count.set(0);
            Some(self.0.bytes.get())
        } else {
            None
        }
    }
}


pub struct Rng32To8<'a> {
    rng: &'a Rng32<'a>,
    client: OptionalCell<&'a rng::Client8>,
    randomness: Cell<u32>,
    bytes_consumed: Cell<usize>,

}

impl Rng32To8<'a> {
    pub fn new(rng: &'a Rng32<'a>) -> Rng32To8<'a> {
        Rng32To8 {
            rng: rng,
            client: OptionalCell::empty(),
            randomness: Cell::new(0),
            bytes_consumed: Cell::new(0),
        }
    }
}

impl Rng8<'a> for Rng32To8<'a> {
    fn get(&self) -> ReturnCode {
        self.rng.get()
    }

    /// Cancel acquisition of random numbers.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no oustanding request. No `randomness_available`
    ///     callback will be issued.
    ///   - FAIL: There will be a randomness_available callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode {
        self.rng.cancel()
    }

    fn set_client(&'a self, client: &'a Client8) {
        self.rng.set_client(self);
        self.client.set(client);
    }
}


impl<'a> Client32 for Rng32To8<'a> {
    fn randomness_available(&self,
                            randomness: &mut Iterator<Item = u32>,
                            error: ReturnCode) -> rng::Continue {
        self.client.map_or(rng::Continue::Done, |client| {
            if error != ReturnCode::SUCCESS {
                client.randomness_available(&mut Rng32To8Iter(self), error)
            } else {
                let r = randomness.next();
                match r {
                    None => return rng::Continue::More,
                    Some(val) => {
                        self.randomness.set(val);
                        self.bytes_consumed.set(0);
                    }
                }
                client.randomness_available(&mut Rng32To8Iter(self),
                                            ReturnCode::SUCCESS)
            }
        })
    }
}

struct Rng32To8Iter<'a, 'b: 'a>(&'a Rng32To8<'b>);

impl Iterator for Rng32To8Iter<'a, 'b> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let bytes_consumed = self.0.bytes_consumed.get();
        if bytes_consumed < 4 {
            // Pull out a byte and right shift the u32 so its
            // least significant byte is fresh randomness.
            let randomness = self.0.randomness.get();
            let byte = (randomness & 0xff) as u8;
            self.0.randomness.set(randomness >> 8);
            self.0.bytes_consumed.set(bytes_consumed + 1);
            Some(byte)
        } else {
            None
        }
    }
}
