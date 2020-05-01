//! Virtualize the Digest interface to enable multiple users of an underlying
//! Digest hardware peripheral.

use core::cell::Cell;
use core::marker::PhantomData;
use kernel::common::cells::OptionalCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::{ListLink, ListNode};
use kernel::hil::digest;
use kernel::hil::digest::DigestType;
use kernel::ReturnCode;

pub struct VirtualMuxDigest<'a, A: digest::Digest<'a, T>, T: DigestType> {
    mux: &'a MuxDigest<'a, A, T>,
    next: ListLink<'a, VirtualMuxDigest<'a, A, T>>,
    client: OptionalCell<&'a dyn digest::Client<'a, T>>,
    id: u32,
}

impl<A: digest::Digest<'a, T>, T: DigestType> ListNode<'a, VirtualMuxDigest<'a, A, T>>
    for VirtualMuxDigest<'a, A, T>
{
    fn next(&self) -> &'a ListLink<VirtualMuxDigest<'a, A, T>> {
        &self.next
    }
}

impl<A: digest::Digest<'a, T>, T: DigestType> VirtualMuxDigest<'a, A, T> {
    pub fn new(mux_digest: &'a MuxDigest<'a, A, T>) -> VirtualMuxDigest<'a, A, T> {
        let id = mux_digest.next_id.get();
        mux_digest.next_id.set(id + 1);

        VirtualMuxDigest {
            mux: mux_digest,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            id: id,
        }
    }
}

impl<A: digest::Digest<'a, T>, T: DigestType> digest::Digest<'a, T> for VirtualMuxDigest<'a, A, T> {
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, T>) {
        self.client.set(client);
    }

    /// Add data to the digest IP.
    /// All data passed in is fed to the Digest hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.digest.add_data(data)
        } else if self.mux.running_id.get() == self.id {
            self.mux.digest.add_data(data)
        } else {
            Err((ReturnCode::EBUSY, data.take()))
        }
    }

    /// Request the hardware block to generate a Digest
    /// This doesn't return anything, instead the client needs to have
    /// set a `hash_done` handler.
    fn run(&'a self, digest: &'static mut T) -> Result<(), (ReturnCode, &'static mut T)> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.digest.run(digest)
        } else if self.mux.running_id.get() == self.id {
            self.mux.digest.run(digest)
        } else {
            Err((ReturnCode::EBUSY, digest))
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

impl<A: digest::Digest<'a, T>, T: DigestType> digest::Client<'a, T> for VirtualMuxDigest<'a, A, T> {
    fn add_data_done(&'a self, result: Result<(), ReturnCode>, data: &'static mut [u8]) {
        self.client
            .map(move |client| client.add_data_done(result, data));
    }

    fn hash_done(&'a self, result: Result<(), ReturnCode>, digest: &'static mut T) {
        self.client
            .map(move |client| client.hash_done(result, digest));
    }
}

impl<A: digest::Digest<'a, T> + digest::HMACSha256, T: DigestType> digest::HMACSha256
    for VirtualMuxDigest<'a, A, T>
{
    fn set_mode_hmacsha256(&self, key: &[u8; 32]) -> Result<(), ReturnCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.digest.set_mode_hmacsha256(key)
        } else if self.mux.running_id.get() == self.id {
            self.mux.digest.set_mode_hmacsha256(key)
        } else {
            Err(ReturnCode::EBUSY)
        }
    }
}

/// Calling a 'set_mode*()' function from a `VirtualMuxDigest` will mark that
/// `VirtualMuxDigest` as the one that has been enabled and running. Until that
/// Mux calls `clear_data()` it will be the only `VirtualMuxDigest` that can
/// interact with the underlying device.
pub struct MuxDigest<'a, A: digest::Digest<'a, T>, T: DigestType> {
    digest: &'a A,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
    phantom: PhantomData<&'a T>,
}

impl<A: digest::Digest<'a, T>, T: DigestType> MuxDigest<'a, A, T> {
    pub const fn new(digest: &'a A) -> MuxDigest<'a, A, T> {
        MuxDigest {
            digest: digest,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
            phantom: PhantomData,
        }
    }
}
