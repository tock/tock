// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test that AES ECB mode is working properly.

use crate::tests::run_kernel_op;
use crate::{AES, PERIPHERALS};
use capsules_aes_gcm::aes_gcm::Aes128Gcm;
use capsules_core::virtualizers::virtual_aes_ccm;
use capsules_extra::test::aes::{TestAes128Cbc, TestAes128Ctr, TestAes128Ecb};
use capsules_extra::test::aes_ccm;
use capsules_extra::test::aes_gcm;
use earlgrey::aes::Aes;
use kernel::debug;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::static_init;

/// The only 'test_case' for aes_test as directly invoked by the test runner,
/// this calls all the other tests, preserving the order in which they must
/// be ran.
#[test_case]
fn aes_tester() {
    run_aes128_ccm();
    run_aes128_gcm();
    run_aes128_ecb();
    run_aes128_cbc();
    run_aes128_ctr();
}

fn run_aes128_ccm() {
    debug!("check run AES128 CCM... ");
    run_kernel_op(100);

    unsafe {
        let aes = AES.unwrap();

        let t = static_init_test_ccm(&aes);
        kernel::hil::symmetric_encryption::AES128CCM::set_client(aes, t);

        t.run();
    }
    run_kernel_op(10000);
    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_test_ccm(
    aes: &'static Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
) -> &'static aes_ccm::Test<
    'static,
    Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
> {
    let buf = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);

    static_init!(
        aes_ccm::Test<
            'static,
            Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
        >,
        aes_ccm::Test::new(aes, buf)
    )
}

fn run_aes128_gcm() {
    debug!("check run AES128 GCM... ");
    run_kernel_op(100);

    unsafe {
        let aes = AES.unwrap();

        let t = static_init_test_gcm(&aes);
        kernel::hil::symmetric_encryption::AES128GCM::set_client(aes, t);

        #[cfg(feature = "hardware_tests")]
        t.run();
    }
    run_kernel_op(10000);
    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_test_gcm(
    aes: &'static Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
) -> &'static aes_gcm::Test<
    'static,
    Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
> {
    let buf = static_init!([u8; 9 * AES128_BLOCK_SIZE], [0; 9 * AES128_BLOCK_SIZE]);

    static_init!(
        aes_gcm::Test<
            'static,
            Aes128Gcm<'static, virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>>,
        >,
        aes_gcm::Test::new(aes, buf)
    )
}

fn run_aes128_ecb() {
    debug!("check run AES128 ECB... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_ecb(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        {
            while !aes.idle() {}
            t.run();
        }
    }
    run_kernel_op(1000);
    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_test_ecb(aes: &'static Aes) -> &'static TestAes128Ecb<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);

    static_init!(
        TestAes128Ecb<'static, Aes>,
        TestAes128Ecb::new(aes, key, source, data, true)
    )
}

fn run_aes128_cbc() {
    debug!("check run AES128 CBC... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_cbc(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        {
            while !aes.idle() {}
            t.run();
        }
    }
    run_kernel_op(1000);
    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_test_cbc(aes: &'static Aes) -> &'static TestAes128Cbc<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);

    static_init!(
        TestAes128Cbc<'static, Aes>,
        TestAes128Cbc::new(aes, key, iv, source, data, true)
    )
}

fn run_aes128_ctr() {
    debug!("check run AES128 CTR... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_ctr(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        {
            while !aes.idle() {}
            t.run();
        }
    }
    run_kernel_op(1000);
    debug!("    [ok]");
    run_kernel_op(100);
}

unsafe fn static_init_test_ctr(aes: &'static Aes) -> &'static TestAes128Ctr<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);

    static_init!(
        TestAes128Ctr<'static, Aes>,
        TestAes128Ctr::new(aes, key, iv, source, data, true)
    )
}
