//! Capsule used for testing in-kernel port binding, sending, and receiving.
//!
//! This capsule takes in a src port on which to receive/send from and a dst port to send to.
//! It binds to the src port and sends packets to the dst port. Any UDP packets received on the
//! src port are printed to the console, along with the address/port combo they were sent from.
//! Example use of this capsule can be found in `udp_lowpan_test.rs` in the Imix board directory.

use crate::net::ipv6::ip_utils::IPAddr;
use crate::net::udp::udp_port_table::UdpPortManager;
use crate::net::udp::udp_recv::{UDPReceiver, UDPRecvClient};
use crate::net::udp::udp_send::{UDPSendClient, UDPSender};
use core::cell::Cell;
use kernel::common::cells::MapCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::hil::time::{Alarm, AlarmClient, Frequency, Ticks};
use kernel::{debug, ReturnCode};

pub const DST_ADDR: IPAddr = IPAddr([
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
    0x1f,
    // 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
    // 0x0f,
]);
pub const PAYLOAD_LEN: usize = 192;
pub const SEND_INTERVAL_SECONDS: u32 = 5;

pub struct MockUdp<'a, A: Alarm<'a>> {
    id: u16,
    pub alarm: &'a A,
    udp_sender: &'a dyn UDPSender<'a>,
    udp_receiver: &'a UDPReceiver<'a>,
    port_table: &'static UdpPortManager,
    udp_dgram: MapCell<LeasableBuffer<'static, u8>>,
    src_port: Cell<u16>,
    dst_port: Cell<u16>,
    send_loop: Cell<bool>,
}

impl<'a, A: Alarm<'a>> MockUdp<'a, A> {
    pub fn new(
        id: u16,
        alarm: &'a A,
        udp_sender: &'a dyn UDPSender<'a>,
        udp_receiver: &'a UDPReceiver<'a>,
        port_table: &'static UdpPortManager,
        udp_dgram: LeasableBuffer<'static, u8>,
        dst_port: u16,
    ) -> MockUdp<'a, A> {
        MockUdp {
            id: id,
            alarm: alarm,
            udp_sender: udp_sender,
            udp_receiver: udp_receiver,
            port_table: port_table,
            udp_dgram: MapCell::new(udp_dgram),
            src_port: Cell::new(0), // invalid initial value
            dst_port: Cell::new(dst_port),
            send_loop: Cell::new(false),
        }
    }

    // starts sending packets every 5 seconds.
    pub fn start_sending(&self) {
        // Set alarm bc if you try to send immediately there are initialization issues
        self.send_loop.set(true);
        self.alarm
            .set_alarm_from_now(A::ticks_from_seconds(SEND_INTERVAL_SECONDS));
    }

    pub fn stop_sending(&self) {
        self.alarm.disable();
    }

    // Binds to passed port. If already bound to a port,
    // unbinds currently bound to port and binds to passed port.
    pub fn bind(&self, src_port: u16) {
        self.src_port.set(src_port);
        if self.udp_sender.is_bound() != self.udp_receiver.is_bound() {
            debug!("Error: bindings should match.");
        }
        match self.udp_sender.is_bound() {
            true => {
                match self.port_table.unbind(
                    self.udp_sender.get_binding().expect("missing1"),
                    self.udp_receiver.get_binding().expect("missing2"),
                ) {
                    Ok(sock) => match self.port_table.bind(sock, self.src_port.get()) {
                        Ok((send_bind, rcv_bind)) => {
                            self.udp_sender.set_binding(send_bind);
                            self.udp_receiver.set_binding(rcv_bind);
                        }
                        Err(_sock) => {
                            debug!("Binding error in mock_udp");
                            // dropping sock destroys it!
                        }
                    },
                    Err((_send_bind, _rcv_bind)) => {
                        debug!("TEST FAIL: attempted to unbind with mismatched bindings.");
                    }
                }
            }
            false => {
                // Bind for the first time.
                let socket = self.port_table.create_socket();
                match socket {
                    Ok(sock) => {
                        match self.port_table.bind(sock, self.src_port.get()) {
                            Ok((send_bind, rcv_bind)) => {
                                self.udp_sender.set_binding(send_bind);
                                self.udp_receiver.set_binding(rcv_bind);
                            }
                            Err(_sock) => {
                                debug!("Binding error in mock_udp (passed 0 as src_port?)");
                                // dropping sock destroys it!
                            }
                        }
                    }
                    Err(_return_code) => {
                        debug!("Socket error in mock_udp");
                        return;
                    }
                }
            }
        }
    }

    pub fn set_dst(&self, dst_port: u16) {
        self.dst_port.set(dst_port);
    }

    // Sends a packet containing a single 2 byte number.
    pub fn send(&self, value: u16) -> ReturnCode {
        match self.udp_dgram.take() {
            Some(mut dgram) => {
                dgram[0] = (value >> 8) as u8;
                dgram[1] = (value & 0x00ff) as u8;
                dgram.slice(0..2);
                match self
                    .udp_sender
                    .send_to(DST_ADDR, self.dst_port.get(), dgram)
                {
                    Ok(_) => ReturnCode::SUCCESS,
                    Err(mut buf) => {
                        buf.reset();
                        self.udp_dgram.replace(buf);
                        ReturnCode::ERESERVE
                    }
                }
            }
            None => {
                debug!("ERROR: udp_dgram not present.");
                ReturnCode::FAIL
            }
        }
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for MockUdp<'a, A> {
    fn fired(&self) {
        if self.send_loop.get() {
            self.send(self.id);
        }
    }
}

impl<'a, A: Alarm<'a>> UDPSendClient for MockUdp<'a, A> {
    fn send_done(&self, result: ReturnCode, mut dgram: LeasableBuffer<'static, u8>) {
        debug!("Mock UDP done sending. Result: {:?}", result);
        dgram.reset();
        self.udp_dgram.replace(dgram);
        debug!("");
        self.alarm
            .set_alarm(self.alarm.now().wrapping_add(A::Ticks::from(
                <A::Frequency>::frequency() * SEND_INTERVAL_SECONDS,
            )));
    }
}

impl<'a, A: Alarm<'a>> UDPRecvClient for MockUdp<'a, A> {
    fn receive(
        &self,
        src_addr: IPAddr,
        _dst_addr: IPAddr,
        src_port: u16,
        _dst_port: u16,
        payload: &[u8],
    ) {
        debug!(
            "[MOCK_UDP {:?}] Received packet from {:?}:{:?}, contents: {:?}\n",
            self.id, src_addr, src_port, payload
        );
    }
}
