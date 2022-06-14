//! Virtualize the SHA interface to enable multiple users of an underlying
//! SHA hardware peripheral.

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::digest::{self, ClientHash, ClientVerify};
use kernel::hil::digest::{ClientData, DigestData};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{
    LeasableBuffer, LeasableBufferDynamic, LeasableMutableBuffer,
};
use kernel::ErrorCode;

use crate::virtual_digest::{Mode, Operation};

pub struct VirtualMuxSha<'a, A: digest::Digest<'a, L>, const L: usize> {
    mux: &'a MuxSha<'a, A, L>,
    next: ListLink<'a, VirtualMuxSha<'a, A, L>>,
    client: OptionalCell<&'a dyn digest::Client<L>>,
    data: OptionalCell<LeasableBufferDynamic<'static, u8>>,
    data_len: Cell<usize>,
    digest: TakeCell<'static, [u8; L]>,
    verify: Cell<bool>,
    mode: Cell<Mode>,
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
            data: OptionalCell::empty(),
            data_len: Cell::new(0),
            digest: TakeCell::empty(),
            verify: Cell::new(false),
            mode: Cell::new(Mode::None),
            id: id,
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestData<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    /// Add data to the sha IP.
    /// All data passed in is fed to the SHA hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.sha.add_data(data)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.data.is_none() {
                let len = data.len();
                self.data.replace(LeasableBufferDynamic::Immutable(data));
                self.data_len.set(len);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    /// Add data to the sha IP.
    /// All data passed in is fed to the SHA hardware block.
    /// Returns the number of bytes written on success
    fn add_mut_data(
        &self,
        data: LeasableMutableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableMutableBuffer<'static, u8>)> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.sha.add_mut_data(data)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.data.is_none() {
                let len = data.len();
                self.data.replace(LeasableBufferDynamic::Mutable(data));
                self.data_len.set(len);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    /// Disable the SHA hardware and clear the keys and any other sensitive
    /// data
    fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mode.set(Mode::None);
            self.mux.sha.clear_data()
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestHash<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    /// Request the hardware block to generate a SHA
    /// This doesn't return anything, instead the client needs to have
    /// set a `hash_done` handler.
    fn run(
        &'a self,
        digest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.sha.run(digest)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.digest.is_none() {
                self.digest.replace(digest);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, digest))
            }
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestVerify<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    fn verify(
        &self,
        compare: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled
        if self.mux.running_id.get() == self.id {
            self.mux.sha.verify(compare)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.digest.is_none() {
                self.digest.replace(compare);
                self.verify.set(true);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, compare))
            }
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Digest<'a, L>
    for VirtualMuxSha<'a, A, L>
{
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, client: &'a dyn digest::Client<L>) {
        let node = self.mux.users.iter().find(|node| node.id == self.id);
        if node.is_none() {
            self.mux.users.push_head(self);
        }
        self.mux.sha.set_client(client);
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > digest::ClientData<L> for VirtualMuxSha<'a, A, L>
{
    fn add_data_done(&self, result: Result<(), ErrorCode>, data: LeasableBuffer<'static, u8>) {
        self.client
            .map(move |client| client.add_data_done(result, data));
        self.mux.do_next_op();
    }

    fn add_mut_data_done(
        &self,
        result: Result<(), ErrorCode>,
        data: LeasableMutableBuffer<'static, u8>,
    ) {
        self.client
            .map(move |client| client.add_mut_data_done(result, data));
        self.mux.do_next_op();
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > digest::ClientHash<L> for VirtualMuxSha<'a, A, L>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        self.client
            .map(move |client| client.hash_done(result, digest));

        // Forcefully clear the data to allow other apps to use the HMAC
        self.clear_data();
        self.mux.do_next_op();
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > digest::ClientVerify<L> for VirtualMuxSha<'a, A, L>
{
    fn verification_done(&self, result: Result<bool, ErrorCode>, digest: &'static mut [u8; L]) {
        self.client
            .map(move |client| client.verification_done(result, digest));

        // Forcefully clear the data to allow other apps to use the HMAC
        self.clear_data();
        self.mux.do_next_op();
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
            self.mode.set(Mode::Sha(Operation::Sha256));
            self.mux.sha.set_mode_sha256()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha256));
            Ok(())
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
            self.mode.set(Mode::Sha(Operation::Sha384));
            self.mux.sha.set_mode_sha384()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha384));
            Ok(())
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
            self.mode.set(Mode::Sha(Operation::Sha512));
            self.mux.sha.set_mode_sha512()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha512));
            Ok(())
        }
    }
}

pub struct MuxSha<'a, A: digest::Digest<'a, L>, const L: usize> {
    sha: &'a A,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
    users: List<'a, VirtualMuxSha<'a, A, L>>,
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > MuxSha<'a, A, L>
{
    pub const fn new(sha: &'a A) -> MuxSha<'a, A, L> {
        MuxSha {
            sha,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
            users: List::new(),
        }
    }

    fn do_next_op(&self) {
        let mnode = self.users.iter().find(|node| node.mode.get() != Mode::None);
        mnode.map(|node| {
            self.running.set(true);
            self.running_id.set(node.id);

            match node.mode.get() {
                Mode::None => {}
                Mode::Hmac(_) => {}
                Mode::Sha(op) => {
                    match op {
                        Operation::Sha256 => {
                            self.sha.set_mode_sha256().unwrap();
                        }
                        Operation::Sha384 => {
                            self.sha.set_mode_sha384().unwrap();
                        }
                        Operation::Sha512 => {
                            self.sha.set_mode_sha512().unwrap();
                        }
                    }
                    return;
                }
            }

            if node.data.is_some() {
                let leasable = node.data.take().unwrap();
                match leasable {
                    LeasableBufferDynamic::Mutable(mut b) => {
                        b.slice(0..node.data_len.get());
                        if let Err((err, slice)) = self.sha.add_mut_data(b) {
                            node.add_mut_data_done(Err(err), slice);
                        }
                    }
                    LeasableBufferDynamic::Immutable(mut b) => {
                        b.slice(0..node.data_len.get());
                        if let Err((err, slice)) = self.sha.add_data(b) {
                            node.add_data_done(Err(err), slice);
                        }
                    }
                }
                return;
            }

            if node.digest.is_some() {
                if node.verify.get() {
                    if let Err((err, compare)) = self.sha.verify(node.digest.take().unwrap()) {
                        node.verification_done(Err(err), compare);
                    }
                } else {
                    if let Err((err, data)) = self.sha.run(node.digest.take().unwrap()) {
                        node.hash_done(Err(err), data);
                    }
                }
            }
        });
    }
}
