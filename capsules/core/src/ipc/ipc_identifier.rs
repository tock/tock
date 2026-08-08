// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Identifier for processes interacting via IPC.
//!
//! This identifier is intentionally opaque to userspace, only guaranteeing that
//! it exists as a u64. There is no promise that the internal implementation
//! will not change in the future.

use kernel::ProcessId;

/// Identifier used for IPC communication between processes.
///
/// An IPC Identifier is used as a handle to a potential process for targetting
/// IPC communication. The process it refers to does not necessarily exist.
///
/// This identifier is guaranteed to be encodable as a 64-bit value. Each IPD
/// Identifier corresponds to a unique u64 value, and each u64 value
/// corresponds to a unique IPC Identifier.
///
/// This should be treated as an opaque type, with no consideration of the
/// internal implementation. The internal implementation is subject to change.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IpcIdentifier {
    process_id_num: u32,
    short_id_num: u32,
}

impl IpcIdentifier {
    /// Create a new IpcIdentifier from a ProcessId. This is typically used to
    /// create an IpcIdentifier from a process whose grant space a capsule is
    /// interacting with.
    pub fn new_from_processid(processid: ProcessId) -> Self {
        Self {
            process_id_num: processid.id() as u32,
            short_id_num: match processid.short_app_id() {
                kernel::process::ShortId::LocallyUnique => 0,
                kernel::process::ShortId::Fixed(non_zero) => non_zero.get(),
            },
        }
    }

    /// Create a new IpcIdentifier from two 32-bit values, a lower and upper
    /// value. This is typically used to create an IpcIdentifier from values
    /// passed in from userspace.
    pub fn new_from_halves(lower: u32, upper: u32) -> Self {
        Self {
            process_id_num: lower,
            short_id_num: upper,
        }
    }

    /// Get the lower 32 bits of the IpcIdentifier 64-bit encoding. Typically
    /// used to send IpcIdentifier values to userspace.
    pub fn lower(&self) -> u32 {
        self.process_id_num
    }

    /// Get the upper 32 bits of the IpcIdentifier 64-bit encoding. Typically
    /// used to send IpcIdentifier values to userspace.
    pub fn upper(&self) -> u32 {
        self.short_id_num
    }
}

impl From<IpcIdentifier> for u64 {
    /// Conversion from an IpcIdentifier to its 64-bit encoding.
    fn from(id: IpcIdentifier) -> Self {
        ((id.upper() as u64) << 32) | (id.lower() as u64)
    }
}

impl From<u64> for IpcIdentifier {
    /// Conversion from a 64-bit encoding to an IpcIdentifier.
    fn from(encoding: u64) -> Self {
        let lower: u32 = encoding as u32;
        let upper: u32 = (encoding >> 32) as u32;
        IpcIdentifier::new_from_halves(lower, upper)
    }
}
