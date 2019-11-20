//! This file contains the definition and implementation for the UDP reception
//! interface. It follows the same virtualization model as that described in `udp_send.rs`,
//! except that no queueing is needed because received packets are immediately dispatched to the
//! appropriate capsule / app. Once again, port binding for userspace apps is managed seperately
//! by the UDP userspace driver, which must correctly check bindings of kernel apps to ensure
//! correctness when dispatching received packets to the appropriate client.

use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6::IP6Header;
use crate::net::ipv6::ipv6_recv::IP6RecvClient;
use crate::net::udp::driver::UDPDriver;
use crate::net::udp::udp::UDPHeader;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::udp_port_table::{PortQuery, UdpReceiverBinding};

pub struct MuxUdpReceiver<'a> {
    rcvr_list: List<'a, UDPReceiver<'a>>,
    driver: OptionalCell<&'static UDPDriver<'static>>,
}

impl<'a> MuxUdpReceiver<'a> {
    pub fn new() -> MuxUdpReceiver<'a> {
        MuxUdpReceiver {
            rcvr_list: List::new(),
            driver: OptionalCell::empty(),
        }
    }

    pub fn add_client(&self, rcvr: &'a UDPReceiver<'a>) {
        self.rcvr_list.push_tail(rcvr);
    }

    pub fn set_driver(&self, driver_ref: &'static UDPDriver) {
        self.driver.replace(driver_ref);
    }
}

impl<'a> IP6RecvClient for MuxUdpReceiver<'a> {
    fn receive(&self, ip_header: IP6Header, payload: &[u8]) {
        match UDPHeader::decode(payload).done() {
            Some((offset, udp_header)) => {
                let len = udp_header.get_len() as usize;
                let dst_port = udp_header.get_dst_port();
                if len > payload.len() {
                    debug!("[UDP_RECV] Error: Received UDP length too long");
                    return;
                }
                for rcvr in self.rcvr_list.iter() {
                    match rcvr.binding.take() {
                        Some(binding) => {
                            if binding.get_port() == dst_port {
                                rcvr.client.map(|client| {
                                    client.receive(
                                        ip_header.get_src_addr(),
                                        ip_header.get_dst_addr(),
                                        udp_header.get_src_port(),
                                        udp_header.get_dst_port(),
                                        &payload[offset..],
                                    );
                                });
                                rcvr.binding.replace(binding);
                                break;
                            }
                            rcvr.binding.replace(binding);
                        }
                        // The UDPReceiver used by the driver will not have a binding
                        None => match self.driver.take() {
                            Some(driver) => {
                                if driver.is_bound(dst_port) {
                                    driver.receive(
                                        ip_header.get_src_addr(),
                                        ip_header.get_dst_addr(),
                                        udp_header.get_src_port(),
                                        udp_header.get_dst_port(),
                                        &payload[offset..],
                                    );
                                    self.driver.replace(driver);
                                    break;
                                }
                                self.driver.replace(driver);
                            }
                            None => {}
                        },
                    }
                }
            }
            None => {}
        }
    }
}

/// The UDP driver implements this client interface trait to receive
/// packets passed up the network stack to the UDPReceiver, and then
/// distributes them to userland applications from there.
/// Kernel apps can also instantiate structs that implement this trait
/// in order to receive UDP packets
pub trait UDPRecvClient {
    fn receive(
        &self,
        src_addr: IPAddr,
        dst_addr: IPAddr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    );
}

/// This struct is set as the client of the MuxUdpReceiver, and passes
/// received packets up to whatever app layer client assigns itself
/// as the UDPRecvClient held by this UDPReciever.
pub struct UDPReceiver<'a> {
    client: OptionalCell<&'a dyn UDPRecvClient>,
    binding: MapCell<UdpReceiverBinding>,
    next: ListLink<'a, UDPReceiver<'a>>,
}

impl<'a> ListNode<'a, UDPReceiver<'a>> for UDPReceiver<'a> {
    fn next(&'a self) -> &'a ListLink<'a, UDPReceiver<'a>> {
        &self.next
    }
}

impl<'a> UDPReceiver<'a> {
    pub fn new() -> UDPReceiver<'a> {
        UDPReceiver {
            client: OptionalCell::empty(),
            binding: MapCell::empty(),
            next: ListLink::empty(),
        }
    }

    pub fn set_client(&self, client: &'a dyn UDPRecvClient) {
        self.client.set(client);
    }

    pub fn get_binding(&self) -> Option<UdpReceiverBinding> {
        self.binding.take()
    }

    fn is_bound(&self) -> bool {
        self.binding.is_some()
    }

    pub fn set_binding(&self, binding: UdpReceiverBinding) -> Option<UdpReceiverBinding> {
        self.binding.replace(binding)
    }
}
