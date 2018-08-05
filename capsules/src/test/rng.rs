//! Test random number generators. Usually, to test the full RNG library, these RNGs
//! should be through two layers of translation. For example, if your platform provides
//! an Rng32, then test Rng32 -> Rng32to8 -> Rng8to32 -> TestRng32. They simply ask
//! for ELEMENTS random numbers and print them in hex to console.

use core::cell::Cell;
use kernel::hil::rng;
use kernel::ReturnCode;

const ELEMENTS: usize = 8;

// Use this test if the underlying RNG is an Rng32
pub struct TestRng32<'a> {
    rng: &'a rng::Rng32<'a>,
    pool: Cell<[u32; ELEMENTS]>,
    count: Cell<usize>,
}

impl<'a> TestRng32<'a> {
    pub fn new(rng: &'a rng::Rng32<'a>) -> TestRng32<'a> {
        TestRng32 {
            rng: rng,
            pool: Cell::new([0xeeeeeeee; ELEMENTS]),
            count: Cell::new(0),
        }
    }

    pub fn run(&self) {
        match self.rng.get() {
            ReturnCode::SUCCESS => debug!("RNG test: first get SUCCESS"),
            _ => panic!("RNG test: unable to get random numbers")
        }
    }
}

impl<'a> rng::Client32 for TestRng32<'a> {

    fn randomness_available(&self,
                            randomness: &mut Iterator<Item = u32>,
                            error: ReturnCode) -> rng::Continue {
        let mut val = randomness.next();
        if error != ReturnCode::SUCCESS {
            panic!("RNG test: randomness_available called with error {:?}", error);
        }
        while val.is_some() {
            //debug!("RNG test: iterator returned Some.");
            let data = val.unwrap();

            let mut pool = self.pool.get();
            let mut count = self.count.get();
            pool[count] = data;
            count = count + 1;
            self.pool.set(pool);
            self.count.set(count);

            if count >= ELEMENTS {
                debug!("RNG test: obtained all {} values. They are:", count);
                for i in 0..pool.len() {
                    debug!("[{:02x}]: {:08x}", i, pool[i]);
                }
                return rng::Continue::Done;
            } else {
                val = randomness.next();
            }
        }
        // val must be None: out of randomness, ask for more
        rng::Continue::More
    }
}

// Use this test if the underlying RNG is an Rng8
pub struct TestRng8<'a> {
    rng: &'a rng::Rng8<'a>,
    pool: Cell<[u8; ELEMENTS]>,
    count: Cell<usize>,
}

impl<'a> TestRng8<'a> {
    pub fn new(rng: &'a rng::Rng8<'a>) -> TestRng8<'a> {
        TestRng8 {
            rng: rng,
            pool: Cell::new([0xee; ELEMENTS]),
            count: Cell::new(0),
        }
    }

    pub fn run(&self) {
        match self.rng.get() {
            ReturnCode::SUCCESS => debug!("RNG test: first get SUCCESS"),
            _ => panic!("RNG test: unable to get random numbers")
        }
    }
}

impl<'a> rng::Client8 for TestRng8<'a> {

    fn randomness_available(&self,
                            randomness: &mut Iterator<Item = u8>,
                            error: ReturnCode) -> rng::Continue {
        let mut val = randomness.next();
        if error != ReturnCode::SUCCESS {
            panic!("RNG test: randomness_available called with error {:?}", error);
        }
        while val.is_some() {
            debug!("RNG test: randomness_available iterator returned Some, adding.");
            let data = val.unwrap();

            let mut pool = self.pool.get();
            let mut count = self.count.get();
            pool[count] = data;
            count = count + 1;
            self.pool.set(pool);
            self.count.set(count);

            if count >= ELEMENTS {
                debug!("RNG test: obtained {} values. They are:", count);
                for i in 0..pool.len() {
                    debug!("[{:02x}]: {:02x}", i, pool[i]);
                }
                return rng::Continue::Done;
            } else {
                val = randomness.next();
            }
        }
        debug!("RNG test: randomness_available iterator returned None, requesting more.");
        // val must be None: out of randomness, ask for more
        rng::Continue::More
    }
}
