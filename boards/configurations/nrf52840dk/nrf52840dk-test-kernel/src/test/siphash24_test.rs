// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! This tests a software SipHash24 implementation. To run this test,
//! add this line to the boot sequence:
//! ```
//! test::siphash24_test::run_siphash24();
//! ```

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use capsules_extra::sip_hash::SipHasher24;
use capsules_extra::test::siphash24::TestSipHash24;
use kernel::static_init;

pub unsafe fn run_siphash24(client: &'static dyn CapsuleTestClient) {
    let t = static_init_test_siphash24(client);
    t.run();
}

unsafe fn static_init_test_siphash24(
    client: &'static dyn CapsuleTestClient,
) -> &'static TestSipHash24 {
    let sha = static_init!(SipHasher24<'static>, SipHasher24::new());
    kernel::deferred_call::DeferredCallClient::register(sha);

    let hstring = static_init!([u8; 15], *b"tickv-super-key");
    let hbuf = static_init!([u8; 64], [0; 64]);

    // Copy to the 64 byte buffer because we always hash 64 bytes.
    hbuf[..15].copy_from_slice(&hstring[..]);

    let hhash = static_init!([u8; 8], [0; 8]);
    let chash = static_init!([u8; 8], [0xd1, 0xdc, 0x3b, 0x92, 0xc2, 0x5a, 0x1b, 0x30]);

    let test = static_init!(TestSipHash24, TestSipHash24::new(sha, hbuf, hhash, chash));

    test.set_client(client);

    test
}
