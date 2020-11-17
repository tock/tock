// Test file for virtual_rng
// Currently tests out getting rng sequentially

use crate::virtual_rng::VirtualRngMasterDevice;
use kernel::debug;
use kernel::hil::rng::Rng;
use kernel::ReturnCode;

const ELEMENTS: usize = 8;

pub struct TestRng<'a, R: Rng<'a>> {
    rng: &'a VirtualRngMasterDevice<'a, R>,
}

impl<'a, R: Rng<'a>> TestRng<'a, R> {
    pub fn new(rng: &'a VirtualRngMasterDevice<'a, R>) -> TestRng<'a, R> {
        debug!("Initialized virtual_rng tester");
        TestRng { rng: rng }
    }

    pub fn run(&self) {
        debug!("Starting virtual_rng get tests:");
        for x in 1..ELEMENTS {
            match self.rng.get() {
                ReturnCode::SUCCESS => debug!("virtual_rng test: get {} SUCCESS", x),
                _ => panic!("Virtual RNG test: unable to get random numbers"),
            }
        }
    }
}
