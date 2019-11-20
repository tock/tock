//! UDP port table implementation for enforcing port binding. This table is
//! checked when packets are sent/received by a UDPSendStruct/UDPReceiver.
//! UdpPortBinding provides an opaque descriptor object that allows the holder
//! interact with the bound port table. Only the holder of the UdpPortBinding
//! object can interact with its own corresponding location in the bound port
//! table. In order to bind to a particular port as sending/receiving, one must
//! obtain the corresponding sender/receiving binding from UdpPortBinding.
use crate::returncode::ReturnCode;
use core::fmt;
use tock_cells::optional_cell::OptionalCell;
use tock_cells::take_cell::TakeCell;

//#![allow(dead_code)]
const MAX_NUM_BOUND_PORTS: usize = 16;

/// The PortEntry struct is stored in the table and conveys what port is bound
/// at the given index if one is bound. If no port is bound, the value stored
/// at location is Unbound.
#[derive(Clone, Copy, PartialEq)]
pub enum PortEntry {
    Port(u16),
    Unbound,
}

// We need Option<PortEntry> to distinguish between the case in which we have
// a UdpPortSocket that is not bound to a port and an index where there is no
// UdpPortSocket allocated.
static mut PORT_TABLE: [Option<PortEntry>; MAX_NUM_BOUND_PORTS] = [None; MAX_NUM_BOUND_PORTS];

/// The PortQuery trait enables the UdpPortTable to query the userspace bound
/// ports in the UDP driver. The UDP driver struct implements this trait.
pub trait PortQuery {
    fn is_bound(&self, port: u16) -> bool;
}

/// A UdpPortSocket provides a handle into the bound port table. When binding to
/// a port, the socket is consumed and stored inside a UdpPortBinding. When
/// undbinding, the socket is returned and can be used to bind to other ports.
#[derive(Debug)]
pub struct UdpPortSocket {
    idx: usize,
    port_table: &'static UdpPortTable,
}

/// The UdpPortTable maintains a reference the port_array, which manages what
/// ports are bound at any given moment, and user_ports, which provides a
/// handle to userspace port bindings in the UDP driver.
pub struct UdpPortTable {
    port_array: TakeCell<'static, [Option<PortEntry>]>,
    user_ports: OptionalCell<&'static dyn PortQuery>,
}

impl fmt::Debug for UdpPortTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Port Table]")
    }
}

impl UdpPortSocket {
    // important that this function is not public. If it were, capsules could
    // obtain access to ports bound by other capsules
    fn new(idx: usize, pt: &'static UdpPortTable) -> UdpPortSocket {
        UdpPortSocket {
            idx: idx,
            port_table: pt,
        }
    }
}

impl Drop for UdpPortSocket {
    fn drop(&mut self) {
        self.port_table.destroy_socket(self);
    }
}

/// An opaque descriptor that allows the holder to obtain a binding on a port
/// for receiving UDP packets.
#[derive(Debug)]
pub struct UdpReceiverBinding {
    idx: usize,
    port: u16,
}

/// An opaque descriptor that allows the holder to obtain a binding on a port
/// for sending UDP packets.
#[derive(Debug)]
pub struct UdpSenderBinding {
    idx: usize,
    port: u16,
}

impl UdpSenderBinding {
    fn new(idx: usize, port: u16) -> UdpSenderBinding {
        UdpSenderBinding {
            idx: idx,
            port: port,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl UdpReceiverBinding {
    fn new(idx: usize, port: u16) -> UdpReceiverBinding {
        UdpReceiverBinding {
            idx: idx,
            port: port,
        }
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
            port_array: TakeCell::new(&mut PORT_TABLE),
            user_ports: OptionalCell::empty(),
        }
    }

    pub unsafe fn set_user_ports(&self, user_ports_ref: &'static dyn PortQuery) {
        self.user_ports.replace(user_ports_ref);
    }

    pub fn create_socket(&'static self) -> Result<UdpPortSocket, ReturnCode> {
        self.port_array
            .map(|table| {
                let mut result: Result<UdpPortSocket, ReturnCode> = Err(ReturnCode::FAIL);
                for i in 0..MAX_NUM_BOUND_PORTS {
                    match table[i] {
                        None => {
                            result = Ok(UdpPortSocket::new(i, &self));
                            table[i] = Some(PortEntry::Unbound);
                            break;
                        }
                        _ => (),
                    }
                }
                result
            })
            .expect("failed to create socket")
    }

    pub fn destroy_socket(&self, socket: &mut UdpPortSocket) {
        self.port_array.map(|table| {
            // only free slot if it is unbound! Current design means that drop is also called
            // when port table consumes socket on call to bind(), but dont want to drop the
            // bindings. Alternate approach is to have port table store the sockets in the port
            // table itself rather than consuming sockets on bind() and creating them on unbind(),
            // but this approach would bloat the size of the table
            match table[socket.idx] {
                Some(entry) => {
                    if entry == PortEntry::Unbound {
                        table[socket.idx] = None;
                    }
                }
                _ => {}
            }
        });
    }

    pub fn is_bound(&self, port: u16) -> Result<bool, ()> {
        // First, check the user bindings.
        if self.user_ports.is_none() {
            return Err(());
        }
        let user_bound = self
            .user_ports
            .map_or(true, |port_query| port_query.is_bound(port));
        if self.user_ports.is_none() {}
        if user_bound {
            return Ok(true);
        };
        let ret = self
            .port_array
            .map(|table| {
                let mut port_exists = false;
                for i in 0..MAX_NUM_BOUND_PORTS {
                    match table[i] {
                        Some(PortEntry::Port(p)) => {
                            if p == port {
                                port_exists = true;
                                break;
                            }
                        }
                        _ => (),
                    }
                }
                port_exists
            })
            .unwrap();
        Ok(ret)
    }

    // On success, a UdpPortBinding is returned. On failure, the same
    // UdpPortSocket is returned.
    pub fn bind(
        &self,
        socket: UdpPortSocket,
        port: u16,
    ) -> Result<(UdpSenderBinding, UdpReceiverBinding), UdpPortSocket> {
        match self.is_bound(port) {
            Ok(bound) => {
                if bound {
                    Err(socket)
                } else {
                    self.port_array
                        .map(|table| {
                            table[socket.idx] = Some(PortEntry::Port(port));
                            let binding_pair = (
                                UdpSenderBinding::new(socket.idx, port),
                                UdpReceiverBinding::new(socket.idx, port),
                            );
                            // Add socket to the linked list.
                            Ok(binding_pair)
                        })
                        .unwrap()
                }
            }
            Err(_) => Err(socket),
        }
    }

    /// Disassociate the port from the given binding. Return the socket that was
    /// contained within the binding object.
    pub fn unbind(
        &'static self,
        sender_binding: UdpSenderBinding,
        receiver_binding: UdpReceiverBinding,
    ) -> Result<UdpPortSocket, (UdpSenderBinding, UdpReceiverBinding)> {
        // Verfify that the indices match up
        if sender_binding.idx != receiver_binding.idx {
            return Err((sender_binding, receiver_binding));
        }
        let idx = sender_binding.idx;
        self.port_array.map(|table| {
            table[idx] = Some(PortEntry::Unbound);
        });
        // Search the list and return the appropriate socket
        Ok(UdpPortSocket::new(idx, &self))
    }
}
