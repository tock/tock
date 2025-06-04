// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the software implementation of ECDSA over the P256 curve by signing a
//! hash and checking it against the expected signature value. Since the
//! implementation of EcdsaP256SignatureSigner uses the deterministic nonce
//! generation algorithm as detailed in RFC 6979, the signature output by
//! this test will always be the same, meaning we can simply test it against
//! a known correct value.

use crate::p256_signer::EcdsaP256SignatureSigner;
use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient, CapsuleTestError};
use kernel::debug;
use kernel::hil::public_key_crypto::signature;
use kernel::hil::public_key_crypto::signature::SignatureSign;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub struct TestEcdsaP256Sign {
    ecdsa: &'static EcdsaP256SignatureSigner<'static>,
    hash: TakeCell<'static, [u8; 32]>,      // The hash to sign
    signature: TakeCell<'static, [u8; 64]>, // The resulting signature
    correct_signature: TakeCell<'static, [u8; 64]>, // The expected signature
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl TestEcdsaP256Sign {
    pub fn new(
        ecdsa: &'static EcdsaP256SignatureSigner<'static>,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
        correct_signature: &'static mut [u8; 64],
    ) -> Self {
        TestEcdsaP256Sign {
            ecdsa,
            hash: TakeCell::new(hash),
            signature: TakeCell::new(signature),
            correct_signature: TakeCell::new(correct_signature),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&'static self) {
        self.ecdsa.set_sign_client(self);
        let hash = self.hash.take().unwrap();
        let signature = self.signature.take().unwrap();
        let r = self.ecdsa.sign(hash, signature);
        if r.is_err() {
            panic!("EcdsaP256SignTest: failed to sign: {:?}", r);
        }
    }
}

impl signature::ClientSign<32, 64> for TestEcdsaP256Sign {
    fn signing_done(
        &self,
        result: Result<(), ErrorCode>,
        _hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
    ) {
        match result {
            Ok(()) => {
                let correct = self.correct_signature.take().unwrap();
                let res = if signature == correct {
                    debug!("EcdsaP256SignTest passed (signatures match)");
                    Ok(())
                } else {
                    debug!("EcdsaP256SignTest failed (signatures don't match)");
                    Err(CapsuleTestError::IncorrectResult)
                };
                self.client.map(|client| client.done(res));
            }
            Err(e) => {
                panic!("EcdsaP256SignTest: signing failed: {:?}", e);
            }
        }
    }
}

impl CapsuleTest for TestEcdsaP256Sign {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
