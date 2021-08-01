//! notice that there will be 18 tests, 6 for each,
//! and the test output will make the debug buffer full,
//! please go to boards/components/src/debug_writer.rs and change
//!     let buf = static_init!([u8; 1024], [0; 1024]);
//! to
//!     let buf = static_init!([u8; 4096], [0; 4096]);
//! Thanks!
//! To run this test, include the code
//! ```rust
//!    test::virtual_aes_ccm_test::run();
//! ```
//! In the boot sequence. If it runs correctly, you should see the following
//! output:
//!
//! AES CCM* encryption/decryption tests
//! AES CCM* encryption/decryption tests
//! AES CCM* encryption/decryption tests
//! Initialization complete. Entering main loop
//! aes_ccm_test passed: (current_test=0, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=0, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=1, encrypting=false, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=true, tag_is_valid=true)
//! aes_ccm_test passed: (current_test=2, encrypting=false, tag_is_valid=true)
use capsules::test::aes_ccm::Test;
use capsules::virtual_aes_ccm;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::hil::symmetric_encryption::{AES128, AES128CCM, AES128_BLOCK_SIZE};
use kernel::static_init;
use sam4l::aes::Aes;

type AESCCMMUX = virtual_aes_ccm::MuxAES128CCM<'static, Aes<'static>>;
type AESCCMCLIENT = virtual_aes_ccm::VirtualAES128CCM<'static, Aes<'static>>;

pub unsafe fn run(
    aes: &'static sam4l::aes::Aes,
    dynamic_deferred_caller: &'static DynamicDeferredCall,
) {
    // mux
    let ccm_mux = static_init!(
        AESCCMMUX,
        virtual_aes_ccm::MuxAES128CCM::new(aes, dynamic_deferred_caller)
    );
    ccm_mux.initialize_callback_handle(
        dynamic_deferred_caller
            .register(ccm_mux)
            .expect("no deferred call slot available for ccm mux"),
    );
    aes.set_client(ccm_mux);
    // ---------------- ONE CLIENT ---------------------
    // client 1
    const CRYPT_SIZE: usize = 7 * AES128_BLOCK_SIZE;
    let crypt_buf1 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
    let ccm_client1 = static_init!(
        AESCCMCLIENT,
        virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf1)
    );
    ccm_client1.setup();
    // test 1
    let data1 = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
    let t1 = static_init!(Test<'static, AESCCMCLIENT>, Test::new(ccm_client1, data1));
    ccm_client1.set_client(t1);
    // ---------------- ANOTHER CLIENT ---------------------
    // client 2
    let crypt_buf2 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
    let ccm_client2 = static_init!(
        AESCCMCLIENT,
        virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf2)
    );
    ccm_client2.setup();
    // test 2
    let data2 = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
    let t2 = static_init!(Test<'static, AESCCMCLIENT>, Test::new(ccm_client2, data2));
    ccm_client2.set_client(t2);

    // client 3
    let crypt_buf3 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
    let ccm_client3 = static_init!(
        AESCCMCLIENT,
        virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf3)
    );
    ccm_client3.setup();
    // test 3
    let data3 = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
    let t3 = static_init!(Test<'static, AESCCMCLIENT>, Test::new(ccm_client3, data3));
    ccm_client3.set_client(t3);
    // ----------------- RUN TESTS NOW ----------------------
    // run
    t1.run();
    t2.run();
    t3.run();
}
