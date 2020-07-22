//! Library of randomness structures, including a system call driver for
//! userspace applications to request randomness, entropy conversion, entropy
//! to randomness conversion, and synchronous random number generation.
//!
//!
//! The RNG accepts a user-defined callback and buffer to hold received
//! randomness. A single command starts the RNG, the callback is called when the
//! requested amount of randomness is received, or the buffer is filled.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let rng = static_init!(
//!         capsules::rng::RngDriver<'static, sam4l::trng::Trng>,
//!         capsules::rng::RngDriver::new(&sam4l::trng::TRNG, board_kernel.create_grant(&grant_cap)));
//! sam4l::trng::TRNG.set_client(rng);
//! ```

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::hil::entropy;
use kernel::hil::entropy::{Entropy32, Entropy8};
use kernel::hil::rng;
use kernel::hil::rng::{Client, Continue, Random, Rng};
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Rng as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
    remaining: usize,
    idx: usize,
}

pub struct RngDriver<'a> {
    rng: &'a dyn Rng<'a>,
    apps: Grant<App>,
    getting_randomness: Cell<bool>,
}

impl<'a> RngDriver<'a> {
    pub fn new(rng: &'a dyn Rng<'a>, grant: Grant<App>) -> RngDriver<'a> {
        RngDriver {
            rng: rng,
            apps: grant,
            getting_randomness: Cell::new(false),
        }
    }
}

impl rng::Client for RngDriver<'_> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> rng::Continue {
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

impl<'a> Driver for RngDriver<'a> {
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
                })
                .unwrap_or_else(|err| err.into()),
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
                })
                .unwrap_or_else(|err| err.into()),

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
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

pub struct Entropy32ToRandom<'a> {
    egen: &'a dyn Entropy32<'a>,
    client: OptionalCell<&'a dyn rng::Client>,
}

impl<'a> Entropy32ToRandom<'a> {
    pub fn new(egen: &'a dyn Entropy32<'a>) -> Entropy32ToRandom<'a> {
        Entropy32ToRandom {
            egen: egen,
            client: OptionalCell::empty(),
        }
    }
}

impl<'a> Rng<'a> for Entropy32ToRandom<'a> {
    fn get(&self) -> ReturnCode {
        self.egen.get()
    }

    fn cancel(&self) -> ReturnCode {
        self.egen.cancel()
    }

    fn set_client(&'a self, client: &'a dyn rng::Client) {
        self.egen.set_client(self);
        self.client.set(client);
    }
}

impl entropy::Client32 for Entropy32ToRandom<'_> {
    fn entropy_available(
        &self,
        entropy: &mut dyn Iterator<Item = u32>,
        error: ReturnCode,
    ) -> entropy::Continue {
        self.client.map_or(entropy::Continue::Done, |client| {
            if error != ReturnCode::SUCCESS {
                match client.randomness_available(&mut Entropy32ToRandomIter(entropy), error) {
                    rng::Continue::More => entropy::Continue::More,
                    rng::Continue::Done => entropy::Continue::Done,
                }
            } else {
                match client
                    .randomness_available(&mut Entropy32ToRandomIter(entropy), ReturnCode::SUCCESS)
                {
                    rng::Continue::More => entropy::Continue::More,
                    rng::Continue::Done => entropy::Continue::Done,
                }
            }
        })
    }
}

struct Entropy32ToRandomIter<'a>(&'a mut dyn Iterator<Item = u32>);

impl Iterator for Entropy32ToRandomIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.0.next()
    }
}

pub struct Entropy8To32<'a> {
    egen: &'a dyn Entropy8<'a>,
    client: OptionalCell<&'a dyn entropy::Client32>,
    count: Cell<usize>,
    bytes: Cell<u32>,
}

impl<'a> Entropy8To32<'a> {
    pub fn new(egen: &'a dyn Entropy8<'a>) -> Entropy8To32<'a> {
        Entropy8To32 {
            egen: egen,
            client: OptionalCell::empty(),
            count: Cell::new(0),
            bytes: Cell::new(0),
        }
    }
}

impl<'a> Entropy32<'a> for Entropy8To32<'a> {
    fn get(&self) -> ReturnCode {
        self.egen.get()
    }

    /// Cancel acquisition of random numbers.
    ///
    /// There are two valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no outstanding request. No `randomness_available`
    ///     callback will be issued.
    ///   - FAIL: There will be a randomness_available callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode {
        self.egen.cancel()
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.egen.set_client(self);
        self.client.set(client);
    }
}

impl entropy::Client8 for Entropy8To32<'_> {
    fn entropy_available(
        &self,
        entropy: &mut dyn Iterator<Item = u8>,
        error: ReturnCode,
    ) -> entropy::Continue {
        self.client.map_or(entropy::Continue::Done, |client| {
            if error != ReturnCode::SUCCESS {
                client.entropy_available(&mut Entropy8To32Iter(self), error)
            } else {
                let mut count = self.count.get();
                // Read in one byte at a time until we have 4;
                // return More if we need more, else return the value
                // of the upper randomness_available, as if it needs more
                // we'll need more from the underlying Rng8.
                while count < 4 {
                    let byte = entropy.next();
                    match byte {
                        None => {
                            return entropy::Continue::More;
                        }
                        Some(val) => {
                            let current = self.bytes.get();
                            let bits = val as u32;
                            let result = current | (bits << (8 * count));
                            count = count + 1;
                            self.count.set(count);
                            self.bytes.set(result)
                        }
                    }
                }
                let rval =
                    client.entropy_available(&mut Entropy8To32Iter(self), ReturnCode::SUCCESS);
                self.bytes.set(0);
                rval
            }
        })
    }
}

struct Entropy8To32Iter<'a, 'b: 'a>(&'a Entropy8To32<'b>);

impl Iterator for Entropy8To32Iter<'_, '_> {
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

pub struct Entropy32To8<'a> {
    egen: &'a dyn Entropy32<'a>,
    client: OptionalCell<&'a dyn entropy::Client8>,
    entropy: Cell<u32>,
    bytes_consumed: Cell<usize>,
}

impl<'a> Entropy32To8<'a> {
    pub fn new(egen: &'a dyn Entropy32<'a>) -> Entropy32To8<'a> {
        Entropy32To8 {
            egen: egen,
            client: OptionalCell::empty(),
            entropy: Cell::new(0),
            bytes_consumed: Cell::new(0),
        }
    }
}

impl<'a> Entropy8<'a> for Entropy32To8<'a> {
    fn get(&self) -> ReturnCode {
        self.egen.get()
    }

    /// Cancel acquisition of random numbers.
    ///
    /// There are two valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no outstanding request. No `randomness_available`
    ///     callback will be issued.
    ///   - FAIL: There will be a randomness_available callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode {
        self.egen.cancel()
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client8) {
        self.egen.set_client(self);
        self.client.set(client);
    }
}

impl entropy::Client32 for Entropy32To8<'_> {
    fn entropy_available(
        &self,
        entropy: &mut dyn Iterator<Item = u32>,
        error: ReturnCode,
    ) -> entropy::Continue {
        self.client.map_or(entropy::Continue::Done, |client| {
            if error != ReturnCode::SUCCESS {
                client.entropy_available(&mut Entropy32To8Iter(self), error)
            } else {
                let r = entropy.next();
                match r {
                    None => return entropy::Continue::More,
                    Some(val) => {
                        self.entropy.set(val);
                        self.bytes_consumed.set(0);
                    }
                }
                client.entropy_available(&mut Entropy32To8Iter(self), ReturnCode::SUCCESS)
            }
        })
    }
}

struct Entropy32To8Iter<'a, 'b: 'a>(&'a Entropy32To8<'b>);

impl Iterator for Entropy32To8Iter<'_, '_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let bytes_consumed = self.0.bytes_consumed.get();
        if bytes_consumed < 4 {
            // Pull out a byte and right shift the u32 so its
            // least significant byte is fresh randomness.
            let entropy = self.0.entropy.get();
            let byte = (entropy & 0xff) as u8;
            self.0.entropy.set(entropy >> 8);
            self.0.bytes_consumed.set(bytes_consumed + 1);
            Some(byte)
        } else {
            None
        }
    }
}

pub struct SynchronousRandom<'a> {
    rgen: &'a dyn Rng<'a>,
    seed: Cell<u32>,
}

#[allow(dead_code)]
impl<'a> SynchronousRandom<'a> {
    fn new(rgen: &'a dyn Rng<'a>) -> SynchronousRandom {
        SynchronousRandom {
            rgen: rgen,
            seed: Cell::new(0),
        }
    }
}

impl<'a> Random<'a> for SynchronousRandom<'a> {
    fn initialize(&'a self) {
        self.rgen.set_client(self);
        self.rgen.get();
    }

    fn reseed(&self, seed: u32) {
        self.seed.set(seed);
    }

    // This implementation uses a linear congruential generator due to
    // its efficiency. The parameters for the generator are those
    // recommended in Numerical Recipes by Press, Teukolsky,
    // Vetterling, and Flannery.

    fn random(&self) -> u32 {
        const LCG_MULTIPLIER: u32 = 1_644_525;
        const LCG_INCREMENT: u32 = 1_013_904_223;
        let val = self.seed.get();
        let val = val.wrapping_mul(LCG_MULTIPLIER);
        let val = val.wrapping_add(LCG_INCREMENT);
        self.seed.set(val);
        val
    }
}

impl Client for SynchronousRandom<'_> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        match randomness.next() {
            None => Continue::More,
            Some(val) => {
                self.seed.set(val);
                Continue::Done
            }
        }
    }
}
