use capsules::test::aes::{Test};
use sam4l::aes::{AES, Aes};
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE};

pub fn run() {
    let t = static_init_test();

    unsafe { AES.set_client(t); }

    t.run();
}

fn static_init_test() -> &'static mut Test<'static, Aes<'static>> {
    unsafe {
        let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
        let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
        let key = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);
        let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

        static_init!(Test<'static, Aes>, Test::new(&AES, key, iv, source, data))
    }
}
