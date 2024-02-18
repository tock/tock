// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the software implementation of HMAC-SHA256 by performing a hash and
//! checking it against the expected hash value.

use crate::hmac_sha256::HmacSha256Software;
use crate::sha256::Sha256Software;
use kernel::hil::digest;
use kernel::hil::digest::{DigestAlgorithm, HmacSha256, HmacSha256Hmac};
use kernel::hil::digest::{DigestData, DigestDataHash, DigestHash};
use kernel::utilities::cells::{MapCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

pub struct TestHmacSha256 {
    hmac: &'static HmacSha256Software<'static, Sha256Software<'static>>,
    key: TakeCell<'static, [u8]>,  // The key to use for HMAC
    data: TakeCell<'static, [u8]>, // The data to hash
    digest: MapCell<&'static mut HmacSha256Hmac>, // The supplied hash
    correct: &'static mut HmacSha256Hmac, // The supplied hash
}

impl TestHmacSha256 {
    pub fn new(
        hmac: &'static HmacSha256Software<'static, Sha256Software<'static>>,
        key: &'static mut [u8],
        data: &'static mut [u8],
        digest: &'static mut HmacSha256Hmac,
        correct: &'static mut HmacSha256Hmac,
    ) -> Self {
        TestHmacSha256 {
            hmac,
            key: TakeCell::new(key),
            data: TakeCell::new(data),
            digest: MapCell::new(digest),
            correct,
        }
    }

    pub fn run(&'static self) {
        self.hmac.set_client(self);
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

impl digest::ClientData<HmacSha256Hmac> for TestHmacSha256 {
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.data.replace(data.take());

        let r = self.hmac.run(self.digest.take().unwrap());
        if r.is_err() {
            panic!("HmacSha256Test: failed to run HMAC");
        }
    }
}

impl digest::ClientHash<HmacSha256Hmac> for TestHmacSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut HmacSha256Hmac) {
        for i in 0..32 {
            if self.correct.as_slice()[i] != digest.as_slice()[i] {
                panic!("HmacSha256Test: incorrect HMAC output!");
            }
        }
        kernel::debug!("HMAC-SHA256 matches!");
    }
}
