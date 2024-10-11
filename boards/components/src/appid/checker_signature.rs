// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for signature credential checkers.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::{digest, public_key_crypto};

#[macro_export]
macro_rules! app_checker_signature_component_static {
    ($S:ty, $H:ty, $HL:expr, $SL:expr $(,)?) => {{
        let hash_buffer = kernel::static_buf!([u8; $HL]);
        let signature_buffer = kernel::static_buf!([u8; $SL]);
        let checker = kernel::static_buf!(
            capsules_system::process_checker::signature::AppCheckerSignature<
                'static,
                $S,
                $H,
                $HL,
                $SL,
            >
        );

        (checker, hash_buffer, signature_buffer)
    };};
}

pub type AppCheckerSignatureComponentType<S, H, const HL: usize, const SL: usize> =
    capsules_system::process_checker::signature::AppCheckerSignature<'static, S, H, HL, SL>;

pub struct AppCheckerSignatureComponent<
    S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL> + 'static,
    H: kernel::hil::digest::DigestDataHash<'static, HL> + 'static,
    const HL: usize,
    const SL: usize,
> {
    hasher: &'static H,
    verifier: &'static S,
    credential_type: tock_tbf::types::TbfFooterV2CredentialsType,
}

impl<
        S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: kernel::hil::digest::DigestDataHash<'static, HL>,
        const HL: usize,
        const SL: usize,
    > AppCheckerSignatureComponent<S, H, HL, SL>
{
    pub fn new(
        hasher: &'static H,
        verifier: &'static S,
        credential_type: tock_tbf::types::TbfFooterV2CredentialsType,
    ) -> Self {
        Self {
            hasher,
            verifier,
            credential_type,
        }
    }
}

impl<
        S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: kernel::hil::digest::DigestDataHash<'static, HL> + kernel::hil::digest::Digest<'static, HL>,
        const HL: usize,
        const SL: usize,
    > Component for AppCheckerSignatureComponent<S, H, HL, SL>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_system::process_checker::signature::AppCheckerSignature<'static, S, H, HL, SL>,
        >,
        &'static mut MaybeUninit<[u8; HL]>,
        &'static mut MaybeUninit<[u8; SL]>,
    );

    type Output = &'static capsules_system::process_checker::signature::AppCheckerSignature<
        'static,
        S,
        H,
        HL,
        SL,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let hash_buffer = s.1.write([0; HL]);
        let signature_buffer = s.2.write([0; SL]);

        let checker = s.0.write(
            capsules_system::process_checker::signature::AppCheckerSignature::new(
                self.hasher,
                self.verifier,
                hash_buffer,
                signature_buffer,
                self.credential_type,
            ),
        );

        digest::Digest::set_client(self.hasher, checker);
        public_key_crypto::signature::SignatureVerify::set_verify_client(self.verifier, checker);

        checker
    }
}
