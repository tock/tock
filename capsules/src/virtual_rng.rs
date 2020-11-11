use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::rng::{Client, Continue, Random, Rng};
use kernel::ReturnCode;

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    SetClient,
    Initialize,
    Reseed(u32),
    GetRandom,
}

// Struct to manage multiple rng requests
pub struct MuxRngMaster<'a, R: Random<'a> + Rng<'a>> {
    rng: &'a R,
    devices: List<'a, VirtualRngMasterDevice<'a, R>>,
    inflight: OptionalCell<&'a VirtualRngMasterDevice<'a, R>>,
}

impl<'a, R: Random<'a> + Rng<'a>> MuxRngMaster<'a, R> {
    pub const fn new(rng: &'a R) -> MuxRngMaster<'a, R> {
        MuxRngMaster {
            rng: rng,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) -> u32{
        if self.inflight.is_none() {
            let mnode = self
                .devices
                .iter()
                .find(|node| node.operation.get() != Op::Idle);
            mnode.map(|node| {
                let op = node.operation.get();
                // Need to set idle here in case callback changes state
                node.operation.set(Op::Idle);
                match op {
                    // TODO: return values are a hacky way of getting rng.random()
                    //       back to callback. Is there a better way?
                    Op::SetClient => {
                        self.rng.set_client(node);
                        return 0;
                    }
                    Op::Initialize => {
                        self.rng.initialize();
                        return 0;
                    }
                    Op::GetRandom => {
                        self.inflight.set(node);
                        self.rng.random()
                    }
                    Op::Reseed(seed) => {
                        self.rng.reseed(seed);
                        return 0;
                    }
                    Op::Idle => {return 0;} // Can't get here...
                }
            });
        }
        // Return value indicating success
        0
    }
}

impl<'a, R: Random<'a> + Rng<'a>> Client for MuxRngMaster<'a, R> {
    fn randomness_available(
        &self,
        _randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        self.inflight.take().map(move |device| {
            self.do_next_op();
            device.randomness_available(_randomness, _error)
        });
        Continue::Done
    }
}

// Struct for a single rng device
pub struct VirtualRngMasterDevice<'a, R: Random<'a> + Rng<'a>> {
    //reference to the mux
    mux: &'a MuxRngMaster<'a, R>,

    // Pointer to next element in the list of devices
    next: ListLink<'a, VirtualRngMasterDevice<'a, R>>,
    client: OptionalCell<&'a dyn Client>,
    operation: Cell<Op>,
}

// Implement ListNode trait for virtual rng device
impl<'a, R: Random<'a> + Rng<'a>> ListNode<'a, VirtualRngMasterDevice<'a, R>> for VirtualRngMasterDevice<'a, R> {
    fn next(&self) -> &'a ListLink<VirtualRngMasterDevice<'a, R>> {
        &self.next
    }
}

impl<'a, R: Random<'a> + Rng<'a>> VirtualRngMasterDevice<'a, R> {
    pub const fn new(
        mux: &'a MuxRngMaster<'a, R>,
    ) -> VirtualRngMasterDevice<'a, R> {
        VirtualRngMasterDevice {
            mux: mux,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            operation: Cell::new(Op::Idle),
        }
    }

    pub fn set_client(&'a self, client: &'a dyn Client) {
        self.mux.devices.push_head(self);
        self.client.set(client);
    }
}

impl<'a, R: Random<'a> + Rng<'a>> Rng<'a> for VirtualRngMasterDevice<'a, R> {
    fn get(&self) -> ReturnCode {
        return self.mux.rng.get();
    }

    fn cancel(&self) -> ReturnCode {
        return self.mux.rng.cancel();
    }

    fn set_client(&'a self, _: &'a dyn Client) {
        self.operation.set(Op::SetClient);
        self.mux.do_next_op();
    }
}

impl<'a, R: Random<'a> + Rng<'a>> Random<'a> for VirtualRngMasterDevice<'a, R> {
    fn initialize(&'a self) {
        self.operation.set(Op::Initialize);
        self.mux.do_next_op();
    }

    fn reseed(&self, seed: u32) {
        self.operation.set(Op::Reseed(seed));
        self.mux.do_next_op();
    }

    fn random(&self) -> u32 {
        self.operation.set(Op::GetRandom);
        self.mux.do_next_op()
    }
}

impl<'a, R: Random<'a> + Rng<'a>> Client for VirtualRngMasterDevice<'a, R> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        self.client.map(move |client| {
            client.randomness_available(randomness, _error)
        });
        // TODO: is this the valid default return value?
        Continue::Done
    }
}
