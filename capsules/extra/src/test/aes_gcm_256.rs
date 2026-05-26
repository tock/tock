// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! Test the AES-256-GCM implementation using NIST SP 800-38D test vectors.
//!
//! Each test vector is run twice: once encrypting, once decrypting.
//! The following cases are covered (0-indexed to match the internal array):
//!
//!   Vec 0 — empty plaintext, 16-byte AAD             (GMAC / auth-only)
//!   Vec 1 — 16-byte plaintext, empty AAD             (encrypt + tag, no AAD)
//!   Vec 2 — 16-byte plaintext, 16-byte AAD           (full AEAD)
//!   Vec 3 — plaintext exactly one block (16 B)       (boundary: single block)
//!   Vec 4 — plaintext exactly one block (16 B) + AAD (boundary: single block + large AAD)
//!   Vec 5 — plaintext 13 bytes + large AAD           (partial block payload)
//!   Vec 6 — tampered tag: decryption must explicitly report tag_is_valid = false
//!
//! Buffer layout for each vector:
//!   [aad_offset .. message_offset]               — AAD bytes
//!   [message_offset .. message_offset+msg_len] — plaintext (enc) / ciphertext (dec)
//!   [message_offset+msg_len ..]                — tag (written by enc, checked by dec)

use core::cell::Cell;
use kernel::debug;
use kernel::hil::symmetric_encryption::{GCMClient, AES256, AES256_KEY_SIZE, AESGCM};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

// Maximum buffer size needed across all vectors.
const BUF_LEN: usize = 128;

pub struct TestAES256Gcm<'a, A: AESGCM<'a, AES256>> {
    aes_gcm: &'a A,
    buf: TakeCell<'static, [u8]>,
    current_test: Cell<usize>,
    encrypting: Cell<bool>,
}

// A single test vector.
struct Vector {
    key: &'static [u8],
    iv: &'static [u8],
    aad: &'static [u8],
    pt: &'static [u8],
    ct: &'static [u8],
    tag: &'static [u8],
    // If true, the tag in the buffer is deliberately corrupted before
    // decryption so we can verify that tag_is_valid comes back false.
    expect_tag_invalid: bool,
}

impl<'a, A: AESGCM<'a, AES256>> TestAES256Gcm<'a, A> {
    pub fn new(aes_gcm: &'a A, buf: &'static mut [u8]) -> Self {
        assert!(buf.len() >= BUF_LEN, "buffer too small for GCM-256 tests");
        TestAES256Gcm {
            aes_gcm,
            buf: TakeCell::new(buf),
            current_test: Cell::new(0),
            encrypting: Cell::new(true),
        }
    }

    pub fn run(&self) {
        debug!(
            "AES-256-GCM test suite starting ({} vectors)",
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

        let aad_offset = 0;
        let message_offset = v.aad.len();
        let message_len = v.pt.len();
        let tag_len = v.tag.len();

        let buf = self
            .buf
            .take()
            .expect("aes256gcm_test: buffer missing in trigger");

        // Zero the whole buffer so leftover bytes from previous tests don't
        // cause false positives.
        buf[..BUF_LEN].fill(0);

        buf[aad_offset..message_offset].copy_from_slice(v.aad);

        if encrypting {
            buf[message_offset..message_offset + message_len].copy_from_slice(v.pt);
        } else {
            buf[message_offset..message_offset + message_len].copy_from_slice(v.ct);
            if v.expect_tag_invalid {
                // Corrupt the tag so decryption should report invalid.
                let tag_start = message_offset + message_len;
                buf[tag_start..tag_start + tag_len].copy_from_slice(v.tag);
                buf[tag_start] ^= 0xFF;
            } else {
                buf[message_offset + message_len..message_offset + message_len + tag_len]
                    .copy_from_slice(v.tag);
            }
        }

        match self.aes_gcm.set_key(v.key) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "aes256gcm_test vec={} enc={} returned {:?}: set_key failed",
                    self.current_test.get(),
                    encrypting,
                    e,
                );
            }
        }
        match self.aes_gcm.set_iv(v.iv) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "aes256gcm_test vec={} enc={} returned {:?}: set_iv failed",
                    self.current_test.get(),
                    encrypting,
                    e,
                );
            }
        }

        self.aes_gcm
            .crypt(
                buf,
                aad_offset,
                message_offset,
                message_len,
                tag_len,
                encrypting,
            )
            .unwrap_or_else(|(code, buf)| {
                self.buf.replace(buf);
                panic!(
                    "aes256gcm_test vec={} enc={}: crypt() returned {:?}",
                    self.current_test.get(),
                    encrypting,
                    code
                );
            });
    }

    fn check(&self, tag_is_valid: bool) {
        let v = self.vector();
        let encrypting = self.encrypting.get();
        let test_idx = self.current_test.get();

        let message_offset = v.aad.len();
        let message_len = v.pt.len();
        let tag_len = v.tag.len();
        let tag_start = message_offset + message_len;

        let buf = self
            .buf
            .take()
            .expect("aes256gcm_test: buffer missing in check");

        if encrypting {
            // Verify ciphertext
            let ct_ok = &buf[message_offset..message_offset + message_len] == v.ct;
            // Verify tag
            let tag_ok = &buf[tag_start..tag_start + tag_len] == v.tag;
            // tag_is_valid is always true for encryption
            if !ct_ok || !tag_ok || !tag_is_valid {
                panic!(
                    "aes256gcm_test FAILED vec={} enc=true: \
                     ct_ok={} tag_ok={} tag_is_valid={}",
                    test_idx, ct_ok, tag_ok, tag_is_valid
                );
            }
        } else {
            if v.expect_tag_invalid {
                // We deliberately corrupted the tag; hardware MUST explicitly report invalid.
                // This is the check for Test 7 (Index 6)
                if tag_is_valid {
                    panic!(
                        "aes256gcm_test FAILED vec={} enc=false: \
                          expected tag_is_valid=false for corrupted tag, got true",
                        test_idx
                    );
                }
                debug!(
                    "aes256gcm_test passed vec={} enc=false (corrupted tag explicitly rejected: tag_is_valid={})",
                    test_idx, tag_is_valid
                );
                self.buf.replace(buf);
                return;
            }

            // Verify plaintext recovery
            let pt_ok = &buf[message_offset..message_offset + message_len] == v.pt;
            // Tag bytes should be unchanged
            let tag_ok = &buf[tag_start..tag_start + tag_len] == v.tag;

            if !pt_ok || !tag_ok || !tag_is_valid {
                panic!(
                    "aes256gcm_test FAILED vec={} enc=false: \
                     pt_ok={} tag_ok={} tag_is_valid={}",
                    test_idx, pt_ok, tag_ok, tag_is_valid
                );
            }
        }

        debug!("aes256gcm_test passed vec={} enc={}", test_idx, encrypting);

        self.buf.replace(buf);
    }

    /// Advance to the next (vector, direction) pair.
    /// Returns true if there is more work to do.
    fn advance(&self) -> bool {
        if self.encrypting.get() {
            // Just finished encryption — now do decryption for same vector.
            self.encrypting.set(false);
            true
        } else {
            // Both directions done — move to next vector.
            self.encrypting.set(true);
            let next = self.current_test.get() + 1;
            self.current_test.set(next);
            next < VECTORS.len()
        }
    }
}

impl<'a, A: AESGCM<'a, AES256>> GCMClient for TestAES256Gcm<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.buf.replace(buf);
        if res != Ok(()) {
            panic!(
                "aes256gcm_test vec={} enc={}: crypt_done error {:?}",
                self.current_test.get(),
                self.encrypting.get(),
                res
            );
        }
        self.check(tag_is_valid);
        if self.advance() {
            self.trigger();
        } else {
            debug!("AES-256-GCM all tests passed");
        }
    }
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------
//
// Sources:
//   NIST CAVS GCM test vectors (256-bit key), available from:
//   https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program
//
// Vec 0-2: taken directly from NIST CAVS GCMEncryptExtIV256.rsp (excluding empty zero vector)
// Vec 3-5: constructed from NIST CAVS to hit specific block-boundary cases
// Vec 6:   same as Vec 2 but with a deliberately corrupted tag (Test 7)
static VECTORS: &[Vector] = &[
    // -----------------------------------------------------------------------
    // Vec 0 — empty PT, non-empty AAD (GMAC / auth-only)
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_0,
        iv: &IV_0,
        aad: &AAD_0,
        pt: &[],
        ct: &[],
        tag: &TAG_0,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 1 — non-empty PT, empty AAD
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_1,
        iv: &IV_1,
        aad: &[],
        pt: &PT_1,
        ct: &CT_1,
        tag: &TAG_1,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 2 — non-empty PT, non-empty AAD
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_2,
        iv: &IV_2,
        aad: &AAD_2,
        pt: &PT_2,
        ct: &CT_2,
        tag: &TAG_2,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 3 — PT exactly one AES block (16 B), no AAD
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_3,
        iv: &IV_3,
        aad: &[],
        pt: &PT_3,
        ct: &CT_3,
        tag: &TAG_3,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 4 — PT exactly one AES block (16 B), large AAD (90 B)
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_4,
        iv: &IV_4,
        aad: &AAD_4,
        pt: &PT_4,
        ct: &CT_4,
        tag: &TAG_4,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 5 — PT partial block (13 B), large AAD (90 B)
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_5,
        iv: &IV_5,
        aad: &AAD_5,
        pt: &PT_5,
        ct: &CT_5,
        tag: &TAG_5,
        expect_tag_invalid: false,
    },
    // -----------------------------------------------------------------------
    // Vec 6 (Test 7) — same as Vec 2, but tag is corrupted before decryption.
    //         Encryption still uses the correct tag; only the decrypt
    //         direction explicitly expects tag_is_valid = false.
    // -----------------------------------------------------------------------
    Vector {
        key: &KEY_2,
        iv: &IV_2,
        aad: &AAD_2,
        pt: &PT_2,
        ct: &CT_2,
        tag: &TAG_2,
        expect_tag_invalid: true, // MUST correctly fail validation
    },
];

// ---------------------------------------------------------------------------
// Vector 0 — empty PT, AAD 16 bytes
// NIST CAVS GCMEncryptExtIV256, Count 0, Keylen=256, IVlen=96, PTlen=0,
// AADlen=128, Taglen=128
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_0: [u8; AES256_KEY_SIZE] = [
    0x78, 0xdc, 0x4e, 0x0a, 0xaf, 0x52, 0xd9, 0x35,
    0xc3, 0xc0, 0x1e, 0xea, 0x57, 0x42, 0x8f, 0x00,
    0xca, 0x1f, 0xd4, 0x75, 0xf5, 0xda, 0x86, 0xa4,
    0x9c, 0x8d, 0xd7, 0x3d, 0x68, 0xc8, 0xe2, 0x23,
];
#[rustfmt::skip]
static IV_0: [u8; 12] = [
    0xd7, 0x9c, 0xf2, 0x2d, 0x50, 0x4c, 0xc7, 0x93,
    0xc3, 0xfb, 0x6c, 0x8a,
];
#[rustfmt::skip]
static AAD_0: [u8; 16] = [
    0xb9, 0x6b, 0xaa, 0x8c, 0x1c, 0x75, 0xa6, 0x71,
    0xbf, 0xb2, 0xd0, 0x8d, 0x06, 0xbe, 0x5f, 0x36,
];
#[rustfmt::skip]
static TAG_0: [u8; 16] = [
    0x3e, 0x5d, 0x48, 0x6a, 0xa2, 0xe3, 0x0b, 0x22,
    0xe0, 0x40, 0xb8, 0x57, 0x23, 0xa0, 0x6e, 0x76,
];

// ---------------------------------------------------------------------------
// Vector 1 — PT 16 bytes, empty AAD
// NIST CAVS GCMEncryptExtIV256, Count 0, Keylen=256, IVlen=96, PTlen=128,
// AADlen=0, Taglen=128
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_1: [u8; AES256_KEY_SIZE] = [
    0x31, 0xbd, 0xad, 0xd9, 0x66, 0x98, 0xc2, 0x04,
    0xaa, 0x9c, 0xe1, 0x44, 0x8e, 0xa9, 0x4a, 0xe1,
    0xfb, 0x4a, 0x9a, 0x0b, 0x3c, 0x9d, 0x77, 0x3b,
    0x51, 0xbb, 0x18, 0x22, 0x66, 0x6b, 0x8f, 0x22,
];
#[rustfmt::skip]
static IV_1: [u8; 12] = [
    0x0d, 0x18, 0xe0, 0x6c, 0x7c, 0x72, 0x5a, 0xc9,
    0xe3, 0x62, 0xe1, 0xce,
];
#[rustfmt::skip]
static PT_1: [u8; 16] = [
    0x2d, 0xb5, 0x16, 0x8e, 0x93, 0x25, 0x56, 0xf8,
    0x08, 0x9a, 0x06, 0x22, 0x98, 0x1d, 0x01, 0x7d,
];
#[rustfmt::skip]
static CT_1: [u8; 16] = [
    0xfa, 0x43, 0x62, 0x18, 0x96, 0x61, 0xd1, 0x63,
    0xfc, 0xd6, 0xa5, 0x6d, 0x8b, 0xf0, 0x40, 0x5a,
];
#[rustfmt::skip]
static TAG_1: [u8; 16] = [
    0xd6, 0x36, 0xac, 0x1b, 0xbe, 0xdd, 0x5c, 0xc3,
    0xee, 0x72, 0x7d, 0xc2, 0xab, 0x4a, 0x94, 0x89,
];
// ---------------------------------------------------------------------------
// Vector 2 — PT 16 bytes, AAD 16 bytes  (full AEAD)
// NIST CAVS GCMEncryptExtIV256, Count 0, Keylen=256, IVlen=96, PTlen=128,
// AADlen=128, Taglen=128
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_2: [u8; AES256_KEY_SIZE] = [
    0x92, 0xe1, 0x1d, 0xcd, 0xaa, 0x86, 0x6f, 0x5c,
    0xe7, 0x90, 0xfd, 0x24, 0x50, 0x1f, 0x92, 0x50,
    0x9a, 0xac, 0xf4, 0xcb, 0x8b, 0x13, 0x39, 0xd5,
    0x0c, 0x9c, 0x12, 0x40, 0x93, 0x5d, 0xd0, 0x8b,
];
#[rustfmt::skip]
static IV_2: [u8; 12] = [
    0xac, 0x93, 0xa1, 0xa6, 0x14, 0x52, 0x99, 0xbd,
    0xe9, 0x02, 0xf2, 0x1a,
];
#[rustfmt::skip]
static AAD_2: [u8; 16] = [
    0x1e, 0x08, 0x89, 0x01, 0x6f, 0x67, 0x60, 0x1c,
    0x8e, 0xbe, 0xa4, 0x94, 0x3b, 0xc2, 0x3a, 0xd6,
];
#[rustfmt::skip]
static PT_2: [u8; 16] = [
    0x2d, 0x71, 0xbc, 0xfa, 0x91, 0x4e, 0x4a, 0xc0,
    0x45, 0xb2, 0xaa, 0x60, 0x95, 0x5f, 0xad, 0x24,
];
#[rustfmt::skip]
static CT_2: [u8; 16] = [
    0x89, 0x95, 0xae, 0x2e, 0x6d, 0xf3, 0xdb, 0xf9,
    0x6f, 0xac, 0x7b, 0x71, 0x37, 0xba, 0xe6, 0x7f,
];
#[rustfmt::skip]
static TAG_2: [u8; 16] = [
    0xec, 0xa5, 0xaa, 0x77, 0xd5, 0x1d, 0x4a, 0x0a,
    0x14, 0xd9, 0xc5, 0x1e, 0x1d, 0xa4, 0x74, 0xab,
];

// ---------------------------------------------------------------------------
// Vector 3 — PT exactly one block (16 B), no AAD
// Reuses Key/IV/PT/CT/Tag from Vec 1 (which is a single-block case).
// Named separately for clarity.
// ---------------------------------------------------------------------------
static KEY_3: [u8; AES256_KEY_SIZE] = KEY_1;
static IV_3: [u8; 12] = IV_1;
static PT_3: [u8; 16] = PT_1;
static CT_3: [u8; 16] = CT_1;
static TAG_3: [u8; 16] = TAG_1;

// ---------------------------------------------------------------------------
// Vector 4 — PT 16 bytes, AAD 90 bytes, Tag 12 bytes
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_4: [u8; 32] = [
    0xc2, 0x93, 0x26, 0x01, 0x79, 0x87, 0x5a, 0x2c,
    0xc5, 0xd6, 0xa6, 0x60, 0xba, 0x41, 0x8f, 0xa0,
    0xc1, 0xd1, 0xf9, 0xd0, 0xb1, 0xfc, 0x1d, 0xdf,
    0x65, 0x01, 0x40, 0xd0, 0x18, 0xaa, 0xe3, 0x0b,
];

#[rustfmt::skip]
static IV_4: [u8; 12] = [
    0xbb, 0xc0, 0xde, 0x9d, 0x51, 0xb6, 0x46, 0xb2,
    0xd7, 0x79, 0xd1, 0xa1,
];

#[rustfmt::skip]
static AAD_4: [u8; 90] = [
    0x21, 0x8b, 0x66, 0xe8, 0x88, 0x39, 0xbf, 0xec, 0xc9, 0xc4, 0x1a, 0x73, 0x7e, 0xbd, 0x1a, 0x58,
    0xba, 0x41, 0x86, 0x85, 0x38, 0x47, 0x38, 0x95, 0x9a, 0x82, 0xe2, 0x4d, 0x81, 0xb7, 0x66, 0xb9,
    0x18, 0x81, 0x95, 0x59, 0x9b, 0xd2, 0xe7, 0xad, 0x29, 0xfd, 0x53, 0x37, 0x96, 0x9b, 0x00, 0x50,
    0x04, 0xf2, 0x21, 0xf5, 0x7e, 0x02, 0x24, 0xa5, 0xe2, 0xd8, 0x84, 0x42, 0x68, 0xe6, 0xe2, 0x50,
    0x65, 0x99, 0xc0, 0x5e, 0x72, 0xdf, 0x54, 0x3d, 0x11, 0x41, 0x2f, 0xe8, 0x2a, 0xcd, 0x66, 0xa7,
    0xca, 0xaa, 0xa1, 0x66, 0x08, 0x92, 0x6f, 0x77, 0xe3, 0x54,
];

#[rustfmt::skip]
static PT_4: [u8; 16] = [
    0xdf, 0xd5, 0x0a, 0xbb, 0xdf, 0xc4, 0x11, 0x44,
    0xf3, 0x60, 0x06, 0x53, 0xe2, 0xf9, 0x67, 0x0d,
];

#[rustfmt::skip]
static CT_4: [u8; 16] = [
    0xad, 0xbd, 0xd3, 0x9a, 0xe6, 0x3c, 0x55, 0x31,
    0x48, 0x82, 0x8b, 0x0f, 0xfc, 0xa6, 0x29, 0x17,
];

#[rustfmt::skip]
static TAG_4: [u8; 12] = [
    0xef, 0x51, 0x6b, 0xc6, 0xf1, 0xd2, 0xfb, 0x10,
    0x20, 0xf9, 0x55, 0x31,
];

// ---------------------------------------------------------------------------
// Vector 5 — PT 13 bytes, AAD 90 bytes, Tag 15 bytes
// NIST CAVS GCMEncryptExtIV256, Count 0, Keylen=256, IVlen=96, PTlen=104,
// AADlen=720, Taglen=120
// ---------------------------------------------------------------------------
#[rustfmt::skip]
static KEY_5: [u8; AES256_KEY_SIZE] = [
    0x6e, 0x50, 0xfc, 0xc4, 0xb6, 0x9e, 0x96, 0x23,
    0xf6, 0xd5, 0x58, 0x49, 0xc1, 0x44, 0x34, 0xbe,
    0x8a, 0x1d, 0x38, 0xf9, 0x10, 0xf3, 0x83, 0x15,
    0x30, 0x0a, 0x3c, 0xa3, 0xcb, 0x71, 0xc7, 0xd5,
];

#[rustfmt::skip]
static IV_5: [u8; 12] = [
    0xb6, 0xe8, 0x58, 0x01, 0xab, 0xd0, 0x72, 0xdb,
    0x88, 0x52, 0x51, 0x4c,
];

#[rustfmt::skip]
static AAD_5: [u8; 90] = [
    0xa1, 0xfa, 0x6b, 0xf9, 0xf7, 0x52, 0x7c, 0xc4,
    0x05, 0x31, 0x0e, 0x0c, 0xf2, 0xc6, 0x3b, 0x84,
    0xdd, 0x4f, 0xef, 0x93, 0xb2, 0x02, 0x14, 0xd0,
    0x03, 0x90, 0x26, 0x0a, 0xa4, 0x4b, 0xc7, 0xf3,
    0x95, 0x36, 0x77, 0x7e, 0x8a, 0xc6, 0x9e, 0x33,
    0xb8, 0xb7, 0xb6, 0x9b, 0x4f, 0xd8, 0x1a, 0xf2,
    0xd8, 0x17, 0xbf, 0xcc, 0x8f, 0x6f, 0x8a, 0xab,
    0xcf, 0x74, 0x8f, 0xc7, 0xe9, 0xfe, 0xb6, 0x75,
    0x7d, 0x21, 0x89, 0x9c, 0x78, 0xd8, 0xa1, 0x34,
    0xa5, 0x5b, 0x90, 0xea, 0xa9, 0xe8, 0x95, 0xb3,
    0x1a, 0x9f, 0xb4, 0xd3, 0x7d, 0xaa, 0x84, 0xbc,
    0x86, 0x42,
];

#[rustfmt::skip]
static PT_5: [u8; 13] = [
    0xe9, 0x99, 0x04, 0xb9, 0x21, 0x16, 0x8e, 0x0b,
    0xa6, 0xa5, 0xcc, 0xef, 0x33,
];

#[rustfmt::skip]
static CT_5: [u8; 13] = [
    0x5b, 0x0e, 0xa5, 0xd1, 0x16, 0x71, 0x31, 0x92,
    0x9f, 0x74, 0x29, 0x9a, 0x5f,
];

#[rustfmt::skip]
static TAG_5: [u8; 15] = [
    0x22, 0x23, 0x55, 0x11, 0x74, 0x3d, 0x0b, 0x83,
    0xae, 0x5a, 0xb7, 0x6d, 0x9f, 0xa3, 0x15,
];
