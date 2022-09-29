//! Test the software implementation of SHA256 by performing a hash
//! and checking it against the expected hash value. It uses
//! DigestData::add_date and DigestVerify::verify through the
//! Digest trait.

use core::cell::Cell;
use core::cmp;

use crate::sha256::Sha256Software;
use kernel::debug;
use kernel::hil::digest;
use kernel::hil::digest::{Digest, DigestData, DigestVerify};
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::ErrorCode;

pub struct TestSha256 {
    sha: &'static Sha256Software<'static>,
    data: TakeCell<'static, [u8]>,     // The data to hash
    hash: TakeCell<'static, [u8; 32]>, // The supplied hash
    position: Cell<usize>,             // Keep track of position in data
    correct: Cell<bool>,               // Whether supplied hash is correct
}

// We add data in chunks of 12 bytes to ensure that the underlying
// buffering mechanism works correctly (it can handle filling blocks
// as well as zeroing out incomplete blocks).
const CHUNK_SIZE: usize = 12;

impl TestSha256 {
    pub fn new(
        sha: &'static Sha256Software<'static>,
        data: &'static mut [u8],
        hash: &'static mut [u8; 32],
        correct: bool,
    ) -> Self {
        TestSha256 {
            sha: sha,
            data: TakeCell::new(data),
            hash: TakeCell::new(hash),
            position: Cell::new(0),
            correct: Cell::new(correct),
        }
    }

    pub fn run(&'static self) {
        self.sha.set_client(self);
        let data = self.data.take().unwrap();
        let chunk_size = cmp::min(CHUNK_SIZE, data.len());
        self.position.set(chunk_size);
        let mut buffer = LeasableMutableBuffer::new(data);
        buffer.slice(0..chunk_size);
        let r = self.sha.add_mut_data(buffer);
        if r.is_err() {
            panic!("Sha256Test: failed to add data: {:?}", r);
        }
    }
}

impl digest::ClientData<32> for TestSha256 {
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: LeasableBuffer<'static, u8>) {
        unimplemented!()
    }

    fn add_mut_data_done(
        &self,
        result: Result<(), ErrorCode>,
        mut data: LeasableMutableBuffer<'static, u8>,
    ) {
        if data.len() != 0 {
            let r = self.sha.add_mut_data(data);
            if r.is_err() {
                panic!("Sha256Test: failed to add data: {:?}", r);
            }
        } else {
            data.reset();
            if self.position.get() < data.len() {
                let new_position = cmp::min(data.len(), self.position.get() + CHUNK_SIZE);
                data.slice(self.position.get()..new_position);
                debug!(
                    "Sha256Test: Setting slice to {}..{}",
                    self.position.get(),
                    new_position
                );
                let r = self.sha.add_mut_data(data);
                if r.is_err() {
                    panic!("Sha256Test: failed to add data: {:?}", r);
                }
                self.position.set(new_position);
            } else {
                data.reset();
                self.data.put(Some(data.take()));
                match result {
                    Ok(()) => {
                        let v = self.sha.verify(self.hash.take().unwrap());
                        if v.is_err() {
                            panic!("Sha256Test: failed to verify: {:?}", v);
                        }
                    }
                    Err(e) => {
                        panic!("Sha256Test: adding data failed: {:?}", e);
                    }
                }
            }
        }
    }
}

impl digest::ClientVerify<32> for TestSha256 {
    fn verification_done(&self, result: Result<bool, ErrorCode>, compare: &'static mut [u8; 32]) {
        self.hash.put(Some(compare));
        debug!("Sha256Test: Verification result: {:?}", result);
        match result {
            Ok(success) => {
                if success != self.correct.get() {
                    panic!(
                        "Sha256Test: Verification should have been {}, was {}",
                        self.correct.get(),
                        success
                    );
                }
            }
            Err(e) => {
                panic!("Sha256Test: Error in verification: {:?}", e);
            }
        }
    }
}

impl digest::ClientHash<32> for TestSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32]) {}
}
