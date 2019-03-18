// Capsule used for testing in-kernel port binding using the PortTable interface
// Author: Armin + Hudson

use kernel::{debug, ReturnCode};
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::udp::udp::UDPHeader;
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};

pub const DST_ADDR: IPAddr =     IPAddr([
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f,
        // 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        // 0x0f,
]);
pub const SRC_PORT: u16 = 15123;
pub const DST_PORT: u16 = 16123;
pub const PAYLOAD_LEN: usize = 192;
const UDP_HDR_SIZE: usize = 8;
static UDP_DGRAM: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];


pub struct MockUdp1<'a> {
    id: u16,
    udp_sender: &'a UDPSender<'a>,
}

impl<'a> MockUdp1<'a> {
    pub fn new(id: u16,
               udp_sender: &'a UDPSender<'a>) -> MockUdp1<'a> {
        MockUdp1 {
            id: id,
            udp_sender: udp_sender,
        }
    }

    pub fn send(&self, value: u16) {
        // Performs little-endian conversion.

        // UDP_DGRAM[0] = (value >> 8) as u8;
        // UDP_DGRAM[1] = (value & 0x00ff) as u8;
        let tmp = self.udp_sender
            .send_to(DST_ADDR, DST_PORT, SRC_PORT, &UDP_DGRAM);
        debug!("mock_udp1 retval: {:?}", tmp);

    }
}

impl<'a> UDPSendClient for MockUdp1<'a> {
    fn send_done(&self, result: ReturnCode) {
        debug!("Done sending. Result: {:?}", result);
    }
}
