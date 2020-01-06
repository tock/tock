//! Component to initialize the udp/6lowpan interface.
//!
//! This provides one Component, UDPMuxComponent. This component
//! exposes a MuxUdpSender that other components can implement
//! UDPSenders on top of to use the UDP/6Lowpan stack.
//!
//! Usage
//! -----
//! ```rust
//!    let (udp_mux, udp_recv) = UDPMuxComponent::new(
//!        mux_mac,
//!        DEFAULT_CTX_PREFIX_LEN,
//!        DEFAULT_CTX_PREFIX,
//!        DST_MAC_ADDR,
//!        src_mac_from_serial_num,
//!        local_ip_ifaces,
//!        mux_alarm,
//!        PAYLOAD_LEN,
//!    )
//!    .finalize();
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last Modified: 5/21/2019

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::ieee802154::device::MacDevice;
use capsules::net::ieee802154::MacAddress;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::ipv6::ipv6::{IP6Packet, IPPayload, TransportHeader};
use capsules::net::ipv6::ipv6_recv::IP6Receiver;
use capsules::net::ipv6::ipv6_send::IP6SendStruct;
use capsules::net::ipv6::ipv6_send::IP6Sender;
use capsules::net::sixlowpan::{sixlowpan_compression, sixlowpan_state};
use capsules::net::udp::udp::UDPHeader;
use capsules::net::udp::udp_port_table::{SocketBindingEntry, UdpPortManager, MAX_NUM_BOUND_PORTS};
use capsules::net::udp::udp_recv::MuxUdpReceiver;
use capsules::net::udp::udp_send::MuxUdpSender;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::radio;
use kernel::hil::time::{Alarm, Ticks32Bits};
use kernel::static_init;
use sam4l;

// The UDP stack requires exactly one of several packet buffers:
//
//   1. RF233_BUF: buffer the IP6_Sender uses to pass frames to the radio after fragmentation
//   2. SIXLOWPAN_RX_BUF: Buffer to hold full IP packets after they are decompressed by 6LoWPAN
//   3. udp_dgram: The payload of the IP6_Packet, which holds full IP Packets before they are tx'd.
//
//   Additionally, every capsule using the stack needs an additional buffer to craft packets for
//   tx which can then be passed to the MuxUdpSender for tx.

static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];
static mut SIXLOWPAN_RX_BUF: [u8; 1280] = [0x00; 1280];

pub const PAYLOAD_LEN: usize = 200; //The max size UDP message that can be sent by userspace apps or capsules
const UDP_HDR_SIZE: usize = 8;
static mut UDP_DGRAM: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];

// Rather than require a data structure with 65535 slots (number of UDP ports), we
// use a structure that can hold up to 16 port bindings. Any given capsule can bind
// at most one port. When a capsule obtains a socket, it is assigned a slot in this table.
// MAX_NUM_BOUND_PORTS represents the total number of capsules that can bind to different
// ports simultaneously within the Tock kernel.
// Each slot in the table tracks one socket that has been given to a capsule. If no
// slots in the table are free, no slots remain to be given out. If a socket is used to bind to
// a port, the port that is bound is saved in the slot to ensure that subsequent bindings do
// not also attempt to bind that port number.
static mut USED_KERNEL_PORTS: [Option<SocketBindingEntry>; MAX_NUM_BOUND_PORTS] =
    [None; MAX_NUM_BOUND_PORTS];

pub struct UDPMuxComponent {
    mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    ctx_pfix_len: u8,
    ctx_pfix: [u8; 16],
    dst_mac_addr: MacAddress,
    src_mac_addr: MacAddress,
    interface_list: &'static [IPAddr],
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
}

impl UDPMuxComponent {
    pub fn new(
        mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
        ctx_pfix_len: u8,
        ctx_pfix: [u8; 16],
        dst_mac_addr: MacAddress,
        src_mac_addr: MacAddress,
        interface_list: &'static [IPAddr],
        alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
    ) -> UDPMuxComponent {
        UDPMuxComponent {
            mux_mac: mux_mac,
            ctx_pfix_len: ctx_pfix_len,
            ctx_pfix: ctx_pfix,
            dst_mac_addr: dst_mac_addr,
            src_mac_addr: src_mac_addr,
            interface_list: interface_list,
            alarm_mux: alarm,
        }
    }
}

impl Component for UDPMuxComponent {
    type StaticInput = ();
    type Output = (
        &'static MuxUdpSender<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        >,
        &'static MuxUdpReceiver<'static>,
        &'static UdpPortManager,
    );

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let ipsender_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let udp_mac = static_init!(
            capsules::ieee802154::virtual_mac::MacUser<'static>,
            capsules::ieee802154::virtual_mac::MacUser::new(self.mux_mac)
        );
        self.mux_mac.add_user(udp_mac);

        let sixlowpan = static_init!(
            sixlowpan_state::Sixlowpan<
                'static,
                sam4l::ast::Ast<'static>,
                sixlowpan_compression::Context,
            >,
            sixlowpan_state::Sixlowpan::new(
                sixlowpan_compression::Context {
                    prefix: self.ctx_pfix,
                    prefix_len: self.ctx_pfix_len,
                    id: 0,
                    compress: false,
                },
                &sam4l::ast::AST
            )
        );

        let sixlowpan_state = sixlowpan as &dyn sixlowpan_state::SixlowpanState<Ticks32Bits>;
        let sixlowpan_tx = sixlowpan_state::TxState::new(sixlowpan_state);
        let default_rx_state = static_init!(
            sixlowpan_state::RxState<'static, Ticks32Bits>,
            sixlowpan_state::RxState::new(&mut SIXLOWPAN_RX_BUF)
        );
        sixlowpan_state.add_rx_state(default_rx_state);
        udp_mac.set_receive_client(sixlowpan);

        let tr_hdr = TransportHeader::UDP(UDPHeader::new());
        let ip_pyld: IPPayload = IPPayload {
            header: tr_hdr,
            payload: &mut UDP_DGRAM,
        };
        let ip6_dg = static_init!(IP6Packet<'static>, IP6Packet::new(ip_pyld));

        // In current design, all udp senders share same IP sender, and the IP sender
        // holds the destination mac address. This means all UDP senders must send to
        // the same mac address...this works fine under the assumption
        // of all packets being routed via a single gateway router, but doesn't work
        // if multiple senders want to send to different addresses on a local network.
        // This will be fixed once we have an ipv6_nd cache mapping IP addresses to dst macs
        let ip_send = static_init!(
            capsules::net::ipv6::ipv6_send::IP6SendStruct<
                'static,
                VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            >,
            capsules::net::ipv6::ipv6_send::IP6SendStruct::new(
                ip6_dg,
                ipsender_virtual_alarm,
                &mut RF233_BUF,
                sixlowpan_tx,
                udp_mac,
                self.dst_mac_addr,
                self.src_mac_addr,
            )
        );
        ipsender_virtual_alarm.set_client(ip_send);

        // Initially, set src IP of the sender to be the first IP in the Interface
        // list. Userland apps can change this if they so choose.
        // Notably, the src addr is the same regardless of if messages are sent from
        // userland or capsules.
        ip_send.set_addr(self.interface_list[0]);
        udp_mac.set_transmit_client(ip_send);

        let ip_receive = static_init!(
            capsules::net::ipv6::ipv6_recv::IP6RecvStruct<'static>,
            capsules::net::ipv6::ipv6_recv::IP6RecvStruct::new()
        );
        sixlowpan_state.set_rx_client(ip_receive);
        let udp_recv_mux = static_init!(MuxUdpReceiver<'static>, MuxUdpReceiver::new());
        ip_receive.set_client(udp_recv_mux);

        let udp_send_mux = static_init!(
            MuxUdpSender<
                'static,
                capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
                >,
            >,
            MuxUdpSender::new(ip_send)
        );
        ip_send.set_client(udp_send_mux);

        let create_table_cap = create_capability!(capabilities::CreatePortTableCapability);
        let udp_port_table = static_init!(
            UdpPortManager,
            UdpPortManager::new(&create_table_cap, &mut USED_KERNEL_PORTS)
        );

        (udp_send_mux, udp_recv_mux, udp_port_table)
    }
}
