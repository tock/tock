//! Component to initialize the userland UDP driver.
//!
//! This provides one Component, UDPDriverComponent. This component initializes
//! a userspace UDP driver that allows apps to use the UDP stack.
//!
//! Usage
//! -----
//! ```rust
//!    let udp_driver = UDPDriverComponent::new(
//!        board_kernel,
//!        udp_send_mux,
//!        udp_recv_mux,
//!        udp_port_table,
//!        local_ip_ifaces,
//!        PAYLOAD_LEN,
//!     )
//!     .finalize(components::udp_driver_component_static!());
//! ```

use core::mem::MaybeUninit;
use core_capsules;
use core_capsules::virtual_alarm::VirtualMuxAlarm;
use extra_capsules::net::ipv6::ip_utils::IPAddr;
use extra_capsules::net::ipv6::ipv6_send::IP6SendStruct;
use extra_capsules::net::network_capabilities::{
    AddrRange, NetworkCapability, PortRange, UdpVisibilityCapability,
};
use extra_capsules::net::udp::udp_port_table::UdpPortManager;
use extra_capsules::net::udp::udp_recv::MuxUdpReceiver;
use extra_capsules::net::udp::udp_recv::UDPReceiver;
use extra_capsules::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use kernel;
use kernel::capabilities;
use kernel::capabilities::NetworkCapabilityCreationCapability;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time::Alarm;

const MAX_PAYLOAD_LEN: usize = super::udp_mux::MAX_PAYLOAD_LEN;

// Setup static space for the objects.
#[macro_export]
macro_rules! udp_driver_component_static {
    ($A:ty $(,)?) => {{
        use components::udp_mux::MAX_PAYLOAD_LEN;

        let udp_send = kernel::static_buf!(
            extra_capsules::net::udp::udp_send::UDPSendStruct<
                'static,
                extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                >,
            >
        );
        let udp_vis_cap =
            kernel::static_buf!(extra_capsules::net::network_capabilities::UdpVisibilityCapability);
        let net_cap =
            kernel::static_buf!(extra_capsules::net::network_capabilities::NetworkCapability);
        let udp_driver = kernel::static_buf!(extra_capsules::net::udp::UDPDriver<'static>);
        let buffer = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);
        let udp_recv =
            kernel::static_buf!(extra_capsules::net::udp::udp_recv::UDPReceiver<'static>);

        (udp_send, udp_vis_cap, net_cap, udp_driver, buffer, udp_recv)
    };};
}

pub struct UDPDriverComponent<A: Alarm<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    udp_send_mux:
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    interface_list: &'static [IPAddr],
}

impl<A: Alarm<'static>> UDPDriverComponent<A> {
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
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            udp_send_mux,
            udp_recv_mux,
            port_table,
            interface_list,
        }
    }
}

impl<A: Alarm<'static>> Component for UDPDriverComponent<A> {
    type StaticInput = (
        &'static mut MaybeUninit<
            UDPSendStruct<
                'static,
                extra_capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<
            extra_capsules::net::network_capabilities::UdpVisibilityCapability,
        >,
        &'static mut MaybeUninit<extra_capsules::net::network_capabilities::NetworkCapability>,
        &'static mut MaybeUninit<extra_capsules::net::udp::UDPDriver<'static>>,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<UDPReceiver<'static>>,
    );
    type Output = &'static extra_capsules::net::udp::UDPDriver<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        // TODO: change initialization below
        let create_cap = create_capability!(NetworkCapabilityCreationCapability);
        let udp_vis = s.1.write(UdpVisibilityCapability::new(&create_cap));
        let udp_send = s.0.write(UDPSendStruct::new(self.udp_send_mux, udp_vis));

        // Can't use create_capability bc need capability to have a static lifetime
        // so that UDP driver can use it as needed
        struct DriverCap;
        unsafe impl capabilities::UdpDriverCapability for DriverCap {}
        static DRIVER_CAP: DriverCap = DriverCap;

        let net_cap = s.2.write(NetworkCapability::new(
            AddrRange::Any,
            PortRange::Any,
            PortRange::Any,
            &create_cap,
        ));

        let buffer = s.4.write([0; MAX_PAYLOAD_LEN]);

        let udp_driver = s.3.write(extra_capsules::net::udp::UDPDriver::new(
            udp_send,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.interface_list,
            MAX_PAYLOAD_LEN,
            self.port_table,
            kernel::utilities::leasable_buffer::LeasableMutableBuffer::new(buffer),
            &DRIVER_CAP,
            net_cap,
        ));
        udp_send.set_client(udp_driver);
        self.port_table.set_user_ports(udp_driver, &DRIVER_CAP);

        let udp_driver_rcvr = s.5.write(UDPReceiver::new());
        self.udp_recv_mux.set_driver(udp_driver);
        self.udp_recv_mux.add_client(udp_driver_rcvr);
        udp_driver
    }
}
