// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::UIntLike;
use core::ptr::{read_volatile, write_volatile};

/// The bus design on some systems (*cough* LiteX) may require mapping a
/// read/write operation for a single register on the peripheral into multiple
/// operations on the CPU design. `BusAdapter` implements "access a peripheral
/// register" operations using the series of volatile reads and writes that are
/// needed on a particular system. `DirectBus` is an implementation of
/// `BusAdapter` for systems with no unusual pointering requirements.
///
/// # Safety
/// `SIZE` must be correct. If `SIZE` is incorrect, then Mmio* structs generated
/// by peripheral! may generate incorrect offsets for later registers, leading
/// to UB.
pub unsafe trait BusAdapter<Value: UIntLike>: Copy {
    /// The amount of MMIO pointer space this type takes.
    const SIZE: usize;

    /// Reads the value at the provided pointer.
    /// # Safety
    /// `pointer` must be a valid pointer to a readable register of type `Value`
    /// attached to a bus type this BusAdapter supports. If that register has
    /// hardware-specific safety requirements, the caller must comply with those
    /// as well.
    unsafe fn read(self, pointer: *const ()) -> Value;

    /// Writes the value at the provided pointer.
    /// # Safety
    /// `pointer` must be a valid pointer to a writable register of type `Value`
    /// attached to a bus type this BusAdapter supports. If that register has
    /// hardware-specific safety requirements, the caller must comply with those
    /// as well.
    unsafe fn write(self, pointer: *mut (), value: Value);
}

/// A `BusAdapter` for systems with no unusual pointering requirements.
#[derive(Clone, Copy)]
pub struct DirectBus;

unsafe impl<Value: UIntLike> BusAdapter<Value> for DirectBus {
    // Safety: On a direct-mapping bus, the size of a register of type T is just
    // the size of type T.
    const SIZE: usize = core::mem::size_of::<Value>();

    unsafe fn read(self, pointer: *const ()) -> Value {
        // Safety:
        // By using DirectBus, the caller has promised that pointer points to a
        // readable MMIO register of type Value with no memory mapping required.
        // The caller has met any hardware-specific safety requirements that
        // register has.
        unsafe { read_volatile(pointer.cast()) }
    }

    unsafe fn write(self, pointer: *mut (), value: Value) {
        // Safety:
        // By using DirectBus, the caller has promised that pointer points to a
        // writable MMIO register of type Value with no memory mapping required.
        // The caller has met any hardware-specific safety requirements that
        // register has.
        unsafe { write_volatile(pointer.cast(), value) }
    }
}
