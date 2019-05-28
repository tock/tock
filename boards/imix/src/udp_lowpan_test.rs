//! `udp_lowpan_test.rs`: Test kernel space sending of
//! UDP Packets over 6LoWPAN.
//!
//! Currently this file only tests sending messages. It sends two long UDP messages
//! (long enough that each requires multiple fragments). The payload of each message
//! is all 0's. Tests for UDP reception exist in userspace, but not in the kernel
//! at this point in time. At the conclusion of the test, it prints "Test completed successfully."
//!
//! To use this test suite, allocate space for a new LowpanTest structure, and
//! call the `initialize_all` function, which performs
//! the initialization routines for the 6LoWPAN, TxState, RxState, and Sixlowpan
//! structs. Insert the code into `boards/imix/src/main.rs` as follows:
//!
//! ...
//! // Radio initialization code
//! ...
//!    let udp_lowpan_test = udp_lowpan_test::initialize_all(
//!        mux_mac,
//!        mux_alarm as &'static MuxAlarm<'static, sam4l::ast::Ast>,
//!    );
//! ...
//! // Imix initialization
//! ...
//! udp_lowpan_test.start();

use capsules::ieee802154::device::MacDevice;
use capsules::net::buffer::Buffer;
use capsules::net::ieee802154::MacAddress;
use capsules::net::ipv6::ip_utils::{ip6_nh, IPAddr};
use capsules::net::ipv6::ipv6::{IP6Header, IP6Packet, IPPayload, TransportHeader};
use capsules::net::ipv6::ipv6_send::{IP6SendStruct, IP6Sender};
use capsules::net::sixlowpan::sixlowpan_compression;
use capsules::net::sixlowpan::sixlowpan_state::{Sixlowpan, SixlowpanState, TxState};
use capsules::net::udp::udp::UDPHeader;
use capsules::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::common::cells::MapCell;
use kernel::debug;
use kernel::hil::radio;
use kernel::hil::time;
use kernel::hil::time::Frequency;
use kernel::static_init;
use kernel::ReturnCode;

use kernel::udp_port_table::{UdpPortTable, UdpSenderBinding};

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
pub const TEST_LOOP: bool = true;
static mut UDP_PAYLOAD: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN]; //Becomes payload of UDP packet

pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0 as u8; radio::MAX_BUF_SIZE];

//Use a global variable option, initialize as None, then actually initialize in initialize all

pub struct LowpanTest<'a, A: time::Alarm> {
    alarm: A,
    test_counter: Cell<usize>,
    udp_sender: &'a UDPSender<'a>,
    port_table: &'static UdpPortTable,
    dgram: MapCell<Buffer<'static, u8>>,
    send_bind: MapCell<UdpSenderBinding>,
}
//TODO: Initialize UDP sender/send_done client in initialize all
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

    let ipsender_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );

    let ip6_sender = static_init!(
        IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        IP6SendStruct::new(
            ip6_dg,
            ipsender_virtual_alarm,
            &mut RF233_BUF,
            sixlowpan_tx,
            radio_mac,
            DST_MAC_ADDR,
            SRC_MAC_ADDR
        )
    );
    radio_mac.set_transmit_client(ip6_sender);

    let udp_port_table = static_init!(UdpPortTable, UdpPortTable::new());

    let udp_mux = static_init!(
        MuxUdpSender<
            'static,
            capsules::net::ipv6::ipv6_send::IP6SendStruct<
                'static,
                VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
            >,
        >,
        MuxUdpSender::new(ip6_sender)
    );

    let udp_send_struct = static_init!(
        UDPSendStruct<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        >,
        UDPSendStruct::new(udp_mux)
    );

    let udp_lowpan_test = static_init!(
        LowpanTest<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        LowpanTest::new(
            //sixlowpan_tx,
            //radio_mac,
            VirtualMuxAlarm::new(mux_alarm),
            udp_send_struct,
            udp_port_table,
            &mut UDP_PAYLOAD,
        )
    );
    ip6_sender.set_client(udp_mux);
    udp_send_struct.set_client(udp_lowpan_test);
    udp_lowpan_test.alarm.set_client(udp_lowpan_test);
    ipsender_virtual_alarm.set_client(ip6_sender);

    udp_lowpan_test
}

impl<'a, A: time::Alarm> capsules::net::udp::udp_send::UDPSendClient for LowpanTest<'a, A> {
    fn send_done(&self, result: ReturnCode, mut dgram: Buffer<'static, u8>) {
        dgram.reset();
        self.dgram.replace(dgram);
        match result {
            ReturnCode::SUCCESS => {
                debug!("Packet Sent!");
                match self.test_counter.get() {
                    2 => debug!("Test completed successfully."),
                    _ => self.schedule_next(),
                }
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
        port_table: &'static UdpPortTable,
        dgram: &'static mut [u8],
    ) -> LowpanTest<'a, A> {
        LowpanTest {
            alarm: alarm,
            //sixlowpan_tx: sixlowpan_tx,
            //radio: radio,
            test_counter: Cell::new(0),
            udp_sender: udp_sender,
            port_table: port_table,
            dgram: MapCell::new(Buffer::new(dgram)),
            send_bind: MapCell::empty(),
        }
    }

    pub fn start(&self) {
        let socket = self.port_table.create_socket();
        let src_port = 12345;
        match socket {
            Ok(sock) => {
                debug!("Socket successfully created in udp_lowpan_test");
                match self.port_table.bind(sock, src_port) {
                    Ok((send_bind, _rcv_bind)) => {
                        debug!("Binding successfully created in udp_lowpan_test");
                        self.send_bind.replace(send_bind);
                    }
                    Err(sock) => {
                        debug!("Binding error in udp_lowpan_test");
                        self.port_table.destroy_socket(sock);
                    }
                }
            }
            Err(_return_code) => {
                debug!("Socket error in udp_lowpan_test");
                return;
            }
        }
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
        2 // 3
    }

    fn run_test(&self, test_id: usize) {
        debug!("Running test {}:", test_id);
        match test_id {
            //0 => self.port_table_test(),//self.ipv6_send_packet_test(),
            0 => self.ipv6_send_packet_test(), //self.ipv6_send_packet_test(),
            1 => self.ipv6_send_packet_test(),
            //1 => self.port_table_test(),
            _ => {}
        }
    }

    fn port_table_test(&self) {
        // Initialize bindings.
        let socket1 = self.port_table.create_socket().unwrap();
        let socket2 = self.port_table.create_socket().unwrap();
        let _socket3 = self.port_table.create_socket().unwrap();
        debug!("Finished creating sockets");
        // Attempt to bind to a port that has already been bound.
        let (send_bind, recv_bind) = self.port_table.bind(socket1, 80).ok().unwrap();
        // TODO: socket "memory-leak"?
        assert!(self.port_table.bind(socket2, 80).is_err());
        // debug!("After return code assertions for binding");
        // // Ensure that only the first binding is able to send
        assert_eq!(send_bind.get_port(), 80);
        assert_eq!(recv_bind.get_port(), 80);
        let _new_sock1 = self.port_table.unbind(send_bind, recv_bind);

        // let binding_socket = match ret1 {
        //     Ok(binding) => {
        //         let send_binding = binding.get_sender().unwrap();
        //         // Make sure correct port is bound
        //         assert_eq!(send_binding.get_port(), 80);
        //         // Disallow getting sender twice
        //         let err = binding.get_sender();
        //         match err {
        //             Ok(_) => assert!(false),
        //             Err(_) => assert!(true),
        //             _ => assert!(false),
        //         }
        //         assert!(binding.put_sender(send_binding).is_ok());
        //         let send_binding2 = binding.get_sender().unwrap();
        //         // Make sure correct port is bound
        //         assert_eq!(send_binding2.get_port(), 80);
        //         // Cannot unbind until we call put_sender
        //         let attempt = self.port_table.unbind(binding);
        //         let binding = attempt.err().unwrap();
        //         assert!(binding.put_sender(send_binding2).is_ok());
        //         self.port_table.unbind(binding).ok()
        //     },
        //     Err(x) => {
        //         assert!(false);
        //         None
        //     },
        // };
        // // // See if the third binding can successfully bind once the first is
        // // // unbound.
        // assert!(self.port_table.bind(socket3, 80).is_ok());
        // assert!(self.port_table.bind(binding_socket.unwrap(), 20).is_ok());
        debug!("port_table_test passed");
    }

    // TODO: add a test that involves sending/receiving.

    fn ipv6_send_packet_test(&self) {
        unsafe {
            self.send_ipv6_packet();
        }
    }

    unsafe fn send_ipv6_packet(&self) {
        self.send_next();
    }

    fn send_next(&self) {
        let dst_port: u16 = 32123;
        let send_bind = self.send_bind.take().expect("missing bind");
        debug!("before send_to");
        match self.dgram.take() {
            Some(dgram) => {
                self.udp_sender
                    .send_to(DST_ADDR, dst_port, dgram, &send_bind);
            }
            None => debug!("UDP_LOWPAN_TEST: DGRAM Missing - Err"),
        }
        self.send_bind.replace(send_bind);
        debug!("send_next done");
    }
}

impl<'a, A: time::Alarm> time::Client for LowpanTest<'a, A> {
    fn fired(&self) {
        self.run_test_and_increment();
    }
}
