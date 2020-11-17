//! Test entropy and random number generators. Usually, to test the
//! full library, these generators should be through two layers of
//! translation for entropy then converted to randomness. For example,
//! if your platform provides an Entropy32, then test Entropy32 ->
//! Entropy32to8 -> Entropy8to32 -> Entropy32ToRandom. Then simply ask
//! for ELEMENTS random numbers and print them in hex to console.

// TODO: Revert

use crate::virtual_rng::VirtualRngMasterDevice;
use core::cell::Cell;
use kernel::debug;
use kernel::hil::entropy;
use kernel::hil::rng;
use kernel::hil::rng::{Rng, Random};
use kernel::ReturnCode;

const ELEMENTS: usize = 8;

trait FooAndBar<'a>: Random<'a> + Rng<'a> {}
impl<'a, T> FooAndBar<'a> for T where T: Random<'a> + Rng<'a> {}

// Use this test to test an Rng
pub struct TestRng<'a, R: Random<'a> + Rng<'a>> {
    rng: &'a VirtualRngMasterDevice<'a, R>,
    pool: Cell<[u32; ELEMENTS]>,
    count: Cell<usize>,
}

impl<'a, R: Random<'a> + Rng<'a>> TestRng<'a, R> {
    pub fn new(rng: &'a VirtualRngMasterDevice<'a, R>) -> TestRng<'a, R> {
        debug!("Hi from inside virtual rng tester new!");
        TestRng {
            rng: rng,
            pool: Cell::new([0xeeeeeeee; ELEMENTS]),
            count: Cell::new(0),
        }
    }

    pub fn run(&self) {
        debug!("Hi from inside virtual rng tester!");
        match self.rng.get() {
            ReturnCode::SUCCESS => debug!("Virtual RNG test: first get SUCCESS"),
            _ => panic!("Virtual RNG test: unable to get random numbers"),
        }
    }
}
