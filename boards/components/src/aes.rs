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

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil;
use kernel::hil::symmetric_encryption::{
    AES, AES128, AESCBC, AESCCM, AESCtr, AESECB, AESGCM, AESKeySize,
};

const CRYPT_SIZE: usize = 7 * hil::symmetric_encryption::AES_BLOCK_SIZE;

#[macro_export]
macro_rules! aes_mux_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, $A>)
    };};
}

pub type AesMuxComponentType<A> =
    capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>;

pub struct AesMuxComponent<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> {
    aes: &'static A,
}

impl<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> AesMuxComponent<A> {
    pub fn new(aes: &'static A) -> Self {
        Self { aes }
    }
}

impl<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> Component
    for AesMuxComponent<A>
{
    type StaticInput = &'static mut MaybeUninit<
        capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>,
    >;
    type Output = &'static capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let aes_mux = static_buffer
            .write(capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM::new(self.aes));

        DeferredCallClient::register(aes_mux);
        hil::symmetric_encryption::AES::set_client(self.aes, aes_mux);

        aes_mux
    }
}

#[macro_export]
macro_rules! aes_virtual_component_static {
    ($A:ty $(,)?) => {{
        const CRYPT_SIZE: usize = 7 * kernel::hil::symmetric_encryption::AES_BLOCK_SIZE;
        let virtual_aes = kernel::static_buf!(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $A>
        );
        let crypt_buf = kernel::static_buf!([u8; CRYPT_SIZE]);

        (virtual_aes, crypt_buf)
    };};
}

#[macro_export]
macro_rules! aes_driver_component_static {
    ($A:ty, $K:ty $(,)?) => {{
        const CRYPT_SIZE: usize = 7 * kernel::hil::symmetric_encryption::AES_BLOCK_SIZE;
        let aes_src_buffer = kernel::static_buf!([u8; 32]);
        let aes_dst_buffer = kernel::static_buf!([u8; CRYPT_SIZE]);
        let aes_driver = kernel::static_buf!(
            capsules_extra::symmetric_encryption::aes::AesDriver<'static, $A, $K>
        );

        (aes_driver, aes_src_buffer, aes_dst_buffer)
    };};
}

pub struct AesVirtualComponent<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> {
    aes_mux: &'static capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>,
}

impl<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> AesVirtualComponent<A> {
    pub fn new(
        aes_mux: &'static capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM<'static, A>,
    ) -> Self {
        Self { aes_mux }
    }
}

impl<A: 'static + AES<'static, AES128> + AESCtr + AESCBC + AESECB> Component
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

pub struct AesDriverComponent<
    K: AESKeySize,
    A: AES<'static, K> + AESCCM<'static, K> + 'static,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    aes: &'static A,
    mem_cap: CAP,
    _phantom: PhantomData<K>,
}

impl<
    K: AESKeySize,
    A: AES<'static, K> + AESCCM<'static, K> + 'static,
    CAP: MemoryAllocationCapability + 'static,
> AesDriverComponent<K, A, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        aes: &'static A,

        mem_cap: CAP,
    ) -> AesDriverComponent<K, A, CAP> {
        AesDriverComponent {
            board_kernel,
            driver_num,
            aes,
            mem_cap,

            _phantom: PhantomData::<K>,
        }
    }
}

impl<
    K: AESKeySize + 'static,
    A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    CAP: MemoryAllocationCapability + 'static,
> Component for AesDriverComponent<K, A, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_extra::symmetric_encryption::aes::AesDriver<'static, A, K>,
        >,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
    );
    type Output = &'static capsules_extra::symmetric_encryption::aes::AesDriver<'static, A, K>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let aes_src_buf = static_buffer.1.write([0; 32]);
        let aes_dst_buf = static_buffer.2.write([0; CRYPT_SIZE]);

        let aes_driver =
            static_buffer
                .0
                .write(capsules_extra::symmetric_encryption::aes::AesDriver::new(
                    self.aes,
                    aes_src_buf,
                    aes_dst_buf,
                    self.board_kernel
                        .create_grant(self.driver_num, &self.mem_cap),
                ));

        hil::symmetric_encryption::AESCCM::set_client(self.aes, aes_driver);
        hil::symmetric_encryption::AES::set_client(self.aes, aes_driver);

        aes_driver
    }
}
