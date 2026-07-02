// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for collections of SHA.
//!
//! Usage
//! -----
//! ```rust
//!    type Sha = components::sha::ShaSoftware256ComponentType;
//!    const SHA_DIGEST_LEN: usize = 32;
//!    type ShaDriver = components::sha::ShaDriverComponentType<Sha, SHA_DIGEST_LEN>;
//!
//!    let sha_driver = components::sha::ShaDriverComponent::new(
//!        board_kernel,
//!        capsules_extra::sha::DRIVER_NUM,
//!        sha,
//!    )
//!    .finalize(components::sha_driver_component_static!(
//!        Sha,
//!        SHA_DIGEST_LEN
//!    ));
//! ```

use capsules_extra::sha256_driver::ShaDriver;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil::digest;

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_driver_component_static {
    ($A:ty, $L:expr$(,)?) => {{
        let sha_driver =
            kernel::static_buf!(capsules_extra::sha256_driver::ShaDriver<'static, $A, $L>);

        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!([u8; $L]);

        (sha_driver, data_buffer, dest_buffer)
    };};
}

pub type ShaDriverComponentType<A, const L: usize> = ShaDriver<'static, A, L>;

pub struct ShaDriverComponent<
    A: 'static + digest::DigestDataHash<'static, L>,
    const L: usize,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sha: &'static A,
    mem_cap: CAP,
}

impl<
    A: 'static + digest::DigestDataHash<'static, L>,
    const L: usize,
    CAP: MemoryAllocationCapability + 'static,
> ShaDriverComponent<A, L, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sha: &'static A,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            sha,
            mem_cap,
        }
    }
}

impl<
    A: kernel::hil::digest::Sha256 + 'static + digest::DigestDataHash<'static, L>,
    const L: usize,
    CAP: MemoryAllocationCapability + 'static,
> Component for ShaDriverComponent<A, L, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<ShaDriver<'static, A, L>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; L]>,
    );

    type Output = &'static ShaDriver<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let data_buffer = s.1.write([0; 64]);
        let dest_buffer = s.2.write([0; L]);

        let sha = s.0.write(ShaDriver::new(
            self.sha,
            data_buffer,
            dest_buffer,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        self.sha.set_client(sha);

        sha
    }
}

#[macro_export]
macro_rules! sha_software_256_component_static {
    ($(,)?) => {{
        kernel::static_buf!(capsules_extra::sha256::Sha256Software<'static>)
    };};
}

pub type ShaSoftware256ComponentType = capsules_extra::sha256::Sha256Software<'static>;

pub struct ShaSoftware256Component {}

impl ShaSoftware256Component {
    pub fn new() -> ShaSoftware256Component {
        ShaSoftware256Component {}
    }
}

impl Component for ShaSoftware256Component {
    type StaticInput = &'static mut MaybeUninit<capsules_extra::sha256::Sha256Software<'static>>;

    type Output = &'static capsules_extra::sha256::Sha256Software<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let sha_256_sw = s.write(capsules_extra::sha256::Sha256Software::new());

        kernel::deferred_call::DeferredCallClient::register(sha_256_sw);

        sha_256_sw
    }
}
