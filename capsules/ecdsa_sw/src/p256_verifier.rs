// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! ECDSA Signature Verifier for P256 signatures.

use p256::ecdsa;
use p256::ecdsa::signature::hazmat::PrehashVerifier;

use core::cell::Cell;
use kernel::hil;
use kernel::hil::public_key_crypto::keys::SetKeyBySliceClient;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

enum State {
    Verifying,
    ChangingKey(&'static mut [u8; 64]),
}

pub struct EcdsaP256SignatureVerifier<'a> {
    verified: Cell<bool>,
    client: OptionalCell<&'a dyn hil::public_key_crypto::signature::ClientVerify<32, 64>>,
    client_key_set: OptionalCell<&'a dyn hil::public_key_crypto::keys::SetKeyBySliceClient<64>>,
    verifying_key: TakeCell<'static, [u8; 64]>,
    hash_storage: TakeCell<'static, [u8; 32]>,
    signature_storage: TakeCell<'static, [u8; 64]>,
    deferred_call: kernel::deferred_call::DeferredCall,
    state: OptionalCell<State>,
}

impl EcdsaP256SignatureVerifier<'_> {
    pub fn new(verifying_key: &'static mut [u8; 64]) -> Self {
        Self {
            verified: Cell::new(false),
            client: OptionalCell::empty(),
            client_key_set: OptionalCell::empty(),
            verifying_key: TakeCell::new(verifying_key),
            hash_storage: TakeCell::empty(),
            signature_storage: TakeCell::empty(),
            deferred_call: kernel::deferred_call::DeferredCall::new(),
            state: OptionalCell::empty(),
        }
    }
}

impl<'a> hil::public_key_crypto::signature::SignatureVerify<'a, 32, 64>
    for EcdsaP256SignatureVerifier<'a>
{
    fn set_verify_client(
        &self,
        client: &'a dyn hil::public_key_crypto::signature::ClientVerify<32, 64>,
    ) {
        self.client.replace(client);
    }

    fn verify(
        &self,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; 32],
            &'static mut [u8; 64],
        ),
    > {
        if self.verifying_key.is_some() {
            if let Ok(sig) = ecdsa::Signature::from_slice(signature) {
                self.verifying_key
                    .map(|vkey| {
                        let vkey: &[u8; 64] = vkey;
                        let ep = p256::EncodedPoint::from_untagged_bytes(vkey.into());
                        let key = ecdsa::VerifyingKey::from_encoded_point(&ep);
                        key.map(|ecdsa_key| {
                            self.verified
                                .set(ecdsa_key.verify_prehash(hash, &sig).is_ok());
                            self.hash_storage.replace(hash);
                            self.signature_storage.replace(signature);
                            self.state.set(State::Verifying);
                            self.deferred_call.set();
                            Ok(())
                        })
                        .unwrap()
                    })
                    .unwrap()
            } else {
                Err((kernel::ErrorCode::INVAL, hash, signature))
            }
        } else {
            Err((kernel::ErrorCode::FAIL, hash, signature))
        }
    }
}

impl<'a> hil::public_key_crypto::keys::SetKeyBySlice<'a, 64> for EcdsaP256SignatureVerifier<'a> {
    fn set_key(
        &self,
        key: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 64])> {
        // Just wait for the deferred call to make the change so we can keep
        // both the old and the new key in the meantime.
        self.state.set(State::ChangingKey(key));
        self.deferred_call.set();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn SetKeyBySliceClient<64>) {
        self.client_key_set.replace(client);
    }
}

impl kernel::deferred_call::DeferredCallClient for EcdsaP256SignatureVerifier<'_> {
    fn handle_deferred_call(&self) {
        if let Some(s) = self.state.take() {
            match s {
                State::Verifying => {
                    self.client.map(|client| {
                        if let Some(h) = self.hash_storage.take() {
                            if let Some(s) = self.signature_storage.take() {
                                client.verification_done(Ok(self.verified.get()), h, s);
                            }
                        }
                    });
                }
                State::ChangingKey(key) => {
                    self.verifying_key.map(|vkey| {
                        vkey.copy_from_slice(key);
                    });

                    self.client_key_set.map(|client| {
                        client.set_key_done(key, Ok(()));
                    });
                }
            }
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
