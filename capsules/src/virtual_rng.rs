use core::cell::Cell;
use core::marker::PhantomData;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
// TODO: use these for every reference to Rng
use kernel::hil::rng::{Client, Continue, Random, Rng};
use kernel::ReturnCode;

// The Mux struct manages multiple Rng clients. Each client may have
// at most one outstanding Rng request.
pub struct MuxRngMaster<'a, Rng: kernel::hil::rng::Rng<'a>> {
    rng: &'a Rng,
    // TODO: check if valid (stolen from virtual digest)
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
    // Additional data storage needed to implement virtualization logic
    inflight: OptionalCell<&'a VirtualRngMasterDevice<'a, Rng>>,
}

impl<'a, Rng: kernel::hil::rng::Rng<'a>> MuxRngMaster<'a, Rng> {
    pub const fn new(rng: &'a Rng) -> MuxRngMaster<'a, Rng> {
        MuxRngMaster {
            rng: rng,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
            inflight: OptionalCell::empty(),
        }
    }

    // TODO: Implement virtualization logic helper functions
}

pub struct VirtualRngMasterDevice<'a, Rng: kernel::hil::rng::Rng<'a>> {
    //reference to the mux
    mux: &'a MuxRngMaster<'a, Rng>,

    // Pointer to next element in the list of devices
    next: ListLink<'a, VirtualRngMasterDevice<'a, Rng>>,
    client: OptionalCell<&'a dyn kernel::hil::rng::Client>,
}

impl<'a, Rng: kernel::hil::rng::Rng<'a>> VirtualRngMasterDevice<'a, Rng> {
    pub const fn new(
        mux: &'a MuxRngMaster<'a, Rng>,
    ) -> VirtualRngMasterDevice<'a, Rng> {
        VirtualRngMasterDevice {
            mux: mux,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    // Most virtualizers will use a set_client method that looks exactly like this
    pub fn set_client(&'a self, client: &'a dyn kernel::hil::rng::Client) {
        // self.mux.devices.push_head(self);
        // TODO: check whether we need to use a linked list or not
        self.client.set(client);
    }
}

impl<'a, Rng: kernel::hil::rng::Rng<'a>> kernel::hil::rng::Rng<'a> for VirtualRngMasterDevice<'a, Rng> {
    fn get(&self) -> ReturnCode {
        // TODO: return random get
        // self.egen.get()
        return ReturnCode::SUCCESS; //TODO: remove filler
    }

    fn cancel(&self) -> ReturnCode {
        // TODO: cancel random get
        return ReturnCode::SUCCESS; //TODO: remove filler
    }

    fn set_client(&'a self, _: &'a dyn Client) {
        // TODO: set the client (refer to rng)
    }
}

impl<'a, Rng: kernel::hil::rng::Rng<'a>> kernel::hil::rng::Random<'a> for VirtualRngMasterDevice<'a, Rng> {
    fn initialize(&'a self) {
        // TODO: init mux
    }

    fn reseed(&self, seed: u32) {
        // TODO: reseed the capsule
    }

    fn random(&self) -> u32 {
        // TODO: return a random number
        return 0;
    }
}

impl<'a, Rng: kernel::hil::rng::Rng<'a>> kernel::hil::rng::Client for VirtualRngMasterDevice<'a, Rng> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        error: ReturnCode,
    ) -> Continue {
        match randomness.next() {
            None => Continue::More,
            Some(val) => {
                // TODO: set the random seed
                // self.seed.set(val);
                Continue::Done
            }
        }
    }
}
