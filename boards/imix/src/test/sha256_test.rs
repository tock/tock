//! This tests a software SHA256 implementation. To run this test,
//! add this line to the imix boot sequence:
//! ```
//!     test::sha256_test::run_sha256(dynamic_deferred_caller);
//! ```
//! This test takes a dynamic deferred call (for callbacks). It tries to
//! hash 'hello world' and uses Digest::validate to check that the hash
//! is correct.
//!
//! The expected output is
//! Sha256Test: Verification result: Ok(true)
//!
//! This tests whether the SHA-256 hash of the string "hello hello
//! hello hello hello hello hello hello hello hello hello hello "
//! hashes correctly. This string is 12 repetitions of "hello ", so is
//! 72 bytes long. As SHA uses 64-byte/512 bit blocks, this verifies
//! that multi-block hashes work correctly.

use extra_capsules::sha256::Sha256Software;
use extra_capsules::test::sha256::TestSha256;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::static_init;

pub unsafe fn run_sha256(call: &'static DynamicDeferredCall) {
    let t = static_init_test_sha256(call);
    t.run();
}

// HSTRING is "hello world" and HHASH is the SHA-256 hash of this string.
pub static mut HSTRING: [u8; 11] = *b"hello world";

pub static mut HHASH: [u8; 32] = [
    0xB9, 0x4D, 0x27, 0xB9, 0x93, 0x4D, 0x3E, 0x08, 0xA5, 0x2E, 0x52, 0xD7, 0xDA, 0x7D, 0xAB, 0xFA,
    0xC4, 0x84, 0xEF, 0xE3, 0x7A, 0x53, 0x80, 0xEE, 0x90, 0x88, 0xF7, 0xAC, 0xE2, 0xEF, 0xCD, 0xE9,
];

// LSTRING is 12 repetitions of "hello " (72 bytes long) and LHASH is
// the SHA-256 hash of this string.
pub static mut LSTRING: [u8; 72] = [0; 72];
pub static mut LHASH: [u8; 32] = [
    0x59, 0x42, 0xc3, 0x71, 0x6f, 0x02, 0x82, 0x89, 0x3f, 0xbe, 0x04, 0x9b, 0xa2, 0x0e, 0x56, 0x0e,
    0x45, 0x94, 0xd5, 0xee, 0x15, 0xcb, 0x8a, 0x1e, 0x28, 0x7c, 0x20, 0x12, 0xc2, 0xce, 0xb5, 0xa9,
];

unsafe fn static_init_test_sha256(call: &'static DynamicDeferredCall) -> &'static TestSha256 {
    let sha = static_init!(Sha256Software<'static>, Sha256Software::new(call));
    sha.initialize_callback_handle(call.register(sha).unwrap());
    let bytes = b"hello ";
    for i in 0..12 {
        for j in 0..6 {
            LSTRING[i * 6 + j] = bytes[j];
        }
    }
    // We expect LSTRING to hash to LHASH, so final argument is true
    let test = static_init!(
        TestSha256,
        TestSha256::new(sha, &mut LSTRING, &mut LHASH, true)
    );

    test
}
