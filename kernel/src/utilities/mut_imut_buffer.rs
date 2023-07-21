// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! An enum that can contain a reference to either a mutable or
//! an immutable buffer.
//!
//! This type is intended for internal use in implementations of HILs
//! or abstractions which need to handle both mutable and immutable
//! buffers.
//!
//! One motivating use case is keys for public key
//! cryptography. Public keys are often distributed as constant values
//! in the flash of a kernel image (e.g., to verify signatures), which
//! requires they be immutable.  Copying them to RAM is expensive
//! because these keys can be very large (e.g., 512 bytes for a
//! 4096-bit RSA key). At the same time, some clients may use
//! dynamically generated or received keys, which are stored in
//! mutable RAM. Requiring that keys be immutable would
//! discard mut on this memory. An
//! implementation can use this type to store either mutable and
//! immutable buffers. The OTBN (OpenTitan Big Number accelerator) is
//! one example use of MutImutBuffer.
//!
//! Because this type requires dynamic runtime checks that types
//! match, it should not be used in any HILs or standard, external
//! APIs. It is intended only for internal use in implementations.
//!
//! Author: Alistair Francis
//!
//! Usage
//! -----
//!
//!  ```rust
//! use kernel::utilities::mut_imut_buffer::MutImutBuffer;
//!
//! let mut mutable = ['a', 'b', 'c', 'd'];
//! let immutable = ['e', 'f', 'g', 'h'];
//!
//! let shared_buf = MutImutBuffer::Mutable(&mut mutable);
//! let shared_buf2 = MutImutBuffer::Immutable(&immutable);
//!  ```

/// An enum which can hold either a mutable or an immutable buffer
pub enum MutImutBuffer<'a, T> {
    Mutable(&'a mut [T]),
    Immutable(&'a [T]),
}

impl<'a, T> MutImutBuffer<'a, T> {
    /// Returns the length of the underlying buffer
    pub fn len(&self) -> usize {
        match self {
            MutImutBuffer::Mutable(buf) => buf.len(),
            MutImutBuffer::Immutable(buf) => buf.len(),
        }
    }
}
