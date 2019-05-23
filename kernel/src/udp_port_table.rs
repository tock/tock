//! UDP port table implementation for enforcing port binding. This table is
//! checked when packets are sent/received by a UDPSendStruct/UDPReceiver.
//! UdpPortBinding provides an opaque descriptor object that allows the holder
//! interact with the bound port table. Only the holder of the UdpPortBinding
//! object can interact with its own corresponding location in the bound port
//! table. In order to bind to a particular port as sending/receiving, one must
//! obtain the corresponding sender/receiving binding from UdpPortBinding.
use tock_cells::take_cell::TakeCell;
use tock_cells::optional_cell::OptionalCell;
use core::cell::Cell;
use crate::net_permissions::{AddrRange, PortRange}; // testing
use crate::capabilities;
use crate::returncode::ReturnCode;
use crate::create_capability;

//#![allow(dead_code)]
const MAX_NUM_BOUND_PORTS: usize = 16;

#[derive(Clone, Copy)] // TODO: must we derive these traits?
pub enum PortEntry {
    Port(u16),
    Unbound,
}

// We need Option<PortEntry> to distinguish between the case in which we have
// a UdpPortSocket that is not bound to a port and an index where there is no
// UdpPortSocket allocated.
static mut port_table: [Option<PortEntry>; MAX_NUM_BOUND_PORTS] = [None; MAX_NUM_BOUND_PORTS];

pub trait PortQuery {
    fn is_bound(&self, port: u16) -> bool;
}

// A UdpPortSocket provides a handle into the bound port table. When binding to
// a port, the socket is consumed and stored inside a UdpPortBinding. When
// undbinding, the socket is returned and can be used to bind to other ports.
pub struct UdpPortSocket {
    idx: usize,
}

pub struct UdpPortTable {
    port_array: TakeCell<'static, [Option<PortEntry>]>,
    user_ports: OptionalCell<&'static PortQuery>,
}

impl UdpPortSocket {
    fn new(idx: usize) -> UdpPortSocket {
        UdpPortSocket {idx: idx}
    }
}

// An opaque descriptor that allows the holder to obtain a binding on a port
// for receiving UDP packets.
pub struct UdpReceiverBinding {
    idx: usize,
    port: u16,
}

// An opaque descriptor that allows the holder to obtain a binding on a port
// for sending UDP packets.
pub struct UdpSenderBinding {
    idx: usize,
    port: u16,
}

impl UdpSenderBinding {
    fn new(idx: usize, port: u16)
        -> UdpSenderBinding {
        UdpSenderBinding {idx: idx, port: port}

    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl UdpReceiverBinding {
    fn new(idx: usize, port: u16)
        -> UdpReceiverBinding {
        UdpReceiverBinding {idx: idx, port: port}

    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}



impl UdpPortTable {
    // Mark new as unsafe so that the port table is only generated in trusted
    // code.
    pub unsafe fn new() -> UdpPortTable {
        UdpPortTable {
            port_array: TakeCell::new(&mut port_table),
            user_ports: OptionalCell::empty(),
        }
    }

    pub unsafe fn set_user_ports(&self, user_ports_ref: &'static PortQuery) {
        self.user_ports.replace(user_ports_ref);
    }

    pub fn create_socket(&self) -> Result<UdpPortSocket, ReturnCode> {
        self.port_array.map(|table| {
            let mut result: Result<UdpPortSocket, ReturnCode> = Err(ReturnCode::FAIL);
            for i in 0..MAX_NUM_BOUND_PORTS {
                match table[i] {
                    None => {
                        result = Ok(UdpPortSocket::new(i));
                        table[i] = Some(PortEntry::Unbound);
                        break;
                    },
                    _ => (),
                }
            };
            result
        }).unwrap()
    }

    pub fn destroy_socket(&self, socket: UdpPortSocket) {
        self.port_array.map(|table| {
            table[socket.idx] = None;
        });
    }

    pub fn is_bound(&self, port: u16) -> bool {
        // TODO: return error if self.user_ports is empty!!!!!
        // First, check the user bindings.
        if self.user_ports.is_none() {
            debug!("empty user ports.");
        } else {
            debug!("not empty user ports.");
        }
        // TODO: Change is_bound to return ReturnCode or Result so we can
        // seperately handle error case of user_ports not existing.
        // Currently, if user_ports doesnt exist we just pretend that
        // the requested port is already bound.
        let user_bound = self.user_ports.map_or(true, |port_query| {
            port_query.is_bound(port)
        });
        if self.user_ports.is_none() {
            debug!("I am gone.");
        }
        if user_bound {
            return true;
        };
        self.port_array.map(|table| {
            let mut port_exists = false;
            for i in 0..MAX_NUM_BOUND_PORTS {
                match table[i] {
                    Some(PortEntry::Port(p)) => {
                        if (p == port) {
                            port_exists = true;
                            break;
                        }
                    },
                    _ => (),
                }
            };
            port_exists
        }).unwrap()
    }

    // On success, a UdpPortBinding is returned. On failure, the same
    // UdpPortSocket is returned.
    pub fn bind(&self, socket: UdpPortSocket, port: u16, /*cap: &UdpCapability*/) ->
        Result<(UdpSenderBinding, UdpReceiverBinding), UdpPortSocket> {
        debug!("Checking binding on: {:?}", port);
        if self.is_bound(port) {
            Err(socket)
        } else {
            self.port_array.map(|table| {
                table[socket.idx] = Some(PortEntry::Port(port));
                let binding_pair = (UdpSenderBinding::new(socket.idx, port),
                    UdpReceiverBinding::new(socket.idx, port));
                // Add socket to the linked list.
                Ok(binding_pair)
            }).unwrap()
        }
    }



    // Disassociate the port from the given binding. Return the socket that was
    // contained within the binding object.
    pub fn unbind(&self, sender_binding: UdpSenderBinding,
        receiver_binding: UdpReceiverBinding,
        /*cap: &capabilities::UDPBindCapability*/)
    -> Result<UdpPortSocket, (UdpSenderBinding, UdpReceiverBinding)> {
        // Verfify that the indices match up
        if sender_binding.idx != receiver_binding.idx {
            return Err((sender_binding, receiver_binding));
        }
        let idx = sender_binding.idx;
        self.port_array.map(|table| {
            table[idx] = None;
        });
        // Search the list and return the appropriate socket
        Ok(UdpPortSocket::new(idx))
    }


}
