//! Test the AES hardware

use kernel::ReturnCode;
use kernel::hil;
use kernel::hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128, AES128Ctr, AES128CBC};
use sam4l::aes::{AES};

struct Test {
    encrypting: bool,
    mode_ctr: bool,
    use_source: bool,
}

static mut T: Test =
    // Test::new_ctr(true, true);
    // Test::new_ctr(false, true);
    // Test::new_ctr(true, false);
    // Test::new_ctr(false, false);
       Test::new_cbc(true, true);
    // Test::new_cbc(false, true);
    // Test::new_cbc(true, false);
    // Test::new_cbc(false, false);

pub fn run() {
    unsafe {
        AES.set_client(&T);
        T.run()
    }
}

impl Test {
    pub const fn new_ctr(encrypting: bool, use_source: bool) -> Test {
        Test {
            encrypting: encrypting,
            mode_ctr: true,
            use_source: use_source,
        }
    }

    pub const fn new_cbc(encrypting: bool, use_source: bool) -> Test {
        Test {
            encrypting: encrypting,
            mode_ctr: false,
            use_source: use_source,
        }
    }

    pub fn run(&self) { unsafe {
        AES.enable();

        assert!(AES.set_key(&KEY) == ReturnCode::SUCCESS);

        let iv = if self.mode_ctr { &IV_CTR } else { &IV_CBC };
        assert!(AES.set_iv(iv) == ReturnCode::SUCCESS);

        let source = if self.encrypting { &PTXT }
                     else { if self.mode_ctr { &CTXT_CTR }
                            else { &CTXT_CBC }
                          };
        if self.use_source {
            assert!(AES.set_source(Some(source)) == ReturnCode::SUCCESS);
        } else {
            assert!(AES.set_source(None) == ReturnCode::SUCCESS);

            // Copy source into dest and then crypt in-place
            for (i, b) in source.iter().enumerate() {
                DATA[DATA_OFFSET + i] = *b;
            }
        }

        assert!(AES.put_dest(Some(&mut DATA)) == ReturnCode::SUCCESS);

        if self.mode_ctr {
            AES.set_mode_aes128ctr(self.encrypting);
        } else {
            AES.set_mode_aes128cbc(self.encrypting);
        }
        AES.start_message();

        let start = DATA_OFFSET;
        let stop = DATA_OFFSET + DATA_LEN;
        assert!(AES.crypt(start, stop) == ReturnCode::SUCCESS);

        // await crypt_done()
    }}
}

impl hil::symmetric_encryption::Client for Test {
    fn crypt_done(&self) { unsafe {
        let dest = AES.take_dest().unwrap().unwrap();
        let expected = if self.encrypting { if self.mode_ctr { &CTXT_CTR }
                                            else { &CTXT_CBC } }
                       else { &PTXT };

        if &dest[DATA_OFFSET .. DATA_OFFSET + DATA_LEN] == expected.as_ref() {
            debug!("OK!");
        } else {
            debug!("FAIL");
            debug!("{:?}", dest);
        }
        AES.disable();
    }}
}

static mut DATA: [u8; 6 * AES128_BLOCK_SIZE] = [0; 6 * AES128_BLOCK_SIZE];

const DATA_OFFSET: usize = AES128_BLOCK_SIZE;
const DATA_LEN: usize = 4 * AES128_BLOCK_SIZE;

static KEY: [u8; AES128_BLOCK_SIZE] = [
    0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
    0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c
];

static IV_CTR: [u8; AES128_BLOCK_SIZE] = [
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
];

static IV_CBC: [u8; AES128_BLOCK_SIZE] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f
];

static PTXT: [u8; 4 * AES128_BLOCK_SIZE] = [
    0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96,
    0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c,
    0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11,
    0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
    0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
    0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10
];

static CTXT_CTR: [u8; 4 * AES128_BLOCK_SIZE] = [
    0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26,
    0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
    0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff,
    0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
    0x5a, 0xe4, 0xdf, 0x3e, 0xdb, 0xd5, 0xd3, 0x5e,
    0x5b, 0x4f, 0x09, 0x02, 0x0d, 0xb0, 0x3e, 0xab,
    0x1e, 0x03, 0x1d, 0xda, 0x2f, 0xbe, 0x03, 0xd1,
    0x79, 0x21, 0x70, 0xa0, 0xf3, 0x00, 0x9c, 0xee
];

static CTXT_CBC: [u8; 4 * AES128_BLOCK_SIZE] = [
    0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46,
    0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9, 0x19, 0x7d,
    0x50, 0x86, 0xcb, 0x9b, 0x50, 0x72, 0x19, 0xee,
    0x95, 0xdb, 0x11, 0x3a, 0x91, 0x76, 0x78, 0xb2,
    0x73, 0xbe, 0xd6, 0xb8, 0xe3, 0xc1, 0x74, 0x3b,
    0x71, 0x16, 0xe6, 0x9e, 0x22, 0x22, 0x95, 0x16,
    0x3f, 0xf1, 0xca, 0xa1, 0x68, 0x1f, 0xac, 0x09,
    0x12, 0x0e, 0xca, 0x30, 0x75, 0x86, 0xe1, 0xa7
];
