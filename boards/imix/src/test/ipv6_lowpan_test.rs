//! `ipv6_lowpan_test.rs`: 6LoWPAN Fragmentation Test Suite
//!
//! This implements a simple testing framework for 6LoWPAN fragmentation and
//! compression. Two Imix boards run this code, one for receiving and one for
//! transmitting. The transmitting board must call the `start` function in
//! the `main.rs` file. The transmitting Imix then sends a variety of packets
//! to the receiving Imix, relying on the 6LoWPAN fragmentation and reassembly
//! layer. Note that this layer also performs 6LoWPAN compression (invisible
//! to the upper layers), so this test suite is also dependent on the
//! correctness of the compression/decompression implementation; for this
//! reason, tests solely for compression/decompression have been left in a
//! different file.
//!
//! This test suite will print out whether a receive packet is different than
//! the expected packet. For this test to work correctly, and for both sides
//! to remain in sync, they must both be started at the same time. Any dropped
//! frames will prevent the test from completing successfully.
//!
//! To use this test suite, allocate space for a new LowpanTest structure, and
//! set it as the client for the Sixlowpan struct and for the respective TxState
//! struct. For the transmit side, call the LowpanTest::start method. The
//! `initialize_all` function performs this initialization; simply call this
//! function in `boards/imix/src/main.rs` as follows:
//!
//! Alternatively, you can call the `initialize_all` function, which performs
//! the initialization routines for the 6LoWPAN, TxState, RxState, and Sixlowpan
//! structs. Insert the code into `boards/imix/src/main.rs` as follows:
//!
//! ...
//! // Radio initialization code
//! ...
//! let lowpan_frag_test = test::ipv6_lowpan_test::initialize_all(
//!    mux_mac,
//!    mux_alarm as &'static MuxAlarm<'static, sam4l::ast::Ast>,
//! );
//! ...
//! // Imix initialization
//! ...
//! lowpan_frag_test.start(); // If flashing the transmitting Imix

use capsules::ieee802154::device::{MacDevice, TxClient};
use capsules::net::ieee802154::MacAddress;
use capsules::net::ipv6::ip_utils::{ip6_nh, IPAddr};
use capsules::net::ipv6::ipv6::{IP6Header, IP6Packet, IPPayload, TransportHeader};
use capsules::net::sixlowpan::sixlowpan_compression;
use capsules::net::sixlowpan::sixlowpan_state::{
    RxState, Sixlowpan, SixlowpanRxClient, SixlowpanState, TxState,
};
use capsules::net::udp::udp::UDPHeader;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use core::ptr;
use kernel::debug;
use kernel::hil::radio;
use kernel::hil::time::Frequency;
use kernel::hil::time::{self, Alarm};
use kernel::static_init;
use kernel::ReturnCode;

pub const MLP: [u8; 8] = [0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7];

/* pub const SRC_ADDR: IPAddr = IPAddr([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

pub const DST_ADDR: IPAddr = IPAddr([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);*/

pub const SRC_ADDR: IPAddr = IPAddr([
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
]);
pub const DST_ADDR: IPAddr = IPAddr([
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
]);
pub const SRC_MAC_ADDR: MacAddress =
    MacAddress::Long([0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17]);
//pub const DST_MAC_ADDR: MacAddress =
//    MacAddress::Long([0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]);
pub const DST_MAC_ADDR: MacAddress = MacAddress::Short(57326);
//TODO: No longer pass MAC addresses to 6lowpan code, so these values arent used rn
pub const IP6_HDR_SIZE: usize = 40;
pub const UDP_HDR_SIZE: usize = 8;
pub const PAYLOAD_LEN: usize = 200;
pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0 as u8; radio::MAX_BUF_SIZE];

/* 6LoWPAN Constants */
const DEFAULT_CTX_PREFIX_LEN: u8 = 8;
static DEFAULT_CTX_PREFIX: [u8; 16] = [0x0 as u8; 16];
static mut RX_STATE_BUF: [u8; 1280] = [0x0; 1280];

#[derive(Copy, Clone, Debug, PartialEq)]
enum TF {
    Inline = 0b00,
    Traffic = 0b01,
    Flow = 0b10,
    TrafficFlow = 0b11,
}

#[derive(Copy, Clone, Debug)]
enum SAC {
    Inline,
    LLP64,
    LLP16,
    LLPIID,
    Unspecified,
    Ctx64,
    Ctx16,
    CtxIID,
}

#[derive(Copy, Clone, Debug)]
enum DAC {
    Inline,
    LLP64,
    LLP16,
    LLPIID,
    Ctx64,
    Ctx16,
    CtxIID,
    McastInline,
    Mcast48,
    Mcast32,
    Mcast8,
    McastCtx,
}

pub const TEST_DELAY_MS: u32 = 10000;
pub const TEST_LOOP: bool = false;
static mut SUCCESS_COUNT: usize = 0;
// Below was IP6_DGRAM before change to typed buffers
//static mut IP6_DGRAM: [u8; IP6_HDR_SIZE + PAYLOAD_LEN] = [0; IP6_HDR_SIZE + PAYLOAD_LEN];
static mut UDP_DGRAM: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE]; //Becomes payload of UDP

//Use a global variable option, initialize as None, then actually initialize in initialize all

static mut IP6_DG_OPT: Option<IP6Packet> = None;
//END changes

pub struct LowpanTest<'a, A: time::Alarm<'a>> {
    alarm: A,
    sixlowpan_tx: TxState<'a>,
    radio: &'a dyn MacDevice<'a>,
    test_counter: Cell<usize>,
}

pub unsafe fn initialize_all(
    mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    mux_alarm: &'static MuxAlarm<'static, sam4l::ast::Ast>,
) -> &'static LowpanTest<
    'static,
    capsules::virtual_alarm::VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
> {
    let radio_mac = static_init!(
        capsules::ieee802154::virtual_mac::MacUser<'static>,
        capsules::ieee802154::virtual_mac::MacUser::new(mux_mac)
    );
    mux_mac.add_user(radio_mac);
    let default_rx_state = static_init!(RxState<'static>, RxState::new(&mut RX_STATE_BUF));

    let sixlowpan = static_init!(
        Sixlowpan<'static, sam4l::ast::Ast<'static>, sixlowpan_compression::Context>,
        Sixlowpan::new(
            sixlowpan_compression::Context {
                prefix: DEFAULT_CTX_PREFIX,
                prefix_len: DEFAULT_CTX_PREFIX_LEN,
                id: 0,
                compress: false,
            },
            &sam4l::ast::AST
        )
    );

    let sixlowpan_state = sixlowpan as &dyn SixlowpanState;
    let sixlowpan_tx = TxState::new(sixlowpan_state);

    sixlowpan_tx.init(SRC_MAC_ADDR, DST_MAC_ADDR, radio_mac.get_pan(), None);

    let lowpan_frag_test = static_init!(
        LowpanTest<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        LowpanTest::new(sixlowpan_tx, radio_mac, VirtualMuxAlarm::new(mux_alarm))
    );

    sixlowpan_state.add_rx_state(default_rx_state);
    sixlowpan_state.set_rx_client(lowpan_frag_test);
    lowpan_frag_test.alarm.set_client(lowpan_frag_test);

    radio_mac.set_receive_client(sixlowpan);

    // Following code initializes an IP6Packet using the global UDP_DGRAM buffer as the payload
    let mut udp_hdr: UDPHeader = UDPHeader {
        src_port: 0,
        dst_port: 0,
        len: 0,
        cksum: 0,
    };
    udp_hdr.set_src_port(12345);
    udp_hdr.set_dst_port(54321);
    udp_hdr.set_len(PAYLOAD_LEN as u16);
    //checksum is calculated and set later

    let mut ip6_hdr: IP6Header = IP6Header::new();
    ip6_hdr.set_next_header(ip6_nh::UDP);
    ip6_hdr.set_payload_len(PAYLOAD_LEN as u16);
    ip6_hdr.src_addr = SRC_ADDR;
    ip6_hdr.dst_addr = DST_ADDR;

    let tr_hdr: TransportHeader = TransportHeader::UDP(udp_hdr);

    let ip_pyld: IPPayload = IPPayload {
        header: tr_hdr,
        payload: &mut UDP_DGRAM,
    };

    let mut ip6_dg: IP6Packet = IP6Packet {
        header: ip6_hdr,
        payload: ip_pyld,
    };

    ip6_dg.set_transport_checksum(); //calculates and sets UDP cksum

    IP6_DG_OPT = Some(ip6_dg);
    //Now, other places in code should have access to initialized IP6Packet.
    //Note that this code is inherently unsafe and we make no effort to prevent
    //race conditions, as this is merely test code
    radio_mac.set_transmit_client(lowpan_frag_test);
    lowpan_frag_test
}

impl<'a, A: time::Alarm<'a>> LowpanTest<'a, A> {
    pub fn new(
        sixlowpan_tx: TxState<'a>,
        radio: &'a dyn MacDevice<'a>,
        alarm: A,
    ) -> LowpanTest<'a, A> {
        LowpanTest {
            alarm: alarm,
            sixlowpan_tx: sixlowpan_tx,
            radio: radio,
            test_counter: Cell::new(0),
        }
    }

    pub fn start(&self) {
        //self.run_test_and_increment();
        self.schedule_next();
    }

    fn schedule_next(&self) {
        let delta = (A::Frequency::frequency() * TEST_DELAY_MS) / 5000;
        let next = self.alarm.now().wrapping_add(delta);
        self.alarm.set_alarm(next);
    }

    fn run_test_and_increment(&self) {
        let test_counter = self.test_counter.get();
        self.run_test(test_counter);
        match TEST_LOOP {
            true => self.test_counter.set((test_counter + 1) % self.num_tests()),
            false => self.test_counter.set(test_counter + 1),
        };
    }

    fn num_tests(&self) -> usize {
        28
    }

    fn run_test(&self, test_id: usize) {
        debug!("Running test {}:", test_id);
        match test_id {
            // Change TF compression
            0 => self.ipv6_send_packet_test(TF::Inline, 255, SAC::Inline, DAC::Inline),
            1 => self.ipv6_send_packet_test(TF::Traffic, 255, SAC::Inline, DAC::Inline),
            2 => self.ipv6_send_packet_test(TF::Flow, 255, SAC::Inline, DAC::Inline),
            3 => self.ipv6_send_packet_test(TF::TrafficFlow, 255, SAC::Inline, DAC::Inline),

            // Change HL compression
            4 => self.ipv6_send_packet_test(TF::TrafficFlow, 255, SAC::Inline, DAC::Inline),
            5 => self.ipv6_send_packet_test(TF::TrafficFlow, 64, SAC::Inline, DAC::Inline),
            6 => self.ipv6_send_packet_test(TF::TrafficFlow, 1, SAC::Inline, DAC::Inline),
            7 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::Inline, DAC::Inline),

            // Change source compression
            8 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::Inline, DAC::Inline),
            9 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::LLP64, DAC::Inline),
            10 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::LLP16, DAC::Inline),
            11 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::LLPIID, DAC::Inline),
            12 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::Unspecified, DAC::Inline),
            13 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::Ctx64, DAC::Inline),
            14 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::Ctx16, DAC::Inline),
            15 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Inline),

            // Change dest compression
            16 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Inline),
            17 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLP64),
            18 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLP16),
            19 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLPIID),
            20 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Ctx64),
            21 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Ctx16),
            22 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::CtxIID),
            23 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::McastInline),
            24 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast48),
            25 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast32),
            26 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast8),
            27 => self.ipv6_send_packet_test(TF::TrafficFlow, 42, SAC::CtxIID, DAC::McastCtx),

            _ => {}
        }
    }

    fn run_check_test(&self, test_id: usize, buf: &[u8], len: usize) {
        debug!("Running test {}:", test_id);
        let success = match test_id {
            // Change TF compression
            0 => ipv6_check_receive_packet(TF::Inline, 255, SAC::Inline, DAC::Inline, buf, len),
            1 => ipv6_check_receive_packet(TF::Traffic, 255, SAC::Inline, DAC::Inline, buf, len),
            2 => ipv6_check_receive_packet(TF::Flow, 255, SAC::Inline, DAC::Inline, buf, len),
            3 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 255, SAC::Inline, DAC::Inline, buf, len)
            }

            // Change HL compression
            4 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 255, SAC::Inline, DAC::Inline, buf, len)
            }
            5 => ipv6_check_receive_packet(TF::TrafficFlow, 64, SAC::Inline, DAC::Inline, buf, len),
            6 => ipv6_check_receive_packet(TF::TrafficFlow, 1, SAC::Inline, DAC::Inline, buf, len),
            7 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::Inline, DAC::Inline, buf, len),

            // Change source compression
            8 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::Inline, DAC::Inline, buf, len),
            9 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::LLP64, DAC::Inline, buf, len),
            10 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::LLP16, DAC::Inline, buf, len),
            11 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::LLPIID, DAC::Inline, buf, len)
            }
            12 => ipv6_check_receive_packet(
                TF::TrafficFlow,
                42,
                SAC::Unspecified,
                DAC::Inline,
                buf,
                len,
            ),
            13 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::Ctx64, DAC::Inline, buf, len),
            14 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::Ctx16, DAC::Inline, buf, len),
            15 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Inline, buf, len)
            }

            // Change dest compression
            16 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Inline, buf, len)
            }
            17 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLP64, buf, len),
            18 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLP16, buf, len),
            19 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::LLPIID, buf, len)
            }
            20 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Ctx64, buf, len),
            21 => ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Ctx16, buf, len),
            22 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::CtxIID, buf, len)
            }
            23 => ipv6_check_receive_packet(
                TF::TrafficFlow,
                42,
                SAC::CtxIID,
                DAC::McastInline,
                buf,
                len,
            ),
            24 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast48, buf, len)
            }
            25 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast32, buf, len)
            }
            26 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::Mcast8, buf, len)
            }
            27 => {
                ipv6_check_receive_packet(TF::TrafficFlow, 42, SAC::CtxIID, DAC::McastCtx, buf, len)
            }

            _ => {
                debug!("Finished tests");
                false
            }
        };
        if success {
            unsafe {
                SUCCESS_COUNT += 1;
            }
        }
        if test_id == self.num_tests() - 1 {
            unsafe {
                if SUCCESS_COUNT == self.num_tests() {
                    debug!("All Tests completed successfully!");
                } else {
                    debug!(
                        "Successfully completed {:?}/{:?} tests",
                        SUCCESS_COUNT,
                        self.num_tests()
                    );
                }
            }
        }
    }
    fn ipv6_send_packet_test(&self, tf: TF, hop_limit: u8, sac: SAC, dac: DAC) {
        ipv6_prepare_packet(tf, hop_limit, sac, dac);
        unsafe {
            self.send_ipv6_packet(&MLP);
        }
    }

    unsafe fn send_ipv6_packet(&self, _: &[u8]) {
        self.send_next(&mut RF233_BUF);
    }

    fn send_next(&self, tx_buf: &'static mut [u8]) {
        unsafe {
            match IP6_DG_OPT {
                Some(ref ip6_packet) => {
                    match self
                        .sixlowpan_tx
                        .next_fragment(&ip6_packet, tx_buf, self.radio)
                    {
                        Ok((is_done, frame)) => {
                            //TODO: Fix ordering so that debug output does not indicate extra frame sent
                            if is_done {
                                self.schedule_next();
                            } else {
                                // TODO: Handle err (not just debug statement)
                                let (retcode, _opt) = self.radio.transmit(frame);
                                match retcode {
                                    ReturnCode::SUCCESS => {}
                                    _ => debug!("Error in radio transmit"),
                                }
                            }
                        }
                        Err((retcode, _buf)) => {
                            debug!("ERROR!: {:?}", retcode);
                        }
                    }
                }
                None => debug!("Error! tried to send uninitialized IP6Packet"),
            }
        }
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for LowpanTest<'a, A> {
    fn fired(&self) {
        self.run_test_and_increment();
    }
}

impl<'a, A: time::Alarm<'a>> SixlowpanRxClient for LowpanTest<'a, A> {
    fn receive(&self, buf: &[u8], len: usize, retcode: ReturnCode) {
        debug!("Receive completed: {:?}", retcode);
        let test_num = self.test_counter.get();
        self.test_counter.set((test_num + 1) % self.num_tests());
        self.run_check_test(test_num, buf, len)
    }
}

static mut ARRAY: [u8; 100] = [0x0; 100]; //used in introducing delay between frames
impl<'a, A: time::Alarm<'a>> TxClient for LowpanTest<'a, A> {
    fn send_done(&self, tx_buf: &'static mut [u8], _acked: bool, result: ReturnCode) {
        match result {
            ReturnCode::SUCCESS => {}
            _ => debug!("sendDone indicates error"),
        }
        unsafe {
            //This unsafe block introduces a delay between frames to prevent
            // a race condition on the receiver
            //it is sorta complicated bc I was having some trouble with dead code elimination
            let mut i = 0;
            while i < 4000000 {
                ARRAY[i % 100] = (i % 100) as u8;
                i = i + 1;
                if i % 1000000 == 0 {
                    i = i + 2;
                }
            }
        }
        self.send_next(tx_buf);
    }
}

#[allow(clippy::cast_ptr_alignment)]
fn ipv6_check_receive_packet(
    tf: TF,
    hop_limit: u8,
    sac: SAC,
    dac: DAC,
    recv_packet: &[u8],
    len: usize,
) -> bool {
    ipv6_prepare_packet(tf, hop_limit, sac, dac);
    let mut test_success = true;
    unsafe {
        // First, need to check header fields match:
        // Do this by casting first 48 bytes of rcvd packet as IP/UDP headers
        let rcvip6hdr: IP6Header = ptr::read(recv_packet.as_ptr() as *const _);
        let rcvudphdr: UDPHeader =
            ptr::read((recv_packet.as_ptr().offset(IP6_HDR_SIZE as isize)) as *const _);

        // Now compare to the headers that would be being sent by prepare packet
        // (as we know prepare packet is running in parallel on sender to generate tx packets)
        match IP6_DG_OPT {
            Some(ref ip6_packet) => {
                //First check IP headers
                if rcvip6hdr.get_version() != ip6_packet.header.get_version() {
                    test_success = false;
                    debug!("Mismatched IP ver");
                }

                if rcvip6hdr.get_traffic_class() != ip6_packet.header.get_traffic_class() {
                    debug!("Mismatched tc");
                    test_success = false;
                }
                if rcvip6hdr.get_dscp() != ip6_packet.header.get_dscp() {
                    debug!("Mismatched dcsp");
                    test_success = false;
                }
                if rcvip6hdr.get_ecn() != ip6_packet.header.get_ecn() {
                    debug!("Mismatched ecn");
                    test_success = false;
                }
                if rcvip6hdr.get_payload_len() != ip6_packet.header.get_payload_len() {
                    debug!("Mismatched IP len");
                    test_success = false;
                }
                if rcvip6hdr.get_next_header() != ip6_packet.header.get_next_header() {
                    debug!(
                        "Mismatched next hdr. Rcvd is: {:?}, expctd is: {:?}",
                        rcvip6hdr.get_next_header(),
                        ip6_packet.header.get_next_header()
                    );
                    test_success = false;
                }
                if rcvip6hdr.get_hop_limit() != ip6_packet.header.get_hop_limit() {
                    debug!("Mismatched hop limit");
                    test_success = false;
                }

                //Now check UDP headers

                match ip6_packet.payload.header {
                    TransportHeader::UDP(ref sent_udp_pkt) => {
                        if rcvudphdr.get_src_port() != sent_udp_pkt.get_src_port() {
                            debug!(
                                "Mismatched src_port. Rcvd is: {:?}, expctd is: {:?}",
                                rcvudphdr.get_src_port(),
                                sent_udp_pkt.get_src_port()
                            );
                            test_success = false;
                        }

                        if rcvudphdr.get_dst_port() != sent_udp_pkt.get_dst_port() {
                            debug!(
                                "Mismatched dst_port. Rcvd is: {:?}, expctd is: {:?}",
                                rcvudphdr.get_dst_port(),
                                sent_udp_pkt.get_dst_port()
                            );
                            test_success = false;
                        }

                        if rcvudphdr.get_len() != sent_udp_pkt.get_len() {
                            debug!(
                                "Mismatched udp_len. Rcvd is: {:?}, expctd is: {:?}",
                                rcvudphdr.get_len(),
                                sent_udp_pkt.get_len()
                            );
                            test_success = false;
                        }

                        if rcvudphdr.get_cksum() != sent_udp_pkt.get_cksum() {
                            debug!(
                                "Mismatched cksum. Rcvd is: {:?}, expctd is: {:?}",
                                rcvudphdr.get_cksum(),
                                sent_udp_pkt.get_cksum()
                            );
                            test_success = false;
                        }
                    }
                    _ => {
                        debug!(
                            "Error: For some reason prepare packet is not
                                    preparing a UDP payload"
                        );
                    }
                }
            }
            None => debug!("Error! tried to read uninitialized IP6Packet"),
        }

        // Finally, check bytes of UDP Payload
        let mut payload_success = true;
        for i in (IP6_HDR_SIZE + UDP_HDR_SIZE)..len {
            if recv_packet[i] != UDP_DGRAM[i - (IP6_HDR_SIZE + UDP_HDR_SIZE)] {
                test_success = false;
                payload_success = false;
                debug!(
                    "Packets differ at idx: {} where recv = {}, ref = {}",
                    i - (IP6_HDR_SIZE + UDP_HDR_SIZE),
                    recv_packet[i],
                    UDP_DGRAM[i - (IP6_HDR_SIZE + UDP_HDR_SIZE)]
                );
                //break; //Comment this in to help prevent debug buffer overflows
            }
        }
        if !payload_success {
            debug!("Packet payload did not match.");
        }
        debug!("Individual Test success is: {}", test_success);
        test_success
    }
}

//TODO: Change this function to modify IP6Packet struct instead of raw buffer
fn ipv6_prepare_packet(tf: TF, hop_limit: u8, sac: SAC, dac: DAC) {
    {
        let payload = unsafe { &mut UDP_DGRAM[0..] };
        for i in 0..(PAYLOAD_LEN - UDP_HDR_SIZE) {
            payload[i] = i as u8;
        }
    }
    unsafe {
        //Had to use unsafe here bc IP6_DG_OPT is mutable static

        match IP6_DG_OPT {
            Some(ref mut ip6_packet) => {
                {
                    let ip6_header: &mut IP6Header = &mut ip6_packet.header;
                    ip6_header.set_payload_len(PAYLOAD_LEN as u16);

                    if tf != TF::TrafficFlow {
                        ip6_header.set_ecn(0b01);
                    }
                    if (tf as u8) & (TF::Traffic as u8) != 0 {
                        ip6_header.set_dscp(0b000000);
                    } else {
                        ip6_header.set_dscp(0b101010);
                    }

                    if (tf as u8) & (TF::Flow as u8) != 0 {
                        ip6_header.set_flow_label(0);
                    } else {
                        ip6_header.set_flow_label(0xABCDE);
                    }

                    ip6_header.set_next_header(ip6_nh::UDP);

                    ip6_header.set_hop_limit(hop_limit);

                    match sac {
                        SAC::Inline => {
                            ip6_header.src_addr = SRC_ADDR;
                        }
                        SAC::LLP64 => {
                            // LLP::xxxx:xxxx:xxxx:xxxx
                            ip6_header.src_addr.set_unicast_link_local();
                            ip6_header.src_addr.0[8..16].copy_from_slice(&SRC_ADDR.0[8..16]);
                        }
                        SAC::LLP16 => {
                            // LLP::ff:fe00:xxxx
                            ip6_header.src_addr.set_unicast_link_local();
                            // Distinct from compute_iid because the U/L bit is not flipped
                            ip6_header.src_addr.0[11] = 0xff;
                            ip6_header.src_addr.0[12] = 0xfe;
                            ip6_header.src_addr.0[14..16].copy_from_slice(&SRC_ADDR.0[14..16]);
                        }
                        SAC::LLPIID => {
                            // LLP::IID
                            ip6_header.src_addr.set_unicast_link_local();
                            ip6_header.src_addr.0[8..16].copy_from_slice(
                                &sixlowpan_compression::compute_iid(&SRC_MAC_ADDR),
                            );
                        }
                        SAC::Unspecified => {}
                        SAC::Ctx64 => {
                            // MLP::xxxx:xxxx:xxxx:xxxx
                            ip6_header.src_addr.set_prefix(&MLP, 64);
                            ip6_header.src_addr.0[8..16].copy_from_slice(&SRC_ADDR.0[8..16]);
                        }
                        SAC::Ctx16 => {
                            // MLP::ff:fe00:xxxx
                            ip6_header.src_addr.set_prefix(&MLP, 64);
                            // Distinct from compute_iid because the U/L bit is not flipped
                            ip6_header.src_addr.0[11] = 0xff;
                            ip6_header.src_addr.0[12] = 0xfe;
                            ip6_header.src_addr.0[14..16].copy_from_slice(&SRC_ADDR.0[14..16]);
                        }
                        SAC::CtxIID => {
                            // MLP::IID
                            ip6_header.src_addr.set_prefix(&MLP, 64);
                            ip6_header.src_addr.0[8..16].copy_from_slice(
                                &sixlowpan_compression::compute_iid(&SRC_MAC_ADDR),
                            );
                        }
                    }

                    match dac {
                        DAC::Inline => {
                            ip6_header.dst_addr = DST_ADDR;
                        }
                        DAC::LLP64 => {
                            // LLP::xxxx:xxxx:xxxx:xxxx
                            ip6_header.dst_addr.set_unicast_link_local();
                            ip6_header.dst_addr.0[8..16].copy_from_slice(&DST_ADDR.0[8..16]);
                        }
                        DAC::LLP16 => {
                            // LLP::ff:fe00:xxxx
                            ip6_header.dst_addr.set_unicast_link_local();
                            // Distinct from compute_iid because the U/L bit is not flipped
                            ip6_header.dst_addr.0[11] = 0xff;
                            ip6_header.dst_addr.0[12] = 0xfe;
                            ip6_header.dst_addr.0[14..16].copy_from_slice(&SRC_ADDR.0[14..16]);
                        }
                        DAC::LLPIID => {
                            // LLP::IID
                            ip6_header.dst_addr.set_unicast_link_local();
                            ip6_header.dst_addr.0[8..16].copy_from_slice(
                                &sixlowpan_compression::compute_iid(&DST_MAC_ADDR),
                            );
                        }
                        DAC::Ctx64 => {
                            // MLP::xxxx:xxxx:xxxx:xxxx
                            ip6_header.dst_addr.set_prefix(&MLP, 64);
                            ip6_header.dst_addr.0[8..16].copy_from_slice(&SRC_ADDR.0[8..16]);
                        }
                        DAC::Ctx16 => {
                            // MLP::ff:fe00:xxxx
                            ip6_header.dst_addr.set_prefix(&MLP, 64);
                            // Distinct from compute_iid because the U/L bit is not flipped
                            ip6_header.dst_addr.0[11] = 0xff;
                            ip6_header.dst_addr.0[12] = 0xfe;
                            ip6_header.dst_addr.0[14..16].copy_from_slice(&SRC_ADDR.0[14..16]);
                        }
                        DAC::CtxIID => {
                            // MLP::IID
                            ip6_header.dst_addr.set_prefix(&MLP, 64);
                            ip6_header.dst_addr.0[8..16].copy_from_slice(
                                &sixlowpan_compression::compute_iid(&DST_MAC_ADDR),
                            );
                        }
                        DAC::McastInline => {
                            // first byte is ff, that's all we know
                            ip6_header.dst_addr = DST_ADDR;
                            ip6_header.dst_addr.0[0] = 0xff;
                        }
                        DAC::Mcast48 => {
                            // ffXX::00XX:XXXX:XXXX
                            ip6_header.dst_addr.0[0] = 0xff;
                            ip6_header.dst_addr.0[1] = DST_ADDR.0[1];
                            ip6_header.dst_addr.0[11..16].copy_from_slice(&DST_ADDR.0[11..16]);
                        }
                        DAC::Mcast32 => {
                            // ffXX::00XX:XXXX
                            ip6_header.dst_addr.0[0] = 0xff;
                            ip6_header.dst_addr.0[1] = DST_ADDR.0[1];
                            ip6_header.dst_addr.0[13..16].copy_from_slice(&DST_ADDR.0[13..16]);
                        }
                        DAC::Mcast8 => {
                            // ff02::00XX
                            ip6_header.dst_addr.0[0] = 0xff;
                            ip6_header.dst_addr.0[1] = DST_ADDR.0[1];
                            ip6_header.dst_addr.0[15] = DST_ADDR.0[15];
                        }
                        DAC::McastCtx => {
                            // ffXX:XX + plen + pfx64 + XXXX:XXXX
                            ip6_header.dst_addr.0[0] = 0xff;
                            ip6_header.dst_addr.0[1] = DST_ADDR.0[1];
                            ip6_header.dst_addr.0[2] = DST_ADDR.0[2];
                            ip6_header.dst_addr.0[3] = 64 as u8;
                            ip6_header.dst_addr.0[4..12].copy_from_slice(&MLP);
                            ip6_header.dst_addr.0[12..16].copy_from_slice(&DST_ADDR.0[12..16]);
                        }
                    }
                } //This bracket ends mutable borrow of ip6_packet for header
                  //Now that packet is fully prepared, set checksum
                ip6_packet.set_transport_checksum(); //calculates and sets UDP cksum
            } //End of Some{}
            None => debug!("Error! tried to prepare uninitialized IP6Packet"),
        }
    }

    debug!(
        "Packet with tf={:?} hl={} sac={:?} dac={:?}",
        tf, hop_limit, sac, dac
    );
}
