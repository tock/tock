//! This file contains the definition and implementation for the UDP reception
//! interface.

use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6::IP6Header;
use crate::net::ipv6::ipv6_recv::IP6RecvClient;
use crate::net::udp::udp::UDPHeader;
use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::udp_port_table::UdpPortTable;

pub struct MuxUdpReceiver<'a> {
    rcvr_list: List<'a, UDPReceiver<'a>>,
}

impl<'a> MuxUdpReceiver<'a> {
    pub fn new() -> MuxUdpReceiver<'a> {
        MuxUdpReceiver {
            rcvr_list: List::new(),
        }
    }

    pub fn add_client(&self, rcvr: &'a UDPReceiver<'a>) {
        self.rcvr_list.push_tail(rcvr);
    }
}

impl<'a> IP6RecvClient for MuxUdpReceiver<'a> {
    fn receive(&self, ip_header: IP6Header, payload: &[u8]) {
        // TODO: add call to port_table.can_recv here
        // TODO: change from ret code to bool.
        match UDPHeader::decode(payload).done() {
            Some((offset, udp_header)) => {
                let len = udp_header.get_len() as usize;
                if len > payload.len() {
                    debug!("[UDP_RECV] Error: Received UDP length too long");
                    return;
                }
                for rcvr in rcvr_list.iter() {
                    match rcvr.binding.take() {
                        Some(binding) => {
                            if binding.get_port() == udp_header.get_dst_port() {
                                rcvr.map(|client| {
                                    client.receive(
                                        ip_header.get_src_addr(),
                                        ip_header.get_dst_addr(),
                                        udp_header.get_src_port(),
                                        udp_header.get_dst_port(),
                                        &payload[offset..],
                                    );
                                });
                                break;
                            }
                        }
                        None => {}
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
    client: OptionalCell<&'a UDPRecvClient>,
    binding: MapCell<UdpPortBinding>,
}

impl<'a> UDPReceiver<'a> {
    pub fn new() -> UDPReceiver<'a> {
        UDPReceiver {
            client: OptionalCell::empty(),
            binding: MapCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'a UDPRecvClient) {
        self.client.set(client);
    }
}
