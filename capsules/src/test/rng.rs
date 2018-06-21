//! Test RNG hardware

use kernel::hil::rng::{RNG, Continue, Client};

pub struct TestRng<'a> {
    rng: &'a RNG<'a>,
}

impl<'a> TestRng<'a> {
    pub fn new(
        rng: &'a RNG<'a>,
    ) -> Self {
        TestRng {
            rng: rng,
        }
    }

    pub fn run(&self) {
        self.rng.init();
        self.rng.get();
    }
}

impl<'a> Client for TestRng<'a> {
    fn randomness_available(&self, randomness: &mut Iterator<Item = u32>) -> Continue {
        debug!("Randomness: \r");
        randomness.take(5).for_each(|r| debug!("  [{:x}]\r", r));
        Continue::Done
    }
}
