// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for IEEE 802.15.4 radio syscall interface.
//!
//! This provides one Component, `Ieee802154Component`, which implements a
//! userspace syscall interface to a full 802.15.4 stack with a always-on MAC
//! implementation, as well as multiplexed access to that MAC implementation.
//!
//! Usage
//! -----
//! ```rust
//! let aes_mux = components::ieee802154::MuxAes128ccmComponent::new(
//!     &base_peripherals.ecb,
//! )
//!  .finalize(components::mux_aes128ccm_component_static!(
//!     nrf52840::aes::AesECB
//! ));
//!
//! let (radio, mux_mac) = components::ieee802154::Ieee802154Component::new(
//!     board_kernel,
//!     capsules_extra::ieee802154::DRIVER_NUM,
//!     &nrf52::ieee802154_radio::RADIO,
//!     aes_mux,
//!     PAN_ID,
//!     SRC_MAC,
//!     deferred_caller,
//! )
//! .finalize(components::ieee802154_component_static!(
//!     nrf52::ieee802154_radio::Radio,
//!     nrf52::aes::AesECB<'static>
//! ));
//! ```

use capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM;
use capsules_extra::ieee802154::device::MacDevice;
use capsules_extra::ieee802154::mac::{AwakeMac, Mac};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::radio::{self, MAX_BUF_SIZE};
use kernel::hil::symmetric_encryption::{self, AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB};

// This buffer is used as an intermediate buffer for AES CCM encryption. An
// upper bound on the required size is `3 * BLOCK_SIZE + radio::MAX_BUF_SIZE`.
pub const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;

#[macro_export]
macro_rules! mux_aes128ccm_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, $A>)
    };};
}

pub type MuxAes128ccmComponentType<A> = MuxAES128CCM<'static, A>;

pub struct MuxAes128ccmComponent<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> {
    aes: &'static A,
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> MuxAes128ccmComponent<A> {
    pub fn new(aes: &'static A) -> Self {
        Self { aes }
    }
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> Component
    for MuxAes128ccmComponent<A>
{
    type StaticInput = &'static mut MaybeUninit<MuxAES128CCM<'static, A>>;
    type Output = &'static MuxAES128CCM<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let aes_mux = static_buffer.write(MuxAES128CCM::new(self.aes));
        kernel::deferred_call::DeferredCallClient::register(aes_mux);
        self.aes.set_client(aes_mux);

        aes_mux
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! ieee802154_component_static {
    ($R:ty, $A:ty $(,)?) => {{
        let virtual_aes = kernel::static_buf!(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>
        );
        let awake_mac = kernel::static_buf!(capsules_extra::ieee802154::mac::AwakeMac<'static, $R>);
        let framer = kernel::static_buf!(
            capsules_extra::ieee802154::framer::Framer<
                'static,
                capsules_extra::ieee802154::mac::AwakeMac<'static, $R>,
                capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>,
            >
        );

        let mux_mac = kernel::static_buf!(
            capsules_extra::ieee802154::virtual_mac::MuxMac<
                'static,
                capsules_extra::ieee802154::framer::Framer<
                    'static,
                    capsules_extra::ieee802154::mac::AwakeMac<'static, $R>,
                    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>,
                >,
            >
        );
        let mac_user = kernel::static_buf!(
            capsules_extra::ieee802154::virtual_mac::MacUser<
                'static,
                capsules_extra::ieee802154::framer::Framer<
                    'static,
                    capsules_extra::ieee802154::mac::AwakeMac<'static, $R>,
                    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>,
                >,
            >
        );
        let radio_driver = kernel::static_buf!(
            capsules_extra::ieee802154::RadioDriver<
                'static,
                capsules_extra::ieee802154::virtual_mac::MacUser<
                    'static,
                    capsules_extra::ieee802154::framer::Framer<
                        'static,
                        capsules_extra::ieee802154::mac::AwakeMac<'static, $R>,
                        capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>,
                    >,
                >,
            >
        );

        let radio_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let radio_rx_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let crypt_buf = kernel::static_buf!([u8; components::ieee802154::CRYPT_SIZE]);
        let radio_rx_crypt_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);

        (
            virtual_aes,
            awake_mac,
            framer,
            mux_mac,
            mac_user,
            radio_driver,
            radio_buf,
            radio_rx_buf,
            crypt_buf,
            radio_rx_crypt_buf,
        )
    };};
}

pub type Ieee802154ComponentType<R, A> = capsules_extra::ieee802154::RadioDriver<
    'static,
    capsules_extra::ieee802154::virtual_mac::MacUser<
        'static,
        capsules_extra::ieee802154::framer::Framer<
            'static,
            capsules_extra::ieee802154::mac::AwakeMac<'static, R>,
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
        >,
    >,
>;

pub type Ieee802154ComponentMacDeviceType<R, A> = capsules_extra::ieee802154::framer::Framer<
    'static,
    capsules_extra::ieee802154::mac::AwakeMac<'static, R>,
    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
>;

pub struct Ieee802154Component<
    R: 'static + kernel::hil::radio::Radio<'static>,
    A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    radio: &'static R,
    aes_mux: &'static MuxAES128CCM<'static, A>,
    pan_id: capsules_extra::net::ieee802154::PanID,
    short_addr: u16,
    long_addr: [u8; 8],
}

impl<
        R: 'static + kernel::hil::radio::Radio<'static>,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
    > Ieee802154Component<R, A>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        radio: &'static R,
        aes_mux: &'static MuxAES128CCM<'static, A>,
        pan_id: capsules_extra::net::ieee802154::PanID,
        short_addr: u16,
        long_addr: [u8; 8],
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            radio,
            aes_mux,
            pan_id,
            short_addr,
            long_addr,
        }
    }
}

impl<
        R: 'static + kernel::hil::radio::Radio<'static>,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
    > Component for Ieee802154Component<R, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
        >,
        &'static mut MaybeUninit<capsules_extra::ieee802154::mac::AwakeMac<'static, R>>,
        &'static mut MaybeUninit<
            capsules_extra::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, R>,
                capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
            >,
        >,
        &'static mut MaybeUninit<
            capsules_extra::ieee802154::virtual_mac::MuxMac<
                'static,
                capsules_extra::ieee802154::framer::Framer<
                    'static,
                    AwakeMac<'static, R>,
                    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<
            capsules_extra::ieee802154::virtual_mac::MacUser<
                'static,
                capsules_extra::ieee802154::framer::Framer<
                    'static,
                    AwakeMac<'static, R>,
                    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<
            capsules_extra::ieee802154::RadioDriver<
                'static,
                capsules_extra::ieee802154::virtual_mac::MacUser<
                    'static,
                    capsules_extra::ieee802154::framer::Framer<
                        'static,
                        AwakeMac<'static, R>,
                        capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
                    >,
                >,
            >,
        >,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
    );
    type Output = (
        &'static capsules_extra::ieee802154::RadioDriver<
            'static,
            capsules_extra::ieee802154::virtual_mac::MacUser<
                'static,
                capsules_extra::ieee802154::framer::Framer<
                    'static,
                    AwakeMac<'static, R>,
                    capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
                >,
            >,
        >,
        &'static capsules_extra::ieee802154::virtual_mac::MuxMac<
            'static,
            capsules_extra::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, R>,
                capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
            >,
        >,
    );

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let crypt_buf = static_buffer.8.write([0; CRYPT_SIZE]);
        let aes_ccm = static_buffer.0.write(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM::new(
                self.aes_mux,
                crypt_buf,
            ),
        );
        aes_ccm.setup();

        // Keeps the radio on permanently; pass-through layer.
        let radio_rx_buf = static_buffer.7.write([0; radio::MAX_BUF_SIZE]);
        let awake_mac = static_buffer.1.write(AwakeMac::new(self.radio));
        self.radio.set_transmit_client(awake_mac);
        self.radio.set_receive_client(awake_mac);
        self.radio.set_receive_buffer(radio_rx_buf);

        let radio_rx_crypt_buf = static_buffer.9.write([0; MAX_BUF_SIZE]);

        let mac_device = static_buffer
            .2
            .write(capsules_extra::ieee802154::framer::Framer::new(
                awake_mac,
                aes_ccm,
                kernel::utilities::leasable_buffer::SubSliceMut::new(radio_rx_crypt_buf),
            ));
        AES128CCM::set_client(aes_ccm, mac_device);
        awake_mac.set_transmit_client(mac_device);
        awake_mac.set_receive_client(mac_device);
        awake_mac.set_config_client(mac_device);

        let mux_mac = static_buffer
            .3
            .write(capsules_extra::ieee802154::virtual_mac::MuxMac::new(
                mac_device,
            ));
        mac_device.set_transmit_client(mux_mac);
        mac_device.set_receive_client(mux_mac);

        let userspace_mac =
            static_buffer
                .4
                .write(capsules_extra::ieee802154::virtual_mac::MacUser::new(
                    mux_mac,
                ));
        mux_mac.add_user(userspace_mac);

        let radio_buffer = static_buffer.6.write([0; radio::MAX_BUF_SIZE]);
        let radio_driver = static_buffer
            .5
            .write(capsules_extra::ieee802154::RadioDriver::new(
                userspace_mac,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                radio_buffer,
            ));
        kernel::deferred_call::DeferredCallClient::register(radio_driver);

        mac_device.set_key_procedure(radio_driver);
        mac_device.set_device_procedure(radio_driver);
        userspace_mac.set_transmit_client(radio_driver);
        userspace_mac.set_receive_client(radio_driver);
        userspace_mac.set_pan(self.pan_id);
        userspace_mac.set_address(self.short_addr);
        userspace_mac.set_address_long(self.long_addr);

        (radio_driver, mux_mac)
    }
}

// IEEE 802.15.4 RAW DRIVER

// Setup static space for the objects.
#[macro_export]
macro_rules! ieee802154_raw_component_static {
    ($R:ty $(,)?) => {{
        let radio_driver =
            kernel::static_buf!(capsules_extra::ieee802154::phy_driver::RadioDriver<$R>);
        let tx_buffer = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let rx_buffer = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);

        (radio_driver, tx_buffer, rx_buffer)
    };};
}

pub type Ieee802154RawComponentType<R> =
    capsules_extra::ieee802154::phy_driver::RadioDriver<'static, R>;

pub struct Ieee802154RawComponent<R: 'static + kernel::hil::radio::Radio<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    radio: &'static R,
}

impl<R: 'static + kernel::hil::radio::Radio<'static>> Ieee802154RawComponent<R> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        radio: &'static R,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            radio,
        }
    }
}

impl<R: 'static + kernel::hil::radio::Radio<'static>> Component for Ieee802154RawComponent<R> {
    type StaticInput = (
        &'static mut MaybeUninit<capsules_extra::ieee802154::phy_driver::RadioDriver<'static, R>>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
    );
    type Output = &'static capsules_extra::ieee802154::phy_driver::RadioDriver<'static, R>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let tx_buffer = static_buffer.1.write([0; MAX_BUF_SIZE]);
        let radio_rx_buf = static_buffer.2.write([0; radio::MAX_BUF_SIZE]);

        let radio_driver =
            static_buffer
                .0
                .write(capsules_extra::ieee802154::phy_driver::RadioDriver::new(
                    self.radio,
                    self.board_kernel.create_grant(self.driver_num, &grant_cap),
                    tx_buffer,
                ));

        self.radio.set_transmit_client(radio_driver);
        self.radio.set_receive_client(radio_driver);
        self.radio.set_receive_buffer(radio_rx_buf);

        radio_driver
    }
}
