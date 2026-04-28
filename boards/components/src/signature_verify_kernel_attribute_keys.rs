// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for SignatureVerifyKernelAttributeKeys.

use capsules_extra::signature_verify_kernel_attribute_keys::SignatureVerifyKernelAttributeKeys;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;

#[macro_export]
macro_rules! signature_verify_kernel_attribute_keys_component_static {
    ($S:ty, $ALGORITHM_ID:expr, $KL:expr, $HL:expr, $SL:expr $(,)?) => {{
        let verifier = kernel::static_buf!(
            capsules_extra::signature_verify_kernel_attribute_keys::SignatureVerifyKernelAttributeKeys<
                'static,
                $S,
                $ALGORITHM_ID,
                $KL,
                $HL,
                $SL,
            >
        );
        let key_buffer = kernel::static_buf!([u8; $KL]);
        (verifier, key_buffer)
    }};
}

pub type SignatureVerifyKernelAttributeKeysComponentType<
    S,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> = SignatureVerifyKernelAttributeKeys<'static, S, ALGORITHM_ID, KL, HL, SL>;

pub struct SignatureVerifyKernelAttributeKeysComponent<
    S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>
        + kernel::hil::public_key_crypto::keys::SetKeyBySlice<'static, KL>
        + 'static,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> {
    verifier: &'static S,
    attributes: &'static [u8],
}

impl<
    S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>
        + kernel::hil::public_key_crypto::keys::SetKeyBySlice<'static, KL>
        + 'static,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> SignatureVerifyKernelAttributeKeysComponent<S, ALGORITHM_ID, KL, HL, SL>
{
    pub fn new(verifier: &'static S, attributes: &'static [u8]) -> Self {
        Self {
            verifier,
            attributes,
        }
    }
}

impl<
    S: kernel::hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>
        + kernel::hil::public_key_crypto::keys::SetKeyBySlice<'static, KL>
        + 'static,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> Component for SignatureVerifyKernelAttributeKeysComponent<S, ALGORITHM_ID, KL, HL, SL>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            SignatureVerifyKernelAttributeKeys<'static, S, ALGORITHM_ID, KL, HL, SL>,
        >,
        &'static mut MaybeUninit<[u8; KL]>,
    );

    type Output = &'static SignatureVerifyKernelAttributeKeys<'static, S, ALGORITHM_ID, KL, HL, SL>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let (verifier_buf, key_buf) = s;
        let key_buffer = key_buf.write([0u8; KL]);
        let verifier_multiple_keys = verifier_buf.write(SignatureVerifyKernelAttributeKeys::new(
            self.verifier,
            self.attributes,
            key_buffer,
        ));
        self.verifier.set_client(verifier_multiple_keys);
        verifier_multiple_keys.register();
        verifier_multiple_keys
    }
}
