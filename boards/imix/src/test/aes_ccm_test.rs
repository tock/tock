//! To run this test, include the code
//! ```
//!    test::aes_ccm_test::run();
//! ```
//! In the boot sequence. If it runs correctly, you should see the following
//! output:
//!
//! aes_ccm_test passed: (current_test=0, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=false, tag_is_valid=true)

use capsules::aes_ccm;
use capsules::test::aes_ccm::Test;
use kernel::hil::symmetric_encryption::{AES128, AES128CCM, AES128_BLOCK_SIZE};
use kernel::static_init;
use sam4l::aes::Aes;

pub unsafe fn run(aes: &'static Aes) {
    let ccm = static_init_ccm(aes);
    aes.set_client(ccm);

    let t = static_init_test(ccm);
    ccm.set_client(t);

    t.run();
}

unsafe fn static_init_ccm(aes: &'static Aes) -> &'static aes_ccm::AES128CCM<'static, Aes<'static>> {
    const CRYPT_SIZE: usize = 7 * AES128_BLOCK_SIZE;
    let crypt_buf = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
    static_init!(
        aes_ccm::AES128CCM<'static, Aes<'static>>,
        aes_ccm::AES128CCM::new(&aes, crypt_buf)
    )
}

type AESCCM = aes_ccm::AES128CCM<'static, Aes<'static>>;

unsafe fn static_init_test(aes_ccm: &'static AESCCM) -> &'static Test<'static, AESCCM> {
    let data = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
    static_init!(Test<'static, AESCCM>, Test::new(aes_ccm, data))
}
