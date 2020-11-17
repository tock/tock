// Virtualizer for the RNG
use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::rng::{Client, Continue, Rng};
use kernel::ReturnCode;

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Get,
    SetClient,
}

// Struct to manage multiple rng requests
pub struct MuxRngMaster<'a, R: Rng<'a>> {
    rng: &'a dyn Rng<'a>,
    devices: List<'a, VirtualRngMasterDevice<'a, R>>,
    inflight: OptionalCell<&'a VirtualRngMasterDevice<'a, R>>,
}

impl<'a, R: Rng<'a>> MuxRngMaster<'a, R> {
    pub const fn new(rng: &'a dyn Rng<'a>) -> MuxRngMaster<'a, R> {
        MuxRngMaster {
            rng: rng,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    // TODO: return type is a hacky way of surfacing the return code from rng.get()
    fn do_next_op(&self) -> ReturnCode {
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
                    Op::SetClient => {
                        self.rng.set_client(node);
                        ReturnCode::SUCCESS
                    }
                    Op::Get => self.rng.get(),
                    Op::Idle => ReturnCode::SUCCESS, // Can't get here...
                }
            });
        }
        ReturnCode::SUCCESS
    }
}

impl<'a, R: Rng<'a>> Client for MuxRngMaster<'a, R> {
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
pub struct VirtualRngMasterDevice<'a, R: Rng<'a>> {
    //reference to the mux
    mux: &'a MuxRngMaster<'a, R>,

    // Pointer to next element in the list of devices
    next: ListLink<'a, VirtualRngMasterDevice<'a, R>>,
    client: OptionalCell<&'a dyn Client>,
    operation: Cell<Op>,
}

// Implement ListNode trait for virtual rng device
impl<'a, R: Rng<'a>> ListNode<'a, VirtualRngMasterDevice<'a, R>> for VirtualRngMasterDevice<'a, R> {
    fn next(&self) -> &'a ListLink<VirtualRngMasterDevice<'a, R>> {
        &self.next
    }
}

impl<'a, R: Rng<'a>> VirtualRngMasterDevice<'a, R> {
    pub const fn new(mux: &'a MuxRngMaster<'a, R>) -> VirtualRngMasterDevice<'a, R> {
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

impl<'a, R: Rng<'a>> Rng<'a> for VirtualRngMasterDevice<'a, R> {
    fn get(&self) -> ReturnCode {
        self.operation.set(Op::Get);
        self.mux.do_next_op()
    }

    fn cancel(&self) -> ReturnCode {
        self.mux.rng.cancel()
    }

    fn set_client(&'a self, _: &'a dyn Client) {
        self.operation.set(Op::SetClient);
        self.mux.do_next_op();
    }
}

impl<'a, R: Rng<'a>> Client for VirtualRngMasterDevice<'a, R> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        self.client
            .map(move |client| client.randomness_available(randomness, _error));
        Continue::Done
    }
}
