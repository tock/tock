// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Minimal implementation of Multiboot V1
//!
//! <https://www.gnu.org/software/grub/manual/multiboot/multiboot.html>

/// Magic number for Multboot V1 header
const MAGIC_NUMBER: u32 = 0x1BADB002;

/// Minimal Multiboot V1 header structure
#[repr(C)]
pub struct MultibootV1Header {
    magic: u32,
    flags: u32,
    checksum: u32,
}

impl MultibootV1Header {
    /// Constructs a new Multiboot header instance using the given flags
    ///
    /// This function automatically computes an appropriate checksum value for the header.
    pub const fn new(flags: u32) -> Self {
        let mut checksum: u32 = 0;
        checksum = checksum.wrapping_sub(MAGIC_NUMBER);
        checksum = checksum.wrapping_sub(flags);

        Self {
            magic: MAGIC_NUMBER,
            flags,
            checksum,
        }
    }
}
