//! In-kernel structure for tracking UDP ports bound by capsules.
//!
//! When kernel capsules wish to send or receive UDP packets, the UDP sending / receiving
//! capsules will only allow this if the capsule has bound to the port it wishes to
//! send from / receive on. Binding to a port is accompished via calls on the
//! `UdpPortManager` struct defined in this file. Calls to bind on this table enforce that only
//! one capsule can be bound to a given port at any time. Once capsules succesfully bind
//! using this table, they receive back binding structures (`UdpPortBindingTx`/`UdpPortBindingRx`)
//! that act as proof that the holder
//! is bound to that port. These structures can only be created within this file, and calls
//! to unbind must consume these structures, enforcing this invariant.
//! The UDP tx/rx capsules require these bindings be passed in order to send/receive on a given
//! port. Seperate bindings are used for sending and receiving because the UdpReceiver must
//! hold onto the binding for as long as a capsule wishes to receive packets on a port, so
//! a seperate binding must be available to enable sending packets on a port while
//! listening on the same port.
//!
//! To reduce the size of data structures required for this task, a fixed size
//! array is used to store bindings in the kernel. This means that a limited
//! number of bindings can be stored at any point in time. Reserving a slot
//! in this table is done by requesting a socket, which represents a reserved slot.
//! These sockets are then used to request bindings on a particular port.
//!
//! This file only stores information about which ports are bound by capsules.
//! The files `udp_send.rs` and `udp_recv.rs` enforce that only capsules possessing
//! the correct bindings can actually send / recv on a given port.
//!
//! Userspace port bindings are managed seperately by the userspace UDP driver
//! (`capsules/src/net/udp/driver.rs`), because apps can be dynamically added or
//! removed. Bindings for userspace apps are stored in the grant regions of each app,
//! such that removing an app automatically unbinds it. This file is able to query the
//! userspace UDP driver to check which ports are bound, and vice-versa, such that
//! exclusive access to ports between userspace apps and capsules is still enforced.

use crate::net::network_capabilities::{NetworkCapability, UdpVisibilityCapability};
use core::fmt;
use kernel::capabilities::{CreatePortTableCapability, UdpDriverCapability};
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::ReturnCode;

// Sets the maximum number of UDP ports that can be bound by capsules. Reducing this number
// can save a small amount of memory, and slightly reduces the overhead of iterating through the
// table to check whether a port is already bound.
pub const MAX_NUM_BOUND_PORTS: usize = 5;

/// The SocketBindingEntry struct is stored in the PORT_TABLE and conveys what port is bound
/// at the given index if one is bound. If no port is bound, the value stored
/// at that location in the table is Unbound.
#[derive(Clone, Copy, PartialEq)]
pub enum SocketBindingEntry {
    Port(u16),
    Unbound,
}

/// The PortQuery trait enables the UdpPortManager to query the userspace bound
/// ports in the UDP driver. The UDP driver struct implements this trait.
pub trait PortQuery {
    fn is_bound(&self, port: u16) -> bool;
}

/// A UdpSocket provides a handle into the bound port table. When binding to
/// a port, the socket is consumed and Udp{Sender, Receiver}Binding structs are returned. When
/// undbinding, the socket is returned and can be used to bind to other ports.
#[derive(Debug)]
pub struct UdpSocket {
    idx: usize,
    port_table: &'static UdpPortManager,
}

/// The UdpPortManager maintains a reference to the port_array, which manages what
/// ports are bound at any given moment, and user_ports, which provides a
/// handle to userspace port bindings in the UDP driver.
pub struct UdpPortManager {
    port_array: TakeCell<'static, [Option<SocketBindingEntry>]>,
    user_ports: OptionalCell<&'static dyn PortQuery>,
    udp_vis: &'static UdpVisibilityCapability,
}

impl fmt::Debug for UdpPortManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Port Table]")
    }
}

impl UdpSocket {
    // important that this function is not public. If it were, capsules could
    // obtain access to ports bound by other capsules
    fn new(idx: usize, pt: &'static UdpPortManager) -> UdpSocket {
        UdpSocket {
            idx: idx,
            port_table: pt,
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.port_table.destroy_socket(self);
    }
}

/// An opaque descriptor that allows the holder to obtain a binding on a port
/// for receiving UDP packets.
#[derive(Debug)]
pub struct UdpPortBindingRx {
    idx: usize,
    port: u16,
}

/// An opaque descriptor that allows the holder to obtain a binding on a port
/// for sending UDP packets.
#[derive(Debug)]
pub struct UdpPortBindingTx {
    idx: usize,
    port: u16,
}

impl UdpPortBindingTx {
    fn new(idx: usize, port: u16) -> UdpPortBindingTx {
        UdpPortBindingTx {
            idx: idx,
            port: port,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl UdpPortBindingRx {
    fn new(idx: usize, port: u16) -> UdpPortBindingRx {
        UdpPortBindingRx {
            idx: idx,
            port: port,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl UdpPortManager {
    // Require capability so that the port table is only created by kernel
    pub fn new(
        _cap: &dyn CreatePortTableCapability,
        used_kernel_ports: &'static mut [Option<SocketBindingEntry>],
        udp_vis: &'static UdpVisibilityCapability,
    ) -> UdpPortManager {
        UdpPortManager {
            port_array: TakeCell::new(used_kernel_ports),
            user_ports: OptionalCell::empty(),
            udp_vis: udp_vis,
        }
    }

    // This function is called to set a reference to the UDP driver, so that the ports
    // bound by applications can be queried from within this file.
    pub fn set_user_ports(
        &self,
        user_ports_ref: &'static dyn PortQuery,
        _driver_cap: &dyn UdpDriverCapability,
    ) {
        self.user_ports.replace(user_ports_ref);
    }

    /// Called by capsules that would like to eventually be able to bind to a
    /// UDP port. This call will succeed unless MAX_NUM_BOUND_PORTS capsules
    /// have already bound to a port.
    pub fn create_socket(&'static self) -> Result<UdpSocket, ReturnCode> {
        self.port_array
            .map_or(Err(ReturnCode::ENOSUPPORT), |table| {
                let mut result: Result<UdpSocket, ReturnCode> = Err(ReturnCode::FAIL);
                for i in 0..MAX_NUM_BOUND_PORTS {
                    match table[i] {
                        None => {
                            result = Ok(UdpSocket::new(i, &self));
                            table[i] = Some(SocketBindingEntry::Unbound);
                            break;
                        }
                        _ => (),
                    }
                }
                result
            })
    }

    /// Called when sockets are dropped to free their slots in the table.
    /// The slot in the table is only freed if the socket that is dropped is
    /// unbound. If the slot is bound, the socket is being dropped after a call to
    /// bind(), and the slot in the table should remain reserved.
    fn destroy_socket(&self, socket: &mut UdpSocket) {
        self.port_array.map(|table| match table[socket.idx] {
            Some(entry) => {
                if entry == SocketBindingEntry::Unbound {
                    table[socket.idx] = None;
                }
            }
            _ => {}
        });
    }

    /// Check if a given port is already bound, by either an app or capsule.
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
                        Some(SocketBindingEntry::Port(p)) => {
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

    /// Called by capsules that have already reserved a socket to attempt to bind to
    /// a UDP port. The socket is passed by value.
    /// On success, bindings is returned. On failure, the same
    /// UdpSocket is returned.
    pub fn bind(
        &self,
        socket: UdpSocket,
        port: u16,
        net_cap: &'static NetworkCapability,
    ) -> Result<(UdpPortBindingTx, UdpPortBindingRx), UdpSocket> {
        if net_cap.local_port_valid(port, self.udp_vis) {
            match self.is_bound(port) {
                Ok(bound) => {
                    if bound {
                        Err(socket)
                    } else {
                        self.port_array
                            .map(|table| {
                                table[socket.idx] = Some(SocketBindingEntry::Port(port));
                                let binding_pair = (
                                    UdpPortBindingTx::new(socket.idx, port),
                                    UdpPortBindingRx::new(socket.idx, port),
                                );
                                // Add socket to the linked list.
                                Ok(binding_pair)
                            })
                            .unwrap()
                    }
                }
                Err(_) => Err(socket),
            }
        } else {
            Err(socket)
        }
    }

    /// Disassociate the port from the given binding. Return the socket associated
    /// with the passed bindings. On Err, return the passed bindings.
    pub fn unbind(
        &'static self,
        sender_binding: UdpPortBindingTx,
        receiver_binding: UdpPortBindingRx,
    ) -> Result<UdpSocket, (UdpPortBindingTx, UdpPortBindingRx)> {
        // Verfify that the indices match up
        if sender_binding.idx != receiver_binding.idx {
            return Err((sender_binding, receiver_binding));
        }
        let idx = sender_binding.idx;
        self.port_array.map(|table| {
            table[idx] = Some(SocketBindingEntry::Unbound);
        });
        // Search the list and return the appropriate socket
        Ok(UdpSocket::new(idx, &self))
    }
}
