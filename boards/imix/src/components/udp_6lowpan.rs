//! Component to initialize the udp/6lowpan interface on imix board.
//!
//! This provides one Component, UDPComponent, which implements a
//! userspace syscall interface to a full udp stack on top of 6lowpan
//!
//! Usage
//! -----
//! ```rust
//! let udp_driver = UDPComponent::new().finalize();
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::ieee802154::device::MacDevice;
use capsules::net::ipv6::ipv6::{IP6Packet, IPPayload, TransportHeader};
use capsules::net::ipv6::ipv6_recv::IP6Receiver;
use capsules::net::ipv6::ipv6_send::IP6Sender;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::sixlowpan::{sixlowpan_compression, sixlowpan_state};
use capsules::net::udp::udp::UDPHeader;
use capsules::net::udp::udp_recv::UDPReceiver;
use capsules::net::udp::udp_send::{UDPSendStruct, UDPSender};

use kernel;
use kernel::component::Component;
use sam4l;

pub struct UDPComponent {
    mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
}

impl UDPComponent {
    pub fn new(
        mux_mac: &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    ) -> UDPComponent {
        UDPComponent {
            mux_mac: mux_mac,
        }
    }
}

// Some constants for configuring the 6LoWPAN stack
const UDP_HDR_SIZE: usize = 8;
const PAYLOAD_LEN: usize = 200;
const DEFAULT_CTX_PREFIX_LEN: u8 = 8;
const DEFAULT_CTX_PREFIX: [u8; 16] = [0x0; 16];

//Source IP Address. TODO: Move somewhere else
const SRC_ADDR: IPAddr = IPAddr([
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
]);

// The UDP stack requires several packet buffers:
//
//   1. IP_BUF: buffer to hold full IP Packets before they are compressed by 6LoWPAN
//   2. SIXLOWPAN_RX_BUF: Buffer to hold full IP packets after they are decompressed by 6LoWPAN
//   3. UDP_BUF: Buffer to hold maximum sized UDP payload that can be passed to userspace
//   4. UDP_DGRAM: ???

static mut IP_BUF: [u8; 1280] = [0x00; 1280];
static mut SIXLOWPAN_RX_BUF: [u8; 1280] = [0x00; 1280];
static mut UDP_BUF: [u8; PAYLOAD_LEN] = [0x00; PAYLOAD_LEN];
static mut UDP_DGRAM: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];

impl Component for UDPComponent {
    type Output = &'static capsules::net::udp::UDPDriver<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {

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
                    prefix: DEFAULT_CTX_PREFIX,
                    prefix_len: DEFAULT_CTX_PREFIX_LEN,
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
        sixlowpan_tx.dst_pan.set(0xABCD);
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
                &mut IP_BUF,
                sixlowpan_tx,
                udp_mac
            )
        );
        ip_send.set_addr(SRC_ADDR);
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
                kernel::Grant::create(),
                &mut UDP_BUF
            )
        );
        udp_send.set_client(udp_driver);
        udp_recv.set_client(udp_driver);
        udp_driver
        }
}
