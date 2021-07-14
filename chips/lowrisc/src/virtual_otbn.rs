//! Virtualise the Accel interface to enable multiple users of an underlying
//! Accel hardware peripheral.

use crate::otbn::{Client, Otbn};
use core::cell::Cell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::{ListLink, ListNode};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct VirtualMuxAccel<'a, const T: usize> {
    mux: &'a MuxAccel<'a, T>,
    next: ListLink<'a, VirtualMuxAccel<'a, T>>,
    client: OptionalCell<&'a dyn Client<'a, T>>,
    id: u32,
}

impl<'a, const T: usize> ListNode<'a, VirtualMuxAccel<'a, T>> for VirtualMuxAccel<'a, T> {
    fn next(&self) -> &'a ListLink<VirtualMuxAccel<'a, T>> {
        &self.next
    }
}

impl<'a, const T: usize> VirtualMuxAccel<'a, T> {
    pub fn new(mux_accel: &'a MuxAccel<'a, T>) -> VirtualMuxAccel<'a, T> {
        let id = mux_accel.next_id.get();
        mux_accel.next_id.set(id + 1);

        VirtualMuxAccel {
            mux: mux_accel,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            id: id,
        }
    }

    pub fn set_client(&'a self, client: &'a dyn Client<'a, T>) {
        self.client.set(client);
    }

    pub fn load_binary(
        &self,
        input: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.load_binary(input)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.load_binary(input)
        } else {
            Err((ErrorCode::BUSY, input.take()))
        }
    }

    pub fn set_property(&self, key: usize, value: usize) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.set_property(key, value)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.set_property(key, value)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn run(
        &'a self,
        output: &'static mut [u8; 1024],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 1024])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if self.mux.running.get() == false {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.run(output)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.run(output)
        } else {
            Err((ErrorCode::BUSY, output))
        }
    }

    /// Disable the Accel hardware and clear the keys and any other sensitive
    /// data
    pub fn clear_data(&self) {
        if self.mux.running_id.get() == self.id {
            self.mux.running.set(false);
            self.mux.accel.clear_data()
        }
    }
}

impl<'a, const T: usize> Client<'a, T> for VirtualMuxAccel<'a, T> {
    fn binary_load_done(&'a self, result: Result<(), ErrorCode>, input: &'static mut [u8]) {
        self.client
            .map(move |client| client.binary_load_done(result, input));
    }

    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8; T]) {
        self.client
            .map(move |client| client.op_done(result, output));
    }
}

/// Calling a 'set_mode*()' function from a `VirtualMuxAccel` will mark that
/// `VirtualMuxAccel` as the one that has been enabled and running. Until that
/// Mux calls `clear_data()` it will be the only `VirtualMuxAccel` that can
/// interact with the underlying device.
pub struct MuxAccel<'a, const T: usize> {
    accel: &'a Otbn<'a>,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
}

impl<'a, const T: usize> MuxAccel<'a, T> {
    pub const fn new(accel: &'a Otbn<'a>) -> MuxAccel<'a, T> {
        MuxAccel {
            accel,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
        }
    }
}
