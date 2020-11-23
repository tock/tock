// Test file for virtual_rng
// Currently tests out getting rng sequentially

// TODO: REMOVE SIZED MAN

use crate::virtual_rng::{MuxRngMaster, VirtualRngMasterDevice};
use kernel::debug;
use kernel::hil::rng::Rng;
use kernel::ReturnCode;

const ELEMENTS: usize = 4;

pub struct TestRng<'a, R: Rng<'a>+ ?Sized> {
    mux: &'a MuxRngMaster<'a, R>,
}

impl<'a, R: Rng<'a>+ ?Sized> TestRng<'a, R> {
    pub fn new(mux: &'a MuxRngMaster<'a, R>) -> TestRng<'a, R> {
        debug!("Initialized virtual_rng tester");
        TestRng { mux: mux }
    }

    pub fn run(&self) {
        debug!("Starting virtual_rng get tests:");

        // Setup clients:
        let client1 = VirtualRngMasterDevice::new(self.mux);
        let client2 = VirtualRngMasterDevice::new(self.mux);
        let client3 = VirtualRngMasterDevice::new(self.mux);

        // Check clients are able to get random numbers sequentially
        for x in 1..ELEMENTS {
            match client1.get() {
                ReturnCode::SUCCESS => debug!("virtual_rng test: get {} SUCCESS", x),
                _ => panic!("Virtual RNG test: unable to get random numbers"),
            }

            match client2.get() {
                ReturnCode::SUCCESS => debug!("virtual_rng test: get {} SUCCESS", x),
                _ => panic!("Virtual RNG test: unable to get random numbers"),
            }

            match client3.get() {
                ReturnCode::SUCCESS => debug!("virtual_rng test: get {} SUCCESS", x),
                _ => panic!("Virtual RNG test: unable to get random numbers"),
            }
        }
    }
}
