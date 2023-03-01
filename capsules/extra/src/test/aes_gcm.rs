//! Test the AES GCM implementation on top of AES hardware.

use core::cell::Cell;
use kernel::debug;
use kernel::hil::symmetric_encryption::{GCMClient, AES128GCM, AES128_KEY_SIZE};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

pub struct Test<'a, A: AES128GCM<'a>> {
    aes_gcm: &'a A,

    buf: TakeCell<'static, [u8]>,
    current_test: Cell<usize>,
    encrypting: Cell<bool>,

    // (key, iv, pt, aad, ct, tag)
    tests: [(
        &'static [u8],
        &'static [u8],
        &'static [u8],
        &'static [u8],
        &'static [u8],
        &'static [u8],
    ); 3],
}

impl<'a, A: AES128GCM<'a>> Test<'a, A> {
    pub fn new(aes_gcm: &'a A, buf: &'static mut [u8]) -> Test<'a, A> {
        Test {
            aes_gcm: aes_gcm,
            buf: TakeCell::new(buf),
            current_test: Cell::new(0),
            encrypting: Cell::new(true),
            tests: [
                (&KEY_128_ZERO, &IV_128_ZERO, &[], &[], &[], &TAG_128_ZERO),
                (
                    &KEY_128_TWELVE,
                    &IV_128_TWELVE,
                    &[],
                    &AAD_128_TWELVE,
                    &[],
                    &TAG_128_TWELVE,
                ),
                (
                    &KEY_128_THIRTEEN,
                    &IV_128_THIRTEEN,
                    &PT_128_THIRTEEN,
                    &[],
                    &CT_128_THIRTEEN,
                    &TAG_128_THIRTEEN,
                ),
            ],
        }
    }

    pub fn run(&self) {
        debug!("AES GCM* encryption/decryption tests");
        self.trigger_test();
    }

    fn next_test(&self) -> bool {
        if self.encrypting.get() {
            self.encrypting.set(false);
        } else {
            self.encrypting.set(true);
            self.current_test.set(self.current_test.get() + 1);
            if self.current_test.get() >= self.tests.len() {
                return false;
            }
        }
        true
    }

    fn trigger_test(&self) {
        let (key, iv, pt, aad, ct, tag) = self.tests[self.current_test.get()];
        let (aad_off, pt_off, pt_len) = (0, aad.len(), pt.len());
        let encrypting = self.encrypting.get();

        let buf = match self.buf.take() {
            None => panic!("aes_gcm_test failed: buffer is not present in trigger_test."),
            Some(buf) => buf,
        };

        if encrypting {
            buf[aad_off..pt_off].copy_from_slice(aad);
            buf[pt_off..pt_off + pt_len].copy_from_slice(pt);
        } else {
            buf[aad_off..pt_off].copy_from_slice(aad);
            buf[pt_off..pt_off + pt_len].copy_from_slice(ct);
            buf[pt_off + pt_len..(pt_off + pt_len + tag.len())].copy_from_slice(tag);
        }

        if self.aes_gcm.set_key(key) != Ok(()) {
            panic!("aes_gcm_test failed: cannot set key.");
        }

        if self.aes_gcm.set_iv(&iv) != Ok(()) {
            panic!("aes_gcm_test failed: cannot set IV.");
        }

        let _ = self
            .aes_gcm
            .crypt(buf, aad_off, pt_off, pt_len, encrypting)
            .map_err(|(_code, buf)| {
                self.buf.replace(buf);
                panic!("Failed to start test.");
            });
    }

    fn check_test(&self, tag_is_valid: bool) {
        let (_key, _iv, pt, aad, ct, tag) = self.tests[self.current_test.get()];
        let (_aad_off, pt_off, pt_len) = (0, aad.len(), pt.len());
        let encrypting = self.encrypting.get();

        let buf = match self.buf.take() {
            None => panic!("aes_gcm_test failed: buffer is not present in check_test."),
            Some(buf) => buf,
        };

        if encrypting {
            let ct_matches = buf[pt_off..(pt_off + pt_len)]
                .iter()
                .zip(ct.iter())
                .all(|(a, b)| *a == *b);
            let tag_matches = buf[(pt_off + pt_len)..(pt_off + pt_len + tag.len())]
                .iter()
                .zip(tag.iter())
                .all(|(a, b)| *a == *b);

            if ct_matches && tag_matches && tag_is_valid {
                debug!(
                    "aes_gcm_test passed: (current_test={}, encrypting={}, tag_is_valid={})",
                    self.current_test.get(),
                    self.encrypting.get(),
                    tag_is_valid
                );
            } else {
                panic!("aes_gcm_test failed: ct_matches={}, tag_matches={}, (current_test={}, encrypting={}, tag_is_valid={}",
                       ct_matches,
                       tag_matches,
                       self.current_test.get(),
                       self.encrypting.get(),
                       tag_is_valid);
            }
        } else {
            let pt_matches = buf[pt_off..(pt_off + pt_len)]
                .iter()
                .zip(pt.iter())
                .all(|(a, b)| *a == *b);
            let tag_matches = buf[(pt_off + pt_len)..(pt_off + pt_len + tag.len())]
                .iter()
                .zip(tag.iter())
                .all(|(a, b)| *a == *b);

            if pt_matches && tag_matches && tag_is_valid {
                debug!(
                    "aes_gcm_test passed: (current_test={}, encrypting={}, tag_is_valid={})",
                    self.current_test.get(),
                    self.encrypting.get(),
                    tag_is_valid
                );
            } else {
                panic!("aes_gcm_test failed: pt_matches={}, tag_matches={}, (current_test={}, encrypting={}, tag_is_valid={}",
                       pt_matches,
                       tag_matches,
                       self.current_test.get(),
                       self.encrypting.get(),
                       tag_is_valid);
            }
        }

        self.buf.replace(buf);
    }
}

impl<'a, A: AES128GCM<'a>> GCMClient for Test<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.buf.replace(buf);
        if res != Ok(()) {
            panic!("aes_gcm_test failed: crypt_done returned {:?}", res);
        } else {
            self.check_test(tag_is_valid);
            if self.next_test() {
                self.trigger_test()
            }
        }
    }
}

static KEY_128_ZERO: [u8; AES128_KEY_SIZE] = [
    0x11, 0x75, 0x4c, 0xd7, 0x2a, 0xec, 0x30, 0x9b, 0xf5, 0x2f, 0x76, 0x87, 0x21, 0x2e, 0x89, 0x57,
];

static IV_128_ZERO: [u8; 12] = [
    0x3c, 0x81, 0x9d, 0x9a, 0x9b, 0xed, 0x08, 0x76, 0x15, 0x03, 0x0b, 0x65,
];

static TAG_128_ZERO: [u8; AES128_KEY_SIZE] = [
    0x25, 0x03, 0x27, 0xc6, 0x74, 0xaa, 0xf4, 0x77, 0xae, 0xf2, 0x67, 0x57, 0x48, 0xcf, 0x69, 0x71,
];

static KEY_128_TWELVE: [u8; AES128_KEY_SIZE] = [
    0x26, 0x73, 0x0f, 0x1a, 0xd2, 0x4b, 0x76, 0xd6, 0x6f, 0x7a, 0xb8, 0x45, 0x9d, 0xdc, 0xd1, 0x17,
];

static IV_128_TWELVE: [u8; 12] = [
    0x1f, 0xfb, 0x3e, 0x75, 0x71, 0xcb, 0x70, 0x14, 0x5e, 0xa5, 0x16, 0x53,
];

static AAD_128_TWELVE: [u8; 90] = [
    0xbf, 0xc3, 0xa8, 0x08, 0xc0, 0x60, 0xcd, 0xfd, 0x2a, 0xb7, 0x69, 0x1b, 0x32, 0x4a, 0xb3, 0x59,
    0x29, 0xe8, 0x0f, 0x26, 0x2b, 0xf3, 0xb9, 0x4c, 0xc2, 0xf4, 0x5c, 0x62, 0xbb, 0x0f, 0x32, 0xbc,
    0x4e, 0x4b, 0x96, 0x73, 0x69, 0x11, 0x0a, 0x7b, 0x4c, 0x47, 0x82, 0x7e, 0x93, 0xa9, 0xec, 0xd7,
    0xfc, 0xda, 0x5e, 0x6a, 0x97, 0x39, 0xa0, 0xd1, 0x78, 0x6d, 0x6d, 0xc7, 0xa4, 0x5c, 0x9c, 0x1e,
    0x8e, 0xcc, 0x8f, 0x90, 0xdc, 0x70, 0xbc, 0x5a, 0x5a, 0xe1, 0xa0, 0x31, 0x3f, 0xd6, 0xef, 0x87,
    0xd7, 0xb3, 0x6e, 0x3d, 0x48, 0xc4, 0x44, 0x8f, 0x70, 0x3e,
];

static TAG_128_TWELVE: [u8; 14] = [
    0x45, 0xa9, 0xbe, 0x4c, 0x84, 0x9e, 0xcb, 0x25, 0x85, 0x42, 0x1a, 0x1f, 0x08, 0xe6,
];

static KEY_128_THIRTEEN: [u8; AES128_KEY_SIZE] = [
    0x8f, 0x85, 0xd3, 0x66, 0x16, 0xa9, 0x5f, 0xc1, 0x05, 0x86, 0xc3, 0x16, 0xb3, 0x05, 0x37, 0x70,
];

static IV_128_THIRTEEN: [u8; 12] = [
    0xd3, 0x20, 0xb5, 0x00, 0x26, 0x96, 0x09, 0xac, 0xe1, 0xbe, 0x67, 0xce,
];

static PT_128_THIRTEEN: [u8; 32] = [
    0x3a, 0x75, 0x8e, 0xe0, 0x72, 0xfc, 0x70, 0xa6, 0x42, 0x75, 0xb5, 0x6e, 0x72, 0xcb, 0x23, 0xa1,
    0x59, 0x04, 0x58, 0x9c, 0xef, 0xbe, 0xeb, 0x58, 0x48, 0xec, 0x53, 0xff, 0xc0, 0x6c, 0x7a, 0x5d,
];

static CT_128_THIRTEEN: [u8; 32] = [
    0xfb, 0x2f, 0xe3, 0xeb, 0x40, 0xed, 0xfb, 0xd2, 0x2a, 0x51, 0x6b, 0xec, 0x35, 0x9d, 0x4b, 0xb4,
    0x23, 0x8a, 0x07, 0x00, 0xa4, 0x6f, 0xee, 0x11, 0x36, 0xa0, 0x61, 0x85, 0x40, 0x22, 0x9c, 0x41,
];

static TAG_128_THIRTEEN: [u8; 16] = [
    0x42, 0x26, 0x93, 0x16, 0xce, 0xce, 0x7d, 0x88, 0x2c, 0xc6, 0x8c, 0x3e, 0xd9, 0xd2, 0xf0, 0xae,
];
