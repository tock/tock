//! Component to initialize the userland UDP driver.
//!
//! This provides one Component, UDPDriverComponent. This component initializes a userspace
//! UDP driver that allows apps to use the UDP stack.
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
//!     .finalize();
//! ```

use capsules;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::ipv6::ipv6_send::IP6SendStruct;
use capsules::net::network_capabilities::{
    AddrRange, NetworkCapability, PortRange, UdpVisibilityCapability,
};
use capsules::net::udp::udp_port_table::UdpPortManager;
use capsules::net::udp::udp_recv::MuxUdpReceiver;
use capsules::net::udp::udp_recv::UDPReceiver;
use capsules::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use capsules::virtual_alarm::VirtualMuxAlarm;
use core::mem::MaybeUninit;
use kernel;
use kernel::capabilities;
use kernel::capabilities::NetworkCapabilityCreationCapability;
use kernel::component::Component;
use kernel::hil::time::Alarm;
use kernel::{create_capability, static_init, static_init_half};

const UDP_HDR_SIZE: usize = 8;
const MAX_PAYLOAD_LEN: usize = super::udp_mux::MAX_PAYLOAD_LEN;

static mut DRIVER_BUF: [u8; MAX_PAYLOAD_LEN - UDP_HDR_SIZE] = [0; MAX_PAYLOAD_LEN - UDP_HDR_SIZE];

// Setup static space for the objects.
#[macro_export]
macro_rules! udp_driver_component_helper {
    ($A:ty) => {{
        use capsules::net::udp::udp_send::UDPSendStruct;
        use core::mem::MaybeUninit;
        static mut BUF0: MaybeUninit<
            UDPSendStruct<
                'static,
                capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, $A>,
                >,
            >,
        > = MaybeUninit::uninit();
        (&mut BUF0,)
    };};
}

pub struct UDPDriverComponent<A: Alarm<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    udp_send_mux:
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    interface_list: &'static [IPAddr],
}

impl<A: Alarm<'static>> UDPDriverComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
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
                capsules::net::ipv6::ipv6_send::IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>,
            >,
        >,
    );
    type Output = &'static capsules::net::udp::UDPDriver<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        // TODO: change initialization below
        let create_cap = create_capability!(NetworkCapabilityCreationCapability);
        let udp_vis = static_init!(
            UdpVisibilityCapability,
            UdpVisibilityCapability::new(&create_cap)
        );
        let udp_send = static_init_half!(
            static_buffer.0,
            UDPSendStruct<
                'static,
                capsules::net::ipv6::ipv6_send::IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>,
            >,
            UDPSendStruct::new(self.udp_send_mux, udp_vis)
        );

        // Can't use create_capability bc need capability to have a static lifetime
        // so that UDP driver can use it as needed
        struct DriverCap;
        unsafe impl capabilities::UdpDriverCapability for DriverCap {}
        static DRIVER_CAP: DriverCap = DriverCap;

        let net_cap = static_init!(
            NetworkCapability,
            NetworkCapability::new(AddrRange::Any, PortRange::Any, PortRange::Any, &create_cap)
        );

        let udp_driver = static_init!(
            capsules::net::udp::UDPDriver<'static>,
            capsules::net::udp::UDPDriver::new(
                udp_send,
                self.board_kernel.create_grant(&grant_cap),
                self.interface_list,
                MAX_PAYLOAD_LEN,
                self.port_table,
                kernel::common::leasable_buffer::LeasableBuffer::new(&mut DRIVER_BUF),
                &DRIVER_CAP,
                net_cap,
            )
        );
        udp_send.set_client(udp_driver);
        self.port_table.set_user_ports(udp_driver, &DRIVER_CAP);

        let udp_driver_rcvr = static_init!(UDPReceiver<'static>, UDPReceiver::new());
        self.udp_recv_mux.set_driver(udp_driver);
        self.udp_recv_mux.add_client(udp_driver_rcvr);
        udp_driver
    }
}
