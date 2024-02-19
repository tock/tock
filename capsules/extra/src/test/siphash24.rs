// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test the software implementation of SipHash24 by performing a hash
//! and checking it against the expected hash value. It uses
//! DigestData::add_date and DigestVerify::verify through the
//! Digest trait.

use crate::sip_hash::SipHasher24;
use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use kernel::hil::hasher::{Client, Hasher};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;

pub struct TestSipHash24 {
    hasher: &'static SipHasher24<'static>,
    data: TakeCell<'static, [u8]>,            // The data to hash
    hash: TakeCell<'static, [u8; 8]>,         // The supplied hash
    correct_hash: TakeCell<'static, [u8; 8]>, // The correct hash
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

impl TestSipHash24 {
    pub fn new(
        hasher: &'static SipHasher24<'static>,
        data: &'static mut [u8],
        hash: &'static mut [u8; 8],
        correct_hash: &'static mut [u8; 8],
    ) -> Self {
        TestSipHash24 {
            hasher,
            data: TakeCell::new(data),
            hash: TakeCell::new(hash),
            correct_hash: TakeCell::new(correct_hash),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&'static self) {
        self.hasher.set_client(self);
        let data = self.data.take().unwrap();
        let buffer = SubSliceMut::new(data);
        let r = self.hasher.add_mut_data(buffer);
        if r.is_err() {
            panic!("SipHash24Test: failed to add data: {:?}", r);
        }
    }
}

impl Client<8> for TestSipHash24 {
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.data.replace(data.take());
        self.hasher.run(self.hash.take().unwrap()).unwrap();
    }

    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {}

    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; 8]) {
        let correct = self.correct_hash.take().unwrap();

        let mut matches = true;
        for i in 0..8 {
            if correct[i] != digest[i] {
                matches = false;
                kernel::debug!("TestSipHash24: incorrect hash output!");
            }
        }
        kernel::debug!("TestSipHash24 matches!");

        self.hash.replace(digest);
        self.hasher.clear_data();

        self.client.map(|client| {
            let res = if matches { Ok(()) } else { Err(()) };
            client.done(res);
        });
    }
}

impl CapsuleTest for TestSipHash24 {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
