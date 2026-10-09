// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This tests a software SHA256 implementation.
//!
//! This test uses a deferred call (for callbacks). It tries to
//! hash 'hello world' and uses Digest::validate to check that the hash
//! is correct.
//!
//! The expected output is
//! Sha256Test: Verification result: Ok(true)
//!
//! This tests whether the SHA-256 hash of the string "hello hello
//! hello hello hello hello hello hello hello hello hello hello "
//! hashes correctly. This string is 12 repetitions of "hello ", so is
//! 72 bytes long. As SHA uses 64-byte/512 bit blocks, this verifies
//! that multi-block hashes work correctly.

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use capsules_extra::sha256::Sha256Software;
use capsules_extra::test::sha256::TestSha256;
use kernel::static_init;

pub unsafe fn run_sha256(client: &'static dyn CapsuleTestClient) {
    let t = static_init_test_sha256(client);
    t.run();
}

unsafe fn static_init_test_sha256(
    client: &'static dyn CapsuleTestClient,
) -> &'static TestSha256<'static, Sha256Software<'static>> {
    let sha = static_init!(Sha256Software<'static>, Sha256Software::new());
    kernel::deferred_call::DeferredCallClient::register(sha);

    // LSTRING is 12 repetitions of "hello " (72 bytes long) and LHASH is
    // the SHA-256 hash of this string.
    let lstring = static_init!([u8; 72], [0; 72]);
    let bytes = b"hello ";
    for i in 0..12 {
        for j in 0..6 {
            lstring[i * 6 + j] = bytes[j];
        }
    }

    let lhash = static_init!(
        [u8; 32],
        [
            0x59, 0x42, 0xc3, 0x71, 0x6f, 0x02, 0x82, 0x89, 0x3f, 0xbe, 0x04, 0x9b, 0xa2, 0x0e,
            0x56, 0x0e, 0x45, 0x94, 0xd5, 0xee, 0x15, 0xcb, 0x8a, 0x1e, 0x28, 0x7c, 0x20, 0x12,
            0xc2, 0xce, 0xb5, 0xa9,
        ]
    );

    // We expect LSTRING to hash to LHASH, so final argument is true
    let test = static_init!(
        TestSha256<Sha256Software>,
        TestSha256::new(sha, lstring, lhash, true)
    );
    test.set_client(client);

    test
}
