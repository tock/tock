// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for collections of SHA.
//!
//! Usage
//! -----
//! ```rust
//!    let sha = components::sha::ShaComponent::new(
//!        board_kernel,
//!        chip.sha,
//!    )
//!    .finalize(components::sha_component_static!(
//!        lowrisc::sha::Sha,
//!        32,
//!    ));
//! ```

use capsules_extra::sha::ShaDriver;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_component_static {
    ($A:ty, $L:expr$(,)?) => {{
        let sha_driver = kernel::static_buf!(
            capsules_extra::sha::ShaDriver<
                'static,
                capsules_core::virtualizers::virtual_sha::VirtualMuxSha<'static, $A, $L>,
                $L,
            >
        );

        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!([u8; $L]);

        (sha_driver, data_buffer, dest_buffer)
    };};
}

pub struct ShaComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sha: &'static A,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> ShaComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sha: &'static A,
    ) -> ShaComponent<A, L> {
        ShaComponent {
            board_kernel,
            driver_num,
            sha,
        }
    }
}

impl<
        A: kernel::hil::digest::Sha256
            + digest::Sha384
            + digest::Sha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for ShaComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<ShaDriver<'static, A, L>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; L]>,
    );

    type Output = &'static ShaDriver<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let data_buffer = s.1.write([0; 64]);
        let dest_buffer = s.2.write([0; L]);

        let sha = s.0.write(capsules_extra::sha::ShaDriver::new(
            self.sha,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
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
