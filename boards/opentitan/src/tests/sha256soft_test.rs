//! Test SHA256 software implementation

use crate::tests::run_kernel_op;
use crate::SHA256SOFT;
use extra_capsules::test::sha256::TestSha256;
use kernel::debug;
use kernel::static_init;

#[test_case]
fn sha256software_verify() {
    debug!("start SHA256 verify test");

    let sha = unsafe { SHA256SOFT.unwrap() };

    let lstring = unsafe { static_init!([u8; 72], [0; 72]) };
    let bytes = b"hello ";

    for i in 0..12 {
        for j in 0..6 {
            lstring[i * 6 + j] = bytes[j];
        }
    }

    let lhash = unsafe {
        static_init!(
            [u8; 32],
            [
                0x59, 0x42, 0xc3, 0x71, 0x6f, 0x02, 0x82, 0x89, 0x3f, 0xbe, 0x04, 0x9b, 0xa2, 0x0e,
                0x56, 0x0e, 0x45, 0x94, 0xd5, 0xee, 0x15, 0xcb, 0x8a, 0x1e, 0x28, 0x7c, 0x20, 0x12,
                0xc2, 0xce, 0xb5, 0xa9
            ]
        )
    };

    let test = unsafe { static_init!(TestSha256, TestSha256::new(sha, lstring, lhash, true)) };
    test.run();

    run_kernel_op(1000);

    debug!("    [ok]");
    run_kernel_op(100);
}
