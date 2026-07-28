// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! Test the implementation of HMAC-SHA224 by performing a hash and
//! checking it against the expected hash value.

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient, CapsuleTestError};
use kernel::ErrorCode;
use kernel::hil::digest;
use kernel::hil::digest::HmacSha224;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;

const HMAC_SHA224_DIGEST_LEN: usize = 28;

pub struct TestHmacSha224<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN>> {
    hmac: &'a H,
    key: TakeCell<'static, [u8]>,  // The key to use for HMAC
    data: TakeCell<'static, [u8]>, // The data to hash
    digest: TakeCell<'static, [u8; HMAC_SHA224_DIGEST_LEN]>, // The supplied hash
    correct: &'static [u8; HMAC_SHA224_DIGEST_LEN], // The supplied hash
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN> + HmacSha224> TestHmacSha224<'a, H> {
    pub fn new(
        hmac: &'a H,
        key: &'static mut [u8],
        data: &'static mut [u8],
        digest: &'static mut [u8; HMAC_SHA224_DIGEST_LEN],
        correct: &'static [u8; HMAC_SHA224_DIGEST_LEN],
    ) -> Self {
        TestHmacSha224 {
            hmac,
            key: TakeCell::new(key),
            data: TakeCell::new(data),
            digest: TakeCell::new(digest),
            correct,
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&'static self) {
        kernel::hil::digest::Digest::set_client(self.hmac, self);

        let key = self.key.take().unwrap();
        let r = self.hmac.set_mode_hmacsha224(key);
        if r.is_err() {
            panic!("HmacSha224Test: failed to set key: {:?}", r);
        }
        let data = self.data.take().unwrap();
        let buffer = SubSliceMut::new(data);
        let r = self.hmac.add_mut_data(buffer);
        if r.is_err() {
            panic!("HmacSha224Test: failed to add data: {:?}", r);
        }
    }
}

impl<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN>> digest::ClientData<HMAC_SHA224_DIGEST_LEN>
    for TestHmacSha224<'a, H>
{
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.data.replace(data.take());
        match result {
            Ok(()) => {}
            Err(e) => {
                kernel::debug!("HmacSha224Test: failed to add data: {:?}", e);
                self.client.map(|client| {
                    client.done(Err(CapsuleTestError::ErrorCode(e)));
                });
                return;
            }
        }

        let r = self.hmac.run(self.digest.take().unwrap());
        match r {
            Ok(()) => {}
            Err((e, d)) => {
                kernel::debug!("HmacSha224Test: failed to run HMAC: {:?}", e);

                self.digest.replace(d);
                self.client.map(|client| {
                    client.done(Err(CapsuleTestError::ErrorCode(e)));
                });
            }
        }
    }
}

impl<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN>> digest::ClientHash<HMAC_SHA224_DIGEST_LEN>
    for TestHmacSha224<'a, H>
{
    fn hash_done(
        &self,
        _result: Result<(), ErrorCode>,
        digest: &'static mut [u8; HMAC_SHA224_DIGEST_LEN],
    ) {
        let mut error = false;
        for i in 0..HMAC_SHA224_DIGEST_LEN {
            if self.correct[i] != digest[i] {
                error = true;
                break;
            }
        }
        if !error {
            kernel::debug!("HMAC-SHA224 matches!");
            self.client.map(|client| {
                client.done(Ok(()));
            });
        } else {
            kernel::debug!("HmacSha224Test: incorrect HMAC output!");
            self.client.map(|client| {
                client.done(Err(CapsuleTestError::IncorrectResult));
            });
        }
    }
}

impl<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN>> digest::ClientVerify<HMAC_SHA224_DIGEST_LEN>
    for TestHmacSha224<'a, H>
{
    fn verification_done(
        &self,
        _result: Result<bool, ErrorCode>,
        _compare: &'static mut [u8; HMAC_SHA224_DIGEST_LEN],
    ) {
    }
}

impl<'a, H: digest::Digest<'a, HMAC_SHA224_DIGEST_LEN>> CapsuleTest for TestHmacSha224<'a, H> {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
