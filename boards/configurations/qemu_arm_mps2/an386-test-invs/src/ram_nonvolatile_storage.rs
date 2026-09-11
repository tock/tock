// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Implementation of the nonvolatile storage HIL backed by RAM.
//!
//! This is useful on platforms with no nonvolatile storage peripheral at all
//! -- the AN386 does not emulate a flash controller. Contents are lost on
//! reset, but this allows exercising the rest of the nonvolatile storage
//! stack (including
//! `capsules_extra::isolated_nonvolatile_storage_driver` and
//! `kernel::dynamic_binary_storage`) without real hardware support.
//!
//! This is board-local, not a general-purpose capsule: it only exists to
//! stand in for the flash this board doesn't have, so it isn't published
//! from `capsules_extra` for other boards to depend on.
//!
//! Reads and writes complete asynchronously via a deferred call, matching the
//! HIL's expectation that a real (flash-backed) implementation would not
//! complete synchronously with the call that started it.
//!
//! ## Why this accesses memory through raw pointers
//!
//! [`RamNonvolatileStorage::new_at()`] lets an instance stand in for a
//! region that something else also holds a `&'static [u8]` over -- this
//! board uses it that way for the app flash region (`_sapps`..`_eapps`),
//! which `SequentialProcessLoaderMachine` keeps a live `&'static [u8]` to
//! for as long as the kernel runs. Accessing that memory through a `&mut
//! [u8]` here (a second, mutable, live-for-a-scope reference to bytes
//! another live shared reference already points at) would be exactly the
//! aliasing a real flash-backed implementation doesn't run into -- on real
//! hardware, writes go through a peripheral's control registers, not
//! through a reference to the flash's own mapped address range. This
//! mimics that: every access here goes through a raw pointer computed from
//! `memory_base`, copied via [`core::ptr::copy_nonoverlapping`], and no
//! `&`/`&mut` over the target range is ever created. [`Self::new()`] uses
//! the same representation for the (non-aliased) case of owning a private
//! buffer outright, so there is one implementation either way.

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

/// A [`NonvolatileStorage`] implementation backed by RAM rather than a real
/// storage peripheral, accessed by address rather than by holding a
/// reference to it -- see the module docs for why.
pub struct RamNonvolatileStorage<'a> {
    /// Address of the first byte of the backing memory.
    memory_base: usize,
    /// Length, in bytes, of the backing memory.
    memory_len: usize,
    client: OptionalCell<&'a dyn NonvolatileStorageClient>,
    buffer: TakeCell<'static, [u8]>,
    operation: OptionalCell<Operation>,
    deferred_call: DeferredCall,
}

impl<'a> RamNonvolatileStorage<'a> {
    /// Backs this instance with a buffer it owns outright.
    pub fn new(memory: &'static mut [u8]) -> Self {
        // SAFETY: `memory` is a unique `&'static mut`, so nothing else can
        // hold a reference into `[memory_base, memory_base + memory_len)`;
        // taking its address and dropping the reference doesn't change
        // that.
        unsafe { Self::new_at(memory.as_mut_ptr() as usize, memory.len()) }
    }

    /// Backs this instance with the `memory_len` bytes starting at
    /// `memory_base`, without requiring a reference to them.
    ///
    /// # Safety
    ///
    /// `[memory_base, memory_base + memory_len)` must be a valid,
    /// byte-addressable range for the lifetime of this instance. It may be
    /// concurrently read through a `&'static [u8]` held elsewhere (that's
    /// the case this exists for), but not concurrently written by anything
    /// other than this instance.
    pub unsafe fn new_at(memory_base: usize, memory_len: usize) -> Self {
        Self {
            memory_base,
            memory_len,
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
        if address
            .checked_add(length)
            .is_none_or(|end| end > self.memory_len)
        {
            return Err((ErrorCode::INVAL, buffer));
        }

        // SAFETY: bounds checked above; `buffer` is a distinct allocation
        // from our backing memory, so the two ranges cannot overlap. This
        // never forms a `&`/`&mut` over the backing memory -- see the
        // module docs.
        unsafe {
            core::ptr::copy_nonoverlapping(
                (self.memory_base + address) as *const u8,
                buffer.as_mut_ptr(),
                length,
            );
        }

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
        // HACK
        let address = address - 0x40000;

        kernel::debug!("RNS w {:#02x} {}", address, length);

        if self.operation.is_some() {
            return Err((ErrorCode::BUSY, buffer));
        }
        if length > buffer.len() {
            return Err((ErrorCode::INVAL, buffer));
        }
        if address
            .checked_add(length)
            .is_none_or(|end| end > self.memory_len)
        {
            return Err((ErrorCode::INVAL, buffer));
        }

        // SAFETY: bounds checked above; `buffer` is a distinct allocation
        // from our backing memory, so the two ranges cannot overlap. This
        // never forms a `&`/`&mut` over the backing memory -- see the
        // module docs.
        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                (self.memory_base + address) as *mut u8,
                length,
            );
        }

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
