// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Implementation of the nonvolatile storage HIL backed by RAM.
//!
//! This is useful on platforms with no nonvolatile storage peripheral at all
//! -- for example, the QEMU MPS2 AN386 machine, which does not emulate a
//! flash controller. Contents are lost on reset, but this allows exercising
//! the rest of the nonvolatile storage stack (including
//! [`crate::isolated_nonvolatile_storage_driver`]) without real hardware
//! support.
//!
//! Reads and writes complete asynchronously via a deferred call, matching the
//! HIL's expectation that a real (flash-backed) implementation would not
//! complete synchronously with the call that started it.
//!
//! Usage
//! -----
//! ```rust,ignore
//! # use kernel::static_init;
//! static mut STORAGE: [u8; 4096] = [0; 4096];
//! let ram_nonvolatile_storage = static_init!(
//!     capsules_extra::ram_nonvolatile_storage::RamNonvolatileStorage<'static>,
//!     capsules_extra::ram_nonvolatile_storage::RamNonvolatileStorage::new(&mut STORAGE)
//! );
//! ram_nonvolatile_storage.register();
//! ```

use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};

/// The operation that is being deferred until the next tick of the kernel
/// loop so that the client is not called back before `read()`/`write()`
/// return.
#[derive(Clone, Copy)]
enum Operation {
    Read { length: usize },
    Write { length: usize },
}

/// A [`NonvolatileStorage`] implementation backed by a `&'static mut [u8]`
/// buffer rather than a real storage peripheral.
pub struct RamNonvolatileStorage<'a> {
    memory: TakeCell<'static, [u8]>,
    client: OptionalCell<&'a dyn NonvolatileStorageClient>,
    buffer: TakeCell<'static, [u8]>,
    operation: OptionalCell<Operation>,
    deferred_call: DeferredCall,
}

impl<'a> RamNonvolatileStorage<'a> {
    pub fn new(memory: &'static mut [u8]) -> Self {
        Self {
            memory: TakeCell::new(memory),
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            operation: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }
}

impl<'a> NonvolatileStorage<'a> for RamNonvolatileStorage<'a> {
    fn set_client(&self, client: &'a dyn NonvolatileStorageClient) {
        self.client.set(client);
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.operation.is_some() {
            return Err((ErrorCode::BUSY, buffer));
        }
        if length > buffer.len() {
            return Err((ErrorCode::INVAL, buffer));
        }
        let in_bounds = self.memory.map_or(false, |memory| {
            address.checked_add(length) <= Some(memory.len())
        });
        if !in_bounds {
            return Err((ErrorCode::INVAL, buffer));
        }

        self.memory.map(|memory| {
            buffer[..length].copy_from_slice(&memory[address..address + length]);
        });

        self.buffer.replace(buffer);
        self.operation.set(Operation::Read { length });
        self.deferred_call.set();
        Ok(())
    }

    fn write(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.operation.is_some() {
            return Err((ErrorCode::BUSY, buffer));
        }
        if length > buffer.len() {
            return Err((ErrorCode::INVAL, buffer));
        }
        let in_bounds = self.memory.map_or(false, |memory| {
            address.checked_add(length) <= Some(memory.len())
        });
        if !in_bounds {
            return Err((ErrorCode::INVAL, buffer));
        }

        self.memory.map(|memory| {
            memory[address..address + length].copy_from_slice(&buffer[..length]);
        });

        self.buffer.replace(buffer);
        self.operation.set(Operation::Write { length });
        self.deferred_call.set();
        Ok(())
    }
}

impl DeferredCallClient for RamNonvolatileStorage<'_> {
    fn handle_deferred_call(&self) {
        if let Some(operation) = self.operation.take() {
            self.buffer.take().map(|buffer| {
                self.client.map(|client| match operation {
                    Operation::Read { length } => client.read_done(buffer, length),
                    Operation::Write { length } => client.write_done(buffer, length),
                });
            });
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
