// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Oxidos Automotive 2026.

//! Test the implementation of MD5 driver by performing a hash
//! and checking it against the expected hash value. It uses
//! DigestData::add_date and DigestVerify::verify through the
//! Digest trait.

use core::cell::Cell;
use core::cmp;

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use kernel::ErrorCode;
use kernel::debug;
use kernel::hil::digest;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;

pub struct TestMd5<'a, H: digest::Digest<'a, 16>> {
    sha: &'a H,
    data: TakeCell<'static, [u8]>,     // The data to hash
    hash: TakeCell<'static, [u8; 16]>, // The supplied hash
    position: Cell<usize>,             // Keep track of position in data
    correct: Cell<bool>,               // Whether supplied hash is correct
    client: OptionalCell<&'static dyn CapsuleTestClient>,
}

// We add data in chunks of 12 bytes to ensure that the underlying
// buffering mechanism works correctly (it can handle filling blocks
// as well as zeroing out incomplete blocks).
const CHUNK_SIZE: usize = 12;

impl<'a, H: digest::Digest<'a, 16> + digest::Md5> TestMd5<'a, H> {
    pub fn new(
        sha: &'a H,
        data: &'static mut [u8],
        hash: &'static mut [u8; 16],
        correct: bool,
    ) -> Self {
        TestMd5 {
            sha,
            data: TakeCell::new(data),
            hash: TakeCell::new(hash),
            position: Cell::new(0),
            correct: Cell::new(correct),
            client: OptionalCell::empty(),
        }
    }

    pub fn run(&'a self) {
        let r = self.sha.set_mode_md5();
        if r.is_err() {
            panic!("Md5Test: failed to set mode: {:?}", r)
        }
        self.sha.set_client(self);
        let data = self.data.take().unwrap();
        let chunk_size = cmp::min(CHUNK_SIZE, data.len());
        self.position.set(chunk_size);
        let mut buffer = SubSliceMut::new(data);
        buffer.slice(0..chunk_size);
        let r = self.sha.add_mut_data(buffer);
        if r.is_err() {
            panic!("Md5Test: failed to add data: {:?}", r);
        }
    }
}

impl<'a, H: digest::Digest<'a, 16>> digest::ClientData<16> for TestMd5<'a, H> {
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, mut data: SubSliceMut<'static, u8>) {
        if data.len() != 0 {
            let r = self.sha.add_mut_data(data);
            if r.is_err() {
                panic!("Md5Test: failed to add data: {:?}", r);
            }
        } else {
            data.reset();
            if self.position.get() < data.len() {
                let new_position = cmp::min(data.len(), self.position.get() + CHUNK_SIZE);
                data.slice(self.position.get()..new_position);
                debug!(
                    "Md5Test: Setting slice to {}..{}",
                    self.position.get(),
                    new_position
                );
                let r = self.sha.add_mut_data(data);
                if r.is_err() {
                    panic!("Md5Test: failed to add data: {:?}", r);
                }
                self.position.set(new_position);
            } else {
                data.reset();
                self.data.put(Some(data.take()));
                match result {
                    Ok(()) => {
                        let v = self.sha.verify(self.hash.take().unwrap());
                        if v.is_err() {
                            panic!("Md5Test: failed to verify: {:?}", v);
                        }
                    }
                    Err(e) => {
                        panic!("Md5Test: adding data failed: {:?}", e);
                    }
                }
            }
        }
    }
}

impl<'a, H: digest::Digest<'a, 16>> digest::ClientVerify<16> for TestMd5<'a, H> {
    fn verification_done(&self, result: Result<bool, ErrorCode>, compare: &'static mut [u8; 16]) {
        self.hash.put(Some(compare));
        debug!("Md5Test: Verification result: {:?}", result);
        match result {
            Ok(success) => {
                if success != self.correct.get() {
                    panic!(
                        "Md5Test: Verification should have been {}, was {}",
                        self.correct.get(),
                        success
                    );
                } else {
                    self.client.map(|client| {
                        client.done(Ok(()));
                    });
                }
            }
            Err(e) => {
                panic!("Md5Test: Error in verification: {:?}", e);
            }
        }
    }
}

impl<'a, H: digest::Digest<'a, 16>> digest::ClientHash<16> for TestMd5<'a, H> {
    fn hash_done(&self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 16]) {}
}

impl<'a, H: digest::Digest<'a, 16>> CapsuleTest for TestMd5<'a, H> {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
