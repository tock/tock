//! Component to initialize the udp/6lowpan interface on imix board.
//!
//! This provides one Component, UDPComponent, which implements a
//! userspace syscall interface to a full udp stack on top of 6lowpan
//!
//! Usage
//! -----
//! ```rust
//! let udp_driver = UDPComponent::new(mux_mac,
//!                                    DEFAULT_CTX_PREFIX_LEN,
//!                                    DEFAULT_CTX_PREFIX,
//!                                    DST_MAC_ADDR,
//!                                    &LOCAL_IP_IFACES).finalize();
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last Modified: 8/26/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::ieee802154::device::MacDevice;
use capsules::net::ieee802154::MacAddress;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::ipv6::ipv6::{IP6Packet, IPPayload, TransportHeader};
use capsules::net::ipv6::ipv6_recv::IP6Receiver;
use capsules::net::ipv6::ipv6_send::IP6Sender;
use capsules::net::sixlowpan::{sixlowpan_compression, sixlowpan_state};
use capsules::net::udp::udp::UDPHeader;
use capsules::net::udp::udp_recv::UDPReceiver;
use capsules::net::udp::udp_send::{UDPSendStruct, UDPSender};

use kernel;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::radio;
use sam4l;

const PAYLOAD_LEN: usize = 200; //The max size UDP message that can be sent by userland apps

// The UDP stack requires several packet buffers:
//
//   1. RF233_BUF: buffer the IP6_Sender uses to pass frames to the radio after fragmentation
//   2. SIXLOWPAN_RX_BUF: Buffer to hold full IP packets after they are decompressed by 6LoWPAN
//   3. UDP_DGRAM: The payload of the IP6_Packet, which holds full IP Packets before they are tx'd

const UDP_HDR_SIZE: usize = 8;
static mut RF233_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];
static mut SIXLOWPAN_RX_BUF: [u8; 1280] = [0x00; 1280];
static mut UDP_DGRAM: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];

pub struct UDPComponent {
    board_kernel: &'static kernel::Kernel,
    mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    ctx_pfix_len: u8,
    ctx_pfix: [u8; 16],
    dst_mac_addr: MacAddress,
    src_mac_addr: MacAddress,
    interface_list: &'static [IPAddr],
}

impl UDPComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
        ctx_pfix_len: u8,
        ctx_pfix: [u8; 16],
        dst_mac_addr: MacAddress,
        src_mac_addr: MacAddress,
        interface_list: &'static [IPAddr],
    ) -> UDPComponent {
        UDPComponent {
            board_kernel: board_kernel,
            mux_mac: mux_mac,
            ctx_pfix_len: ctx_pfix_len,
            ctx_pfix: ctx_pfix,
            dst_mac_addr: dst_mac_addr,
            src_mac_addr: src_mac_addr,
            interface_list: interface_list,
        }
    }
}

impl Component for UDPComponent {
    type Output = &'static capsules::net::udp::UDPDriver<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

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

        let sixlowpan_state = sixlowpan as &sixlowpan_state::SixlowpanState;
        let sixlowpan_tx = sixlowpan_state::TxState::new(sixlowpan_state);
        let default_rx_state = static_init!(
            sixlowpan_state::RxState<'static>,
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

        let ip_send = static_init!(
            capsules::net::ipv6::ipv6_send::IP6SendStruct<'static>,
            capsules::net::ipv6::ipv6_send::IP6SendStruct::new(
                ip6_dg,
                &mut RF233_BUF,
                sixlowpan_tx,
                udp_mac,
                self.dst_mac_addr,
                self.src_mac_addr
            )
        );

        // Initially, set src IP of the sender to be the first IP in the Interface
        // list. Userland apps can change this if they so choose.
        ip_send.set_addr(self.interface_list[0]);
        udp_mac.set_transmit_client(ip_send);

        let udp_send = static_init!(
            UDPSendStruct<'static, capsules::net::ipv6::ipv6_send::IP6SendStruct<'static>>,
            UDPSendStruct::new(ip_send)
        );
        ip_send.set_client(udp_send);

        let ip_receive = static_init!(
            capsules::net::ipv6::ipv6_recv::IP6RecvStruct<'static>,
            capsules::net::ipv6::ipv6_recv::IP6RecvStruct::new()
        );
        sixlowpan_state.set_rx_client(ip_receive);

        let udp_recv = static_init!(UDPReceiver<'static>, UDPReceiver::new());
        ip_receive.set_client(udp_recv);

        let udp_driver = static_init!(
            capsules::net::udp::UDPDriver<'static>,
            capsules::net::udp::UDPDriver::new(
                udp_send,
                udp_recv,
                self.board_kernel.create_grant(&grant_cap),
                self.interface_list,
                PAYLOAD_LEN
            )
        );
        udp_send.set_client(udp_driver);
        udp_recv.set_client(udp_driver);
        udp_driver
    }
}
