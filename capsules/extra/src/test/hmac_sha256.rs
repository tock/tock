// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the software implementation of HMAC-SHA256 by performing a hash and
//! checking it against the expected hash value.

use crate::hmac_sha256::HmacSha256Software;
use crate::sha256::Sha256Software;
use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient, CapsuleTestError};
use kernel::hil::digest;
use kernel::hil::digest::HmacSha256;
use kernel::hil::digest::{DigestData, DigestHash};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

pub struct TestHmacSha256 {
    hmac: &'static HmacSha256Software<'static, Sha256Software<'static>>,
    key: TakeCell<'static, [u8]>,        // The key to use for HMAC
    data: TakeCell<'static, [u8]>,       // The data to hash
    digest: TakeCell<'static, [u8; 32]>, // The supplied hash
    correct: &'static [u8; 32],          // The supplied hash
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl TestHmacSha256 {
    pub fn new(
        hmac: &'static HmacSha256Software<'static, Sha256Software<'static>>,
        key: &'static mut [u8],
        data: &'static mut [u8],
        digest: &'static mut [u8; 32],
        correct: &'static [u8; 32],
    ) -> Self {
        TestHmacSha256 {
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
        let r = self.hmac.set_mode_hmacsha256(key);
        if r.is_err() {
            panic!("HmacSha256Test: failed to set key: {:?}", r);
        }
        let data = self.data.take().unwrap();
        let buffer = SubSliceMut::new(data);
        let r = self.hmac.add_mut_data(buffer);
        if r.is_err() {
            panic!("HmacSha256Test: failed to add data: {:?}", r);
        }
    }
}

impl digest::ClientData<32> for TestHmacSha256 {
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.data.replace(data.take());

        match result {
            Ok(()) => {}
            Err(e) => {
                kernel::debug!("HmacSha256Test: failed to add data: {:?}", e);
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
                kernel::debug!("HmacSha256Test: failed to run HMAC: {:?}", e);

                self.digest.replace(d);
                self.client.map(|client| {
                    client.done(Err(CapsuleTestError::ErrorCode(e)));
                });
            }
        }
    }
}

impl digest::ClientHash<32> for TestHmacSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; 32]) {
        let mut error = false;
        for i in 0..32 {
            if self.correct[i] != digest[i] {
                error = true;
            }
        }
        if !error {
            kernel::debug!("HMAC-SHA256 matches!");
            self.client.map(|client| {
                client.done(Ok(()));
            });
        } else {
            kernel::debug!("HmacSha256Test: incorrect HMAC output!");
            self.client.map(|client| {
                client.done(Err(CapsuleTestError::IncorrectResult));
            });
        }
    }
}

impl digest::ClientVerify<32> for TestHmacSha256 {
    fn verification_done(&self, _result: Result<bool, ErrorCode>, _compare: &'static mut [u8; 32]) {
    }
}

impl CapsuleTest for TestHmacSha256 {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
