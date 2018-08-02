use core::cell::Cell;
use net::ipv6::ip_utils::IPAddr;
use net::ipv6::ipv6::IP6Header;
use net::ipv6::ipv6_recv::IP6RecvClient;
use net::udp::udp::UDPHeader;

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

/// This struct is set as the client of an IP6Receiver, and passes
/// received packets up to whatever app layer client assigns itself
/// as the UDPRecvClient held by this UDPReciever.
pub struct UDPReceiver<'a> {
    client: Cell<Option<&'a UDPRecvClient>>,
}

impl<'a> UDPReceiver<'a> {
    pub fn new() -> UDPReceiver<'a> {
        UDPReceiver {
            client: Cell::new(None),
        }
    }

    pub fn set_client(&self, client: &'a UDPRecvClient) {
        self.client.set(Some(client));
    }
}

impl<'a> IP6RecvClient for UDPReceiver<'a> {
    fn receive(&self, ip_header: IP6Header, payload: &[u8]) {
        debug!("[UDP_RecvClient] received something");
        match UDPHeader::decode(payload).done() {
            Some((offset, udp_header)) => {
                let len = udp_header.get_len() as usize;
                if len > payload.len() {
                    // TODO: ERROR
                    return;
                }
                self.client.get().map(|client| {
                    client.receive(
                        ip_header.get_src_addr(),
                        ip_header.get_dst_addr(),
                        udp_header.get_src_port(),
                        udp_header.get_dst_port(),
                        &payload[offset..],
                    );
                });
            }
            None => {}
        }
    }
}
