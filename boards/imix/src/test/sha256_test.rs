//! This tests a software SHA256 implementation. To run this test,
//! add this line to the imix boot sequence:
//! ```
//!     test::sha256_test::run_sha256();
//! ```
//! This test takes a dynamic deferred call (for callbacks). It tries to
//! hash 'hello world' and uses Digest::validate to check that the hash
//! is correct.
//!
//! The expected output is
//! Sha256Test: Verification result: Ok(true)
//!

use capsules::sha256::Sha256Software;
use capsules::test::sha256::TestSha256;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::static_init;


pub unsafe fn run_sha256(call: &'static DynamicDeferredCall) {
    let t = static_init_test_sha256(call);
    t.run();
}

pub static mut HSTRING: [u8;11] = ['h' as u8, 'e' as u8, 'l' as u8, 'l' as u8,
                                   'o' as u8, ' ' as u8, 'w' as u8, 'o' as u8,
                                   'r' as u8, 'l' as u8, 'd' as u8];
pub static mut HHASH: [u8;32] = [0xB9, 0x4D, 0x27, 0xB9, 0x93, 0x4D, 0x3E, 0x08,
                                 0xA5, 0x2E, 0x52, 0xD7, 0xDA, 0x7D, 0xAB, 0xFA,
                                 0xC4, 0x84, 0xEF, 0xE3, 0x7A, 0x53, 0x80, 0xEE,
                                 0x90, 0x88, 0xF7, 0xAC, 0xE2, 0xEF, 0xCD, 0xE9];
pub static mut LSTRING: [u8; 34] = [0; 34];
pub static mut LHASH: [u8;32] = [0x2c, 0x81, 0xe8, 0x49, 0x7b, 0x81, 0xfa, 0x37,
                                 0x0a, 0x3d, 0xec, 0x71, 0x10, 0x06, 0xa3, 0xb9,
                                 0xe8, 0x81, 0xe7, 0x78, 0x8e, 0x93, 0x15, 0x32,
                                 0x8d, 0x14, 0xe2, 0xd0, 0xf3, 0xea, 0x16, 0xa1];

unsafe fn static_init_test_sha256(call: &'static DynamicDeferredCall) -> &'static TestSha256 {
    let sha = static_init!(Sha256Software<'static>,
                           Sha256Software::new(call));
    sha.initialize_callback_handle(call.register(sha).unwrap());
    let bytes = "the quick brown fox jumped over the lazy dog".as_bytes();
    for i in 0..34 {
        LSTRING[i] = bytes[i];
    }
    let test = static_init!(TestSha256, TestSha256::new(sha,
                                                        &mut LSTRING,
                                                        &mut LHASH));
    
    test
}
