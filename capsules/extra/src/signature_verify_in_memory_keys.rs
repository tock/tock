// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Mechanism for verifying signatures with multiple in-memory keys.
//!
//! This capsule should be used when a system wants to be able to verify
//! signatures with multiple keys and the underlying signature verifier stores
//! keys in memory and only stores one key at a time.
//!
//! This capsule stores `NUM_KEYS` buffers holding keys. Users should construct
//! this capsule and then call `init_key()` `NUM_KEYS` times to set all of the
//! internal keys to store.
//!
//! The intended layering with this capsule looks like this:
//!
//! ```text
//!   ┌───────────────────────────────────────┐
//!   │                                       │
//!   │         Signature User                │
//!   │ (e.g., `AppCheckerSignature`)         │
//!   │                                       │
//!   └───────────────────────────────────────┘
//!     SignatureVerify + SelectKey      ^
//!   ┌─────────────────────────────┐    │
//!   │                             │    │
//!   │ SignatureVerifyInMemoryKeys │    │SignatureVerifyClient
//!   │    (this module)            │    │
//!   │                             │    │
//!   └─────────────────────────────┘    │
//!     SignatureVerify + SetKeyBySlice  │
//!   ┌───────────────────────────────────────┐
//!   │                                       │
//!   │         Signature Verifier            │
//!   │  (e.g., `EcdsaP256SignatureVerifier`) │
//!   │                                       │
//!   └───────────────────────────────────────┘
//! ```

use kernel::hil;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct SignatureVerifyInMemoryKeys<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const NUM_KEYS: usize,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> {
    verifier: &'a S,

    keys: [MapCell<&'static mut [u8; KL]>; NUM_KEYS],
    active_key: OptionalCell<usize>,

    client_key_select: OptionalCell<&'a dyn hil::public_key_crypto::keys::SelectKeyClient>,

    deferred_call: kernel::deferred_call::DeferredCall,
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
            + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
        const NUM_KEYS: usize,
        const KL: usize,
        const HL: usize,
        const SL: usize,
    > SignatureVerifyInMemoryKeys<'a, S, NUM_KEYS, KL, HL, SL>
{
    pub fn new(verifier: &'a S) -> Self {
        Self {
            verifier,
            keys: [const { MapCell::empty() }; NUM_KEYS],
            active_key: OptionalCell::empty(),
            client_key_select: OptionalCell::empty(),
            deferred_call: kernel::deferred_call::DeferredCall::new(),
        }
    }

    /// Set the keys this module should store.
    pub fn init_key(&self, index: usize, key: &'static mut [u8; KL]) -> Result<(), ()> {
        self.keys.get(index).ok_or(())?.replace(key);
        Ok(())
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
            + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
        const NUM_KEYS: usize,
        const KL: usize,
        const HL: usize,
        const SL: usize,
    > hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
    for SignatureVerifyInMemoryKeys<'a, S, NUM_KEYS, KL, HL, SL>
{
    fn set_verify_client(
        &self,
        client: &'a dyn hil::public_key_crypto::signature::ClientVerify<HL, SL>,
    ) {
        self.verifier.set_verify_client(client);
    }

    fn verify(
        &self,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; HL],
            &'static mut [u8; SL],
        ),
    > {
        self.verifier.verify(hash, signature)
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
            + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
        const NUM_KEYS: usize,
        const KL: usize,
        const HL: usize,
        const SL: usize,
    > hil::public_key_crypto::keys::SelectKey<'a>
    for SignatureVerifyInMemoryKeys<'a, S, NUM_KEYS, KL, HL, SL>
{
    fn get_key_count(&self) -> Result<(), ErrorCode> {
        self.deferred_call.set();
        Ok(())
    }

    fn select_key(&self, index: usize) -> Result<(), ErrorCode> {
        // Extract the key from our stored list of buffers holding keys. Return
        // `INVAL` if the index is greater than the number of keys we have and
        // return `NOMEM` if the key is not in our storage.
        let key = self
            .keys
            .get(index)
            .ok_or(ErrorCode::INVAL)?
            .take()
            .ok_or(ErrorCode::NOMEM)?;

        // Mark which key is now active.
        self.active_key.set(index);

        // Set the key in the verifier. Replace if there is an error.
        self.verifier.set_key(key).map_err(|(e, k)| {
            if let Some(slot) = self.keys.get(self.active_key.get().unwrap_or(0)) {
                slot.replace(k);
            }
            self.active_key.clear();

            e
        })
    }

    fn set_client(&self, client: &'a dyn hil::public_key_crypto::keys::SelectKeyClient) {
        self.client_key_select.replace(client);
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
            + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
        const NUM_KEYS: usize,
        const KL: usize,
        const HL: usize,
        const SL: usize,
    > hil::public_key_crypto::keys::SetKeyBySliceClient<KL>
    for SignatureVerifyInMemoryKeys<'a, S, NUM_KEYS, KL, HL, SL>
{
    fn set_key_done(&self, key: &'static mut [u8; KL], error: Result<(), ErrorCode>) {
        // Re-store the key we just set.
        if let Some(slot) = self.keys.get(self.active_key.get().unwrap_or(0)) {
            slot.replace(key);
        }

        self.client_key_select.map(|client| {
            client.select_key_done(self.active_key.get().unwrap_or(0), error);
        });
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
            + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
        const NUM_KEYS: usize,
        const KL: usize,
        const HL: usize,
        const SL: usize,
    > kernel::deferred_call::DeferredCallClient
    for SignatureVerifyInMemoryKeys<'a, S, NUM_KEYS, KL, HL, SL>
{
    fn handle_deferred_call(&self) {
        self.client_key_select.map(|client| {
            client.get_key_count_done(NUM_KEYS);
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
