// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! RSA Signature Verifier for SHA256 hashes and RSA2048 keys.

use core::cell::Cell;
use kernel::hil;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};

pub struct RsaSignatureVerifier<'a, const H: usize, const S: usize> {
    verified: Cell<bool>,
    client: OptionalCell<&'a dyn hil::public_key_crypto::signature::ClientVerify<H, S>>,
    rsa_public_key: MapCell<rsa::RsaPublicKey>,
    hash_storage: TakeCell<'static, [u8; 32]>,
    signature_storage: TakeCell<'static, [u8; 256]>,

    deferred_call: kernel::deferred_call::DeferredCall,
}

impl<'a, const H: usize, const S: usize> RsaSignatureVerifier<'a, H, S> {
    pub fn new() -> Self {
        // my ACTUAL public key
        let n = rsa::BigUint::parse_bytes(b"24207257266404723702480416527933364039116773666417951609465570931679686940076207109293072612569267113256147168695608811123741758650429326896362330556657608406060222154960934834802893052320574456334624928389491520892685313371199210386475223296696579831840058897720325126562243770933238678183073031561719244791265232863605896837907058881808599654200582034457810596804897754743492491685117186551986141408292570229581725243489406524879436825146596246620952685529114397828868686804212048259156058108264250596765840612650253797791010731257875056647324896013942698591287080293236802963873491363585251097167389150803020633481", 10).unwrap();
        let e = rsa::BigUint::parse_bytes(b"65537", 10).unwrap();

        // Incorrect public key for testing
        // let n = rsa::BigUint::parse_bytes(b"34207257266404723702480416527933364039116773666417951609465570931679686940076207109293072612569267113256147168695608811123741758650429326896362330556657608406060222154960934834802893052320574456334624928389491520892685313371199210386475223296696579831840058897720325126562243770933238678183073031561719244791265232863605896837907058881808599654200582034457810596804897754743492491685117186551986141408292570229581725243489406524879436825146596246620952685529114397828868686804212048259156058108264250596765840612650253797791010731257875056647324896013942698591287080293236802963873491363585251097167389150803020633481", 10).unwrap();
        // let e = rsa::BigUint::parse_bytes(b"65537", 10).unwrap();

        let pub_key =
            rsa::RsaPublicKey::new(n, e).map_or_else(|_e| MapCell::empty(), |v| MapCell::new(v));

        Self {
            verified: Cell::new(false),
            client: OptionalCell::empty(),
            rsa_public_key: pub_key,
            hash_storage: TakeCell::empty(),
            signature_storage: TakeCell::empty(),

            deferred_call: kernel::deferred_call::DeferredCall::new(),
        }
    }
}

impl<'a> hil::public_key_crypto::signature::SignatureVerify<'a, 32, 256>
    for RsaSignatureVerifier<'a, 32, 256>
{
    fn set_verify_client(
        &'a self,
        client: &'a dyn hil::public_key_crypto::signature::ClientVerify<32, 256>,
    ) {
        self.client.replace(client);
    }

    fn verify(
        &'a self,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 256],
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; 32],
            &'static mut [u8; 256],
        ),
    > {
        if self.rsa_public_key.is_some() {
            self.rsa_public_key
                .map(|pub_key| {
                    self.verified.set(
                        pub_key
                            .verify(
                                rsa::Pkcs1v15Sign::new::<rsa::sha2::Sha256>(),
                                hash,
                                signature,
                            )
                            .is_ok(),
                    );
                    self.hash_storage.replace(hash);
                    self.signature_storage.replace(signature);
                    self.deferred_call.set();
                    Ok(())
                })
                .unwrap()
        } else {
            Err((kernel::ErrorCode::FAIL, hash, signature))
        }
    }
}

impl<'a> kernel::deferred_call::DeferredCallClient for RsaSignatureVerifier<'a, 32, 256> {
    fn handle_deferred_call(&self) {
        self.client.map(|client| {
            self.hash_storage.take().map(|h| {
                self.signature_storage.take().map(|s| {
                    client.verification_done(Ok(self.verified.get()), h, s);
                });
            });
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
