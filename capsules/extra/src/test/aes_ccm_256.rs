// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! Test the AES-256-CCM implementation using NIST vectors.
//!
//! Each test vector is run twice: once encrypting, once decrypting.
//! The following cases are covered:
//!
//!   Vec 0 — auth-only (confidential=false), AAD only, no plaintext, 16-byte MIC
//!   Vec 1 — full AEAD: AAD + plaintext, 16-byte MIC
//!   Vec 2 — full AEAD: AAD + plaintext, 16-byte MIC
//!   Vec 3 — full AEAD: AAD + plaintext, 16-byte MIC
//!   Vec 4 — full AEAD: AAD + plaintext, 16-byte MIC
//!   Vec 5 — tampered MIC: decryption must report tag_is_valid = false
//!   Vec 6 — full AEAD: AAD + plaintext, 14-byte MIC
//!
//! Buffer layout for each vector:
//!   [0 .. a_data.len()] — AAD (a_data)
//!   [a_data.len() .. a_data.len() + m_data.len()] — plaintext (enc)
//!   [a_data.len() .. a_data.len() + m_data.len() + tag.len()] — ciphertext + MIC (dec)

use core::cell::Cell;
use kernel::debug;
use kernel::hil::symmetric_encryption::{CCMClient, AES256, AES256_KEY_SIZE, AESCCM};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

const BUF_LEN: usize = 128;

pub struct TestAES256Ccm<'a, A: AESCCM<'a, AES256>> {
    aes_ccm: &'a A,
    buf: TakeCell<'static, [u8]>,
    current_test: Cell<usize>,
    encrypting: Cell<bool>,
}

struct Vector {
    key: &'static [u8],
    nonce: &'static [u8],
    a_data: &'static [u8],
    m_data: &'static [u8],
    c_data: &'static [u8], // ciphertext + MIC concatenated
    mic_len: usize,
    confidential: bool,
    expect_tag_invalid: bool,
}

impl<'a, A: AESCCM<'a, AES256>> TestAES256Ccm<'a, A> {
    pub fn new(aes_ccm: &'a A, buf: &'static mut [u8]) -> Self {
        assert!(buf.len() >= BUF_LEN, "buffer too small for CCM-256 tests");
        TestAES256Ccm {
            aes_ccm,
            buf: TakeCell::new(buf),
            current_test: Cell::new(0),
            encrypting: Cell::new(true),
        }
    }

    pub fn run(&self) {
        debug!(
            "AES-256-CCM test suite starting ({} vectors)",
            VECTORS.len()
        );
        self.trigger();
    }

    fn vector(&self) -> &'static Vector {
        &VECTORS[self.current_test.get()]
    }

    fn trigger(&self) {
        let v = self.vector();
        let encrypting = self.encrypting.get();
        let a_off = 0;
        let m_off = v.a_data.len();
        let m_len = v.m_data.len();

        let buf = self
            .buf
            .take()
            .expect("aes256ccm_test: buffer missing in trigger");

        buf[..BUF_LEN].fill(0);
        buf[a_off..m_off].copy_from_slice(v.a_data);

        if encrypting {
            buf[m_off..m_off + m_len].copy_from_slice(v.m_data);
        } else {
            if v.expect_tag_invalid {
                // Copy correct ciphertext+MIC then corrupt the MIC
                buf[m_off..m_off + m_len + v.mic_len]
                    .copy_from_slice(&v.c_data[0..v.mic_len + m_len]);
                buf[m_off + m_len] ^= 0xFF;
            } else {
                buf[m_off..m_off + v.mic_len + m_len]
                    .copy_from_slice(&v.c_data[0..v.mic_len + m_len]);
            }
        }

        match self.aes_ccm.set_key(v.key) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "aes256ccm_test vec={} enc={} returned {:?}: set_key failed",
                    self.current_test.get(),
                    encrypting,
                    e,
                );
            }
        }
        match self.aes_ccm.set_nonce(v.nonce) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "aes256ccm_test vec={} enc={} returned {:?}: set_nonce failed",
                    self.current_test.get(),
                    encrypting,
                    e,
                );
            }
        }

        self.aes_ccm
            .crypt(
                buf,
                a_off,
                m_off,
                m_len,
                v.mic_len,
                v.confidential,
                encrypting,
            )
            .unwrap_or_else(|(code, buf)| {
                self.buf.replace(buf);
                panic!(
                    "aes256ccm_test vec={} enc={}: crypt() returned {:?}",
                    self.current_test.get(),
                    encrypting,
                    code
                );
            });
    }

    fn check_test(&self, tag_is_valid: bool) {
        let v = self.vector();
        let encrypting = self.encrypting.get();
        let test_idx = self.current_test.get();
        let a_off = 0;
        let m_off = v.a_data.len();
        let m_len = v.m_data.len();
        let tag_len = v.mic_len;

        let buf = self
            .buf
            .take()
            .expect("aes256ccm_test: buffer missing in check_test");

        // AAD must be unchanged in both directions
        let a_ok = &buf[a_off..m_off] == v.a_data;

        if encrypting {
            // Test the payload ciphertext
            let c_ok = buf[m_off..m_off + m_len] == v.c_data[0..m_len];

            // Test the tag separately
            let expected_tag = &v.c_data[m_len..m_len + tag_len];
            let actual_tag = &buf[m_off + m_len..m_off + m_len + tag_len];
            let tag_match = expected_tag == actual_tag;

            if !a_ok || !c_ok || !tag_match || !tag_is_valid {
                panic!(
                    "aes256ccm_test FAILED vec={} enc=true: \
                     a_ok={} c_ok={} tag_match={} tag_is_valid={}",
                    test_idx, a_ok, c_ok, tag_match, tag_is_valid
                );
            }
        } else {
            if v.expect_tag_invalid {
                if tag_is_valid {
                    panic!(
                        "aes256ccm_test FAILED vec={} enc=false: \
                         expected tag_is_valid=false for corrupted MIC, got true",
                        test_idx
                    );
                }
                debug!(
                    "aes256ccm_test passed vec={} enc=false (corrupted MIC correctly rejected)",
                    test_idx
                );
                self.buf.replace(buf);
                return;
            }

            let m_ok = &buf[m_off..m_off + m_len] == v.m_data;
            if !a_ok || !m_ok || !tag_is_valid {
                panic!(
                    "aes256ccm_test FAILED vec={} enc=false: \
                     a_ok={} m_ok={} tag_is_valid={}",
                    test_idx, a_ok, m_ok, tag_is_valid
                );
            }
        }

        debug!("aes256ccm_test passed vec={} enc={}", test_idx, encrypting);
        self.buf.replace(buf);
    }

    fn advance(&self) -> bool {
        if self.encrypting.get() {
            self.encrypting.set(false);
            true
        } else {
            self.encrypting.set(true);
            let next = self.current_test.get() + 1;
            self.current_test.set(next);
            next < VECTORS.len()
        }
    }
}

impl<'a, A: AESCCM<'a, AES256>> CCMClient for TestAES256Ccm<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.buf.replace(buf);
        if res != Ok(()) {
            panic!(
                "aes256ccm_test vec={} enc={}: crypt_done error {:?}",
                self.current_test.get(),
                self.encrypting.get(),
                res
            );
        }
        self.check_test(tag_is_valid);
        if self.advance() {
            self.trigger();
        } else {
            debug!("AES-256-CCM all tests passed");
        }
    }
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

static VECTORS: &[Vector] = &[
    Vector {
        key: &KEY_0,
        nonce: &NONCE_0,
        a_data: &A_DATA_0,
        m_data: &[],
        c_data: &MIC_0,
        mic_len: 16,
        confidential: false,
        expect_tag_invalid: false,
    },
    Vector {
        key: &KEY_1,
        nonce: &NONCE_1,
        a_data: &A_DATA_1,
        m_data: &M_DATA_1,
        c_data: &C_DATA_1,
        mic_len: 16,
        confidential: true,
        expect_tag_invalid: false,
    },
    Vector {
        key: &KEY_2,
        nonce: &NONCE_2,
        a_data: &A_DATA_2,
        m_data: &M_DATA_2,
        c_data: &C_DATA_2,
        mic_len: 16,
        confidential: true,
        expect_tag_invalid: false,
    },
    Vector {
        key: &KEY_3,
        nonce: &NONCE_3,
        a_data: &A_DATA_3,
        m_data: &M_DATA_3,
        c_data: &C_DATA_3,
        mic_len: 16,
        confidential: true,
        expect_tag_invalid: false,
    },
    Vector {
        key: &KEY_4,
        nonce: &NONCE_4,
        a_data: &A_DATA_4,
        m_data: &M_DATA_4,
        c_data: &C_DATA_4,
        mic_len: 16,
        confidential: true,
        expect_tag_invalid: false,
    },
    Vector {
        key: &KEY_5,
        nonce: &NONCE_5,
        a_data: &A_DATA_5,
        m_data: &M_DATA_5,
        c_data: &C_DATA_5,
        mic_len: 16,
        confidential: true,
        expect_tag_invalid: true,
    },
    Vector {
        key: &KEY_6,
        nonce: &NONCE_6,
        a_data: &A_DATA_6,
        m_data: &M_DATA_6,
        c_data: &C_DATA_6,
        mic_len: 14,
        confidential: true,
        expect_tag_invalid: false,
    },
];

// ---------------------------------------------------------------------------
// Vector 0
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_0: [u8; AES256_KEY_SIZE] = [
    0xa4, 0xbc, 0x10, 0xb1, 0xa6, 0x2c, 0x96, 0xd4,
    0x59, 0xfb, 0xaf, 0x3a, 0x5a, 0xa3, 0xfa, 0xce,
    0x73, 0x13, 0xbb, 0x9e, 0x12, 0x53, 0xe6, 0x96,
    0xf9, 0x6a, 0x7a, 0x8e, 0x36, 0x80, 0x10, 0x88,
];
#[rustfmt::skip]
static NONCE_0: [u8; 7] = [
    0xa5, 0x44, 0x21, 0x8d, 0xad, 0xd3, 0xc1,
];
#[rustfmt::skip]
static A_DATA_0: [u8; 32] = [
    0xd3, 0xd5, 0x42, 0x4e, 0x20, 0xfb, 0xec, 0x43,
    0xae, 0x49, 0x53, 0x53, 0xed, 0x83, 0x02, 0x71,
    0x51, 0x5a, 0xb1, 0x04, 0xf8, 0x86, 0x0c, 0x98,
    0x8d, 0x15, 0xb6, 0xd3, 0x6c, 0x03, 0x8e, 0xab,
];
#[rustfmt::skip]
static MIC_0: [u8; 16] = [
    0x93, 0xaf, 0x11, 0xa0, 0x83, 0x79, 0xeb, 0x37,
    0xa1, 0x6a, 0xa2, 0x83, 0x7f, 0x09, 0xd6, 0x9d,
];

// ---------------------------------------------------------------------------
// Vector 1
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_1: [u8; AES256_KEY_SIZE] = [
    0x52, 0x09, 0x02, 0xaa, 0x27, 0xc1, 0x6d, 0xee,
    0x11, 0x28, 0x12, 0xb2, 0xe6, 0x85, 0xaa, 0x20,
    0x3a, 0xeb, 0x8b, 0x86, 0x33, 0xbd, 0x1b, 0xfc,
    0x99, 0x72, 0x8a, 0x48, 0x2d, 0x96, 0xc1, 0xfe,
];
#[rustfmt::skip]
static NONCE_1: [u8; 13] = [
    0xdd, 0xf5, 0x05, 0x02, 0xf4, 0x14, 0xc1, 0xbf,
    0x24, 0x88, 0x8f, 0x13, 0x28,
];
#[rustfmt::skip]
static A_DATA_1: [u8; 15] = [
    0x22, 0xb4, 0xf8, 0xf1, 0xaa, 0xc0, 0x2a, 0x9b,
    0x2e, 0xf7, 0x85, 0xd0, 0xff, 0x6f, 0x93,
];
#[rustfmt::skip]
static M_DATA_1: [u8; 24] = [
    0x53, 0x3f, 0xee, 0x7d, 0x2c, 0x77, 0x40, 0xdb,
    0x55, 0x77, 0x0e, 0x48, 0xcb, 0x1b, 0x54, 0x1d,
    0x99, 0x0e, 0xa3, 0xf8, 0xf0, 0x8e, 0xd1, 0xa6,
];
#[rustfmt::skip]
static C_DATA_1: [u8; 40] = [
    0xfc, 0x86, 0x7b, 0x31, 0x9e, 0x0e, 0x4a, 0xb4,
    0x5e, 0xc5, 0x18, 0xa1, 0xb5, 0xdc, 0xec, 0x4f,
    0x29, 0x98, 0x21, 0x73, 0xf3, 0xab, 0xfd, 0x4d,
    0x8a, 0x8f, 0x8d, 0x14, 0xd2, 0xbd, 0xac, 0x84,
    0xc3, 0x73, 0x7c, 0xfb, 0xd7, 0x5b, 0x7c, 0x0b,
];

// ---------------------------------------------------------------------------
// Vector 2
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_2: [u8; AES256_KEY_SIZE] = [
    0x0e, 0xbd, 0xc6, 0xdd, 0xb4, 0xc5, 0x02, 0x72,
    0x5d, 0xd6, 0xee, 0x8d, 0xa9, 0x5d, 0x56, 0xa0,
    0xd1, 0x04, 0x4b, 0x46, 0x94, 0xd6, 0xba, 0x84,
    0x75, 0xa4, 0x43, 0x4f, 0x23, 0xa8, 0x47, 0x4f,
];
#[rustfmt::skip]
static NONCE_2: [u8; 13] = [
    0xfb, 0x71, 0x7a, 0x8c, 0x82, 0x11, 0x44, 0x77,
    0x25, 0x3a, 0xcc, 0x14, 0xf6,
];
#[rustfmt::skip]
static A_DATA_2: [u8; 19] = [
    0x41, 0xe9, 0xd6, 0x56, 0x32, 0xf7, 0x4f, 0x44,
    0x9a, 0x68, 0x42, 0xd5, 0xe6, 0xc4, 0xa8, 0x6e,
    0xf8, 0x37, 0x91,
];
#[rustfmt::skip]
static M_DATA_2: [u8; 24] = [
    0xc7, 0x36, 0x02, 0x82, 0xc8, 0x54, 0x84, 0xa5,
    0xa3, 0x3a, 0xb1, 0xc6, 0x8d, 0xd7, 0x08, 0x73,
    0xab, 0x4e, 0x74, 0xff, 0xd4, 0xa6, 0x2c, 0xd5,
];
#[rustfmt::skip]
static C_DATA_2: [u8; 40] = [
    0x2e, 0x96, 0x1b, 0x3a, 0x2f, 0xa1, 0x60, 0x9a,
    0x4e, 0x6f, 0xd0, 0x4b, 0xff, 0x6a, 0xc5, 0xe3,
    0x06, 0xae, 0x26, 0x38, 0x70, 0x6f, 0x99, 0x7b,
    0x42, 0xbe, 0x2e, 0x2b, 0xa0, 0x5c, 0x54, 0xb6,
    0x19, 0x85, 0x0d, 0xb5, 0xc9, 0xd6, 0x84, 0xfe,
];

// ---------------------------------------------------------------------------
// Vector 3
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_3: [u8; AES256_KEY_SIZE] = [
    0x4a, 0x75, 0xff, 0x2f, 0x66, 0xda, 0xe2, 0x93,
    0x54, 0x03, 0xcc, 0xe2, 0x7e, 0x82, 0x9a, 0xd8,
    0xbe, 0x98, 0x18, 0x5c, 0x73, 0xf8, 0xbc, 0x61,
    0xd3, 0xce, 0x95, 0x0a, 0x83, 0x00, 0x7e, 0x11,
];
#[rustfmt::skip]
static NONCE_3: [u8; 13] = [
    0x46, 0xeb, 0x39, 0x0b, 0x17, 0x5e, 0x75, 0xda,
    0x61, 0x93, 0xd7, 0xed, 0xb6,
];
#[rustfmt::skip]
static A_DATA_3: [u8; 32] = [
    0x28, 0x2f, 0x05, 0xf7, 0x34, 0xf2, 0x49, 0xc0,
    0x53, 0x5e, 0xe3, 0x96, 0x28, 0x22, 0x18, 0xb7,
    0xc4, 0x91, 0x3c, 0x39, 0xb5, 0x9a, 0xd2, 0xa0,
    0x3f, 0xfa, 0xf5, 0xb0, 0xe9, 0xb0, 0xf7, 0x80,
];
#[rustfmt::skip]
static M_DATA_3: [u8; 24] = [
    0x20, 0x5f, 0x2a, 0x66, 0x4a, 0x85, 0x12, 0xe1,
    0x83, 0x21, 0xa9, 0x1c, 0x13, 0xec, 0x13, 0xb9,
    0xe6, 0xb6, 0x33, 0x22, 0x8c, 0x57, 0xcc, 0x1e,
];
#[rustfmt::skip]
static C_DATA_3: [u8; 40] = [
    0x58, 0xf1, 0x58, 0x4f, 0x76, 0x19, 0x83, 0xbe,
    0xf4, 0xd0, 0x06, 0x07, 0x46, 0xb5, 0xd5, 0xee,
    0x61, 0x0e, 0xcf, 0xda, 0x31, 0x10, 0x1a, 0x7f,
    0x54, 0x60, 0xe9, 0xb7, 0x85, 0x6d, 0x60, 0xa5,
    0xad, 0x98, 0x03, 0xc0, 0x76, 0x2f, 0x81, 0x76,
];

// ---------------------------------------------------------------------------
// Vector 4
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_4: [u8; AES256_KEY_SIZE] = [
    0x9c, 0xde, 0xba, 0xee, 0xe8, 0x69, 0x0b, 0x68,
    0x75, 0x10, 0x70, 0x69, 0x1f, 0x49, 0x59, 0x36,
    0x68, 0xa6, 0xde, 0x12, 0xd3, 0xa9, 0x48, 0xb3,
    0x8d, 0xdb, 0xd3, 0xf7, 0x52, 0x18, 0xb2, 0xd4,
];
#[rustfmt::skip]
static NONCE_4: [u8; 13] = [
    0xaf, 0x1a, 0x97, 0xd4, 0x31, 0x51, 0xf5, 0xea,
    0x9c, 0x48, 0xad, 0x36, 0xa3,
];
#[rustfmt::skip]
static A_DATA_4: [u8; 32] = [
    0xf5, 0x35, 0x3f, 0xb6, 0xbf, 0xc8, 0xf0, 0x9d,
    0x55, 0x61, 0x58, 0x13, 0x2d, 0x6c, 0xbb, 0x97,
    0xd9, 0x04, 0x5e, 0xac, 0xdc, 0x71, 0xf7, 0x82,
    0xbc, 0xef, 0x62, 0xd2, 0x58, 0xb1, 0x95, 0x0a,
];
#[rustfmt::skip]
static M_DATA_4: [u8; 22] = [
    0x3c, 0xbb, 0x08, 0xf1, 0x33, 0x27, 0x0e, 0x44,
    0x54, 0xbc, 0xaa, 0xa0, 0xf2, 0x0f, 0x6d, 0x63,
    0xc3, 0x8b, 0x65, 0x72, 0xe7, 0x66,
];
#[rustfmt::skip]
static C_DATA_4: [u8; 38] = [
    0x39, 0x66, 0x93, 0x0a, 0x2a, 0xe8, 0xfd, 0xd8,
    0xf4, 0x0e, 0x70, 0x07, 0xf3, 0xfd, 0xe0, 0xbd,
    0x6e, 0xb4, 0x8a, 0x46, 0xe6, 0xd2, 0x6e, 0xef,
    0x83, 0xda, 0x9f, 0x63, 0x84, 0xb1, 0xa2, 0xbd,
    0xa1, 0x07, 0x90, 0xda, 0xdb, 0x3f,
];

// ---------------------------------------------------------------------------
// Vector 5
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_5: [u8; AES256_KEY_SIZE] = [
    0x60, 0x82, 0x3b, 0x64, 0xe0, 0xb2, 0xda, 0x3a,
    0x7e, 0xb7, 0x72, 0xbd, 0x59, 0x41, 0xc5, 0x34,
    0xe6, 0xff, 0x94, 0xea, 0x96, 0xb5, 0x64, 0xe2,
    0xb3, 0x8f, 0x82, 0xc7, 0x8b, 0xb5, 0x45, 0x22,
];
#[rustfmt::skip]
static NONCE_5: [u8; 13] = [
    0x48, 0x52, 0x6f, 0x1b, 0xff, 0xc9, 0x7d, 0xd6,
    0x5e, 0x42, 0x90, 0x69, 0x83,
];
#[rustfmt::skip]
static A_DATA_5: [u8; 32] = [
    0xfa, 0xb6, 0x2b, 0x3e, 0x5d, 0xed, 0xa7, 0xa9,
    0xc1, 0x12, 0x86, 0x63, 0xcc, 0x81, 0xc4, 0x4b,
    0x74, 0xab, 0x1b, 0xfe, 0x70, 0xbc, 0x1c, 0x9d,
    0xec, 0x7c, 0x7f, 0xd0, 0x81, 0x73, 0xb8, 0x0a,
];
#[rustfmt::skip]
static M_DATA_5: [u8; 24] = [
    0xa8, 0xbe, 0x79, 0x46, 0x13, 0x83, 0x5c, 0x43,
    0x66, 0xe7, 0x58, 0x17, 0xd2, 0x28, 0x43, 0x8f,
    0x01, 0x1a, 0x2e, 0xc8, 0xa8, 0x6f, 0x97, 0x97,
];
#[rustfmt::skip]
static C_DATA_5: [u8; 40] = [
    0xcc, 0x3e, 0xfe, 0x04, 0xd8, 0x4a, 0x4e, 0xc5,
    0xcb, 0x6a, 0x6c, 0x28, 0xdc, 0x2c, 0x2d, 0x38,
    0x6a, 0x35, 0x9d, 0x95, 0x50, 0xdb, 0xde, 0xc9,
    0x63, 0xdd, 0xd5, 0x64, 0x64, 0xae, 0xd6, 0xd0,
    0x61, 0x31, 0x59, 0xd1, 0xaa, 0x18, 0x1d, 0xcb,
];

// ---------------------------------------------------------------------------
// Vector 6 (14-Byte Tag Test / Variable Length)
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_6: [u8; AES256_KEY_SIZE] = [
    0x5c, 0x8b, 0x59, 0xd3, 0xe7, 0x98, 0x6c, 0x27,
    0x7d, 0x5a, 0xd5, 0x1e, 0x4a, 0x22, 0x33, 0x25,
    0x10, 0x76, 0x80, 0x9e, 0xbf, 0x59, 0x46, 0x3f,
    0x47, 0xcd, 0x10, 0xb4, 0xaa, 0x95, 0x1f, 0x8c,
];
#[rustfmt::skip]
static NONCE_6: [u8; 13] = [
    0x21, 0xff, 0x89, 0x2b, 0x74, 0x3d, 0x66, 0x11,
    0x89, 0xe2, 0x05, 0xc7, 0xf3,
];
#[rustfmt::skip]
static A_DATA_6: [u8; 32] = [
    0xf1, 0xe0, 0xaf, 0x18, 0x51, 0x80, 0xd2, 0xeb,
    0x63, 0xe5, 0x0e, 0x37, 0xba, 0x69, 0x26, 0x47,
    0xca, 0xc2, 0xc6, 0xa1, 0x49, 0xd7, 0x0c, 0x81,
    0xdb, 0xd3, 0x46, 0x85, 0xed, 0x78, 0xfe, 0xaa,
];
#[rustfmt::skip]
static M_DATA_6: [u8; 24] = [
    0x13, 0x8e, 0xe5, 0x3b, 0x19, 0x14, 0xd3, 0x32,
    0x2c, 0x2d, 0xd0, 0xa4, 0xe0, 0x2f, 0xaa, 0xb2,
    0x23, 0x65, 0x55, 0x13, 0x1d, 0x5e, 0xea, 0x08,
];
#[rustfmt::skip]
static C_DATA_6: [u8; 38] = [
    0x5b, 0x2f, 0x30, 0x26, 0xf3, 0x0f, 0xdd, 0x50,
    0xac, 0xcc, 0x40, 0xdd, 0xd0, 0x93, 0xb7, 0x99,
    0x7f, 0x23, 0xd7, 0xc6, 0xd3, 0xc8, 0xbc, 0x42,
    0x5f, 0x82, 0xc8, 0x28, 0x41, 0x36, 0x43, 0xb8,
    0x79, 0x44, 0x94, 0xcb, 0x52, 0x36,
];
