// Capsule used for testing in-kernel port binding using the PortTable interface
// Author: Armin + Hudson

use crate::net::buffer::Buffer;
use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::udp::udp_recv::{UDPReceiver, UDPRecvClient};
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};
use core::cell::Cell;
use kernel::common::cells::MapCell;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::udp_port_table::UdpPortTable;
use kernel::{debug, ReturnCode};

pub const DST_ADDR: IPAddr = IPAddr([
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
    0x1f,
    // 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
    // 0x0f,
]);
pub const SRC_PORT: u16 = 15123;
pub const DST_PORT: u16 = 81;
pub const PAYLOAD_LEN: usize = 192;

pub struct MockUdp1<'a, A: Alarm + 'a> {
    id: u16,
    pub alarm: A,
    udp_sender: &'a UDPSender<'a>,
    udp_receiver: &'a UDPReceiver<'a>,
    port_table: &'static UdpPortTable,
    first: Cell<bool>,
    udp_dgram: MapCell<Buffer<'static, u8>>,
}

impl<'a, A: Alarm> MockUdp1<'a, A> {
    pub fn new(
        id: u16,
        alarm: A,
        udp_sender: &'a UDPSender<'a>,
        udp_receiver: &'a UDPReceiver<'a>,
        port_table: &'static UdpPortTable,
        udp_dgram: Buffer<'static, u8>,
    ) -> MockUdp1<'a, A> {
        MockUdp1 {
            id: id,
            alarm: alarm,
            udp_sender: udp_sender,
            udp_receiver: udp_receiver,
            port_table: port_table,
            first: Cell::new(true),
            udp_dgram: MapCell::new(udp_dgram),
        }
    }

    pub fn start(&self) {
        // Set alarm bc if you try to send immediately there are initialization issues
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() * 10),
        );
    }

    pub fn send(&self, value: u16) {
        match self.udp_dgram.take() {
            Some(mut dgram) => {
                dgram[0] = (value >> 8) as u8;
                dgram[1] = (value & 0x00ff) as u8;
                dgram.slice(0..2);
                match self.udp_sender.send_to(DST_ADDR, DST_PORT, dgram) {
                    ReturnCode::SUCCESS => {}
                    _ => debug!("Mock UDP Send Failed."),
                }
            }
            None => debug!("udp_dgram not present."),
        }
    }
}

impl<'a, A: Alarm> time::Client for MockUdp1<'a, A> {
    fn fired(&self) {
        if self.first.get() {
            self.first.set(false);
            let socket = self.port_table.create_socket();
            match socket {
                Ok(sock) => {
                    debug!("Socket successfully created in mock_udp");
                    match self.port_table.bind(sock, 81) {
                        Ok((send_bind, rcv_bind)) => {
                            debug!("Binding successfully created in mock_udp");
                            self.udp_sender.set_binding(send_bind);
                            self.udp_receiver.set_binding(rcv_bind);
                        }
                        Err(sock) => {
                            debug!("Binding error in mock_udp");
                            self.port_table.destroy_socket(sock);
                        }
                    }
                }
                Err(_return_code) => {
                    debug!("Socket error in mock_udp");
                    return;
                }
            }
        }
        self.send(self.id);
    }
}

impl<'a, A: Alarm> UDPSendClient for MockUdp1<'a, A> {
    fn send_done(&self, result: ReturnCode, mut dgram: Buffer<'static, u8>) {
        debug!("Mock UDP done sending. Result: {:?}", result);
        dgram.reset();
        self.udp_dgram.replace(dgram);
        debug!("");
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() * 5),
        );
    }
}

impl<'a, A: Alarm> UDPRecvClient for MockUdp1<'a, A> {
    fn receive(
        &self,
        src_addr: IPAddr,
        _dst_addr: IPAddr,
        src_port: u16,
        _dst_port: u16,
        payload: &[u8],
    ) {
        debug!(
            "[MOCK_UDP] Received packet from {:?}:{:?}, contents: {:?}",
            src_addr, src_port, payload
        );
    }
}
