use capsules::test::aes::TestAes128Cbc;
use capsules::test::aes::TestAes128Ctr;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE};
use sam4l::aes::{AES, Aes};

pub fn run_aes128_ctr() {
    let t = static_init_test_ctr();

    unsafe {
        AES.set_client(t);
    }

    t.run();
}

pub fn run_aes128_cbc() {
    let t = static_init_test_cbc();

    unsafe {
        AES.set_client(t);
    }

    t.run();
}


fn static_init_test_ctr() -> &'static mut TestAes128Ctr<'static, Aes<'static>> {
    unsafe {
        let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
        let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
        let key = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);
        let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

        static_init!(TestAes128Ctr<'static, Aes>, TestAes128Ctr::new(&AES, key, iv, source, data))
    }
}

fn static_init_test_cbc() -> &'static mut TestAes128Cbc<'static, Aes<'static>> {
    unsafe {
        let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
        let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
        let key = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);
        let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

        static_init!(TestAes128Cbc<'static, Aes>, TestAes128Cbc::new(&AES, key, iv, source, data))
    }
}
