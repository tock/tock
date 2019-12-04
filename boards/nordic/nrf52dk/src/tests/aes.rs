use capsules::test::aes::TestAes128Ctr;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::static_init;
use nrf52832::aes::{AesECB, AESECB};

/// To run the tests add the following `main.rs::reset_handler` somewhere after that the AES
/// peripheral has been initialized:
///
/// ```rustc
///     aes::run();
/// ```
///
pub unsafe fn run() {
    let t = static_init_test();
    AESECB.set_client(t);
    t.run();
}

unsafe fn static_init_test() -> &'static mut TestAes128Ctr<'static, AesECB<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

    static_init!(
        TestAes128Ctr<'static, AesECB>,
        TestAes128Ctr::new(&AESECB, key, iv, source, data)
    )
}
