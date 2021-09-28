//! Defines an emum that can be used to hold either a mutable static or an
//! immutable static.
//!
//! This allows callers to decide if they want to use a mutable static or an
//! immutable static. This is useful when we want to allow passing read only
//! static buffers while also allowing read/write buffers.
//!
//! See the RSA Key implementation for an example of this use case.
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
