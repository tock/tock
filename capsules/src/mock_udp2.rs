// Capsule used for testing in-kernel port binding using the PortTable interface
// Author: Armin + Hudson

use kernel::{debug, ReturnCode};
use kernel::hil::time::{self, Alarm, Frequency};
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::ipv6::ipv6_send::{IP6SendStruct, IP6Sender};
use crate::net::udp::udp::UDPHeader;
use crate::net::udp::udp_send::{UDPSendClient, UDPSender, UDPSendStruct};
use kernel::common::cells::TakeCell;
use kernel::udp_port_table::{UdpPortTable, UdpPortSocket, UdpSenderBinding};

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

// TODO: can we re-use code in mock_udp.rs here?
pub struct MockUdp2<'a, A: Alarm + 'a> {
    id: u16,
    alarm: &'a A,
    udp_sender: &'a UDPSender<'a>,
    port_table: &'static UdpPortTable,
    // socket: &'a TakeCell<UdpPortSocket>,
    // binding: &'a TakeCell<UdpSenderBinding>,
}

impl<'a, A: Alarm> MockUdp2<'a, A> {
    pub fn new(id: u16,
               alarm: &'a A,
               udp_sender:&'a UDPSender<'a>,
               port_table: &'static UdpPortTable)
            -> MockUdp2<'a, A> {
        MockUdp2 {
            id: id,
            alarm: alarm,
            udp_sender: udp_sender,
            port_table: port_table,
            // socket: TakeCell::empty(),
            // binding: TakeCell::empty(),
        }
    }

    pub fn start(&self) {
        debug!("Start called in mock_udp2");
        self.alarm.set_alarm(self.alarm.now().
                             wrapping_add(<A::Frequency>::frequency()));
        let socket = self.port_table.create_socket();
        if socket.is_ok() {
            debug!("Socket successfully created in mock_udp2");
        } else {
            debug!("Socket error in mock_udp2");
            return;
        }
        let socket = socket.ok().unwrap();
        let binding = self.port_table.bind(socket, 80);
        if binding.is_ok() {
            debug!("Binding successfully created in mock_udp2");
        } else {
            debug!("Binding error in mock_udp2");
        }

        // self.socket.replace(socket);
        // self.binding.replace(binding);
    }

    pub fn send(&self, value: u16) {
        // Performs little-endian conversion.

        // UDP_DGRAM[0] = (value >> 8) as u8;
        // UDP_DGRAM[1] = (value & 0x00ff) as u8;
        //debug!("in send in mock");
        let tmp = self.udp_sender
            .send_to(DST_ADDR, DST_PORT, SRC_PORT, &UDP_DGRAM);
    }
}

impl<'a, A: Alarm> time::Client for MockUdp2<'a, A> {
    fn fired(&self) {
        //debug!("timer fired....");
        // self.send(17);
        debug!("2: timer fired....");
        // self.start();
    }
}

impl<'a, A: Alarm> UDPSendClient for MockUdp2<'a, A> {
    fn send_done(&self, result: ReturnCode) {
        debug!("Done sending. Result: {:?}", result);
    }
}
