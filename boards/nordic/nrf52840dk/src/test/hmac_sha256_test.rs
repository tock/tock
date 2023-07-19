// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This tests a software HMAC-SHA256 implementation. To run this test, add this
//! line to the imix boot sequence:
//!
//! ```
//! test::hmac_sha256_test::run_hmacsha256();
//! ```

use capsules_extra::hmac_sha256::HmacSha256Software;
use capsules_extra::sha256::Sha256Software;
use capsules_extra::test::hmac_sha256::TestHmacSha256;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::digest::Digest;
use kernel::static_init;

pub unsafe fn run_hmacsha256() {
    let t = static_init_test_hmacsha256();
    t.run();
}

pub static mut DIGEST_DATA: [u8; 32] = [0; 32];

// Test from https://en.wikipedia.org/wiki/HMAC#Examples
pub static mut WIKI_STR: [u8; 43] = *b"The quick brown fox jumps over the lazy dog";
pub static mut WIKI_KEY: [u8; 3] = *b"key";
pub static mut WIKI_HMAC: [u8; 32] = [
    0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f, 0xb1, 0x43,
    0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc, 0x2d, 0x1a, 0x3c, 0xd8,
];

unsafe fn static_init_test_hmacsha256() -> &'static TestHmacSha256 {
    let sha256_hash_buf = static_init!([u8; 64], [0; 64]);

    let sha256 = static_init!(Sha256Software<'static>, Sha256Software::new());
    sha256.register();

    let hmacsha256 = static_init!(
        HmacSha256Software<'static, Sha256Software<'static>>,
        HmacSha256Software::new(sha256, sha256_hash_buf)
    );
    sha256.set_client(hmacsha256);

    let test = static_init!(
        TestHmacSha256,
        TestHmacSha256::new(
            hmacsha256,
            &mut WIKI_KEY,
            &mut WIKI_STR,
            &mut DIGEST_DATA,
            &mut WIKI_HMAC
        )
    );

    test
}
