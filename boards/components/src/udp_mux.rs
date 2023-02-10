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
//!        MAX_PAYLOAD_LEN,
//!    )
//!    .finalize(components::udp_mux_component_static!());
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last Modified: 5/21/2019

use core::mem::MaybeUninit;
use core_capsules;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use extra_capsules::ieee802154::device::MacDevice;
use extra_capsules::net::ieee802154::MacAddress;
use extra_capsules::net::ipv6::ip_utils::IPAddr;
use extra_capsules::net::ipv6::ipv6_recv::IP6Receiver;
use extra_capsules::net::ipv6::ipv6_recv::IP6RecvStruct;
use extra_capsules::net::ipv6::ipv6_send::IP6SendStruct;
use extra_capsules::net::ipv6::ipv6_send::IP6Sender;
use extra_capsules::net::ipv6::{IP6Packet, IPPayload, TransportHeader};
use extra_capsules::net::network_capabilities::{IpVisibilityCapability, UdpVisibilityCapability};
use extra_capsules::net::sixlowpan::{sixlowpan_compression, sixlowpan_state};
use extra_capsules::net::udp::udp_port_table::{
    SocketBindingEntry, UdpPortManager, MAX_NUM_BOUND_PORTS,
};
use extra_capsules::net::udp::udp_recv::MuxUdpReceiver;
use extra_capsules::net::udp::udp_send::MuxUdpSender;
use extra_capsules::net::udp::UDPHeader;
use kernel;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::radio;
use kernel::hil::time::Alarm;

// The UDP stack requires several packet buffers:
//
//   1. RADIO_BUF: buffer the IP6_Sender uses to pass frames to the radio after fragmentation
//   2. SIXLOWPAN_RX_BUF: Buffer to hold full IP packets after they are decompressed by 6LoWPAN
//   3. UDP_DGRAM: The payload of the IP6_Packet, which holds full IP Packets before they are tx'd.
//
//   Additionally, every capsule using the stack needs an additional buffer to craft packets for
//   tx which can then be passed to the MuxUdpSender for tx.

pub const MAX_PAYLOAD_LEN: usize = 200; //The max size UDP message that can be sent by userspace apps or capsules

// Setup static space for the objects.
#[macro_export]
macro_rules! udp_mux_component_static {
    ($A:ty $(,)?) => {{
        use components::udp_mux::MAX_PAYLOAD_LEN;
        use core::mem::MaybeUninit;
        use core_capsules;
        use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
        use extra_capsules::net::sixlowpan::{sixlowpan_compression, sixlowpan_state};
        use extra_capsules::net::udp::udp_send::MuxUdpSender;

        let alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let mac_user =
            kernel::static_buf!(extra_capsules::ieee802154::virtual_mac::MacUser<'static>);
        let sixlowpan = kernel::static_buf!(
            sixlowpan_state::Sixlowpan<
                'static,
                VirtualMuxAlarm<'static, $A>,
                sixlowpan_compression::Context,
            >
        );
        let rx_state = kernel::static_buf!(sixlowpan_state::RxState<'static>);
        let ip6_send = kernel::static_buf!(
            extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                'static,
                VirtualMuxAlarm<'static, $A>,
            >
        );
        let mux_udp_send = kernel::static_buf!(
            MuxUdpSender<
                'static,
                extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, $A>,
                >,
            >
        );
        let mux_udp_recv =
            kernel::static_buf!(extra_capsules::net::udp::udp_recv::MuxUdpReceiver<'static>);
        let udp_port_manager =
            kernel::static_buf!(extra_capsules::net::udp::udp_port_table::UdpPortManager);

        let ip6_packet = kernel::static_buf!(extra_capsules::net::ipv6::IP6Packet<'static>);
        let ip6_receive =
            kernel::static_buf!(extra_capsules::net::ipv6::ipv6_recv::IP6RecvStruct<'static>);

        // Rather than require a data structure with 65535 slots (number of UDP ports),
        // we use a structure that can hold up to 16 port bindings. Any given capsule
        // can bind at most one port. When a capsule obtains a socket, it is assigned a
        // slot in this table. MAX_NUM_BOUND_PORTS represents the total number of
        // capsules that can bind to different ports simultaneously within the Tock
        // kernel.
        //
        // Each slot in the table tracks one socket that has been given to a capsule. If
        // no slots in the table are free, no slots remain to be given out. If a socket
        // is used to bind to a port, the port that is bound is saved in the slot to
        // ensure that subsequent bindings do not also attempt to bind that port number.
        let used_ports = kernel::static_buf!(
            [Option<extra_capsules::net::udp::udp_port_table::SocketBindingEntry>;
                extra_capsules::net::udp::udp_port_table::MAX_NUM_BOUND_PORTS]
        );

        let radio_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let sixlowpan_rx = kernel::static_buf!([u8; 1280]);
        let udp_dgram = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);

        let udp_vis_cap =
            kernel::static_buf!(extra_capsules::net::network_capabilities::UdpVisibilityCapability);
        let ip_vis_cap =
            kernel::static_buf!(extra_capsules::net::network_capabilities::IpVisibilityCapability);

        (
            alarm,
            mac_user,
            sixlowpan,
            rx_state,
            ip6_send,
            mux_udp_send,
            mux_udp_recv,
            udp_port_manager,
            ip6_packet,
            ip6_receive,
            used_ports,
            radio_buf,
            sixlowpan_rx,
            udp_dgram,
            udp_vis_cap,
            ip_vis_cap,
        )
    };};
}

pub struct UDPMuxComponent<A: Alarm<'static> + 'static> {
    mux_mac: &'static extra_capsules::ieee802154::virtual_mac::MuxMac<'static>,
    ctx_pfix_len: u8,
    ctx_pfix: [u8; 16],
    dst_mac_addr: MacAddress,
    src_mac_addr: MacAddress,
    interface_list: &'static [IPAddr],
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: Alarm<'static> + 'static> UDPMuxComponent<A> {
    pub fn new(
        mux_mac: &'static extra_capsules::ieee802154::virtual_mac::MuxMac<'static>,
        ctx_pfix_len: u8,
        ctx_pfix: [u8; 16],
        dst_mac_addr: MacAddress,
        src_mac_addr: MacAddress,
        interface_list: &'static [IPAddr],
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> Self {
        Self {
            mux_mac,
            ctx_pfix_len,
            ctx_pfix,
            dst_mac_addr,
            src_mac_addr,
            interface_list,
            alarm_mux,
        }
    }
}

impl<A: Alarm<'static> + 'static> Component for UDPMuxComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<extra_capsules::ieee802154::virtual_mac::MacUser<'static>>,
        &'static mut MaybeUninit<
            sixlowpan_state::Sixlowpan<
                'static,
                VirtualMuxAlarm<'static, A>,
                sixlowpan_compression::Context,
            >,
        >,
        &'static mut MaybeUninit<sixlowpan_state::RxState<'static>>,
        &'static mut MaybeUninit<
            extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                'static,
                VirtualMuxAlarm<'static, A>,
            >,
        >,
        &'static mut MaybeUninit<
            MuxUdpSender<
                'static,
                extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<MuxUdpReceiver<'static>>,
        &'static mut MaybeUninit<UdpPortManager>,
        &'static mut MaybeUninit<IP6Packet<'static>>,
        &'static mut MaybeUninit<IP6RecvStruct<'static>>,
        &'static mut MaybeUninit<[Option<SocketBindingEntry>; MAX_NUM_BOUND_PORTS]>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; 1280]>,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<UdpVisibilityCapability>,
        &'static mut MaybeUninit<IpVisibilityCapability>,
    );
    type Output = (
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
        &'static MuxUdpReceiver<'static>,
        &'static UdpPortManager,
    );

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let ipsender_virtual_alarm = s.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        ipsender_virtual_alarm.setup();

        let udp_mac =
            s.1.write(extra_capsules::ieee802154::virtual_mac::MacUser::new(
                self.mux_mac,
            ));
        self.mux_mac.add_user(udp_mac);
        let create_cap = create_capability!(capabilities::NetworkCapabilityCreationCapability);
        let udp_vis = s.14.write(UdpVisibilityCapability::new(&create_cap));
        let ip_vis = s.15.write(IpVisibilityCapability::new(&create_cap));

        let sixlowpan = s.2.write(sixlowpan_state::Sixlowpan::new(
            sixlowpan_compression::Context {
                prefix: self.ctx_pfix,
                prefix_len: self.ctx_pfix_len,
                id: 0,
                compress: false,
            },
            ipsender_virtual_alarm, // OK to reuse bc only used to get time, not set alarms
        ));

        let sixlowpan_rx_buffer = s.12.write([0; 1280]);
        let sixlowpan_state = sixlowpan as &dyn sixlowpan_state::SixlowpanState;
        let sixlowpan_tx = sixlowpan_state::TxState::new(sixlowpan_state);
        let default_rx_state =
            s.3.write(sixlowpan_state::RxState::new(sixlowpan_rx_buffer));
        sixlowpan_state.add_rx_state(default_rx_state);
        udp_mac.set_receive_client(sixlowpan);

        let udp_dgram_buffer = s.13.write([0; MAX_PAYLOAD_LEN]);
        let tr_hdr = TransportHeader::UDP(UDPHeader::new());
        let ip_pyld: IPPayload = IPPayload {
            header: tr_hdr,
            payload: udp_dgram_buffer,
        };
        let ip6_dg = s.8.write(IP6Packet::new(ip_pyld));

        let radio_buf = s.11.write([0; radio::MAX_BUF_SIZE]);

        // In current design, all udp senders share same IP sender, and the IP
        // sender holds the destination mac address. This means all UDP senders
        // must send to the same mac address...this works fine under the
        // assumption of all packets being routed via a single gateway router,
        // but doesn't work if multiple senders want to send to different
        // addresses on a local network. This will be fixed once we have an
        // ipv6_nd cache mapping IP addresses to dst macs
        let ip_send =
            s.4.write(extra_capsules::net::ipv6::ipv6_send::IP6SendStruct::new(
                ip6_dg,
                ipsender_virtual_alarm,
                radio_buf,
                sixlowpan_tx,
                udp_mac,
                self.dst_mac_addr,
                self.src_mac_addr,
                ip_vis,
            ));
        ipsender_virtual_alarm.set_alarm_client(ip_send);

        // Initially, set src IP of the sender to be the first IP in the
        // Interface list. Userland apps can change this if they so choose.
        // Notably, the src addr is the same regardless of if messages are sent
        // from userland or capsules.
        ip_send.set_addr(self.interface_list[0]);
        udp_mac.set_transmit_client(ip_send);

        let ip_receive =
            s.9.write(extra_capsules::net::ipv6::ipv6_recv::IP6RecvStruct::new());
        sixlowpan_state.set_rx_client(ip_receive);
        let udp_recv_mux = s.6.write(MuxUdpReceiver::new());
        ip_receive.set_client(udp_recv_mux);

        let udp_send_mux = s.5.write(MuxUdpSender::new(ip_send));
        ip_send.set_client(udp_send_mux);

        let kernel_ports = s.10.write([None; MAX_NUM_BOUND_PORTS]);
        let create_table_cap = create_capability!(capabilities::CreatePortTableCapability);
        let udp_port_table = s.7.write(UdpPortManager::new(
            &create_table_cap,
            kernel_ports,
            udp_vis,
        ));

        (udp_send_mux, udp_recv_mux, udp_port_table)
    }
}
