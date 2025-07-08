// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! ECDSA Signer for P256 signatures.

use p256::ecdsa;
use p256::ecdsa::signature::hazmat::PrehashSigner;

use kernel::hil;
use kernel::hil::public_key_crypto::keys::SetKeyBySliceClient;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

enum State {
    Signing,
    ChangingKey(&'static mut [u8; 64]),
}

pub struct EcdsaP256SignatureSigner<'a> {
    client: OptionalCell<&'a dyn hil::public_key_crypto::signature::ClientSign<32, 64>>,
    client_key_set: OptionalCell<&'a dyn hil::public_key_crypto::keys::SetKeyBySliceClient<64>>,
    signing_key: TakeCell<'static, [u8; 32]>,
    hash_storage: TakeCell<'static, [u8; 32]>,
    signature_storage: TakeCell<'static, [u8; 64]>,
    deferred_call: kernel::deferred_call::DeferredCall,
    state: OptionalCell<State>,
}

impl EcdsaP256SignatureSigner<'_> {
    pub fn new(signing_key: &'static mut [u8; 32]) -> Self {
        Self {
            client: OptionalCell::empty(),
            client_key_set: OptionalCell::empty(),
            signing_key: TakeCell::new(signing_key),
            hash_storage: TakeCell::empty(),
            signature_storage: TakeCell::empty(),
            deferred_call: kernel::deferred_call::DeferredCall::new(),
            state: OptionalCell::empty(),
        }
    }
}

impl<'a> hil::public_key_crypto::signature::SignatureSign<'a, 32, 64>
    for EcdsaP256SignatureSigner<'a>
{
    fn set_sign_client(
        &self,
        client: &'a dyn hil::public_key_crypto::signature::ClientSign<32, 64>,
    ) {
        self.client.replace(client);
    }

    fn sign(
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
        if self.signing_key.is_some() {
            self.signing_key
                .map(|skey| {
                    let skey: &[u8; 32] = skey;
                    if let Ok(ecdsa_key) = ecdsa::SigningKey::from_bytes(skey.into()) {
                        let maybe_sig: Result<ecdsa::Signature, _> = ecdsa_key.sign_prehash(hash);
                        if let Ok(sig) = maybe_sig {
                            signature.copy_from_slice(&sig.to_bytes());
                            self.hash_storage.replace(hash);
                            self.signature_storage.replace(signature);
                            self.state.set(State::Signing);
                            self.deferred_call.set();
                            Ok(())
                        } else {
                            Err((kernel::ErrorCode::FAIL, hash, signature))
                        }
                    } else {
                        Err((kernel::ErrorCode::INVAL, hash, signature))
                    }
                })
                .unwrap()
        } else {
            Err((kernel::ErrorCode::FAIL, hash, signature))
        }
    }
}

impl<'a> hil::public_key_crypto::keys::SetKeyBySlice<'a, 64> for EcdsaP256SignatureSigner<'a> {
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

impl kernel::deferred_call::DeferredCallClient for EcdsaP256SignatureSigner<'_> {
    fn handle_deferred_call(&self) {
        if let Some(s) = self.state.take() {
            match s {
                State::Signing => {
                    self.client.map(|client| {
                        if let Some(h) = self.hash_storage.take() {
                            if let Some(s) = self.signature_storage.take() {
                                client.signing_done(Ok(()), h, s);
                            }
                        }
                    });
                }
                State::ChangingKey(key) => {
                    self.signing_key.map(|skey| {
                        skey.copy_from_slice(key);
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
