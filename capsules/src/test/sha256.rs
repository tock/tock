//! Test the software implementation of SHA256

use crate::sha256::Sha256Software;
use kernel::debug;
use kernel::hil::digest;
use kernel::hil::digest::{Digest, DigestData, DigestVerify};
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::ErrorCode;

pub struct TestSha256 {
    sha: &'static Sha256Software<'static>,

    data: TakeCell<'static, [u8]>,
    hash: TakeCell<'static, [u8; 32]>,
}

impl TestSha256 {
    pub fn new(
        sha: &'static Sha256Software<'static>,
        data: &'static mut [u8],
        hash: &'static mut [u8; 32],
    ) -> Self {
        TestSha256 {
            sha: sha,
            data: TakeCell::new(data),
            hash: TakeCell::new(hash),
        }
    }

    pub fn run(&'static self) {
        if self.sha.initialize().is_err() {
            debug!("Sha256Test: failed to initialize Sha256Software");
            return;
        }
        self.sha.set_client(self);
        let r = self
            .sha
            .add_data(LeasableBuffer::new(self.data.take().unwrap()));
        if r.is_err() {
            debug!("Sha256Test: failed to add data: {:?}", r);
        }
    }
}

impl digest::ClientData<'static, 32> for TestSha256 {
    fn add_data_done(&'static self, result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        debug!("Sha256Test: Adding data result: {:?}", result);
        self.data.put(Some(data));
        match result {
            Ok(()) => {
                let v = self.sha.verify(self.hash.take().unwrap());
                if v.is_err() {
                    debug!("Sha256Test: failed to verify: {:?}", v);
                }
            }
            Err(e) => {
                debug!("Sha256Test: adding data failed: {:?}", e);
            }
        }
    }
}

impl digest::ClientVerify<'static, 32> for TestSha256 {
    fn verification_done(
        &'static self,
        result: Result<bool, ErrorCode>,
        compare: &'static mut [u8; 32],
    ) {
        self.hash.put(Some(compare));
        debug!("Sha256Test: Verification result: {:?}", result);
    }
}

impl digest::ClientHash<'static, 32> for TestSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32]) {}
}
