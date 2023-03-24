// Virtualizer for the RNG
use core::cell::Cell;
use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::rng::{Client, Continue, Rng};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Get,
}

// Struct to manage multiple rng requests
pub struct MuxRngMaster<'a> {
    rng: &'a dyn Rng<'a>,
    devices: List<'a, VirtualRngMasterDevice<'a>>,
    inflight: OptionalCell<&'a VirtualRngMasterDevice<'a>>,
}

impl<'a> MuxRngMaster<'a> {
    pub const fn new(rng: &'a dyn Rng<'a>) -> MuxRngMaster<'a> {
        MuxRngMaster {
            rng: rng,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) -> Result<(), ErrorCode> {
        if self.inflight.is_none() {
            let mnode = self
                .devices
                .iter()
                .find(|node| node.operation.get() != Op::Idle);

            let return_code = mnode.map(|node| {
                let op = node.operation.get();
                let operation_code = match op {
                    Op::Get => {
                        let success_code = self.rng.get();

                        // Only set inflight to node if we successfully initiated rng
                        if success_code == Ok(()) {
                            self.inflight.set(node);
                        }
                        success_code
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
                Err(ErrorCode::FAIL)
            }
        } else {
            Ok(())
        }
    }
}

impl<'a> Client for MuxRngMaster<'a> {
    fn randomness_available(
        &self,
        _randomness: &mut dyn Iterator<Item = u32>,
        _error: Result<(), ErrorCode>,
    ) -> Continue {
        // Try find if randomness is available, or return done
        self.inflight.take().map_or(Continue::Done, |device| {
            let cont_code = device.randomness_available(_randomness, _error);

            if cont_code == Continue::Done {
                let _ = self.do_next_op();
            }

            cont_code
        })
    }
}

// Struct for a single rng device
pub struct VirtualRngMasterDevice<'a> {
    //reference to the mux
    mux: &'a MuxRngMaster<'a>,

    // Pointer to next element in the list of devices
    next: ListLink<'a, VirtualRngMasterDevice<'a>>,
    client: OptionalCell<&'a dyn Client>,
    operation: Cell<Op>,
}

// Implement ListNode trait for virtual rng device
impl<'a> ListNode<'a, VirtualRngMasterDevice<'a>> for VirtualRngMasterDevice<'a> {
    fn next(&self) -> &'a ListLink<VirtualRngMasterDevice<'a>> {
        &self.next
    }
}

impl<'a> VirtualRngMasterDevice<'a> {
    pub const fn new(mux: &'a MuxRngMaster<'a>) -> VirtualRngMasterDevice<'a> {
        VirtualRngMasterDevice {
            mux: mux,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            operation: Cell::new(Op::Idle),
        }
    }
}

impl<'a> PartialEq<VirtualRngMasterDevice<'a>> for VirtualRngMasterDevice<'a> {
    fn eq(&self, other: &VirtualRngMasterDevice<'a>) -> bool {
        // Check whether two rng devices point to the same device
        self as *const VirtualRngMasterDevice<'a> == other as *const VirtualRngMasterDevice<'a>
    }
}

impl<'a> Rng<'a> for VirtualRngMasterDevice<'a> {
    fn get(&self) -> Result<(), ErrorCode> {
        self.operation.set(Op::Get);
        self.mux.do_next_op()
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        // Set current device to idle
        self.operation.set(Op::Idle);

        self.mux.inflight.map_or_else(
            || {
                // If no node inflight, just set node to idle and return
                Ok(())
            },
            |current_node| {
                // Find if current device is the one in flight or not
                if *current_node == self {
                    self.mux.rng.cancel()
                } else {
                    Ok(())
                }
            },
        )
    }

    fn set_client(&'a self, client: &'a dyn Client) {
        self.mux.devices.push_head(&self);

        // Set client to handle callbacks for current device
        self.client.set(client);

        // Set client for rng to be current virtualizer
        self.mux.rng.set_client(self.mux);
    }
}

impl<'a> Client for VirtualRngMasterDevice<'a> {
    fn randomness_available(
        &self,
        randomness: &mut dyn Iterator<Item = u32>,
        error: Result<(), ErrorCode>,
    ) -> Continue {
        self.client.map_or(Continue::Done, move |client| {
            client.randomness_available(randomness, error)
        })
    }
}
