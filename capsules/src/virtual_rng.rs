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

    fn do_next_op(&self) -> ReturnCode {
        if self.inflight.is_none() {
            let mnode = self
                .devices
                .iter()
                .find(|node| node.operation.get() != Op::Idle);
            let return_code = mnode.map(|node| {
                let op = node.operation.get();
                let operation_code = match op {
                    Op::Get => {
                        node.client.map_or(ReturnCode::FAIL, |client| {
                            self.rng.set_client(*client);
                            let success_code = self.rng.get();

                            // Only set inflight to node if we successfully initiated rng
                            if success_code == ReturnCode::SUCCESS {
                                self.inflight.set(node);
                            }
                            success_code
                        })
                    }
                    Op::Idle => unreachable!("Attempted to run idle operation in virtual_rng!"), // Can't get here...
                };

                // Mark operation as done
                node.operation.set(Op::Idle);
                operation_code
            });

            // Check if return code has a value
            if let Some(r) = return_code {
                r
            } else {
                ReturnCode::FAIL
            }
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl<'a, R: Rng<'a>> Client for MuxRngMaster<'a, R> {
    fn randomness_available(
        &self,
        _randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        // Try find if randomness is available, or return done
        self.inflight.take().map_or(Continue::Done, |device| {
            self.do_next_op();
            device.randomness_available(_randomness, _error)
        })
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
}

impl<'a, R: Rng<'a>> PartialEq<VirtualRngMasterDevice<'a, R>> for VirtualRngMasterDevice<'a, R> {
    fn eq(&self, other: &VirtualRngMasterDevice<'a, R>) -> bool {
        // Check whether two rng devices point to the same device
        self as *const VirtualRngMasterDevice<'a, R>
            == other as *const VirtualRngMasterDevice<'a, R>
    }
}

impl<'a, R: Rng<'a>> Rng<'a> for VirtualRngMasterDevice<'a, R> {
    fn get(&self) -> ReturnCode {
        self.operation.set(Op::Get);
        self.mux.do_next_op()
    }

    fn cancel(&self) -> ReturnCode {
        self.mux.inflight.map_or_else(
            || {
                // If no node inflight, just set node to idle and return
                self.operation.set(Op::Idle);
                ReturnCode::SUCCESS
            },
            |current_node| {
                // Find if current device is the one in flight or not
                if *current_node == self {
                    self.mux.rng.cancel()
                } else {
                    self.operation.set(Op::Idle);
                    ReturnCode::SUCCESS
                }
            },
        )
    }

    fn set_client(&'a self, client: &'a dyn Client) {
        self.mux.devices.push_head(self);
        self.client.set(client);
    }
}

impl<'a, R: Rng<'a>> Client for VirtualRngMasterDevice<'a, R> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        _error: ReturnCode,
    ) -> Continue {
        self.client.map_or(Continue::Done, move |client| {
            client.randomness_available(randomness, _error)
        })
    }
}
