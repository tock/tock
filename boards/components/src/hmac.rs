// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for collections of HMACs.
//!
//! Usage
//! -----
//! ```rust
//!    let hmac = components::hmac::HmacComponent::new(
//!        board_kernel,
//!        chip.hmac,
//!    )
//!    .finalize(components::hmac_component_static!(
//!        lowrisc::hmac::Hmac,
//!        32
//!    ));
//! ```

use capsules_extra::hmac::HmacDriver;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;
use kernel::hil::digest::{DigestAlgorithm, HmacSha256, Sha256, ShaAlgorithm};

#[macro_export]
macro_rules! hmac_component_static {
    ($H:ty, $D:ty, $S:ty $(,)?) => {{
        let hmac = kernel::static_buf!(capsules_extra::hmac::HmacDriver<'static, $H, $D, $S>);

        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!(<$D as kernel::hil::digest::DigestAlgorithm>::Digest);

        (hmac, data_buffer, dest_buffer)
    };};
}

pub type HmacComponentType<H, D, S> = capsules_extra::hmac::HmacDriver<'static, H, D, S>;

pub struct HmacComponent<
    H: 'static + digest::Digest<'static, D>,
    D: DigestAlgorithm,
    S: ShaAlgorithm,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    hmac: &'static H,
    _phantom_d: core::marker::PhantomData<D>,
    _phantom_s: core::marker::PhantomData<S>,
}

impl<H: 'static + digest::Digest<'static, D>, D: DigestAlgorithm, S: ShaAlgorithm>
    HmacComponent<H, D, S>
{
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, hmac: &'static H) -> Self {
        Self {
            board_kernel,
            driver_num,
            hmac,
            _phantom_d: core::marker::PhantomData,
            _phantom_s: core::marker::PhantomData,
        }
    }
}

impl<
        H: digest::Digest<'static, D> + digest::HmacSha<S> + 'static,
        D: DigestAlgorithm + 'static,
        S: ShaAlgorithm + 'static,
    > Component for HmacComponent<H, D, S>
{
    type StaticInput = (
        &'static mut MaybeUninit<HmacDriver<'static, H, D, S>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<D::Digest>,
    );
    type Output = &'static HmacDriver<'static, H, D, S>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let data_buffer = s.1.write([0; 64]);
        let dest_buffer = s.2.write(D::Digest::default());

        let hmac = s.0.write(capsules_extra::hmac::HmacDriver::new(
            self.hmac,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        self.hmac.set_client(hmac);

        hmac
    }
}

#[macro_export]
macro_rules! hmac_sha256_software_component_static {
    ($S:ty $(,)?) => {{
        let hmac_sha256 =
            kernel::static_buf!(capsules_extra::hmac_sha256::HmacSha256Software<'static, $S>);

        let data_buffer = kernel::static_buf!([u8; 64]);
        let sha_buffer = kernel::static_buf!(kernel::hil::digest::Sha256Hash);
        let verify_buffer = kernel::static_buf!(kernel::hil::digest::HmacSha256);

        (hmac_sha256, data_buffer, sha_buffer, verify_buffer)
    };};
}

pub type HmacSha256SoftwareComponentType<S> =
    capsules_extra::hmac_sha256::HmacSha256Software<'static, S>;

pub struct HmacSha256SoftwareComponent<
    S: digest::DigestDataHash<'static, Sha256> + digest::Digest<'static, Sha256> + 'static,
> {
    sha_256: &'static S,
}

impl<S: digest::DigestDataHash<'static, Sha256> + digest::Digest<'static, Sha256>>
    HmacSha256SoftwareComponent<S>
{
    pub fn new(sha_256: &'static S) -> HmacSha256SoftwareComponent<S> {
        HmacSha256SoftwareComponent { sha_256 }
    }
}

impl<S: digest::DigestDataHash<'static, Sha256> + digest::Digest<'static, Sha256> + 'static>
    Component for HmacSha256SoftwareComponent<S>
{
    type StaticInput = (
        &'static mut MaybeUninit<capsules_extra::hmac_sha256::HmacSha256Software<'static, S>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<<Sha256 as DigestAlgorithm>::Digest>,
        &'static mut MaybeUninit<<HmacSha256 as DigestAlgorithm>::Digest>,
    );
    type Output = &'static capsules_extra::hmac_sha256::HmacSha256Software<'static, S>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let data_buffer = s.1.write([0; 64]);
        let sha_buffer = s.2.write(<Sha256 as DigestAlgorithm>::Digest::default());
        let verify_buffer =
            s.3.write(<HmacSha256 as DigestAlgorithm>::Digest::default());

        let hmac_sha256_sw =
            s.0.write(capsules_extra::hmac_sha256::HmacSha256Software::new(
                self.sha_256,
                data_buffer,
                sha_buffer,
                verify_buffer,
            ));

        kernel::hil::digest::Digest::set_client(self.sha_256, hmac_sha256_sw);

        hmac_sha256_sw
    }
}
