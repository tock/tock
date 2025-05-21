// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for signature credential checkers.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::{digest, public_key_crypto};

#[macro_export]
macro_rules! app_checker_signature_component_static {
    ($SignatureKind:ty, $HashKind:ty, $HASH_LEN:expr, $SIGNATURE_LEN:expr $(,)?) => {{
        let hash_buffer = kernel::static_buf!([u8; $HASH_LEN]);
        let signature_buffer = kernel::static_buf!([u8; $SIGNATURE_LEN]);
        let checker = kernel::static_buf!(
            capsules_system::process_checker::signature::AppCheckerSignature<
                'static,
                $SignatureKind,
                $HashKind,
                $HASH_LEN,
                $SIGNATURE_LEN,
            >
        );

        (checker, hash_buffer, signature_buffer)
    };};
}

pub type AppCheckerSignatureComponentType<
    SignatureKind,
    HashKind,
    const HASH_LEN: usize,
    const SIGNATURE_LEN: usize,
> = capsules_system::process_checker::signature::AppCheckerSignature<
    'static,
    SignatureKind,
    HashKind,
    HASH_LEN,
    SIGNATURE_LEN,
>;

pub struct AppCheckerSignatureComponent<
    SignatureKind: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>
        + kernel::hil::public_key_crypto::keys::SelectKey<'static>
        + 'static,
    HashKind: kernel::hil::digest::DigestDataHash<'static, HASH_LEN> + 'static,
    const HASH_LEN: usize,
    const SIGNATURE_LEN: usize,
> {
    hasher: &'static HashKind,
    verifier: &'static SignatureKind,
    credential_type: tock_tbf::types::TbfFooterV2CredentialsType,
}

impl<
        SignatureKind: kernel::hil::public_key_crypto::signature::SignatureVerify<
                'static,
                HASH_LEN,
                SIGNATURE_LEN,
            > + kernel::hil::public_key_crypto::keys::SelectKey<'static>,
        HashKind: kernel::hil::digest::DigestDataHash<'static, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > AppCheckerSignatureComponent<SignatureKind, HashKind, HASH_LEN, SIGNATURE_LEN>
{
    pub fn new(
        hasher: &'static HashKind,
        verifier: &'static SignatureKind,
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
        SignatureKind: kernel::hil::public_key_crypto::signature::SignatureVerify<
                'static,
                HASH_LEN,
                SIGNATURE_LEN,
            > + kernel::hil::public_key_crypto::keys::SelectKey<'static>,
        HashKind: kernel::hil::digest::DigestDataHash<'static, HASH_LEN>
            + kernel::hil::digest::Digest<'static, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > Component for AppCheckerSignatureComponent<SignatureKind, HashKind, HASH_LEN, SIGNATURE_LEN>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_system::process_checker::signature::AppCheckerSignature<
                'static,
                SignatureKind,
                HashKind,
                HASH_LEN,
                SIGNATURE_LEN,
            >,
        >,
        &'static mut MaybeUninit<[u8; HASH_LEN]>,
        &'static mut MaybeUninit<[u8; SIGNATURE_LEN]>,
    );

    type Output = &'static capsules_system::process_checker::signature::AppCheckerSignature<
        'static,
        SignatureKind,
        HashKind,
        HASH_LEN,
        SIGNATURE_LEN,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let hash_buffer = s.1.write([0; HASH_LEN]);
        let signature_buffer = s.2.write([0; SIGNATURE_LEN]);

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
        kernel::hil::public_key_crypto::keys::SelectKey::set_client(self.verifier, checker);
        public_key_crypto::signature::SignatureVerify::set_verify_client(self.verifier, checker);

        checker
    }
}
