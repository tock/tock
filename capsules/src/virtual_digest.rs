//! Virtualize the Digest interface to enable multiple users of an underlying
//! Digest hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::{ListLink, ListNode};
use kernel::hil::digest;
use kernel::ErrorCode;

#[derive(Clone, Copy)]
enum Mode {
    Hmac,
    Sha,
}

pub struct VirtualMuxDigest<'a, A: digest::Digest<'a, L>, const L: usize> {
    mux: &'a MuxDigest<'a, A, L>,
    next: ListLink<'a, VirtualMuxDigest<'a, A, L>>,
    sha_client: OptionalCell<&'a dyn digest::Client<'a, L>>,
    hmac_client: OptionalCell<&'a dyn digest::Client<'a, L>>,
    mode: Cell<Mode>,
    id: u32,
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> ListNode<'a, VirtualMuxDigest<'a, A, L>>
    for VirtualMuxDigest<'a, A, L>
{
    fn next(&self) -> &'a ListLink<VirtualMuxDigest<'a, A, L>> {
        &self.next
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> VirtualMuxDigest<'a, A, L> {
    pub fn new(mux_digest: &'a MuxDigest<'a, A, L>) -> VirtualMuxDigest<'a, A, L> {
        let id = mux_digest.next_id.get();
        mux_digest.next_id.set(id + 1);

        VirtualMuxDigest {
            mux: mux_digest,
            next: ListLink::empty(),
            sha_client: OptionalCell::empty(),
            hmac_client: OptionalCell::empty(),
            mode: Cell::new(Mode::Hmac),
            id: id,
        }
    }

    pub fn set_hmac_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.hmac_client.set(client);
    }

    pub fn set_sha_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.sha_client.set(client);
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Digest<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, _client: &'a dyn digest::Client<'a, L>) {
        unimplemented!()
    }

    /// Add data to the digest IP.
    /// All data passed in is fed to the Digest hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.digest.add_data(data)
        } else if self.mux.running_id.get() == self.id {
            self.mux.digest.add_data(data)
        } else {
            Err((ErrorCode::BUSY, data.take()))
        }
    }

    /// Request the hardware block to generate a Digest
    /// This doesn't return anything, instead the client needs to have
    /// set a `hash_done` handler.
    fn run(
        &'a self,
        digest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.digest.run(digest)
        } else if self.mux.running_id.get() == self.id {
            self.mux.digest.run(digest)
        } else {
            Err((ErrorCode::BUSY, digest))
        }
    }

    /// Disable the Digest hardware and clear the keys and any other sensitive
    /// data
    fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mux.digest.clear_data()
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Client<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        match self.mode.get() {
            Mode::Hmac => {
                self.hmac_client
                    .map(move |client| client.add_data_done(result, data));
            }
            Mode::Sha => {
                self.sha_client
                    .map(move |client| client.add_data_done(result, data));
            }
        }
    }

    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        match self.mode.get() {
            Mode::Hmac => {
                self.hmac_client
                    .map(move |client| client.hash_done(result, digest));
            }
            Mode::Sha => {
                self.sha_client
                    .map(move |client| client.hash_done(result, digest));
            }
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha256, const L: usize> digest::HMACSha256
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha256(key)
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha256(key)
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha384, const L: usize> digest::HMACSha384
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha384(key)
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha384(key)
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha512, const L: usize> digest::HMACSha512
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha512(key)
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Hmac);
            self.mux.digest.set_mode_hmacsha512(key)
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha256, const L: usize> digest::Sha256
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha256()
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha256()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha384, const L: usize> digest::Sha384
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha384()
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha384()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha512, const L: usize> digest::Sha512
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha512()
        } else if self.mux.running_id.get() == self.id {
            self.mode.set(Mode::Sha);
            self.mux.digest.set_mode_sha512()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

/// Calling a 'set_mode*()' function from a `VirtualMuxDigest` will mark that
/// `VirtualMuxDigest` as the one that has been enabled and running. Until that
/// Mux calls `clear_data()` it will be the only `VirtualMuxDigest` that can
/// interact with the underlying device.
pub struct MuxDigest<'a, A: digest::Digest<'a, L>, const L: usize> {
    digest: &'a A,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> MuxDigest<'a, A, L> {
    pub const fn new(digest: &'a A) -> MuxDigest<'a, A, L> {
        MuxDigest {
            digest: digest,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
        }
    }
}
