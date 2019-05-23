//! Component to initialize the userland UDP driver.
//!
//! This provides one Component, UDPDriverComponent. This component
//!
//! Usage
//! -----
//! ```rust
//!    let udp_driver = UDPDriverComponent::new(
//!        board_kernel,
//!        udp_mux,
//!        udp_recv,
//!        udp_port_table,
//!        local_ip_ifaces,
//!        PAYLOAD_LEN,
//!     )
//!     .finalize();
//! ```

// Author: Hudson Ayers <hayers@stanford.edu>
// Last Modified: 5/21/2019

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::net::ipv6::ip_utils::IPAddr;
use capsules::net::ipv6::ipv6_send::IP6SendStruct;
use capsules::net::udp::udp_recv::UDPReceiver;
use capsules::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::udp_port_table::UdpPortTable;
use kernel::{create_capability, static_init};

use kernel;
use kernel::capabilities;
use kernel::component::Component;
use sam4l;

const PAYLOAD_LEN: usize = 200; //The max size UDP message that can be sent by userspace apps or capsules
const UDP_HDR_SIZE: usize = 8;

static mut DRIVER_BUF: [u8; PAYLOAD_LEN - UDP_HDR_SIZE] = [0; PAYLOAD_LEN - UDP_HDR_SIZE];

pub struct UDPDriverComponent {
    board_kernel: &'static kernel::Kernel,
    udp_mux: &'static MuxUdpSender<
        'static,
        IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
    >,
    udp_recv: &'static UDPReceiver<'static>,
    port_table: &'static UdpPortTable,
    interface_list: &'static [IPAddr],
    payload_len: usize,
}

impl UDPDriverComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        udp_mux: &'static MuxUdpSender<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>>,
        >,
        udp_recv: &'static UDPReceiver<'static>,
        port_table: &'static UdpPortTable,
        interface_list: &'static [IPAddr],
        payload_len: usize,
    ) -> UDPDriverComponent {
        UDPDriverComponent {
            board_kernel: board_kernel,
            udp_mux: udp_mux,
            udp_recv: udp_recv,
            port_table: port_table,
            interface_list: interface_list,
            payload_len: payload_len,
        }
    }
}

impl Component for UDPDriverComponent {
    type Output = &'static capsules::net::udp::UDPDriver<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let udp_send = static_init!(
            UDPSendStruct<
                'static,
                capsules::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
                >,
            >,
            UDPSendStruct::new(self.udp_mux)
        );

        let udp_driver = static_init!(
            capsules::net::udp::UDPDriver<'static>,
            capsules::net::udp::UDPDriver::new(
                udp_send,
                self.udp_recv,
                self.board_kernel.create_grant(&grant_cap),
                self.interface_list,
                self.payload_len,
                self.port_table,
                &mut DRIVER_BUF,
            )
        );
        udp_send.set_client(udp_driver);
        self.udp_recv.set_client(udp_driver);
        self.port_table.set_user_ports(udp_driver);

        udp_driver
    }
}
