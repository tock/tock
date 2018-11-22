//! Test that AES (either CTR or CBC mode) is working properly.
//!
//! To test CBC mode, add the following line to the imix boot sequence: 
//! ```
//!     aes_test::run_aes128_cbc();
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
//!     aes_test::run_aes128_ctr();
//! ```
//! You should see the following output: 
//! ```
//!     aes_test CTR passed: (CTR Enc Ctr Src/Dst)
//!     aes_test CTR passed: (CTR Dec Ctr Src/Dst)
//! ```

use capsules::test::aes::TestAes128Cbc;
use capsules::test::aes::TestAes128Ctr;
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use sam4l::aes::{Aes, AES};

pub unsafe fn run_aes128_ctr() {
    let t = static_init_test_ctr();
    AES.set_client(t);

    t.run();
}

pub unsafe fn run_aes128_cbc() {
    let t = static_init_test_cbc();
    AES.set_client(t);

    t.run();
}

unsafe fn static_init_test_ctr() -> &'static mut TestAes128Ctr<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

    static_init!(
        TestAes128Ctr<'static, Aes>,
        TestAes128Ctr::new(&AES, key, iv, source, data)
    )
}

unsafe fn static_init_test_cbc() -> &'static mut TestAes128Cbc<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);
    let iv = static_init!([u8; AES128_BLOCK_SIZE], [0; AES128_BLOCK_SIZE]);

    static_init!(
        TestAes128Cbc<'static, Aes>,
        TestAes128Cbc::new(&AES, key, iv, source, data)
    )
}
