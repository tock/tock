// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test that AES (either CTR or CBC mode) is working properly.
//!
//! To test CBC mode, add the following line to the imix boot sequence:
//! ```
//!     test::aes_test::run_aes128_cbc();
//! ```
//! You should see the following output:
//! ```
//!     aes_test passed (CBC Enc Src/Dst)
//!     aes_test passed (CBC Dec Src/Dst)
//!     aes_test passed (CBC Enc In-place)
//!     aes_test passed (CBC Dec In-place)
//! ```
//! To test CTR mode, add the following line to the imix boot sequence:
//! ```
//!     test::aes_test::run_aes128_ctr();
//! ```
//! You should see the following output:
//! ```
//!     aes_test CTR passed: (CTR Enc Ctr Src/Dst)
//!     aes_test CTR passed: (CTR Dec Ctr Src/Dst)
//! ```

use capsules_extra::test::aes::TestAes128Cbc;
use capsules_extra::test::aes::TestAes128Ctr;
use capsules_extra::test::aes::TestAes128Ecb;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::static_init;
use nrf52840::aes::AesECB;

pub unsafe fn run_aes128_ctr(aes: &'static AesECB) {
    let t = static_init_test_ctr(aes);
    aes.set_client(t);

    t.run();
}

pub unsafe fn run_aes128_cbc(aes: &'static AesECB) {
    let t = static_init_test_cbc(aes);
    aes.set_client(t);

    t.run();
}

pub unsafe fn run_aes128_ecb(aes: &'static AesECB) {
    let t = static_init_test_ecb(aes);
    aes.set_client(t);

    t.run();
}

unsafe fn static_init_test_ctr(
    aes: &'static AesECB,
) -> &'static TestAes128Ctr<'static, AesECB<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

    static_init!(
        TestAes128Ctr<'static, AesECB>,
        TestAes128Ctr::new(&aes, key, iv, source, data)
    )
}

unsafe fn static_init_test_cbc(
    aes: &'static AesECB,
) -> &'static TestAes128Cbc<'static, AesECB<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

    static_init!(
        TestAes128Cbc<'static, AesECB>,
        TestAes128Cbc::new(&aes, key, iv, source, data)
    )
}

unsafe fn static_init_test_ecb(
    aes: &'static AesECB,
) -> &'static TestAes128Ecb<'static, AesECB<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);

    static_init!(
        TestAes128Ecb<'static, AesECB>,
        TestAes128Ecb::new(&aes, key, source, data)
    )
}
