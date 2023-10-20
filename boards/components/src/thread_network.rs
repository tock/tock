// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Component to initialize the Thread Network.
//!
//! This provides one Component, ThreadNetworkComponent. This component initializes
//! a Thread Network controller for maintaining and managing a Thread network.
//!
//! Usage
//! -----
//! ```rust
//!        let thread_driver = components::thread_network::ThreadNetworkComponent::new(
//!             board_kernel,
//!             capsules_extra::net::thread::driver::DRIVER_NUM,
//!             udp_send_mux,
//!             udp_recv_mux,
//!             udp_port_table,
//!             aes_mux,
//!             device_id,
//!             mux_alarm,
//!         )
//!         .finalize(components::thread_network_component_static!(
//!         nrf52840::rtc::Rtc,
//!         nrf52840::aes::AesECB<'static>
//!         ));
//! ```

use capsules_core;
use capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::net::ipv6::ipv6_send::IP6SendStruct;
use capsules_extra::net::network_capabilities::{
    AddrRange, NetworkCapability, PortRange, UdpVisibilityCapability,
};
use kernel::hil::symmetric_encryption::{self, AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB};

use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_extra::net::thread::thread_utils::THREAD_PORT_NUMBER;
use capsules_extra::net::udp::udp_port_table::UdpPortManager;
use capsules_extra::net::udp::udp_recv::MuxUdpReceiver;
use capsules_extra::net::udp::udp_recv::UDPReceiver;
use capsules_extra::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use core::mem::MaybeUninit;
use kernel;
use kernel::capabilities;
use kernel::capabilities::NetworkCapabilityCreationCapability;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::radio;
use kernel::hil::time::Alarm;

const MAX_PAYLOAD_LEN: usize = super::udp_mux::MAX_PAYLOAD_LEN;
pub const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;

// Setup static space for the objects.
#[macro_export]
macro_rules! thread_network_component_static {
    ($A:ty, $B:ty $(,)?) => {{
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
        let thread_network_driver = kernel::static_buf!(
            capsules_extra::net::thread::driver::ThreadNetworkDriver<
                'static,
                VirtualMuxAlarm<'static, $A>,
            >
        );
        let send_buffer = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);
        let recv_buffer = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);
        let udp_recv =
            kernel::static_buf!(capsules_extra::net::udp::udp_recv::UDPReceiver<'static>);
        let crypt_buf = kernel::static_buf!([u8; components::ieee802154::CRYPT_SIZE]);
        let crypt = kernel::static_buf!(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $B>,
        );
        let alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);

        (
            udp_send,
            udp_vis_cap,
            net_cap,
            thread_network_driver,
            send_buffer,
            recv_buffer,
            udp_recv,
            crypt_buf,
            crypt,
            alarm,
        )
    };};
}
pub struct ThreadNetworkComponent<
    A: Alarm<'static> + 'static,
    B: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    udp_send_mux:
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    aes_mux: &'static MuxAES128CCM<'static, B>,
    serial_num: [u8; 8],
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<
        A: Alarm<'static> + 'static,
        B: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + 'static,
    > ThreadNetworkComponent<A, B>
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
        aes_mux: &'static MuxAES128CCM<'static, B>,
        serial_num: [u8; 8],
        alarm_mux: &'static MuxAlarm<'static, A>,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            udp_send_mux,
            udp_recv_mux,
            port_table,
            aes_mux,
            serial_num,
            alarm_mux,
        }
    }
}

impl<
        A: Alarm<'static> + 'static,
        B: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + 'static,
    > Component for ThreadNetworkComponent<A, B>
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
        &'static mut MaybeUninit<
            capsules_extra::net::thread::driver::ThreadNetworkDriver<
                'static,
                VirtualMuxAlarm<'static, A>,
            >,
        >,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<UDPReceiver<'static>>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
        &'static mut MaybeUninit<
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, B>,
        >,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
    );
    type Output = &'static capsules_extra::net::thread::driver::ThreadNetworkDriver<
        'static,
        VirtualMuxAlarm<'static, A>,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let thread_virtual_alarm: &mut VirtualMuxAlarm<'_, A> =
            s.9.write(VirtualMuxAlarm::new(self.alarm_mux));
        thread_virtual_alarm.setup();

        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // AES-128CCM setup
        let crypt_buf = s.7.write([0; CRYPT_SIZE]);
        let aes_ccm = s.8.write(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM::new(
                self.aes_mux,
                crypt_buf,
            ),
        );
        aes_ccm.setup();

        let create_cap = create_capability!(NetworkCapabilityCreationCapability);
        let udp_vis = s.1.write(UdpVisibilityCapability::new(&create_cap));
        let udp_send = s.0.write(UDPSendStruct::new(self.udp_send_mux, udp_vis));

        // Can't use create_capability bc need capability to have a static lifetime
        // so that Thread driver can use it as needed
        struct DriverCap;
        unsafe impl capabilities::UdpDriverCapability for DriverCap {}
        static DRIVER_CAP: DriverCap = DriverCap;

        let net_cap = s.2.write(NetworkCapability::new(
            AddrRange::Any,
            PortRange::Any,
            PortRange::Any,
            &create_cap,
        ));

        let send_buffer = s.4.write([0; MAX_PAYLOAD_LEN]);
        let recv_buffer = s.5.write([0; MAX_PAYLOAD_LEN]);

        let thread_network_driver = s.3.write(
            capsules_extra::net::thread::driver::ThreadNetworkDriver::new(
                udp_send,
                aes_ccm,
                thread_virtual_alarm,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                self.serial_num,
                MAX_PAYLOAD_LEN,
                self.port_table,
                kernel::utilities::leasable_buffer::SubSliceMut::new(send_buffer),
                kernel::utilities::leasable_buffer::SubSliceMut::new(recv_buffer),
                &DRIVER_CAP,
                net_cap,
            ),
        );

        thread_virtual_alarm.set_alarm_client(thread_network_driver);

        udp_send.set_client(thread_network_driver);
        AES128CCM::set_client(aes_ccm, thread_network_driver);

        let udp_driver_rcvr = s.6.write(UDPReceiver::new());
        udp_driver_rcvr.set_client(thread_network_driver);

        // TODO: Thread requires port 19788 for sending/receiving MLE messages.
        // The below implementation binds Thread to the required port and updates
        // the UDP receiving/sending objects. There is a chance that creating a socket
        // fails due to the max number of sockets being exceeded or failing to bind
        // the requested port. In either case, the current implementation panics here
        // as it is impossible to create a Thread network without port 19788 (used for MLE).
        // Future implementations may wish to change this behavior.
        self.port_table
            .create_socket()
            .map(|socket| {
                self.port_table
                    .bind(socket, THREAD_PORT_NUMBER, net_cap)
                    .map_or_else(
                        |_| (),
                        |(tx_bind, rx_bind)| {
                            udp_driver_rcvr.set_binding(rx_bind);
                            udp_send.set_binding(tx_bind);
                        },
                    )
            })
            .unwrap();

        self.udp_recv_mux.add_client(udp_driver_rcvr);

        thread_network_driver
    }
}
