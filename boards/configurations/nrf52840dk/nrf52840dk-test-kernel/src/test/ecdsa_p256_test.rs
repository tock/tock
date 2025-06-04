// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! This tests a software ECDSA P256 implementation. To run this test,
//! add this line to the boot sequence:
//! ```
//! test::ecdsa_p256_test::run_ecdsa_p256();
//! ```

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use core::ptr::addr_of_mut;
use ecdsa_sw::p256_signer::EcdsaP256SignatureSigner;
use ecdsa_sw::test::p256::TestEcdsaP256Sign;
use kernel::static_init;

// HHASH is the hash to sign: this is just the SHA-256 hash of "hello world" as
// in the SHA-256 test.
pub static mut HHASH: [u8; 32] = [
    0xB9, 0x4D, 0x27, 0xB9, 0x93, 0x4D, 0x3E, 0x08, 0xA5, 0x2E, 0x52, 0xD7, 0xDA, 0x7D, 0xAB, 0xFA,
    0xC4, 0x84, 0xEF, 0xE3, 0x7A, 0x53, 0x80, 0xEE, 0x90, 0x88, 0xF7, 0xAC, 0xE2, 0xEF, 0xCD, 0xE9,
];

// SKEY is the secret key used to for signing, encoded as the secret scalar d in
// big-endian byte order.
//
// - `ec-secp256r1-priv-key.pem`:
//
//   -----BEGIN EC PRIVATE KEY-----
//   MHcCAQEEIGU0zCXHLqxDmrHHAWEQP5zNfWRQrAiIpH9YwxHlqysmoAoGCCqGSM49
//   AwEHoUQDQgAE4BM6kKdKNWFRjuFECfFpwc9q239+Uvi3QXniTVdBI1IuthIDs4UQ
//   5fMlB2KPVJWCV0VQvaPiF+g0MIkmTCNisQ==
//   -----END EC PRIVATE KEY-----
//
pub static mut SKEY: [u8; 32] = [
    0x65, 0x34, 0xCC, 0x25, 0xC7, 0x2E, 0xAC, 0x43, 0x9A, 0xB1, 0xC7, 0x01, 0x61, 0x10, 0x3F, 0x9C,
    0xCD, 0x7D, 0x64, 0x50, 0xAC, 0x08, 0x88, 0xA4, 0x7F, 0x58, 0xC3, 0x11, 0xE5, 0xAB, 0x2B, 0x26,
];

// HSIG is the buffer for storing the resulting signature of the hash in HHASH.
pub static mut HSIG: [u8; 64] = [0; 64];

// SSIG is the buffer storing the correct ECDSA P-256 signature using
// deterministic (RFC 6979) nonce generation to compare with, encoded as the
// values r and s both in big-endian byte order concatenated.
pub static mut CSIG: [u8; 64] = [
    0x9E, 0xB8, 0x19, 0x40, 0xD4, 0xA9, 0xE5, 0x5E, 0x84, 0x08, 0xDB, 0xE8, 0xCB, 0x5A, 0x1F, 0x3C,
    0x01, 0x18, 0x1C, 0xD1, 0x92, 0xEC, 0xCE, 0x1E, 0x4B, 0x80, 0x22, 0x94, 0xB1, 0xFB, 0x67, 0x31,
    0xFE, 0xEF, 0xDD, 0x23, 0x08, 0x76, 0x41, 0x0B, 0x03, 0x9E, 0x2A, 0x62, 0xCA, 0xA8, 0x32, 0x03,
    0x4A, 0x63, 0x2C, 0x91, 0xC8, 0xDE, 0xDE, 0x70, 0x5E, 0x67, 0xBA, 0x3A, 0xBE, 0xE1, 0xFE, 0x96,
];

pub unsafe fn run_ecdsa_p256(client: &'static dyn CapsuleTestClient) {
    let t = static_init_test_ecdsa_p256(client);
    t.run();
}

unsafe fn static_init_test_ecdsa_p256(
    client: &'static dyn CapsuleTestClient,
) -> &'static TestEcdsaP256Sign {
    let ecdsa = static_init!(
        EcdsaP256SignatureSigner<'static>,
        EcdsaP256SignatureSigner::new(&mut *addr_of_mut!(SKEY)),
    );
    kernel::deferred_call::DeferredCallClient::register(ecdsa);

    let test = static_init!(
        TestEcdsaP256Sign,
        TestEcdsaP256Sign::new(
            ecdsa,
            &mut *addr_of_mut!(HHASH),
            &mut *addr_of_mut!(HSIG),
            &mut *addr_of_mut!(CSIG)
        )
    );

    test.set_client(client);

    test
}
