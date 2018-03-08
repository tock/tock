use capsules::test::aes::TestAes128Ctr;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use nrf5x::aes::{AesECB, AESECB};

pub fn run() {
    let t = static_init_test();

    unsafe {
        AESECB.set_client(t);
    }

    t.run();
}

fn static_init_test() -> &'static mut TestAes128Ctr<'static, AesECB<'static>> {
    unsafe {
        let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
        let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
        let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
        let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

        static_init!(
            TestAes128Ctr<'static, AesECB>,
            TestAes128Ctr::new(&AESECB, key, iv, source, data)
        )
    }
}
