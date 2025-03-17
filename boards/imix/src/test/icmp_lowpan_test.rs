// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! `icmp_lowpan_test.rs`: Test kernel space sending of
//! ICMP packets over 6LoWPAN
//!
//! Currently this file only tests sending messages.
//!
//! To use this test suite, simply call the `run` function.
//! Insert the code into `boards/imix/src/main.rs` as follows:
//!
//!```rust
//! test::icmp_lowpan_test::run(mux_mac, mux_alarm);
//! ```

use capsules_extra::ieee802154::device::MacDevice;
use capsules_extra::net::icmpv6::icmpv6_send::{ICMP6SendStruct, ICMP6Sender};
use capsules_extra::net::icmpv6::{ICMP6Header, ICMP6Type};
use capsules_extra::net::ieee802154::MacAddress;
use capsules_extra::net::ipv6::ip_utils::IPAddr;
use capsules_extra::net::ipv6::ipv6_send::{IP6SendStruct, IP6Sender};
use capsules_extra::net::ipv6::{IP6Packet, IPPayload, TransportHeader};
use capsules_extra::net::network_capabilities::{
    AddrRange, IpVisibilityCapability, NetworkCapability, PortRange,
};
use capsules_extra::net::sixlowpan::sixlowpan_compression;
use capsules_extra::net::sixlowpan::sixlowpan_state::{Sixlowpan, SixlowpanState, TxState};
use kernel::ErrorCode;

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use core::ptr::addr_of_mut;
use kernel::capabilities::NetworkCapabilityCreationCapability;
use kernel::create_capability;
use kernel::debug;
use kernel::hil::radio;
use kernel::hil::time::{self, Alarm, ConvertTicks};
use kernel::static_init;

pub const SRC_ADDR: IPAddr = IPAddr([
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
]);
pub const DST_ADDR: IPAddr = IPAddr([
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
]);

/* 6LoWPAN Constants */
const DEFAULT_CTX_PREFIX_LEN: u8 = 8;
static DEFAULT_CTX_PREFIX: [u8; 16] = [0x0_u8; 16];
static mut RX_STATE_BUF: [u8; 1280] = [0x0; 1280];
const DST_MAC_ADDR: MacAddress = MacAddress::Short(0x802);
const SRC_MAC_ADDR: MacAddress = MacAddress::Short(0xf00f);

pub const TEST_DELAY_MS: u32 = 10000;
pub const TEST_LOOP: bool = false;

static mut ICMP_PAYLOAD: [u8; 10] = [0; 10];

pub static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0_u8; radio::MAX_BUF_SIZE];

//Use a global variable option, initialize as None, then actually initialize in initialize all

pub struct LowpanICMPTest<'a, A: time::Alarm<'a>> {
    alarm: &'a A,
    test_counter: Cell<usize>,
    icmp_sender: &'a dyn ICMP6Sender<'a>,
    net_cap: &'static NetworkCapability,
}

type Rf233 = capsules_extra::rf233::RF233<
    'static,
    capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
        'static,
        sam4l::spi::SpiHw<'static>,
    >,
>;
type Ieee802154MacDevice =
    components::ieee802154::Ieee802154ComponentMacDeviceType<Rf233, sam4l::aes::Aes<'static>>;

pub unsafe fn run(
    mux_mac: &'static capsules_extra::ieee802154::virtual_mac::MuxMac<'static, Ieee802154MacDevice>,
    mux_alarm: &'static MuxAlarm<'static, sam4l::ast::Ast>,
) {
    let create_cap = create_capability!(NetworkCapabilityCreationCapability);
    let net_cap = static_init!(
        NetworkCapability,
        NetworkCapability::new(AddrRange::Any, PortRange::Any, PortRange::Any, &create_cap)
    );
    let ip_vis = static_init!(
        IpVisibilityCapability,
        IpVisibilityCapability::new(&create_cap)
    );
    let radio_mac = static_init!(
        capsules_extra::ieee802154::virtual_mac::MacUser<'static, Ieee802154MacDevice>,
        capsules_extra::ieee802154::virtual_mac::MacUser::new(mux_mac)
    );
    mux_mac.add_user(radio_mac);
    let ipsender_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    ipsender_virtual_alarm.setup();

    let sixlowpan = static_init!(
        Sixlowpan<
            'static,
            VirtualMuxAlarm<sam4l::ast::Ast<'static>>,
            sixlowpan_compression::Context,
        >,
        Sixlowpan::new(
            sixlowpan_compression::Context {
                prefix: DEFAULT_CTX_PREFIX,
                prefix_len: DEFAULT_CTX_PREFIX_LEN,
                id: 0,
                compress: false,
            },
            ipsender_virtual_alarm
        )
    );

    let sixlowpan_state = sixlowpan as &dyn SixlowpanState;
    let sixlowpan_tx = TxState::new(sixlowpan_state);

    let icmp_hdr = ICMP6Header::new(ICMP6Type::Type128); // Echo Request

    let ip_pyld: IPPayload = IPPayload {
        header: TransportHeader::ICMP(icmp_hdr),
        payload: &mut *addr_of_mut!(ICMP_PAYLOAD),
    };

    let ip6_dg = static_init!(IP6Packet<'static>, IP6Packet::new(ip_pyld));

    let ip6_sender = static_init!(
        IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        IP6SendStruct::new(
            ip6_dg,
            ipsender_virtual_alarm,
            &mut *addr_of_mut!(RF233_BUF),
            sixlowpan_tx,
            radio_mac,
            DST_MAC_ADDR,
            SRC_MAC_ADDR,
            ip_vis
        )
    );
    radio_mac.set_transmit_client(ip6_sender);

    let icmp_send_struct = static_init!(
        ICMP6SendStruct<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        >,
        ICMP6SendStruct::new(ip6_sender)
    );

    let alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    alarm.setup();

    let icmp_lowpan_test = static_init!(
        LowpanICMPTest<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        LowpanICMPTest::new(
            //sixlowpan_tx,
            //radio_mac,
            alarm,
            icmp_send_struct,
            net_cap
        )
    );

    ip6_sender.set_client(icmp_send_struct);
    icmp_send_struct.set_client(icmp_lowpan_test);
    icmp_lowpan_test.alarm.set_alarm_client(icmp_lowpan_test);
    ipsender_virtual_alarm.set_alarm_client(ip6_sender);
    icmp_lowpan_test.start();
}

impl<'a, A: time::Alarm<'a>> capsules_extra::net::icmpv6::icmpv6_send::ICMP6SendClient
    for LowpanICMPTest<'a, A>
{
    fn send_done(&self, result: Result<(), ErrorCode>) {
        match result {
            Ok(()) => {
                debug!("ICMP Echo Request Packet Sent!");
                match self.test_counter.get() {
                    2 => debug!("Test completed successfully."),
                    _ => self.schedule_next(),
                }
            }
            _ => debug!("Failed to send ICMP Packet!"),
        }
    }
}

impl<'a, A: time::Alarm<'a>> LowpanICMPTest<'a, A> {
    pub fn new(
        alarm: &'a A,
        icmp_sender: &'a dyn ICMP6Sender<'a>,
        net_cap: &'static NetworkCapability,
    ) -> LowpanICMPTest<'a, A> {
        LowpanICMPTest {
            alarm,
            test_counter: Cell::new(0),
            icmp_sender,
            net_cap,
        }
    }

    pub fn start(&self) {
        self.schedule_next();
    }

    fn schedule_next(&self) {
        let delta = self.alarm.ticks_from_ms(TEST_DELAY_MS);
        let now = self.alarm.now();
        self.alarm.set_alarm(now, delta);
    }

    fn run_test_and_increment(&self) {
        let test_counter = self.test_counter.get();
        self.run_test(test_counter);
        match TEST_LOOP {
            true => self.test_counter.set((test_counter + 1) % self.num_tests()),
            false => self.test_counter.set(test_counter + 1),
        }
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
        let icmp_hdr = ICMP6Header::new(ICMP6Type::Type128); // Echo Request
        let _ = unsafe {
            self.icmp_sender.send(
                DST_ADDR,
                icmp_hdr,
                &mut *addr_of_mut!(ICMP_PAYLOAD),
                self.net_cap,
            )
        };
    }
}

impl<'a, A: time::Alarm<'a>> time::AlarmClient for LowpanICMPTest<'a, A> {
    fn alarm(&self) {
        self.run_test_and_increment();
    }
}
