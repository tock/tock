//! Test virtual rng for a single device
//! Gets a specified number of random numbers by making sequential calls to get()
//! Full test harness for this can be found in nano33ble/test/virtual_rng_test

use crate::virtual_rng::VirtualRngMasterDevice;

use core::cell::Cell;

use kernel::debug;
use kernel::hil::rng::{Client, Continue, Rng};
use kernel::ErrorCode;

const NUM_REQUESTS: usize = 2;

// Use this test to test an Rng
pub struct TestRng<'a> {
    device_id: usize,
    device: &'a VirtualRngMasterDevice<'a>,
    num_requests: Cell<usize>,
}

impl<'a> TestRng<'a> {
    pub fn new(device_id: usize, device: &'a VirtualRngMasterDevice<'a>) -> TestRng<'a> {
        TestRng {
            device_id: device_id,
            device: device,
            num_requests: Cell::new(NUM_REQUESTS),
        }
    }

    pub fn get_random_nums(&self) {
        match self.device.get() {
            Ok(()) => debug!("Virtual RNG device {}: get Ok(())", self.device_id),
            _ => panic!("Virtual RNG test: unable to get random numbers"),
        }
    }
}

impl<'a> Client for TestRng<'a> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        error: Result<(), ErrorCode>,
    ) -> Continue {
        let val = randomness.next();
        if error != Ok(()) {
            panic!(
                "Virtual RNG device {}: randomness_available called with error {:?}",
                self.device_id, error
            );
        }

        let num_requests_remaining = self.num_requests.get();
        let data = val.unwrap();
        debug!("Random Number from device {}: {:08x}", self.device_id, data);
        self.num_requests.set(num_requests_remaining - 1);
        if num_requests_remaining == 1 {
            Continue::Done
        } else {
            let _ = self.device.get();
            Continue::More
        }
    }
}
