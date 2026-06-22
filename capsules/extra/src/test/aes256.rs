// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the AES hardware for ECB, CBC, and CTR modes using NIST SP 800-38A vectors.
//!
//! Each test struct runs the following steps in sequence:
//!   1. StandardEnc        — out-of-place encryption, full message
//!   2. StandardEncInPlace — in-place encryption, full message
//!   3. StandardDec        — out-of-place decryption, full message  (if test_decrypt)
//!   4. StandardDecInPlace — in-place decryption, full message      (if test_decrypt)
//!   5. ChunkEnc1          — in-place encryption, first half
//!   6. ChunkEnc2          — in-place encryption, second half (no re-init, tests IV chaining)
//!   7. ChunkDec1          — in-place decryption, first half         (if test_decrypt)
//!   8. ChunkDec2          — in-place decryption, second half        (if test_decrypt)

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use core::cell::Cell;
use kernel::debug;
use kernel::hil;
use kernel::hil::symmetric_encryption::{
    AESCtr, AES, AES256, AES256_KEY_SIZE, AESCBC, AESECB, AES_BLOCK_SIZE,
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;

// The data buffer layout is:
//   [0..DATA_OFFSET]              — guard region (never written, detects underflow)
//   [DATA_OFFSET..DATA_OFFSET+DATA_LEN] — ciphertext / plaintext
const DATA_OFFSET: usize = AES_BLOCK_SIZE;
const DATA_LEN: usize = 4 * AES_BLOCK_SIZE;
const CHUNK_LEN: usize = 2 * AES_BLOCK_SIZE; // must divide DATA_LEN evenly

#[derive(Copy, Clone, Debug, PartialEq)]
enum TestStep {
    StandardEnc,
    StandardEncInPlace,
    StandardDec,
    StandardDecInPlace,
    ChunkEnc1,
    ChunkEnc2,
    ChunkDec1,
    ChunkDec2,
    Done,
}

// ---------------------------------------------------------------------------
// ECB
// ---------------------------------------------------------------------------

pub struct TestAES256Ecb<'a, A: 'a> {
    aes: &'a A,
    key: TakeCell<'a, [u8]>,
    source: TakeCell<'static, [u8]>,
    data: TakeCell<'static, [u8]>,
    test_decrypt: bool,
    step: Cell<TestStep>,
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl<'a, A: AES<'a, AES256> + AESECB> TestAES256Ecb<'a, A> {
    pub fn new(
        aes: &'a A,
        key: &'a mut [u8],
        source: &'static mut [u8],
        data: &'static mut [u8],
        test_decrypt: bool,
    ) -> Self {
        TestAES256Ecb {
            aes,
            key: TakeCell::new(key),
            source: TakeCell::new(source),
            data: TakeCell::new(data),
            test_decrypt,
            step: Cell::new(TestStep::StandardEnc),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&self) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        // Re-initialise hardware for every step except the second chunk, which
        // intentionally reuses the hardware state to verify key/IV retention.
        if !is_second_chunk(step) {
            self.aes.enable();
            self.aes.set_mode_aesecb(encrypting).unwrap();
            self.key.map(|key| {
                key[..KEY.len()].copy_from_slice(&KEY);
                assert_eq!(self.aes.set_key(key), Ok(()));
            });
            let src = if encrypting { &PTXT } else { &CTXT_ECB };
            self.source.map(|s| s[..src.len()].copy_from_slice(src));
            self.aes.start_message();
        }

        prepare_in_place(step, in_place, &self.source, &self.data);

        let (start, stop) = chunk_range(step);
        run_crypt(self.aes, in_place, &self.source, &self.data, start, stop);
    }
}

impl<'a, A: AES<'a, AES256> + AESECB> CapsuleTest for TestAES256Ecb<'a, A> {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}

impl<'a, A: AES<'a, AES256> + AESECB> hil::symmetric_encryption::Client<'a>
    for TestAES256Ecb<'a, A>
{
    fn crypt_done(&'a self, source: Option<&'static mut [u8]>, dest: &'static mut [u8]) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        restore_source(in_place, source, &self.source);
        self.data.replace(dest);

        // ECB has no IV chaining, so we can verify after every step except
        // ChunkEnc1/ChunkDec1 where we only have half the ciphertext yet.
        if !is_first_chunk(step) {
            let expected = if encrypting { &CTXT_ECB } else { &PTXT };
            self.data.map(|d| {
                assert_eq!(
                    &d[DATA_OFFSET..DATA_OFFSET + DATA_LEN],
                    expected.as_ref(),
                    "aes_test ECB failed at step {:?}",
                    step
                );
            });
            debug!("aes_test ECB passed step: {:?}", step);
            self.aes.disable();
        }

        let next = next_step(step, self.test_decrypt);
        self.step.set(next);
        if next == TestStep::Done {
            self.client.map(|c| c.done(Ok(())));
        } else {
            self.run();
        }
    }
}

// ---------------------------------------------------------------------------
// CBC
// ---------------------------------------------------------------------------

pub struct TestAES256Cbc<'a, A: 'a> {
    aes: &'a A,
    key: TakeCell<'a, [u8]>,
    iv: TakeCell<'a, [u8]>,
    source: TakeCell<'static, [u8]>,
    data: TakeCell<'static, [u8]>,
    test_decrypt: bool,
    step: Cell<TestStep>,
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl<'a, A: AES<'a, AES256> + AESCBC> TestAES256Cbc<'a, A> {
    pub fn new(
        aes: &'a A,
        key: &'a mut [u8],
        iv: &'a mut [u8],
        source: &'static mut [u8],
        data: &'static mut [u8],
        test_decrypt: bool,
    ) -> Self {
        TestAES256Cbc {
            aes,
            key: TakeCell::new(key),
            iv: TakeCell::new(iv),
            source: TakeCell::new(source),
            data: TakeCell::new(data),
            test_decrypt,
            step: Cell::new(TestStep::StandardEnc),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&self) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        if !is_second_chunk(step) {
            self.aes.enable();
            self.aes.set_mode_aescbc(encrypting).unwrap();
            self.key.map(|key| {
                key[..KEY.len()].copy_from_slice(&KEY);
                assert_eq!(self.aes.set_key(key), Ok(()));
            });
            self.iv.map(|iv| {
                iv[..IV_CBC.len()].copy_from_slice(&IV_CBC);
                assert_eq!(self.aes.set_iv(iv), Ok(()));
            });
            let src = if encrypting { &PTXT } else { &CTXT_CBC };
            self.source.map(|s| s[..src.len()].copy_from_slice(src));
            self.aes.start_message();
        }

        prepare_in_place(step, in_place, &self.source, &self.data);

        let (start, stop) = chunk_range(step);
        run_crypt(self.aes, in_place, &self.source, &self.data, start, stop);
    }
}

impl<'a, A: AES<'a, AES256> + AESCBC> CapsuleTest for TestAES256Cbc<'a, A> {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}

impl<'a, A: AES<'a, AES256> + AESCBC> hil::symmetric_encryption::Client<'a>
    for TestAES256Cbc<'a, A>
{
    fn crypt_done(&'a self, source: Option<&'static mut [u8]>, dest: &'static mut [u8]) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        restore_source(in_place, source, &self.source);
        self.data.replace(dest);

        if !is_first_chunk(step) {
            let expected = if encrypting { &CTXT_CBC } else { &PTXT };
            self.data.map(|d| {
                assert_eq!(
                    &d[DATA_OFFSET..DATA_OFFSET + DATA_LEN],
                    expected.as_ref(),
                    "aes_test CBC failed at step {:?}",
                    step
                );
            });
            debug!("aes_test CBC passed step: {:?}", step);
            self.aes.disable();
        }

        let next = next_step(step, self.test_decrypt);
        self.step.set(next);
        if next == TestStep::Done {
            self.client.map(|c| c.done(Ok(())));
        } else {
            self.run();
        }
    }
}

// ---------------------------------------------------------------------------
// CTR
// ---------------------------------------------------------------------------

pub struct TestAES256Ctr<'a, A: 'a> {
    aes: &'a A,
    key: TakeCell<'a, [u8]>,
    iv: TakeCell<'a, [u8]>,
    source: TakeCell<'static, [u8]>,
    data: TakeCell<'static, [u8]>,
    test_decrypt: bool,
    step: Cell<TestStep>,
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl<'a, A: AES<'a, AES256> + AESCtr> TestAES256Ctr<'a, A> {
    pub fn new(
        aes: &'a A,
        key: &'a mut [u8],
        iv: &'a mut [u8],
        source: &'static mut [u8],
        data: &'static mut [u8],
        test_decrypt: bool,
    ) -> Self {
        TestAES256Ctr {
            aes,
            key: TakeCell::new(key),
            iv: TakeCell::new(iv),
            source: TakeCell::new(source),
            data: TakeCell::new(data),
            test_decrypt,
            step: Cell::new(TestStep::StandardEnc),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&self) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        if !is_second_chunk(step) {
            self.aes.enable();
            self.aes.set_mode_aesctr(encrypting).unwrap();
            self.key.map(|key| {
                key[..KEY.len()].copy_from_slice(&KEY);
                assert_eq!(self.aes.set_key(key), Ok(()));
            });
            self.iv.map(|iv| {
                iv[..IV_CTR.len()].copy_from_slice(&IV_CTR);
                assert_eq!(self.aes.set_iv(iv), Ok(()));
            });
            let src = if encrypting { &PTXT } else { &CTXT_CTR };
            self.source.map(|s| s[..src.len()].copy_from_slice(src));
            self.aes.start_message();
        }

        prepare_in_place(step, in_place, &self.source, &self.data);

        let (start, stop) = chunk_range(step);
        run_crypt(self.aes, in_place, &self.source, &self.data, start, stop);
    }
}

impl<'a, A: AES<'a, AES256> + AESCtr> CapsuleTest for TestAES256Ctr<'a, A> {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}

impl<'a, A: AES<'a, AES256> + AESCtr> hil::symmetric_encryption::Client<'a>
    for TestAES256Ctr<'a, A>
{
    fn crypt_done(&'a self, source: Option<&'static mut [u8]>, dest: &'static mut [u8]) {
        let step = self.step.get();
        let encrypting = is_encrypting(step);
        let in_place = is_in_place(step);

        restore_source(in_place, source, &self.source);
        self.data.replace(dest);

        if !is_first_chunk(step) {
            let expected = if encrypting { &CTXT_CTR } else { &PTXT };
            self.data.map(|d| {
                assert_eq!(
                    &d[DATA_OFFSET..DATA_OFFSET + DATA_LEN],
                    expected.as_ref(),
                    "aes_test CTR failed at step {:?}",
                    step
                );
                // Verify guard region was not touched
                assert_eq!(
                    d[..DATA_OFFSET],
                    [0u8; DATA_OFFSET],
                    "aes_test CTR: guard region corrupted at step {:?}",
                    step
                );
            });
            debug!("aes_test CTR passed step: {:?}", step);
            self.aes.disable();
        }

        let next = next_step(step, self.test_decrypt);
        self.step.set(next);
        if next == TestStep::Done {
            self.client.map(|c| c.done(Ok(())));
        } else {
            self.run();
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn is_encrypting(step: TestStep) -> bool {
    !matches!(
        step,
        TestStep::StandardDec
            | TestStep::StandardDecInPlace
            | TestStep::ChunkDec1
            | TestStep::ChunkDec2
    )
}

fn is_in_place(step: TestStep) -> bool {
    matches!(
        step,
        TestStep::StandardEncInPlace
            | TestStep::StandardDecInPlace
            | TestStep::ChunkEnc1
            | TestStep::ChunkEnc2
            | TestStep::ChunkDec1
            | TestStep::ChunkDec2
    )
}

fn is_first_chunk(step: TestStep) -> bool {
    matches!(step, TestStep::ChunkEnc1 | TestStep::ChunkDec1)
}

fn is_second_chunk(step: TestStep) -> bool {
    matches!(step, TestStep::ChunkEnc2 | TestStep::ChunkDec2)
}

/// Returns (start, stop) indices into the data buffer for this step.
fn chunk_range(step: TestStep) -> (usize, usize) {
    match step {
        TestStep::ChunkEnc1 | TestStep::ChunkDec1 => (DATA_OFFSET, DATA_OFFSET + CHUNK_LEN),
        TestStep::ChunkEnc2 | TestStep::ChunkDec2 => {
            (DATA_OFFSET + CHUNK_LEN, DATA_OFFSET + DATA_LEN)
        }
        _ => (DATA_OFFSET, DATA_OFFSET + DATA_LEN),
    }
}

/// For in-place steps, copy the relevant slice of source into dest at the
/// correct offset so the driver reads plaintext/ciphertext from dest[start..].
fn prepare_in_place(
    step: TestStep,
    in_place: bool,
    source: &TakeCell<'static, [u8]>,
    data: &TakeCell<'static, [u8]>,
) {
    if !in_place {
        return;
    }
    let src_start = match step {
        TestStep::ChunkEnc2 | TestStep::ChunkDec2 => CHUNK_LEN,
        _ => 0,
    };
    let dst_start = match step {
        TestStep::ChunkEnc2 | TestStep::ChunkDec2 => DATA_OFFSET + CHUNK_LEN,
        _ => DATA_OFFSET,
    };
    let copy_len = match step {
        TestStep::ChunkEnc1 | TestStep::ChunkDec1 => DATA_LEN,
        TestStep::ChunkEnc2 | TestStep::ChunkDec2 => 0,
        _ => DATA_LEN,
    };
    source.map(|src| {
        data.map(|dst| {
            dst[dst_start..dst_start + copy_len]
                .copy_from_slice(&src[src_start..src_start + copy_len]);
        });
    });
}

fn run_crypt<'a, A: AES<'a, AES256>>(
    aes: &'a A,
    in_place: bool,
    source: &TakeCell<'static, [u8]>,
    data: &TakeCell<'static, [u8]>,
    start: usize,
    stop: usize,
) {
    let src = if in_place { None } else { source.take() };
    match aes.crypt(src, data.take().unwrap(), start, stop) {
        None => {}
        Some((result, src_back, dest_back)) => {
            source.put(src_back);
            data.put(Some(dest_back));
            panic!("crypt() returned error: {:?}", result);
        }
    }
}

fn restore_source(
    in_place: bool,
    source: Option<&'static mut [u8]>,
    cell: &TakeCell<'static, [u8]>,
) {
    if !in_place {
        cell.replace(source.expect("crypt_done: expected source buffer for out-of-place op"));
    }
}

fn next_step(step: TestStep, test_decrypt: bool) -> TestStep {
    match step {
        TestStep::StandardEnc => TestStep::StandardEncInPlace,
        TestStep::StandardEncInPlace => {
            if test_decrypt {
                TestStep::StandardDec
            } else {
                TestStep::ChunkEnc1
            }
        }
        TestStep::StandardDec => TestStep::StandardDecInPlace,
        TestStep::StandardDecInPlace => TestStep::ChunkEnc1,
        TestStep::ChunkEnc1 => TestStep::ChunkEnc2,
        TestStep::ChunkEnc2 => {
            if test_decrypt {
                TestStep::ChunkDec1
            } else {
                TestStep::Done
            }
        }
        TestStep::ChunkDec1 => TestStep::ChunkDec2,
        TestStep::ChunkDec2 => {
            debug!("All tests passed");
            TestStep::Done
        }
        _ => TestStep::Done,
    }
}

// ---------------------------------------------------------------------------
// NIST test vectors (AES-256)
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const KEY: [u8; AES256_KEY_SIZE] = [
    0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe,
    0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81,
    0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7,
    0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
];

#[rustfmt::skip]
const IV_CTR: [u8; AES_BLOCK_SIZE] = [
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

#[rustfmt::skip]
const IV_CBC: [u8; AES_BLOCK_SIZE] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];

#[rustfmt::skip]
const PTXT: [u8; DATA_LEN] = [
    0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96,
    0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c,
    0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11,
    0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
    0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
    0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
];

#[rustfmt::skip]
const CTXT_CTR: [u8; DATA_LEN] = [
    0x60, 0x1e, 0xc3, 0x13, 0x77, 0x57, 0x89, 0xa5,
    0xb7, 0xa7, 0xf5, 0x04, 0xbb, 0xf3, 0xd2, 0x28,
    0xf4, 0x43, 0xe3, 0xca, 0x4d, 0x62, 0xb5, 0x9a,
    0xca, 0x84, 0xe9, 0x90, 0xca, 0xca, 0xf5, 0xc5,
    0x2b, 0x09, 0x30, 0xda, 0xa2, 0x3d, 0xe9, 0x4c,
    0xe8, 0x70, 0x17, 0xba, 0x2d, 0x84, 0x98, 0x8d,
    0xdf, 0xc9, 0xc5, 0x8d, 0xb6, 0x7a, 0xad, 0xa6,
    0x13, 0xc2, 0xdd, 0x08, 0x45, 0x79, 0x41, 0xa6,
];

#[rustfmt::skip]
const CTXT_CBC: [u8; DATA_LEN] = [
    0xf5, 0x8c, 0x4c, 0x04, 0xd6, 0xe5, 0xf1, 0xba,
    0x77, 0x9e, 0xab, 0xfb, 0x5f, 0x7b, 0xfb, 0xd6,
    0x9c, 0xfc, 0x4e, 0x96, 0x7e, 0xdb, 0x80, 0x8d,
    0x67, 0x9f, 0x77, 0x7b, 0xc6, 0x70, 0x2c, 0x7d,
    0x39, 0xf2, 0x33, 0x69, 0xa9, 0xd9, 0xba, 0xcf,
    0xa5, 0x30, 0xe2, 0x63, 0x04, 0x23, 0x14, 0x61,
    0xb2, 0xeb, 0x05, 0xe2, 0xc3, 0x9b, 0xe9, 0xfc,
    0xda, 0x6c, 0x19, 0x07, 0x8c, 0x6a, 0x9d, 0x1b,
];

#[rustfmt::skip]
const CTXT_ECB: [u8; DATA_LEN] = [
    0xf3, 0xee, 0xd1, 0xbd, 0xb5, 0xd2, 0xa0, 0x3c,
    0x06, 0x4b, 0x5a, 0x7e, 0x3d, 0xb1, 0x81, 0xf8,
    0x59, 0x1c, 0xcb, 0x10, 0xd4, 0x10, 0xed, 0x26,
    0xdc, 0x5b, 0xa7, 0x4a, 0x31, 0x36, 0x28, 0x70,
    0xb6, 0xed, 0x21, 0xb9, 0x9c, 0xa6, 0xf4, 0xf9,
    0xf1, 0x53, 0xe7, 0xb1, 0xbe, 0xaf, 0xed, 0x1d,
    0x23, 0x30, 0x4b, 0x7a, 0x39, 0xf9, 0xf3, 0xff,
    0x06, 0x7d, 0x8d, 0x8f, 0x9e, 0x24, 0xec, 0xc7,
];
