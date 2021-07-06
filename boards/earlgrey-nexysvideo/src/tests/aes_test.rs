//! Test that AES ECB mode is working properly.

use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use capsules::test::aes::{TestAes128Cbc, TestAes128Ctr, TestAes128Ecb};
use earlgrey::aes::Aes;
use kernel::debug;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::static_init;

#[test_case]
fn run_aes128_ecb() {
    debug!("check run AES128 ECB... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_ecb(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        t.run();
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
        TestAes128Ecb::new(aes, key, source, data)
    )
}

#[test_case]
fn run_aes128_cbc() {
    debug!("check run AES128 CBC... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_cbc(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        t.run();
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
        TestAes128Cbc::new(aes, key, iv, source, data)
    )
}

#[test_case]
fn run_aes128_ctr() {
    debug!("check run AES128 CTR... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let aes = &perf.aes;

        let t = static_init_test_ctr(&aes);
        aes.set_client(t);

        #[cfg(feature = "hardware_tests")]
        t.run();
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
        TestAes128Ctr::new(aes, key, iv, source, data)
    )
}
