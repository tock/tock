//! `app_layer_lowpan_frag.rs`: Test application layer sending of
//! 6LoWPAN packets
//!
//! Currently this file only tests sending messages.
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
//! let app_lowpan_frag_test = app_layer_lowpan_frag::initialize_all(radio_mac as &'static Mac,
//!                                                          mux_alarm as &'static
//!                                                             MuxAlarm<'static,
//!                                                                 sam4l::ast::Ast>);
//! radio_mac.set_transmit_client(app_lowpan_frag_test);
//! ...
//! // Imix initialization
//! ...
//! app_lowpan_frag_test.start(); // If flashing the transmitting Imix

use capsules;
extern crate sam4l;
use capsules::ieee802154::device::MacDevice;
use capsules::net::ipv6::ip_utils::{ip6_nh, IPAddr};
use capsules::net::ipv6::ipv6::{IP6Header, IP6Packet, IPPayload, TransportHeader};
use capsules::net::ipv6::ipv6_send::{IP6SendStruct, IP6Sender};
use capsules::net::sixlowpan::sixlowpan_compression;
use capsules::net::sixlowpan::sixlowpan_state::{Sixlowpan, SixlowpanState, TxState};
use capsules::net::udp::udp::UDPHeader;
use capsules::net::udp::udp_send::{UDPSendStruct, UDPSender};
use capsules::net::ieee802154::MacAddress;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::hil::radio;
use kernel::hil::time;
use kernel::hil::time::Frequency;
use kernel::ReturnCode;

pub const SRC_ADDR: IPAddr = IPAddr([
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
]);
pub const DST_ADDR: IPAddr = IPAddr([
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
]);
pub const PAYLOAD_LEN: usize = 200;

/* 6LoWPAN Constants */
const DEFAULT_CTX_PREFIX_LEN: u8 = 8;
static DEFAULT_CTX_PREFIX: [u8; 16] = [0x0 as u8; 16];
static mut RX_STATE_BUF: [u8; 1280] = [0x0; 1280];
const DST_MAC_ADDR: MacAddress = MacAddress::Short(0x802);
const SRC_MAC_ADDR: MacAddress = MacAddress::Short(0xf00f);

pub const TEST_DELAY_MS: u32 = 10000;
pub const TEST_LOOP: bool = false;
static mut UDP_PAYLOAD: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN]; //Becomes payload of UDP

pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0 as u8; radio::MAX_BUF_SIZE];

//Use a global variable option, initialize as None, then actually initialize in initialize all

pub struct LowpanTest<'a, A: time::Alarm> {
    alarm: A,
    //sixlowpan_tx: TxState<'a>,
    //radio: &'a Mac<'a>,
    test_counter: Cell<usize>,
    udp_sender: &'a UDPSender<'a>,
}
//TODO: Initialize UDP sender/send_done client in initialize all
pub unsafe fn initialize_all(
    radio_mac: &'static MacDevice,
    mux_alarm: &'static MuxAlarm<'static, sam4l::ast::Ast>,
) -> &'static LowpanTest<
    'static,
    capsules::virtual_alarm::VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
> {
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

    let sixlowpan_state = sixlowpan as &SixlowpanState;
    let sixlowpan_tx = TxState::new(sixlowpan_state);
    // Following code initializes an IP6Packet using the global UDP_DGRAM buffer as the payload
    let mut udp_hdr: UDPHeader = UDPHeader {
        src_port: 0,
        dst_port: 0,
        len: 0,
        cksum: 0,
    };
    udp_hdr.set_src_port(12345);
    udp_hdr.set_dst_port(54321);
    udp_hdr.set_len(PAYLOAD_LEN as u16 + 8);
    //checksum is calculated and set later

    let mut ip6_hdr: IP6Header = IP6Header::new();
    ip6_hdr.set_next_header(ip6_nh::UDP);
    ip6_hdr.set_payload_len(PAYLOAD_LEN as u16 + 8);
    ip6_hdr.src_addr = SRC_ADDR;
    ip6_hdr.dst_addr = DST_ADDR;

    let tr_hdr: TransportHeader = TransportHeader::UDP(udp_hdr);

    let ip_pyld: IPPayload = IPPayload {
        header: tr_hdr,
        payload: &mut UDP_PAYLOAD,
    };

    let ip6_dg = static_init!(IP6Packet<'static>, IP6Packet::new(ip_pyld));

    let ip6_sender = static_init!(
        IP6SendStruct<'static>,
        IP6SendStruct::new(ip6_dg, &mut RF233_BUF, sixlowpan_tx, radio_mac,
                           DST_MAC_ADDR, SRC_MAC_ADDR)
    );
    radio_mac.set_transmit_client(ip6_sender);

    let udp_send_struct = static_init!(
        UDPSendStruct<'static, IP6SendStruct<'static>>,
        UDPSendStruct::new(ip6_sender)
    );

    let app_lowpan_frag_test = static_init!(
        LowpanTest<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        LowpanTest::new(
            //sixlowpan_tx,
            //radio_mac,
            VirtualMuxAlarm::new(mux_alarm),
            udp_send_struct
        )
    );
    ip6_sender.set_client(udp_send_struct);
    udp_send_struct.set_client(app_lowpan_frag_test);
    app_lowpan_frag_test.alarm.set_client(app_lowpan_frag_test);

    app_lowpan_frag_test
}

impl<'a, A: time::Alarm> capsules::net::udp::udp_send::UDPSendClient for LowpanTest<'a, A> {
    fn send_done(&self, result: ReturnCode) {
        match result {
            ReturnCode::SUCCESS => {
                debug!("Packet Sent!");
                self.schedule_next();
            }
            _ => debug!("Failed to send UDP Packet!"),
        }
    }
}

impl<'a, A: time::Alarm> LowpanTest<'a, A> {
    pub fn new(
        //sixlowpan_tx: TxState<'a>,
        //radio: &'a Mac<'a>,
        alarm: A,
        //ip6_packet: &'static mut IP6Packet<'a>
        udp_sender: &'a UDPSender<'a>,
    ) -> LowpanTest<'a, A> {
        LowpanTest {
            alarm: alarm,
            //sixlowpan_tx: sixlowpan_tx,
            //radio: radio,
            test_counter: Cell::new(0),
            udp_sender: udp_sender,
        }
    }

    pub fn start(&self) {
        //self.run_test_and_increment();
        self.schedule_next();
    }

    fn schedule_next(&self) {
        let delta = (A::Frequency::frequency() * TEST_DELAY_MS) / 1000;
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
        2
    }

    fn run_test(&self, test_id: usize) {
        debug!("Running test {}:", test_id);
        match test_id {
            0 => self.ipv6_send_packet_test(),
            1 => self.ipv6_send_packet_test(),
            _ => {}
        }
    }

    fn ipv6_send_packet_test(&self) {
        unsafe {
            self.send_ipv6_packet();
        }
    }

    unsafe fn send_ipv6_packet(&self) {
        self.send_next();
    }

    fn send_next(&self) {
        //Insert code to send UDP PAYLOAD here.
        let src_port: u16 = 12321;
        let dst_port: u16 = 32123;
        unsafe {
            self.udp_sender
                .send_to(DST_ADDR, src_port, dst_port, &UDP_PAYLOAD)
        };
    }
}

impl<'a, A: time::Alarm> time::Client for LowpanTest<'a, A> {
    fn fired(&self) {
        self.run_test_and_increment();
    }
}
