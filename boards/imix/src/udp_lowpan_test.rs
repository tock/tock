//! `udp_lowpan_test.rs`: Kernel test suite for the UDP/6LoWPAN stack
//!
//! This file tests port binding and sending and receiving messages from kernel space.
//! It has several different test modes. Each test uses the same radio initialization code,
//! but is started with a different function. A description of these tests and the expected output
//! follows.
//!
//! To use this test suite, insert the below code into `boards/imix/src/main.rs` as follows:
//!
//!```
//! ...
//! // Radio initialization code
//! ...
//!    let udp_lowpan_test = udp_lowpan_test::initialize_all(
//!        udp_mux,
//!        mux_alarm as &'static MuxAlarm<'static, sam4l::ast::Ast>,
//!    );
//! ...
//! // Imix initialization
//! ...
//! udp_lowpan_test.start();
//!```
//!
//! Different Initialization functions (pick one):
//!
//! start(),
//! instantiates two capsules that use the UDP stack, and tests various
//! binding and sending orders to ensure that port binding and sending
//! is enforced as expected.
//! The messages sent are long enough to require multiple fragments. The payload of each message
//! is all 0's.
//!
//! start_rx() runs a test where an app and a userspace capsule both verify correctness of port
//! binding across userspace apps and capsules, and then both attempt UDP reception on
//! different ports.
//!
//! start_with_app() tests port binding virtualization between both apps and capsules, and triggers
//! simultaneous sends in apps and capsules to test queueing when both apps and capsules are used.
//!
//! start_dual_rx() tests multiple capsules attempting to bind to different ports and receive
//! messages in quick succession, to test in-kernel distribution of received packets.
//!
//! Depending on the test you want to run, replace the call to start() with calls to
//! start_rx(), start_dual_rx(), or start_with_app().
//! Only one of these should be included at a time. Each is used for a different
//! set of kernel tests, some of which require additional boards or that userland
//! apps be flashed simultaneously.
//!
//! start() is an in-kernel only test. Its expected output follows:
//! -------------------------------------------------------------------------------
//! Running test 0:
//! send_fail test passed
//! Running test 1:
//! port_table_test passed
//! Running test 2:
//! port_table_test2 passed
//! Running test 3:
//! send_test executed, look at printed results once callbacks arrive
//! Mock UDP done sending. Result: SUCCESS
//!
//! Mock UDP done sending. Result: SUCCESS
//!
//! All UDP kernel tests complete.
//! -------------------------------------------------------------------------------
//!
//! start_with_app() should be used alongside the userland app `examples/tests/udp/udp_virt_app_kernel`
//!
//! start_with_app() expected output:
//! -------------------------------------------------------------------------------
//! [UDP VIRT] Starting Kernel Coop UDP Test App.
//! bind_test passed
//! send_test executed, look at printed results once callbacks arrive
//! Mock UDP done sending. Result: SUCCESS
//!
//! Mock UDP done sending. Result: SUCCESS
//!
//! App part of app/kernel test successful!
//! -------------------------------------------------------------------------------
//!
//! start_rx() should be run alongside the userland app
//! `examples/tests/udp/udp_virt_rx_tests/app1`. It also requires that a second board with the
//! normal kernel (no tests) be running simultaneously with both userland apps in
//! `examples/tests/udp/udp_virt_app_tests/` flashed on this second board. Press reset on the
//! second board at least 2 seconds after running `tockloader listen` on the receiving board to run
//! this test.
//!
//! start_rx() expected output:
//! -------------------------------------------------------------------------------
//! [UDP_RCV_APP1]: Rcvd UDP Packet from: 0001:0203:0405:0607:0809:0a0b:0c0d:0e0f : 26411
//! Packet Payload: Hello World - App1
//!
//! [MOCK_UDP 1] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):22222, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 50, 10]
//!
//! [UDP_RCV_APP1]: Rcvd UDP Packet from: 0001:0203:0405:0607:0809:0a0b:0c0d:0e0f : 20480
//! Packet Payload: Hello World - App1
//!
//! [MOCK_UDP 1] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):81, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 50, 10]
//! -------------------------------------------------------------------------------
//!
//! start_dual_rx() has the same instructions as start_rx(), but it should be run with no userspace
//! apps on the receiving board. This test also requires additional changes to main.rs --
//! serial_num_bottom_16 must be replaced with 49138 for this to work. main.rs includes comments
//! showing what should be included and excluded to run this final test. (Normally userspace apps
//! would set the appropriate src mac address of the board under test, but for in-kernel only tests
//! we do not currently expose this functionality to capsules).
//!
//! start_dual_rx() expected output:
//! -------------------------------------------------------------------------------
//![MOCK_UDP 1] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):11111, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 49, 10]
//!
//! [MOCK_UDP 2] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):22222, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 50, 10]
//!
//! [MOCK_UDP 1] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):80, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 49, 10]
//!
//! [MOCK_UDP 2] Received packet from IPAddr([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]):81, contents: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 32, 45, 32, 65, 112, 112, 50, 10]
//! -------------------------------------------------------------------------------

use super::imix_components::test::mock_udp::MockUDPComponent;
use super::imix_components::test::mock_udp2::MockUDPComponent2;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::ipv6::ipv6_send::IP6SendStruct;
use capsules::net::network_capabilities::{AddrRange, NetworkCapability, PortRange};
use capsules::net::udp::udp_port_table::UdpPortManager;
use capsules::net::udp::udp_recv::MuxUdpReceiver;
use capsules::net::udp::udp_send::MuxUdpSender;
use capsules::test::udp::MockUdp;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::capabilities::{NetworkCapabilityCreationCapability, UdpVisibilityCapability};
use kernel::{create_static_capability};
use kernel::component::Component;
use kernel::debug;
use kernel::hil::time::Frequency;
use kernel::hil::time::{self, Alarm};
use kernel::static_init;
use kernel::ReturnCode;

pub const TEST_DELAY_MS: u32 = 2000;
pub const TEST_LOOP: bool = false;
static mut UDP_PAYLOAD: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN]; //Becomes payload of UDP packet

const UDP_HDR_SIZE: usize = 8;
const PAYLOAD_LEN: usize = super::imix_components::udp_mux::PAYLOAD_LEN;
static mut UDP_PAYLOAD1: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];
static mut UDP_PAYLOAD2: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];

#[derive(Copy, Clone)]
enum TestMode {
    DefaultMode,
    WithAppMode,
    RxMode,
    DualRxMode,
}

pub struct LowpanTest<'a, A: time::Alarm<'a>> {
    alarm: &'a A,
    test_counter: Cell<usize>,
    port_table: &'static UdpPortManager,
    mock_udp1: &'a MockUdp<'a, A>,
    mock_udp2: &'a MockUdp<'a, A>,
    test_mode: Cell<TestMode>,
    create_cap: &'static dyn NetworkCapabilityCreationCapability,
}

pub unsafe fn initialize_all(
    udp_send_mux: &'static MuxUdpSender<
        'static,
        IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    >,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    mux_alarm: &'static MuxAlarm<'static, sam4l::ast::Ast>,
) -> &'static LowpanTest<
    'static,
    capsules::virtual_alarm::VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
> {
    let create_cap = create_static_capability!(NetworkCapabilityCreationCapability);
    let net_cap = static_init!(
        NetworkCapability,
        NetworkCapability::new(AddrRange::Any, PortRange::Any, PortRange::Any, create_cap)
    );
    let udp_vis = create_static_capability!(UdpVisibilityCapability);
    let mock_udp1 = MockUDPComponent::new(
        udp_send_mux,
        udp_recv_mux,
        port_table,
        mux_alarm,
        &mut UDP_PAYLOAD1,
        1, //id
        3, //dst_port
        net_cap,
        udp_vis,
    )
    .finalize(());

    let mock_udp2 = MockUDPComponent2::new(
        udp_send_mux,
        udp_recv_mux,
        port_table,
        mux_alarm,
        &mut UDP_PAYLOAD2,
        2, //id
        4, //dst_port
        net_cap,
        udp_vis,
    )
    .finalize(());

    let udp_lowpan_test = static_init!(
        LowpanTest<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        LowpanTest::new(
            static_init!(
                VirtualMuxAlarm<'static, sam4l::ast::Ast>,
                VirtualMuxAlarm::new(mux_alarm)
            ),
            port_table,
            mock_udp1,
            mock_udp2,
            create_cap,
        )
    );

    udp_lowpan_test.alarm.set_client(udp_lowpan_test);

    udp_lowpan_test
}

impl<'a, A: time::Alarm<'a>> LowpanTest<'a, A> {
    pub fn new(
        alarm: &'a A,
        port_table: &'static UdpPortManager,
        mock_udp1: &'static MockUdp<'a, A>,
        mock_udp2: &'static MockUdp<'a, A>,
        create_cap: &'static dyn NetworkCapabilityCreationCapability,
    ) -> LowpanTest<'a, A> {
        LowpanTest {
            alarm: alarm,
            test_counter: Cell::new(0),
            port_table: port_table,
            mock_udp1: mock_udp1,
            mock_udp2: mock_udp2,
            test_mode: Cell::new(TestMode::DefaultMode),
            create_cap: create_cap,
        }
    }

    pub fn start(&self) {
        self.schedule_next();
    }

    pub fn start_with_app(&self) {
        self.test_mode.set(TestMode::WithAppMode);
        self.schedule_next();
    }

    pub fn start_rx(&self) {
        self.test_mode.set(TestMode::RxMode);
        self.schedule_next();
    }

    pub fn start_dual_rx(&self) {
        self.test_mode.set(TestMode::DualRxMode);
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
        10
    }

    fn run_test(&self, test_id: usize) {
        match self.test_mode.get() {
            TestMode::DefaultMode => {
                if test_id < self.num_tests() {
                    debug!("Running test {}:", test_id);
                } else {
                    debug!("All UDP kernel tests complete.");
                }
                match test_id {
                    0 => self.capsule_send_fail(),
                    1 => self.port_table_test(),
                    2 => self.port_table_test(), //self.port_table_test2(),
                    3 => self.capsule_send_test(),
                    4 => self.addr_range_valid_test(),
                    5 => self.port_range_valid_test(),
                    6 => self.capsule_send_valid_net_cap_test(),
                    7 => self.capsule_send_invalid_net_cap_port_test(),
                    8 => self.capsule_send_invalid_net_cap_addr_test(),
                    9 => self.capsule_send_invalid_net_cap_addr_port_test(),
                    _ => return,
                }
            }
            TestMode::RxMode => match test_id {
                0 => self.capsule_receive_test(),
                _ => return,
            },
            TestMode::DualRxMode => match test_id {
                0 => self.capsule_dual_receive_test(),
                _ => return,
            },
            TestMode::WithAppMode => match test_id {
                0 => self.bind_test(),
                1 => self.capsule_send_test(),
                _ => return,
            },
        }
        self.schedule_next();
    }

    // This test ensures that an app and capsule cant bind to the same port
    // but can bind to different ports
    fn bind_test(&self) {
        let net_cap = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(AddrRange::Any, PortRange::Any, PortRange::Any, self.create_cap)
            )
        };
        let mut socket1 = self.port_table.create_socket().unwrap();
        // Attempt to bind to a port that has already been bound by an app.
        let result = self.port_table.bind(socket1, 1000, net_cap);
        assert!(result.is_err());
        socket1 = result.unwrap_err(); // Get the socket back

        //now bind to an open port
        let (_send_bind, _recv_bind) = self
            .port_table
            .bind(socket1, 1001, net_cap)
            .expect("UDP Bind fail");
        //dont unbind, so we can test if app will still be able to bind it

        debug!("bind_test passed");
    }

    // A basic test of port table functionality without using any capsules at all,
    // instead directly creating socket and calling bind/unbind.
    // This test ensures that two capsules could not bind to the same port,
    // that single bindings work correctly,
    fn port_table_test(&self) {
        let net_cap = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(AddrRange::Any, PortRange::Any, PortRange::Any, self.create_cap)
            )
        };
        // Initialize bindings.
        let socket1 = self.port_table.create_socket().unwrap();
        let mut socket2 = self.port_table.create_socket().unwrap();
        let socket3 = self.port_table.create_socket().unwrap();
        //debug!("Finished creating sockets");
        // Attempt to bind to a port that has already been bound.
        let (send_bind, recv_bind) = self
            .port_table
            .bind(socket1, 4000, net_cap)
            .expect("UDP Bind fail1");
        let result = self.port_table.bind(socket2, 4000, net_cap);
        assert!(result.is_err());
        socket2 = result.unwrap_err(); // This is how you get the socket back
        let (send_bind2, recv_bind2) = self
            .port_table
            .bind(socket2, 4001, net_cap)
            .expect("UDP Bind fail2");

        // Ensure that only the first binding is able to send
        assert_eq!(send_bind.get_port(), 4000);
        assert_eq!(recv_bind.get_port(), 4000);
        assert!(self.port_table.unbind(send_bind, recv_bind).is_ok());

        // Show that you can bind to a port once another socket has unbound it
        let (send_bind3, recv_bind3) = self
            .port_table
            .bind(socket3, 4000, net_cap)
            .expect("UDP Bind fail3");

        //clean up remaining bindings
        assert!(self.port_table.unbind(send_bind3, recv_bind3).is_ok());
        assert!(self.port_table.unbind(send_bind2, recv_bind2).is_ok());

        debug!("port_table_test passed");
    }

    fn port_table_test2(&self) {
        // Show that you can create up to 16 sockets before fail, but that destroying allows more
        // (MAX_NUM_BOUND_PORTS is set to 16 in udp_port_table.rs)
        {
            let _socket1 = self.port_table.create_socket().unwrap();
            let _socket2 = self.port_table.create_socket().unwrap();
            let _socket3 = self.port_table.create_socket().unwrap();
            let _socket4 = self.port_table.create_socket().unwrap();
            let _socket5 = self.port_table.create_socket().unwrap();
            let _socket6 = self.port_table.create_socket().unwrap();
            let _socket7 = self.port_table.create_socket().unwrap();
            let _socket8 = self.port_table.create_socket().unwrap();
            let _socket9 = self.port_table.create_socket().unwrap();
            let _socket10 = self.port_table.create_socket().unwrap();
            let _socket11 = self.port_table.create_socket().unwrap();
            let _socket12 = self.port_table.create_socket().unwrap();
            let _socket13 = self.port_table.create_socket().unwrap();
            let _socket14 = self.port_table.create_socket().unwrap();
            let _socket15 = self.port_table.create_socket().unwrap();
            let _socket16 = self.port_table.create_socket().unwrap();
            let willfail = self.port_table.create_socket();
            assert!(willfail.is_err());
            // these sockets table slots are freed once they are dropped, so
            // we can succeed again outside this block
        }
        let willsucceed = self.port_table.create_socket();
        assert!(willsucceed.is_ok());

        debug!("port_table_test2 passed");
    }

    fn capsule_send_fail(&self) {
        let ret = self.mock_udp1.send(0);
        assert!(ret != ReturnCode::SUCCESS); //trying to send while not bound should fail!

        debug!("send_fail test passed")
    }

    fn capsule_send_test(&self) {
        self.mock_udp1.bind(14000);
        self.mock_udp1.set_dst(15000);
        self.mock_udp2.bind(14001);
        self.mock_udp2.set_dst(15001);
        // Send from 2 different capsules in quick succession - second send should execute once
        // first completes!
        self.mock_udp1.send(22);
        self.mock_udp2.send(23);

        debug!("send_test executed, look at printed results once callbacks arrive");
    }

    fn capsule_app_send_test(&self) {
        self.mock_udp1.bind(16124);
        self.mock_udp1.set_dst(15000);
        self.mock_udp1.send(22);

        debug!("app/kernel send_test executed, look at printed results once callbacks arrive");
    }

    fn capsule_receive_test(&self) {
        self.mock_udp1.bind(16124);
    }

    fn capsule_dual_receive_test(&self) {
        self.mock_udp1.bind(16123);
        self.mock_udp2.bind(16124);
    }

    // Test network capability enforcement for addrs
    // TODO: include a positive and negative test for each (as apporpriate)
    fn addr_range_valid_test(&self) {
        let ip_addr1 = IPAddr([
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ]);
        let ip_addr2 = IPAddr([
            0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ]); // differs in first byte from ip_addr1
        let ip_addr3 = IPAddr([
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x00,
        ]); // differs in last byte from ip_addr1
            // Any
        let any_addr = AddrRange::Any;
        for ip_addr in [ip_addr1, ip_addr2, ip_addr3].iter() {
            assert!(any_addr.is_addr_valid(*ip_addr))
        }
        // No addrs
        let no_addrs = AddrRange::NoAddrs;
        for ip_addr in [ip_addr1, ip_addr2, ip_addr3].iter() {
            assert!(!no_addrs.is_addr_valid(*ip_addr))
        }
        let addr_set = AddrRange::AddrSet([
            IPAddr::new(),
            IPAddr::new(),
            IPAddr::new(),
            IPAddr::new(),
            IPAddr::new(),
            IPAddr::new(),
            ip_addr3,
            ip_addr1,
        ]);
        assert!(addr_set.is_addr_valid(ip_addr1));
        assert!(!addr_set.is_addr_valid(ip_addr2));
        assert!(addr_set.is_addr_valid(ip_addr3));
        // Single addr
        let single_addr = AddrRange::Addr(ip_addr1);
        assert!(single_addr.is_addr_valid(ip_addr1));
        assert!(!single_addr.is_addr_valid(ip_addr2));
        // Subnet
        let subnet = AddrRange::Subnet(ip_addr1, 120);
        assert!(subnet.is_addr_valid(ip_addr1));
        assert!(!subnet.is_addr_valid(ip_addr2));
        assert!(subnet.is_addr_valid(ip_addr3));
        debug!("AddrRange tests passed");
    }

    // Test enforcement for ports
    fn port_range_valid_test(&self) {
        // Any
        let any_port = PortRange::Any;
        assert!(any_port.is_port_valid(1000));
        // No ports
        let no_ports = PortRange::NoPorts;
        assert!(!no_ports.is_port_valid(1000));
        // Port set
        let port_set = PortRange::PortSet([80, 22, 100, 200, 300, 400, 500, 8888]);
        for port in [80, 22, 100, 200, 300, 400, 500, 8888].iter() {
            assert!(port_set.is_port_valid(*port));
        }
        assert!(!port_set.is_port_valid(1000));
        // Port range pair
        let port_range = PortRange::Range(1000, 2000);
        assert!(port_range.is_port_valid(1000));
        assert!(port_range.is_port_valid(1500));
        assert!(port_range.is_port_valid(2000));
        assert!(!port_range.is_port_valid(2001));
        assert!(!port_range.is_port_valid(900));
        // Single port
        let single_port = PortRange::Port(8888);
        assert!(single_port.is_port_valid(8888));
        assert!(!single_port.is_port_valid(8000));
        debug!("PortRange tests passed");
    }

    // TODO: check return codes from send_with_cap
    // Valid network capability (valid addr, valid port)

    fn capsule_send_net_cap_test(
        &self,
        net_cap1: &'static NetworkCapability,
        net_cap2: &'static NetworkCapability,
    ) {
        // from capsule send_test
        self.mock_udp1.bind(14000);
        self.mock_udp1.set_dst(15000);
        self.mock_udp2.bind(14001);
        self.mock_udp2.set_dst(15001);
        // Send from 2 different capsules in quick succession - second send should execute once
        // first completes!
        // TODO: construct capabilities.
        self.mock_udp1.send_with_cap(22, net_cap1);
        self.mock_udp2.send_with_cap(23, net_cap2);
        debug!("send_test executed, look at printed results once callbacks arrive");
    }

    fn capsule_send_valid_net_cap_test(&self) {
        let net_cap1 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15000),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        let net_cap2 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15001),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        self.capsule_send_net_cap_test(net_cap1, net_cap2);
        debug!("send_valid_net_cap test executed, look at printed results once callbacks arrive");
    }

    // Invalid network capability (valid addr, invalid port)
    fn capsule_send_invalid_net_cap_port_test(&self) {
        let net_cap1 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15001),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        // net_cap2 has an invalid dst port
        let net_cap2 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15002),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        self.capsule_send_net_cap_test(net_cap1, net_cap2);
        debug!("send_invalid_net_cap_port test executed, look at printed results once callbacks arrive");
    }

    // Invalid network capability (invalid addr, valid port)
    fn capsule_send_invalid_net_cap_addr_test(&self) {
        let net_cap1 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::NoAddrs,
                    PortRange::Port(15000),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        let net_cap2 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15001),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        self.capsule_send_net_cap_test(net_cap1, net_cap2);
        debug!("send_invalid_net_cap_addr executed, look at printed results once callbacks arrive");
    }

    // Invalid network capability (invalid addr, invalid port)
    fn capsule_send_invalid_net_cap_addr_port_test(&self) {
        let net_cap1 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::NoAddrs,
                    PortRange::Port(15000),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        let net_cap2 = unsafe {
            static_init!(
                NetworkCapability,
                NetworkCapability::new(
                    AddrRange::Any,
                    PortRange::Port(15002),
                    PortRange::Any,
                    self.create_cap
                )
            )
        };
        self.capsule_send_net_cap_test(net_cap1, net_cap2);
        debug!("send_invalid_net_cap_addr_port test executed, look at printed results once callbacks arrive");
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for LowpanTest<'a, A> {
    fn fired(&self) {
        self.run_test_and_increment();
    }
}
