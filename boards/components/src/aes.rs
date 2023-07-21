// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for various AES utilities.
//!
//! Usage
//! -----
//! ```rust
//! let aes_driver_device = components::aes::AesVirtualComponent::new(aes_mux).finalize(
//!     components::aes_virtual_component_static!(nrf52840::aes::AesECB<'static>),
//! );
//!
//! let aes = components::aes::AesDriverComponent::new(
//!     board_kernel,
//!     capsules_extra::symmetric_encryption::aes::DRIVER_NUM,
//!     aes_driver_device,
//! )
//! .finalize(components::aes_driver_component_static!(
//!     capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<
//!         'static,
//!         nrf52840::aes::AesECB<'static>,
//!     >
//! ));
//! ```

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::symmetric_encryption::{
    AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB, AES128GCM,
};

const CRYPT_SIZE: usize = 7 * hil::symmetric_encryption::AES128_BLOCK_SIZE;

#[macro_export]
macro_rules! aes_virtual_component_static {
    ($A:ty $(,)?) => {{
        const CRYPT_SIZE: usize = 7 * kernel::hil::symmetric_encryption::AES128_BLOCK_SIZE;
        let virtual_aes = kernel::static_buf!(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>
        );
        let crypt_buf = kernel::static_buf!([u8; CRYPT_SIZE]);

        (virtual_aes, crypt_buf)
    };};
}

#[macro_export]
macro_rules! aes_driver_component_static {
    ($A:ty $(,)?) => {{
        const CRYPT_SIZE: usize = 7 * kernel::hil::symmetric_encryption::AES128_BLOCK_SIZE;
        let aes_src_buffer = kernel::static_buf!([u8; 16]);
        let aes_dst_buffer = kernel::static_buf!([u8; CRYPT_SIZE]);
        let aes_driver =
            kernel::static_buf!(capsules_extra::symmetric_encryption::aes::AesDriver<'static, $A>);

        (aes_driver, aes_src_buffer, aes_dst_buffer)
    };};
}

pub struct AesVirtualComponent<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> {
    aes_mux: &'static capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>,
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> AesVirtualComponent<A> {
    pub fn new(
        aes_mux: &'static capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>,
    ) -> Self {
        Self { aes_mux }
    }
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> Component
    for AesVirtualComponent<A>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
        >,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
    );
    type Output =
        &'static capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let crypt_buf = static_buffer.1.write([0; CRYPT_SIZE]);
        let aes_ccm = static_buffer.0.write(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM::new(
                self.aes_mux,
                crypt_buf,
            ),
        );
        aes_ccm.setup();

        aes_ccm
    }
}

pub struct AesDriverComponent<A: AES128<'static> + AES128CCM<'static> + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    aes: &'static A,
}

impl<A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>>
    AesDriverComponent<A>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        aes: &'static A,
    ) -> AesDriverComponent<A> {
        AesDriverComponent {
            board_kernel,
            driver_num,
            aes,
        }
    }
}

impl<
        A: AES128<'static>
            + AES128Ctr
            + AES128CBC
            + AES128ECB
            + AES128CCM<'static>
            + AES128GCM<'static>,
    > Component for AesDriverComponent<A>
{
    type StaticInput = (
        &'static mut MaybeUninit<capsules_extra::symmetric_encryption::aes::AesDriver<'static, A>>,
        &'static mut MaybeUninit<[u8; 16]>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
    );
    type Output = &'static capsules_extra::symmetric_encryption::aes::AesDriver<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let aes_src_buf = static_buffer.1.write([0; 16]);
        let aes_dst_buf = static_buffer.2.write([0; CRYPT_SIZE]);

        let aes_driver =
            static_buffer
                .0
                .write(capsules_extra::symmetric_encryption::aes::AesDriver::new(
                    self.aes,
                    aes_src_buf,
                    aes_dst_buf,
                    self.board_kernel.create_grant(self.driver_num, &grant_cap),
                ));

        hil::symmetric_encryption::AES128CCM::set_client(self.aes, aes_driver);
        hil::symmetric_encryption::AES128::set_client(self.aes, aes_driver);

        aes_driver
    }
}
