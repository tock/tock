//! Virtualize the Digest interface to enable multiple users of an underlying
//! Digest hardware peripheral.

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::digest::{self, ClientHash, ClientVerify};
use kernel::hil::digest::{ClientData, DigestData};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{
    LeasableBuffer, LeasableBufferDynamic, LeasableMutableBuffer,
};
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
pub enum Operation {
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    None,
    Hmac(Operation),
    Sha(Operation),
}

pub struct VirtualMuxDigest<'a, A: digest::Digest<'a, L>, const L: usize> {
    mux: &'a MuxDigest<'a, A, L>,
    next: ListLink<'a, VirtualMuxDigest<'a, A, L>>,
    sha_client: OptionalCell<&'a dyn digest::Client<L>>,
    hmac_client: OptionalCell<&'a dyn digest::Client<L>>,
    key: TakeCell<'static, [u8]>,
    data: OptionalCell<LeasableBufferDynamic<'static, u8>>,
    digest: TakeCell<'static, [u8; L]>,
    verify: Cell<bool>,
    mode: Cell<Mode>,
    ready: Cell<bool>,
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
    pub fn new(
        mux_digest: &'a MuxDigest<'a, A, L>,
        key: &'static mut [u8],
    ) -> VirtualMuxDigest<'a, A, L> {
        let id = mux_digest.next_id.get();
        mux_digest.next_id.set(id + 1);

        VirtualMuxDigest {
            mux: mux_digest,
            next: ListLink::empty(),
            sha_client: OptionalCell::empty(),
            hmac_client: OptionalCell::empty(),
            key: TakeCell::new(key),
            data: OptionalCell::empty(),
            digest: TakeCell::empty(),
            verify: Cell::new(false),
            mode: Cell::new(Mode::None),
            ready: Cell::new(false),
            id: id,
        }
    }

    pub fn set_hmac_client(&'a self, client: &'a dyn digest::Client<L>) {
        let node = self.mux.users.iter().find(|node| node.id == self.id);
        if node.is_none() {
            self.mux.users.push_head(self);
        }
        self.hmac_client.set(client);
    }

    pub fn set_sha_client(&'a self, client: &'a dyn digest::Client<L>) {
        let node = self.mux.users.iter().find(|node| node.id == self.id);
        if node.is_none() {
            self.mux.users.push_head(self);
        }
        self.sha_client.set(client);
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestData<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    /// Add data to the digest IP.
    /// All data passed in is fed to the Digest hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.digest.add_data(data)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.data.is_none() {
                self.data.replace(LeasableBufferDynamic::Immutable(data));
                Ok(())
            } else {
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    /// Add data to the digest IP.
    /// All data passed in is fed to the Digest hardware block.
    /// Returns the number of bytes written on success
    fn add_mut_data(
        &self,
        data: LeasableMutableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableMutableBuffer<'static, u8>)> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.digest.add_mut_data(data)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.data.is_none() {
                self.data.replace(LeasableBufferDynamic::Mutable(data));
                Ok(())
            } else {
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    /// Disable the Digest hardware and clear the keys and any other sensitive
    /// data
    fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mode.set(Mode::None);
            self.mux.digest.clear_data()
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestHash<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    /// Request the hardware block to generate a Digest
    /// This doesn't return anything, instead the client needs to have
    /// set a `hash_done` handler.
    fn run(
        &'a self,
        digest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.digest.run(digest)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.digest.is_none() {
                self.digest.replace(digest);
                self.ready.set(true);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, digest))
            }
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::DigestVerify<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    fn verify(
        &self,
        compare: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled
        if self.mux.running_id.get() == self.id {
            self.mux.digest.verify(compare)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.digest.is_none() {
                self.digest.replace(compare);
                self.verify.set(true);
                self.ready.set(true);
                Ok(())
            } else {
                Err((ErrorCode::BUSY, compare))
            }
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Digest<'a, L>
    for VirtualMuxDigest<'a, A, L>
{
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, _client: &'a dyn digest::Client<L>) {
        unimplemented!()
    }
}

impl<
        'a,
        A: digest::Digest<'a, L>
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > digest::ClientData<L> for VirtualMuxDigest<'a, A, L>
{
    fn add_data_done(&self, result: Result<(), ErrorCode>, data: LeasableBuffer<'static, u8>) {
        match self.mode.get() {
            Mode::None => {}
            Mode::Hmac(_) => {
                self.hmac_client
                    .map(move |client| client.add_data_done(result, data));
            }
            Mode::Sha(_) => {
                self.sha_client
                    .map(move |client| client.add_data_done(result, data));
            }
        }
        self.mux.do_next_op();
    }

    fn add_mut_data_done(
        &self,
        result: Result<(), ErrorCode>,
        data: LeasableMutableBuffer<'static, u8>,
    ) {
        match self.mode.get() {
            Mode::None => {}
            Mode::Hmac(_) => {
                self.hmac_client
                    .map(move |client| client.add_mut_data_done(result, data));
            }
            Mode::Sha(_) => {
                self.sha_client
                    .map(move |client| client.add_mut_data_done(result, data));
            }
        }
        self.mux.do_next_op();
    }
}

impl<
        'a,
        A: digest::Digest<'a, L>
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > digest::ClientHash<L> for VirtualMuxDigest<'a, A, L>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        match self.mode.get() {
            Mode::None => {}
            Mode::Hmac(_) => {
                self.hmac_client
                    .map(move |client| client.hash_done(result, digest));
            }
            Mode::Sha(_) => {
                self.sha_client
                    .map(move |client| client.hash_done(result, digest));
            }
        }

        // Forcefully clear the data to allow other apps to use the HMAC
        self.clear_data();
        self.mux.do_next_op();
    }
}
impl<
        'a,
        A: digest::Digest<'a, L>
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > digest::ClientVerify<L> for VirtualMuxDigest<'a, A, L>
{
    fn verification_done(&self, result: Result<bool, ErrorCode>, compare: &'static mut [u8; L]) {
        match self.mode.get() {
            Mode::None => {}
            Mode::Hmac(_) => {
                self.hmac_client
                    .map(move |client| client.verification_done(result, compare));
            }
            Mode::Sha(_) => {
                self.sha_client
                    .map(move |client| client.verification_done(result, compare));
            }
        }

        // Forcefully clear the data to allow other apps to use the HMAC
        self.clear_data();
        self.mux.do_next_op();
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HmacSha256, const L: usize> digest::HmacSha256
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha256));
            self.mux.digest.set_mode_hmacsha256(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha256));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HmacSha384, const L: usize> digest::HmacSha384
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha384));
            self.mux.digest.set_mode_hmacsha384(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha384));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HmacSha512, const L: usize> digest::HmacSha512
    for VirtualMuxDigest<'a, A, L>
{
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha512));
            self.mux.digest.set_mode_hmacsha512(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha512));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
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
            self.mode.set(Mode::Sha(Operation::Sha256));
            self.mux.digest.set_mode_sha256()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha256));
            Ok(())
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
            self.mode.set(Mode::Sha(Operation::Sha384));
            self.mux.digest.set_mode_sha384()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha384));
            Ok(())
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
            self.mode.set(Mode::Sha(Operation::Sha512));
            self.mux.digest.set_mode_sha512()
        } else {
            self.mode.set(Mode::Sha(Operation::Sha512));
            Ok(())
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
    users: List<'a, VirtualMuxDigest<'a, A, L>>,
}

impl<
        'a,
        A: digest::Digest<'a, L>
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > MuxDigest<'a, A, L>
{
    pub const fn new(digest: &'a A) -> MuxDigest<'a, A, L> {
        MuxDigest {
            digest: digest,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
            users: List::new(),
        }
    }

    fn do_next_op(&self) {
        // Search for a node that has a mode set and is set as ready.
        // Ready will indicate that `run()` has been called and the operation
        // can complete
        let mnode = self
            .users
            .iter()
            .find(|node| node.mode.get() != Mode::None && node.ready.get());
        mnode.map(|node| {
            self.running.set(true);
            self.running_id.set(node.id);

            match node.mode.get() {
                Mode::None => {}
                Mode::Hmac(op) => {
                    match op {
                        Operation::Sha256 => {
                            node.key.map(|buf| {
                                self.digest.set_mode_hmacsha256(buf).unwrap();
                            });
                        }
                        Operation::Sha384 => {
                            node.key.map(|buf| {
                                self.digest.set_mode_hmacsha384(buf).unwrap();
                            });
                        }
                        Operation::Sha512 => {
                            node.key.map(|buf| {
                                self.digest.set_mode_hmacsha512(buf).unwrap();
                            });
                        }
                    }
                    return;
                }
                Mode::Sha(op) => {
                    match op {
                        Operation::Sha256 => {
                            self.digest.set_mode_sha256().unwrap();
                        }
                        Operation::Sha384 => {
                            self.digest.set_mode_sha384().unwrap();
                        }
                        Operation::Sha512 => {
                            self.digest.set_mode_sha512().unwrap();
                        }
                    }
                    return;
                }
            }

            if node.data.is_some() {
                let leasable = node.data.take().unwrap();
                match leasable {
                    LeasableBufferDynamic::Mutable(b) => {
                        if let Err((err, slice)) = self.digest.add_mut_data(b) {
                            node.add_mut_data_done(Err(err), slice);
                        }
                    }
                    LeasableBufferDynamic::Immutable(b) => {
                        if let Err((err, slice)) = self.digest.add_data(b) {
                            node.add_data_done(Err(err), slice);
                        }
                    }
                }
                return;
            }

            if node.digest.is_some() {
                if node.verify.get() {
                    if let Err((err, compare)) = self.digest.verify(node.digest.take().unwrap()) {
                        node.verification_done(Err(err), compare);
                    }
                } else {
                    if let Err((err, data)) = self.digest.run(node.digest.take().unwrap()) {
                        node.hash_done(Err(err), data);
                    }
                }
            }
        });
    }
}
