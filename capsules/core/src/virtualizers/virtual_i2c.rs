// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Virtualize an I2C master bus.
//!
//! `MuxI2C` provides shared access to a single I2C Master Bus for multiple
//! users. `I2CDevice` provides access to a specific I2C address.

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::i2c::{self, Error, I2CClient, I2CHwMasterClient, NoSMBus};
use kernel::utilities::cells::{OptionalCell, TakeCell};

// `NoSMBus` provides a placeholder for `SMBusMaster` in case the board doesn't have a SMBus
pub struct MuxI2C<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a> = NoSMBus> {
    i2c: &'a I,
    smbus: Option<&'a S>,
    i2c_devices: List<'a, I2CDevice<'a, I, S>>,
    i2c_bus: OptionalCell<&'a I2CMultiDevice<'a, I, S>>,
    smbus_devices: List<'a, SMBusDevice<'a, I, S>>,
    enabled: Cell<usize>,
    i2c_inflight: OptionalCell<&'a I2CDevice<'a, I, S>>,
    i2c_bus_inflight: Cell<bool>,
    smbus_inflight: OptionalCell<&'a SMBusDevice<'a, I, S>>,
    deferred_call: DeferredCall,
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CHwMasterClient for MuxI2C<'a, I, S> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        if self.i2c_inflight.is_some() {
            self.i2c_inflight.take().map(move |device| {
                device.command_complete(buffer, status);
            });
        } else if self.smbus_inflight.is_some() {
            self.smbus_inflight.take().map(move |device| {
                device.command_complete(buffer, status);
            });
        } else if self.i2c_bus_inflight.get() {
            self.i2c_bus_inflight.set(false);
            self.i2c_bus
                .map(|device| device.command_complete(buffer, status));
        }
        self.do_next_op();
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> MuxI2C<'a, I, S> {
    pub fn new(i2c: &'a I, smbus: Option<&'a S>) -> Self {
        Self {
            i2c,
            smbus,
            i2c_devices: List::new(),
            i2c_bus: OptionalCell::empty(),
            smbus_devices: List::new(),
            enabled: Cell::new(0),
            i2c_inflight: OptionalCell::empty(),
            i2c_bus_inflight: Cell::new(false),
            smbus_inflight: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    fn enable(&self) {
        let enabled = self.enabled.get();
        self.enabled.set(enabled + 1);
        if enabled == 0 {
            self.i2c.enable();
        }
    }

    fn disable(&self) {
        let enabled = self.enabled.get();
        self.enabled.set(enabled - 1);
        if enabled == 1 {
            self.i2c.disable();
        }
    }

    fn set_address_check(&self, new_addr: u8) -> bool {
        !self
            .i2c_devices
            .iter()
            .any(|node| node.addr.get() == new_addr)
    }

    fn do_next_op(&self) {
        if self.i2c_inflight.is_none()
            && self.smbus_inflight.is_none()
            && !self.i2c_bus_inflight.get()
        {
            // Nothing is currently in flight

            // Try to do the next I2C operation
            let mnode = self
                .i2c_devices
                .iter()
                .find(|node| node.operation.get() != Op::Idle);
            mnode.map(|node| {
                node.buffer.take().map(|buf| {
                    match node.operation.get() {
                        Op::Write(len) => match self.i2c.write(node.addr.get(), buf, len) {
                            Ok(()) => {}
                            Err((error, buffer)) => {
                                node.buffer.replace(buffer);
                                node.operation.set(Op::CommandComplete(Err(error)));
                                node.mux.do_next_op_async();
                            }
                        },
                        Op::Read(len) => match self.i2c.read(node.addr.get(), buf, len) {
                            Ok(()) => {}
                            Err((error, buffer)) => {
                                node.buffer.replace(buffer);
                                node.operation.set(Op::CommandComplete(Err(error)));
                                node.mux.do_next_op_async();
                            }
                        },
                        Op::WriteRead(wlen, rlen) => {
                            match self.i2c.write_read(node.addr.get(), buf, wlen, rlen) {
                                Ok(()) => {}
                                Err((error, buffer)) => {
                                    node.buffer.replace(buffer);
                                    node.operation.set(Op::CommandComplete(Err(error)));
                                    node.mux.do_next_op_async();
                                }
                            }
                        }
                        Op::CommandComplete(err) => {
                            self.command_complete(buf, err);
                        }
                        Op::Idle => {} // Can't get here...
                    }
                });
                node.operation.set(Op::Idle);
                self.i2c_inflight.set(node);
            });

            if self.i2c_inflight.is_none() && self.smbus.is_some() && !self.i2c_bus_inflight.get() {
                // No I2C operation in flight, try SMBus next
                let mnode = self
                    .smbus_devices
                    .iter()
                    .find(|node| node.operation.get() != Op::Idle);
                mnode.map(|node| {
                    node.buffer.take().map(|buf| match node.operation.get() {
                        Op::Write(len) => {
                            match self.smbus.unwrap().smbus_write(node.addr.get(), buf, len) {
                                Ok(()) => {}
                                Err(e) => {
                                    node.buffer.replace(e.1);
                                    node.operation.set(Op::CommandComplete(Err(e.0)));
                                    node.mux.do_next_op_async();
                                }
                            };
                        }
                        Op::Read(len) => {
                            match self.smbus.unwrap().smbus_read(node.addr.get(), buf, len) {
                                Ok(()) => {}
                                Err(e) => {
                                    node.buffer.replace(e.1);
                                    node.operation.set(Op::CommandComplete(Err(e.0)));
                                    node.mux.do_next_op_async();
                                }
                            };
                        }
                        Op::WriteRead(wlen, rlen) => {
                            match self.smbus.unwrap().smbus_write_read(
                                node.addr.get(),
                                buf,
                                wlen,
                                rlen,
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    node.buffer.replace(e.1);
                                    node.operation.set(Op::CommandComplete(Err(e.0)));
                                    node.mux.do_next_op_async();
                                }
                            };
                        }
                        Op::CommandComplete(err) => {
                            self.command_complete(buf, err);
                        }
                        Op::Idle => unreachable!(),
                    });
                    node.operation.set(Op::Idle);
                    self.smbus_inflight.set(node);
                });
            }

            if self.i2c_inflight.is_none() && self.smbus.is_none() && !self.i2c_bus_inflight.get() {
                self.i2c_bus.map(|node| {
                    if node.operation.get() == Op::Idle {
                        return;
                    }

                    node.buffer.take().map(|buf| {
                        match node.operation.get() {
                            Op::Write(len) => match self.i2c.write(node.addr.get(), buf, len) {
                                Ok(()) => {}
                                Err((error, buffer)) => {
                                    node.buffer.replace(buffer);
                                    node.operation.set(Op::CommandComplete(Err(error)));
                                    node.mux.do_next_op_async();
                                }
                            },
                            Op::Read(len) => match self.i2c.read(node.addr.get(), buf, len) {
                                Ok(()) => {}
                                Err((error, buffer)) => {
                                    node.buffer.replace(buffer);
                                    node.operation.set(Op::CommandComplete(Err(error)));
                                    node.mux.do_next_op_async();
                                }
                            },
                            Op::WriteRead(wlen, rlen) => {
                                match self.i2c.write_read(node.addr.get(), buf, wlen, rlen) {
                                    Ok(()) => {}
                                    Err((error, buffer)) => {
                                        node.buffer.replace(buffer);
                                        node.operation.set(Op::CommandComplete(Err(error)));
                                        node.mux.do_next_op_async();
                                    }
                                }
                            }
                            Op::CommandComplete(err) => {
                                self.command_complete(buf, err);
                            }
                            Op::Idle => {} // Can't get here...
                        }
                    });
                    node.operation.set(Op::Idle);
                    self.i2c_bus_inflight.set(true);
                });
            }
        }
    }

    /// Asynchronously executes the next operation, if any. Used by calls
    /// to trigger do_next_op such that it will execute after the call
    /// returns. This is important in case the operation triggers an error,
    /// requiring a callback with an error condition; if the operation
    /// is executed synchronously, the callback may be reentrant (executed
    /// during the downcall). Please see
    /// <https://github.com/tock/tock/issues/1496>
    fn do_next_op_async(&self) {
        self.deferred_call.set();
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> DeferredCallClient for MuxI2C<'a, I, S> {
    fn handle_deferred_call(&self) {
        self.do_next_op();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Write(usize),
    Read(usize),
    WriteRead(usize, usize),
    CommandComplete(Result<(), Error>),
}

/// A I2CDevice
pub struct I2CDevice<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a> = NoSMBus> {
    mux: &'a MuxI2C<'a, I, S>,
    addr: Cell<u8>,
    enabled: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
    operation: Cell<Op>,
    next: ListLink<'a, I2CDevice<'a, I, S>>,
    client: OptionalCell<&'a dyn I2CClient>,
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CDevice<'a, I, S> {
    pub fn new(mux: &'a MuxI2C<'a, I, S>, addr: u8) -> I2CDevice<'a, I, S> {
        I2CDevice {
            mux,
            addr: Cell::new(addr),
            enabled: Cell::new(false),
            buffer: TakeCell::empty(),
            operation: Cell::new(Op::Idle),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&'a self, client: &'a dyn I2CClient) {
        self.mux.i2c_devices.push_head(self);
        self.client.set(client);
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CClient for I2CDevice<'a, I, S> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        self.client.map(move |client| {
            client.command_complete(buffer, status);
        });
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> ListNode<'a, I2CDevice<'a, I, S>>
    for I2CDevice<'a, I, S>
{
    fn next(&'a self) -> &'a ListLink<'a, I2CDevice<'a, I, S>> {
        &self.next
    }
}

impl<'a, I: i2c::I2CMaster<'a>> i2c::I2CDevice for I2CDevice<'a, I> {
    fn enable(&self) {
        if !self.enabled.get() {
            self.enabled.set(true);
            self.mux.enable();
        }
    }

    fn disable(&self) {
        if self.enabled.get() {
            self.enabled.set(false);
            self.mux.disable();
        }
    }

    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::WriteRead(write_len, read_len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn write(&self, data: &'static mut [u8], len: usize) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::Write(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(buffer);
            self.operation.set(Op::Read(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, buffer))
        }
    }
}

/// A I2CMultiDevice
///
/// This is used to expose the "rest of the bus" when using `I2CDevice`s and
/// also wanting to expose a I2C bus to userspace.
pub struct I2CMultiDevice<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a> = NoSMBus> {
    mux: &'a MuxI2C<'a, I, S>,
    addr: Cell<u8>,
    enabled: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
    operation: Cell<Op>,
    client: OptionalCell<&'a dyn I2CClient>,
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CMultiDevice<'a, I, S> {
    pub fn new(mux: &'a MuxI2C<'a, I, S>) -> I2CMultiDevice<'a, I, S> {
        I2CMultiDevice {
            mux,
            addr: Cell::new(0x00),
            enabled: Cell::new(false),
            buffer: TakeCell::empty(),
            operation: Cell::new(Op::Idle),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&'a self, client: &'a dyn I2CClient) {
        self.mux.i2c_bus.replace(self);
        self.client.set(client);
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CClient for I2CMultiDevice<'a, I, S> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        self.client.map(move |client| {
            client.command_complete(buffer, status);
        });
    }
}

impl<'a, I: i2c::I2CMaster<'a>> i2c::I2CDevice for I2CMultiDevice<'a, I> {
    fn enable(&self) {
        if !self.enabled.get() {
            self.enabled.set(true);
            self.mux.enable();
        }
    }

    fn disable(&self) {
        if self.enabled.get() {
            self.enabled.set(false);
            self.mux.disable();
        }
    }

    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::WriteRead(write_len, read_len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn write(&self, data: &'static mut [u8], len: usize) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::Write(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(buffer);
            self.operation.set(Op::Read(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, buffer))
        }
    }
}

impl<'a, I: i2c::I2CMaster<'a>> i2c::I2CMultiDevice for I2CMultiDevice<'a, I, NoSMBus> {
    fn set_address(&self, addr: u8) {
        if addr == self.addr.get() {
            // Short circuit as address is already set
            return;
        }

        if self.mux.set_address_check(addr) {
            self.addr.set(addr);
        }
    }
}

pub struct SMBusDevice<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> {
    mux: &'a MuxI2C<'a, I, S>,
    addr: Cell<u8>,
    enabled: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
    operation: Cell<Op>,
    next: ListLink<'a, SMBusDevice<'a, I, S>>,
    client: OptionalCell<&'a dyn I2CClient>,
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> SMBusDevice<'a, I, S> {
    pub fn new(mux: &'a MuxI2C<'a, I, S>, addr: u8) -> SMBusDevice<'a, I, S> {
        if mux.smbus.is_none() {
            panic!("There is no SMBus to attach to");
        }

        SMBusDevice {
            mux,
            addr: Cell::new(addr),
            enabled: Cell::new(false),
            buffer: TakeCell::empty(),
            operation: Cell::new(Op::Idle),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&'a self, client: &'a dyn I2CClient) {
        self.mux.smbus_devices.push_head(self);
        self.client.set(client);
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> I2CClient for SMBusDevice<'a, I, S> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        self.client.map(move |client| {
            client.command_complete(buffer, status);
        });
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> ListNode<'a, SMBusDevice<'a, I, S>>
    for SMBusDevice<'a, I, S>
{
    fn next(&'a self) -> &'a ListLink<'a, SMBusDevice<'a, I, S>> {
        &self.next
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> i2c::I2CDevice for SMBusDevice<'a, I, S> {
    fn enable(&self) {
        if !self.enabled.get() {
            self.enabled.set(true);
            self.mux.enable();
        }
    }

    fn disable(&self) {
        if self.enabled.get() {
            self.enabled.set(false);
            self.mux.disable();
        }
    }

    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::WriteRead(write_len, read_len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn write(&self, data: &'static mut [u8], len: usize) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::Write(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(buffer);
            self.operation.set(Op::Read(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, buffer))
        }
    }
}

impl<'a, I: i2c::I2CMaster<'a>, S: i2c::SMBusMaster<'a>> i2c::SMBusDevice
    for SMBusDevice<'a, I, S>
{
    fn smbus_write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::WriteRead(write_len, read_len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn smbus_write(
        &self,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(data);
            self.operation.set(Op::Write(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, data))
        }
    }

    fn smbus_read(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.operation.get() == Op::Idle {
            self.buffer.replace(buffer);
            self.operation.set(Op::Read(len));
            self.mux.do_next_op();
            Ok(())
        } else {
            Err((Error::ArbitrationLost, buffer))
        }
    }
}
