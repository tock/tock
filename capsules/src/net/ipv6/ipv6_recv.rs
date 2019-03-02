use crate::net::ipv6::ipv6::IP6Header;
use crate::net::sixlowpan::sixlowpan_state::SixlowpanRxClient;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::ReturnCode;

// To provide some context for the entire rx chain:
/*
- The radio in the kernel has a single `RxClient`, which is set as the mac layer
  (awake_mac, typically)
- The mac layer (i.e. `AwakeMac`) has a single `RxClient`, which is the
  mac_device(`ieee802154::Framer::framer`)
- The Mac device has a single receive client - `MuxMac` (virtual MAC device).
- The `MuxMac` can have multiple "users" which are of type `MacUser`
- Any received packet is passed to ALL MacUsers, which are expected to filter
  packets themselves accordingly.
- Right now, we initialize two MacUsers in the kernel (in main.rs/components).
  These are the 'radio_mac', which is the MacUser for the RadioDriver that
  enables the userland interface to directly send 802154 frames, and udp_mac,
  the mac layer that is ultimately associated with the udp userland interface.
- The udp_mac MacUser has a single receive client, which is the `sixlowpan_state` struct
- `sixlowpan_state` has a single rx_client, which in our case is a single struct that
  implements the `ip_receive ` trait.
- the `ip_receive` implementing struct (`IP6RecvStruct`) has a single client, which is
  udp_recv, a `UDPReceive` struct.
- The UDPReceive struct is a field of the UDPDriver, which ultimately passes the
  packets up to userland.
*/

pub trait IP6RecvClient {
    // TODO: What should the upper layers receive?
    fn receive(&self, header: IP6Header, payload: &[u8]);
}

/// Currently only one implemetation of this trait should exist,
/// as we do not multiplex received packets based on the address.
/// The receiver receives IP packets destined for any local address.
/// The receiver should drop any packets with destination addresses
/// that are not among the local addresses of this device.
pub trait IP6Receiver<'a> {
    fn set_client(&self, client: &'a IP6RecvClient);
}

pub struct IP6RecvStruct<'a> {
    client: OptionalCell<&'a IP6RecvClient>,
}

impl<'a> IP6Receiver<'a> for IP6RecvStruct<'a> {
    fn set_client(&self, client: &'a IP6RecvClient) {
        self.client.set(client);
    }
}

impl<'a> IP6RecvStruct<'a> {
    pub fn new() -> IP6RecvStruct<'a> {
        IP6RecvStruct {
            client: OptionalCell::empty(),
        }
    }
}

impl<'a> SixlowpanRxClient for IP6RecvStruct<'a> {
    fn receive(&self, buf: &[u8], len: usize, result: ReturnCode) {
        // TODO: Drop here?
        if len > buf.len() || result != ReturnCode::SUCCESS {
            return;
        }
        match IP6Header::decode(buf).done() {
            Some((offset, ip6_header)) => {
                let checksum_result = ip6_header.check_transport_checksum(&buf[offset..len]);
                if checksum_result == ReturnCode::FAIL {
                    debug!("dropped!: {:?}", checksum_result);
                    return; //Dropped.
                }
                // Note: Protocols for which checksum verification is not implemented (TCP, etc.)
                // are automatically assumed as fine, rather than dropped

                self.client
                    .map(|client| client.receive(ip6_header, &buf[offset..len]));
            }
            None => {
                // TODO: Report the error somewhere...
            }
        }
    }
}
