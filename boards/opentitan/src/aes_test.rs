//! Test that AES ECB mode is working properly.
//!
//! To test ECB mode, add the following line to the opentitan boot sequence:
//! ```
//!     aes_test::run_aes128_ecb();
//! ```
//! You should see the following output:
//! ```
//!     aes_test passed (ECB Enc Src/Dst)
//!     aes_test passed (ECB Dec Src/Dst)
//!     aes_test passed (ECB Enc In-place)
//!     aes_test passed (ECB Dec In-place)
//! ```

use capsules::test::aes::TestAes128Ecb;
use ibex::aes::{Aes, AES};
use kernel::hil::symmetric_encryption::{AES128, AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::static_init;

pub unsafe fn run_aes128_ecb() {
    let t = static_init_test_ecb();
    AES.set_client(t);

    t.run();
}

unsafe fn static_init_test_ecb() -> &'static mut TestAes128Ecb<'static, Aes<'static>> {
    let source = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0; 4 * AES128_BLOCK_SIZE]);
    let data = static_init!([u8; 6 * AES128_BLOCK_SIZE], [0; 6 * AES128_BLOCK_SIZE]);
    let key = static_init!([u8; AES128_KEY_SIZE], [0; AES128_KEY_SIZE]);

    static_init!(
        TestAes128Ecb<'static, Aes>,
        TestAes128Ecb::new(&AES, key, source, data)
    )
}
