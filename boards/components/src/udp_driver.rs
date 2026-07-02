// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component to initialize the userland UDP driver.
//!
//! This provides one Component, UDPDriverComponent. This component initializes
//! a userspace UDP driver that allows apps to use the UDP stack.
//!
//! Usage
//! -----
//! ```rust
//! kernel::declare_capability!(UdpDriverCap: kernel::capabilities::UdpDriverCapability);
//! let udp_driver = UDPDriverComponent::new(
//!     board_kernel,
//!     udp_send_mux,
//!     udp_recv_mux,
//!     udp_port_table,
//!     local_ip_ifaces,
//!     PAYLOAD_LEN,
//!     UdpDriverCap,
//! )
//! .finalize(components::udp_driver_component_static!(AlarmType, UdpDriverCap));
//! ```

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::net::ipv6::ip_utils::IPAddr;
use capsules_extra::net::ipv6::ipv6_send::IP6SendStruct;
use capsules_extra::net::network_capabilities::{
    AddrRange, NetworkCapability, PortRange, UdpVisibilityCapability,
};
use capsules_extra::net::udp::udp_port_table::UdpPortManager;
use capsules_extra::net::udp::udp_recv::MuxUdpReceiver;
use capsules_extra::net::udp::udp_recv::UDPReceiver;
use capsules_extra::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use core::mem::MaybeUninit;
use kernel::capabilities::{
    MemoryAllocationCapability, NetworkCapabilityCreationCapability, UdpDriverCapability,
};
use kernel::component::Component;
use kernel::hil::time::Alarm;

const MAX_PAYLOAD_LEN: usize = super::udp_mux::MAX_PAYLOAD_LEN;

// Setup static space for the objects.
#[macro_export]
macro_rules! udp_driver_component_static {
    ($A:ty, $C:ty $(,)?) => {{
        use components::udp_mux::MAX_PAYLOAD_LEN;

        let udp_send = kernel::static_buf!(
            capsules_extra::net::udp::udp_send::UDPSendStruct<
                'static,
                capsules_extra::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                >,
            >
        );
        let udp_vis_cap =
            kernel::static_buf!(capsules_extra::net::network_capabilities::UdpVisibilityCapability);
        let net_cap =
            kernel::static_buf!(capsules_extra::net::network_capabilities::NetworkCapability);
        let udp_driver = kernel::static_buf!(capsules_extra::net::udp::UDPDriver<'static>);
        let buffer = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);
        let udp_recv =
            kernel::static_buf!(capsules_extra::net::udp::udp_recv::UDPReceiver<'static>);
        let driver_cap = kernel::static_buf!($C);

        (
            udp_send,
            udp_vis_cap,
            net_cap,
            udp_driver,
            buffer,
            udp_recv,
            driver_cap,
        )
    };};
}

pub type UDPDriverComponentType = capsules_extra::net::udp::UDPDriver<'static>;

pub struct UDPDriverComponent<
    A: Alarm<'static> + 'static,
    C: UdpDriverCapability + 'static,
    MEM: MemoryAllocationCapability + 'static,
    NET: NetworkCapabilityCreationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    udp_send_mux:
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    interface_list: &'static [IPAddr],
    driver_cap: C,
    mem_cap: MEM,
    create_cap: NET,
}

impl<
        A: Alarm<'static>,
        C: UdpDriverCapability + 'static,
        MEM: MemoryAllocationCapability + 'static,
        NET: NetworkCapabilityCreationCapability + 'static,
    > UDPDriverComponent<A, C, MEM, NET>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        udp_send_mux: &'static MuxUdpSender<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>,
        >,
        udp_recv_mux: &'static MuxUdpReceiver<'static>,
        port_table: &'static UdpPortManager,
        interface_list: &'static [IPAddr],
        driver_cap: C,
        mem_cap: MEM,
        create_cap: NET,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            udp_send_mux,
            udp_recv_mux,
            port_table,
            interface_list,
            driver_cap,
            mem_cap,
            create_cap,
        }
    }
}

impl<
        A: Alarm<'static>,
        C: UdpDriverCapability + 'static,
        MEM: MemoryAllocationCapability + 'static,
        NET: NetworkCapabilityCreationCapability + 'static,
    > Component for UDPDriverComponent<A, C, MEM, NET>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            UDPSendStruct<
                'static,
                capsules_extra::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<
            capsules_extra::net::network_capabilities::UdpVisibilityCapability,
        >,
        &'static mut MaybeUninit<capsules_extra::net::network_capabilities::NetworkCapability>,
        &'static mut MaybeUninit<capsules_extra::net::udp::UDPDriver<'static>>,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<UDPReceiver<'static>>,
        &'static mut MaybeUninit<C>,
    );
    type Output = &'static capsules_extra::net::udp::UDPDriver<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let udp_vis = s.1.write(UdpVisibilityCapability::new(&self.create_cap));
        let udp_send = s.0.write(UDPSendStruct::new(self.udp_send_mux, udp_vis));

        let driver_cap: &'static C = s.6.write(self.driver_cap);

        let net_cap = s.2.write(NetworkCapability::new(
            AddrRange::Any,
            PortRange::Any,
            PortRange::Any,
            &self.create_cap,
        ));

        let buffer = s.4.write([0; MAX_PAYLOAD_LEN]);

        let udp_driver = s.3.write(capsules_extra::net::udp::UDPDriver::new(
            udp_send,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            self.interface_list,
            MAX_PAYLOAD_LEN,
            self.port_table,
            kernel::utilities::leasable_buffer::SubSliceMut::new(buffer),
            driver_cap,
            net_cap,
        ));
        udp_send.set_client(udp_driver);
        self.port_table.set_user_ports(udp_driver, driver_cap);

        let udp_driver_rcvr = s.5.write(UDPReceiver::new());
        self.udp_recv_mux.set_driver(udp_driver);
        self.udp_recv_mux.add_client(udp_driver_rcvr);
        udp_driver
    }
}
