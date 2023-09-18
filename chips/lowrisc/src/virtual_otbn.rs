// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Virtualise the Accel interface to enable multiple users of an underlying
//! Accel hardware peripheral.

use crate::otbn::{Client, Otbn};
use core::cell::Cell;
use kernel::collections::list::{ListLink, ListNode};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct VirtualMuxAccel<'a> {
    mux: &'a MuxAccel<'a>,
    next: ListLink<'a, VirtualMuxAccel<'a>>,
    client: OptionalCell<&'a dyn Client<'a>>,
    id: u32,
}

impl<'a> ListNode<'a, VirtualMuxAccel<'a>> for VirtualMuxAccel<'a> {
    fn next(&self) -> &'a ListLink<VirtualMuxAccel<'a>> {
        &self.next
    }
}

impl<'a> VirtualMuxAccel<'a> {
    pub fn new(mux_accel: &'a MuxAccel<'a>) -> VirtualMuxAccel<'a> {
        let id = mux_accel.next_id.get();
        mux_accel.next_id.set(id + 1);

        VirtualMuxAccel {
            mux: mux_accel,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            id: id,
        }
    }

    pub fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.client.set(client);
    }

    pub fn load_binary(&self, input: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if !self.mux.running.get() {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.load_binary(input)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.load_binary(input)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn load_data(&self, address: usize, data: &[u8]) -> Result<(), ErrorCode> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if !self.mux.running.get() {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.load_data(address, data)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.load_data(address, data)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn run(
        &self,
        address: usize,
        output: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Check if any mux is enabled. If it isn't we enable it for us.
        if !self.mux.running.get() {
            self.mux.running.set(true);
            self.mux.running_id.set(self.id);
            self.mux.accel.run(address, output)
        } else if self.mux.running_id.get() == self.id {
            self.mux.accel.run(address, output)
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

impl<'a> Client<'a> for VirtualMuxAccel<'a> {
    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8]) {
        self.client
            .map(move |client| client.op_done(result, output));
    }
}

/// Calling a 'set_mode*()' function from a `VirtualMuxAccel` will mark that
/// `VirtualMuxAccel` as the one that has been enabled and running. Until that
/// Mux calls `clear_data()` it will be the only `VirtualMuxAccel` that can
/// interact with the underlying device.
pub struct MuxAccel<'a> {
    accel: &'a Otbn<'a>,
    running: Cell<bool>,
    running_id: Cell<u32>,
    next_id: Cell<u32>,
}

impl<'a> MuxAccel<'a> {
    pub const fn new(accel: &'a Otbn<'a>) -> MuxAccel<'a> {
        MuxAccel {
            accel,
            running: Cell::new(false),
            running_id: Cell::new(0),
            next_id: Cell::new(0),
        }
    }
}
