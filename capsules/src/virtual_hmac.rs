//! Virtualize the HMAC interface to enable multiple users of an underlying
//! HMAC hardware peripheral.

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::digest::{self, Client, Digest};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::ErrorCode;

use crate::virtual_digest::{Mode, Operation};

pub struct VirtualMuxHmac<'a, A: digest::Digest<'a, L>, const L: usize> {
    mux: &'a MuxHmac<'a, A, L>,
    next: ListLink<'a, VirtualMuxHmac<'a, A, L>>,
    client: OptionalCell<&'a dyn digest::Client<'a, L>>,
    key: TakeCell<'static, [u8]>,
    data: TakeCell<'static, [u8]>,
    data_len: Cell<usize>,
    digest: TakeCell<'static, [u8; L]>,
    mode: Cell<Mode>,
    id: u32,
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> ListNode<'a, VirtualMuxHmac<'a, A, L>>
    for VirtualMuxHmac<'a, A, L>
{
    fn next(&self) -> &'a ListLink<VirtualMuxHmac<'a, A, L>> {
        &self.next
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> VirtualMuxHmac<'a, A, L> {
    pub fn new(
        mux_hmac: &'a MuxHmac<'a, A, L>,
        key: &'static mut [u8],
    ) -> VirtualMuxHmac<'a, A, L> {
        let id = mux_hmac.next_id.get();
        mux_hmac.next_id.set(id + 1);

        VirtualMuxHmac {
            mux: mux_hmac,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            key: TakeCell::new(key),
            data: TakeCell::empty(),
            data_len: Cell::new(0),
            digest: TakeCell::empty(),
            mode: Cell::new(Mode::None),
            id: id,
        }
    }
}

impl<'a, A: digest::Digest<'a, L>, const L: usize> digest::Digest<'a, L>
    for VirtualMuxHmac<'a, A, L>
{
    /// Set the client instance which will receive `add_data_done()` and
    /// `hash_done()` callbacks
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        let node = self.mux.users.iter().find(|node| node.id == self.id);
        if node.is_none() {
            self.mux.users.push_head(self);
        }
        self.mux.hmac.set_client(client);
    }

    /// Add data to the hmac IP.
    /// All data passed in is fed to the HMAC hardware block.
    /// Returns the number of bytes written on success
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.hmac.add_data(data)
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.data.is_none() {
                let len = data.len();
                self.data.replace(data.take());
                self.data_len.set(len);
                Ok(len)
            } else {
                Err((ErrorCode::BUSY, data.take()))
            }
        }
    }

    /// Request the hardware block to generate a HMAC
    /// This doesn't return anything, instead the client needs to have
    /// set a `hash_done` handler.
    fn run(
        &'a self,
        digest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running_id.get() == self.id {
            self.mux.hmac.run(digest)
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

    /// Disable the HMAC hardware and clear the keys and any other sensitive
    /// data
    fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mode.set(Mode::None);
            self.mux.hmac.clear_data()
        }
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::HMACSha256 + digest::HMACSha384 + digest::HMACSha512,
        const L: usize,
    > digest::Client<'a, L> for VirtualMuxHmac<'a, A, L>
{
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        self.client
            .map(move |client| client.add_data_done(result, data));
        self.mux.do_next_op();
    }

    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        self.client
            .map(move |client| client.hash_done(result, digest));

        // Forcefully clear the data to allow other apps to use the HMAC
        self.clear_data();
        self.mux.do_next_op();
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha256, const L: usize> digest::HMACSha256
    for VirtualMuxHmac<'a, A, L>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha256));
            self.mux.hmac.set_mode_hmacsha256(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha256));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha384, const L: usize> digest::HMACSha384
    for VirtualMuxHmac<'a, A, L>
{
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha384));
            self.mux.hmac.set_mode_hmacsha384(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha384));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
        }
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::HMACSha512, const L: usize> digest::HMACSha512
    for VirtualMuxHmac<'a, A, L>
{
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mode.set(Mode::Hmac(Operation::Sha512));
            self.mux.hmac.set_mode_hmacsha512(key)
        } else {
            self.mode.set(Mode::Hmac(Operation::Sha512));
            self.key.map(|buf| buf.copy_from_slice(key));
            Ok(())
        }
    }
}

pub struct MuxHmac<'a, A: digest::Digest<'a, L>, const L: usize> {
    hmac: &'a A,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
    users: List<'a, VirtualMuxHmac<'a, A, L>>,
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::HMACSha256 + digest::HMACSha384 + digest::HMACSha512,
        const L: usize,
    > MuxHmac<'a, A, L>
{
    pub const fn new(hmac: &'a A) -> MuxHmac<'a, A, L> {
        MuxHmac {
            hmac,
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
                Mode::Sha(_) => {}
                Mode::Hmac(op) => {
                    match op {
                        Operation::Sha256 => {
                            node.key.map(|buf| {
                                self.hmac.set_mode_hmacsha256(buf).unwrap();
                            });
                        }
                        Operation::Sha384 => {
                            node.key.map(|buf| {
                                self.hmac.set_mode_hmacsha384(buf).unwrap();
                            });
                        }
                        Operation::Sha512 => {
                            node.key.map(|buf| {
                                self.hmac.set_mode_hmacsha512(buf).unwrap();
                            });
                        }
                    }
                    return;
                }
            }

            if node.data.is_some() {
                let mut lease = LeasableBuffer::new(node.data.take().unwrap());
                lease.slice(0..node.data_len.get());

                if let Err((err, digest)) = self.hmac.add_data(lease) {
                    node.add_data_done(Err(err), digest);
                }
                return;
            }

            if node.digest.is_some() {
                if let Err((err, data)) = self.hmac.run(node.digest.take().unwrap()) {
                    node.hash_done(Err(err), data);
                }
            }
        });
    }
}
