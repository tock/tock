//! Component to test in kernel udp.
//!
//! Duplicate of mock_udp.rs. Can't call original component twice because it uses static_init!()
//! so have to rely on duplicate files.

// Author: Hudson Ayers <hayers@stanford.edu>

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::net::ipv6::ipv6_send::IP6SendStruct;
use capsules::net::network_capabilities::{NetworkCapability, UdpVisibilityCapability};
use capsules::net::udp::udp_recv::{MuxUdpReceiver, UDPReceiver};
use capsules::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

use capsules::net::udp::udp_port_table::UdpPortManager;
use kernel::common::cells::TakeCell;
use kernel::component::Component;
use kernel::hil::time::Alarm;
use kernel::static_init;

pub struct MockUDPComponent2 {
    // TODO: consider putting bound_port_table in a TakeCell
    udp_send_mux: &'static MuxUdpSender<
        'static,
        IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    >,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    bound_port_table: &'static UdpPortManager,
    alarm_mux: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
    udp_payload: TakeCell<'static, [u8]>,
    id: u16,
    dst_port: u16,
    net_cap: &'static NetworkCapability,
    udp_vis: &'static UdpVisibilityCapability,
}

impl MockUDPComponent2 {
    pub fn new(
        udp_send_mux: &'static MuxUdpSender<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        >,
        udp_recv_mux: &'static MuxUdpReceiver<'static>,
        bound_port_table: &'static UdpPortManager,
        alarm: &'static MuxAlarm<'static, sam4l::ast::Ast<'static>>,
        udp_payload: &'static mut [u8],
        id: u16,
        dst_port: u16,
        net_cap: &'static NetworkCapability,
        udp_vis: &'static UdpVisibilityCapability,
    ) -> MockUDPComponent2 {
        MockUDPComponent2 {
            udp_send_mux: udp_send_mux,
            udp_recv_mux: udp_recv_mux,
            bound_port_table: bound_port_table,
            alarm_mux: alarm,
            udp_payload: TakeCell::new(udp_payload),
            id: id,
            dst_port: dst_port,
            net_cap: net_cap,
            udp_vis: udp_vis,
        }
    }
}

impl Component for MockUDPComponent2 {
    type StaticInput = ();
    type Output = &'static capsules::test::udp::MockUdp<
        'static,
        VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
    >;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let udp_send = static_init!(
            UDPSendStruct<
                'static,
                capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
                >,
            >,
            UDPSendStruct::new(self.udp_send_mux, self.udp_vis)
        );

        let udp_recv = static_init!(UDPReceiver<'static>, UDPReceiver::new());
        self.udp_recv_mux.add_client(udp_recv);

        let udp_alarm = static_init!(
            VirtualMuxAlarm<'static, sam4l::ast::Ast>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let mock_udp = static_init!(
            capsules::test::udp::MockUdp<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
            capsules::test::udp::MockUdp::new(
                self.id,
                udp_alarm,
                udp_send,
                udp_recv,
                self.bound_port_table,
                kernel::common::leasable_buffer::LeasableBuffer::new(
                    self.udp_payload.take().expect("missing payload")
                ),
                self.dst_port,
                self.net_cap,
            )
        );
        udp_send.set_client(mock_udp);
        udp_recv.set_client(mock_udp);
        udp_alarm.set_client(mock_udp);
        mock_udp
    }
}
