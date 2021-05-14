//! Virtualize the SHA interface to enable multiple users of an underlying
//! SHA hardware peripheral.

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::{ListLink, ListNode};
use kernel::hil::digest;
use kernel::ErrorCode;

pub struct VirtualMuxSha<'a, A: digest::Digest<'a, L>, const L: usize> {
    mux: &'a MuxSha<'a, A, L>,
    next: ListLink<'a, VirtualMuxSha<'a, A, L>>,
    client: OptionalCell<&'a dyn digest::Client<'a, L>>,
    id: u32,
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> ListNode<'a, VirtualMuxSha<'a, A, L>>
    for VirtualMuxSha<'a, A, L>
{
    fn next(&self) -> &'a ListLink<VirtualMuxSha<'a, A, L>> {
        &self.next
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> VirtualMuxSha<'a, A, L> {
    pub fn new(mux_sha: &'a MuxSha<'a, A, L>) -> VirtualMuxSha<'a, A, L> {
        let id = mux_sha.next_id.get();
        mux_sha.next_id.set(id + 1);

        VirtualMuxSha {
            mux: mux_sha,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            id: id,
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Digest<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.mux.sha.set_client(client);
    }

    /// Add data to the sha IP.
    /// All data passed in is fed to the SHA hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.sha.add_data(data)
        } else if self.mux.running_id.get() == self.id {
            self.mux.sha.add_data(data)
        } else {
            Err((ErrorCode::BUSY, data.take()))
        }
    }

    /// Request the hardware block to generate a SHA
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
            self.mux.sha.run(digest)
        } else if self.mux.running_id.get() == self.id {
            self.mux.sha.run(digest)
        } else {
            Err((ErrorCode::BUSY, digest))
        }
    }

    /// Disable the SHA hardware and clear the keys and any other sensitive
    /// data
    fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mux.sha.clear_data()
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Client<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        self.client
            .map(move |client| client.add_data_done(result, data));
    }

    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        self.client
            .map(move |client| client.hash_done(result, digest));
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha256, const L: usize> digest::Sha256
    for VirtualMuxSha<'a, A, L>
{
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.sha.set_mode_sha256()
        } else if self.mux.running_id.get() == self.id {
            self.mux.sha.set_mode_sha256()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha384, const L: usize> digest::Sha384
    for VirtualMuxSha<'a, A, L>
{
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.sha.set_mode_sha384()
        } else if self.mux.running_id.get() == self.id {
            self.mux.sha.set_mode_sha384()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::Sha512, const L: usize> digest::Sha512
    for VirtualMuxSha<'a, A, L>
{
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.sha.set_mode_sha512()
        } else if self.mux.running_id.get() == self.id {
            self.mux.sha.set_mode_sha512()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

pub struct MuxSha<'a, A: digest::Digest<'a, L>, const L: usize> {
    sha: &'a A,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> MuxSha<'a, A, L> {
    pub const fn new(sha: &'a A) -> MuxSha<'a, A, L> {
        MuxSha {
            sha,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
        }
    }
}
