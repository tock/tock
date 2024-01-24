// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Defines a SubSlice type to implement leasable buffers.
//!
//! A leasable buffer decouples maintaining a reference to a buffer from the
//! presentation of the accessible buffer. This allows layers to operate on
//! "windows" of the buffer while enabling the original reference (and in effect
//! the entire buffer) to be passed back in a callback.
//!
//! Challenge with Normal Rust Slices
//! ---------------------------------
//!
//! Commonly in Tock we want to partially fill a static buffer with some data,
//! call an asynchronous operation on that data, and then retrieve that buffer
//! via a callback. In common Rust code, that might look something like this
//! (for this example we are transmitting data using I2C).
//!
//! ```rust,ignore
//! // Statically declare the buffer. Make sure it is long enough to handle all
//! // I2C operations we need to perform.
//! let buffer = static_init!([u8; 64], [0; 64]);
//!
//! // Populate the buffer with our current operation.
//! buffer[0] = OPERATION_SET;
//! buffer[1] = REGISTER;
//! buffer[2] = 0x7; // Value to set the register to.
//!
//! // Call the I2C hardware to transmit the data, passing the slice we actually
//! // want to transmit and not the full buffer.
//! i2c.write(buffer[0..3]);
//! ```
//!
//! The issue with this is that within the I2C driver, `buffer` is now only
//! three bytes long. When the I2C driver issues the callback to return the
//! buffer after the transmission completes, the returned buffer will have a
//! length of three. Effectively, the full static buffer is lost.
//!
//! To avoid this, in Tock we always call operations with both the buffer and a
//! separate length. We now have two lengths, the provided `length` parameter
//! which is the size of the buffer actually in use, and `buffer.len()` which is
//! the full size of the static memory.
//!
//! ```rust,ignore
//! // Call the I2C hardware with a reference to the full buffer and the length
//! // of that buffer it should actually consider.
//! i2c.write(buffer, 3);
//! ```
//!
//! Now the I2C driver has a reference to the full buffer, and so when it
//! returns the buffer via callback the client will have access to the full
//! static buffer.
//!
//! Challenge with Buffers + Length
//! -------------------------------
//!
//! Using a reference to the buffer and a separate length parameter is
//! sufficient to address the challenge of needing variable size buffers when
//! using static buffers and complying with Rust's memory management. However,
//! it still has two drawbacks.
//!
//! First, all code in Tock that operates on buffers must correctly handle the
//! separate buffer and length values as though the `buffer` is a `*u8` pointer
//! (as in more traditional C code). We lose many of the benefits of the higher
//! level slice primitive in Rust. For example, calling `buffer.len()` when
//! using data from the buffer is essentially meaningless, as the correct length
//! is the `length` parameter. When copying data _to_ the buffer, however, not
//! overflowing the buffer is critical, and using `buffer.len()` _is_ correct.
//! With separate reference and length managing this is left to the programmer.
//!
//! Second, using only a reference and length assumes that the contents of the
//! buffer will always start at the first entry in the buffer (i.e.,
//! `buffer[0]`). To support more generic use of the buffer, we might want to
//! pass a reference, length, _and offset_, so that we can use arbitrary regions
//! of the buffer, while again retaining a reference to the original buffer to
//! use in callbacks.
//!
//! For example, in networking code it is common to parse headers and then pass
//! the payload to upper layers. With slices, that might look something like:
//!
//! ```rust,ignore
//! // Check for a valid header of size 10.
//! if (valid_header(buffer)) {
//!     self.client.payload_callback(buffer[10..]);
//! }
//! ```
//!
//! The issue is that again the client loses access to the beginning of the
//! buffer and that memory is lost.
//!
//! We might also want to do this when calling lower-layer operations to avoid
//! moving and copying data around. Consider a networking layer that needs to
//! add a header, we might want to do something like:
//!
//! ```rust,ignore
//! buffer[11] = PAYLOAD;
//! network_layer_send(buffer, 11, 1);
//!
//! fn network_layer_send(buffer: &'static [u8], offset: usize, length: usize) {
//!     buffer[0..11] = header;
//!     lower_layer_send(buffer);
//! }
//! ```
//!
//! Now we have to keep track of two parameters which are both redundant with
//! the API provided by Rust slices.
//!
//! Leasable Buffers
//! ----------------
//!
//! A leasable buffer is a data structure that addresses these challenges.
//! Simply, it provides the Rust slice API while internally always retaining a
//! reference to the full underlying buffer. To narrow a buffer, the leasable
//! buffer can be "sliced". To retrieve the full original memory, a leasable
//! buffer can be "reset".
//!
//! A leasable buffer can be sliced multiple times. For example, as a buffer is
//! parsed in a networking stack, each layer can call slice on the leasable
//! buffer to remove that layer's header before passing the buffer to the upper
//! layer.
//!
//! Supporting Mutable and Immutable Buffers
//! ----------------------------------------
//!
//! One challenge with implementing leasable buffers in rust is preserving the
//! mutability of the underlying buffer. If a mutable buffer is passed as an
//! immutable slice, the mutability of that buffer is "lost" (i.e., when passed
//! back in a callback the buffer will be immutable). To address this, we must
//! implement two versions of a leasable buffer: mutable and immutable. That way
//! a mutable buffer remains mutable.
//!
//! Since in Tock most buffers are mutable, the mutable version is commonly
//! used. However, in cases where size is a concern, immutable buffers from
//! flash storage may be preferable. In those cases the immutable version may
//! be used.
//!
//! Usage
//! -----
//!
//! `slice()` is used to set the portion of the `SubSlice` that is accessible.
//! `reset()` makes the entire `SubSlice` accessible again. Typically, `slice()`
//! will be called prior to passing the buffer down to lower layers, and
//! `reset()` will be called once the `SubSlice` is returned via a callback.
//!
//!  ```rust
//! # use kernel::utilities::leasable_buffer::SubSlice;
//!
//! let mut internal = ['a', 'b', 'c', 'd'];
//! let original_base_addr = internal.as_ptr();
//!
//! let mut buffer = SubSlice::new(&mut internal);
//!
//! buffer.slice(1..3);
//!
//! assert_eq!(buffer.as_ptr(), unsafe { original_base_addr.offset(1) });
//! assert_eq!(buffer.len(), 2);
//! assert_eq!((buffer[0], buffer[1]), ('b', 'c'));
//!
//! buffer.reset();
//!
//! assert_eq!(buffer.as_ptr(), original_base_addr);
//! assert_eq!(buffer.len(), 4);
//! assert_eq!((buffer[0], buffer[1]), ('a', 'b'));
//!
//!  ```
//!
//! Author: Amit Levy

use core::ops::{Bound, Range, RangeBounds};
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

/// A mutable leasable buffer implementation.
///
/// A leasable buffer can be used to pass a section of a larger mutable buffer
/// but still get the entire buffer back in a callback.
#[derive(Debug, PartialEq)]
pub struct SubSliceMut<'a, T> {
    internal: &'a mut [T],
    active_range: Range<usize>,
}

/// An immutable leasable buffer implementation.
///
/// A leasable buffer can be used to pass a section of a larger mutable buffer
/// but still get the entire buffer back in a callback.
#[derive(Debug, PartialEq)]
pub struct SubSlice<'a, T> {
    internal: &'a [T],
    active_range: Range<usize>,
}

/// Holder for either a mutable or immutable SubSlice.
///
/// In cases where code needs to support either a mutable or immutable SubSlice,
/// `SubSliceMutImmut` allows the code to store a single type which can
/// represent either option.
pub enum SubSliceMutImmut<'a, T> {
    Immutable(SubSlice<'a, T>),
    Mutable(SubSliceMut<'a, T>),
}

impl<'a, T> SubSliceMutImmut<'a, T> {
    pub fn reset(&mut self) {
        match *self {
            SubSliceMutImmut::Immutable(ref mut buf) => buf.reset(),
            SubSliceMutImmut::Mutable(ref mut buf) => buf.reset(),
        }
    }

    /// Returns the length of the currently accessible portion of the
    /// SubSlice.
    pub fn len(&self) -> usize {
        match *self {
            SubSliceMutImmut::Immutable(ref buf) => buf.len(),
            SubSliceMutImmut::Mutable(ref buf) => buf.len(),
        }
    }

    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        match *self {
            SubSliceMutImmut::Immutable(ref mut buf) => buf.slice(range),
            SubSliceMutImmut::Mutable(ref mut buf) => buf.slice(range),
        }
    }
}

impl<'a, T, I> Index<I> for SubSliceMutImmut<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        match *self {
            SubSliceMutImmut::Immutable(ref buf) => &buf[idx],
            SubSliceMutImmut::Mutable(ref buf) => &buf[idx],
        }
    }
}

impl<'a, T> SubSliceMut<'a, T> {
    /// Create a SubSlice from a passed reference to a raw buffer.
    pub fn new(buffer: &'a mut [T]) -> Self {
        let len = buffer.len();
        SubSliceMut {
            internal: buffer,
            active_range: 0..len,
        }
    }

    fn active_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Retrieve the raw buffer used to create the SubSlice. Consumes the
    /// SubSlice.
    pub fn take(self) -> &'a mut [T] {
        self.internal
    }

    /// Resets the SubSlice to its full size, making the entire buffer
    /// accessible again.
    ///
    /// This should only be called by layer that created the SubSlice, and not
    /// layers that were passed a SubSlice. Layers which are using a SubSlice
    /// should treat the SubSlice as a traditional Rust slice and not consider
    /// any additional size to the underlying buffer.
    ///
    /// Most commonly, this is called once a sliced leasable buffer is returned
    /// through a callback.
    pub fn reset(&mut self) {
        self.active_range = 0..self.internal.len();
    }

    /// Returns the length of the currently accessible portion of the SubSlice.
    pub fn len(&self) -> usize {
        self.active_slice().len()
    }

    /// Returns a pointer to the currently accessible portion of the SubSlice.
    pub fn as_ptr(&self) -> *const T {
        self.active_slice().as_ptr()
    }

    /// Returns a slice of the currently accessible portion of the
    /// LeasableBuffer.
    pub fn as_slice(&mut self) -> &mut [T] {
        &mut self.internal[self.active_range.clone()]
    }

    /// Returns `true` if the LeasableBuffer is sliced internally.
    ///
    /// This is a useful check when switching between code that uses
    /// LeasableBuffers and code that uses traditional slice-and-length. Since
    /// slice-and-length _only_ supports using the entire buffer it is not valid
    /// to try to use a sliced LeasableBuffer.
    pub fn is_sliced(&self) -> bool {
        self.internal.len() != self.len()
    }

    /// Reduces the range of the SubSlice that is accessible.
    ///
    /// This should be called whenever a layer wishes to pass only a portion of
    /// a larger buffer to another layer.
    ///
    /// For example, if the application layer has a 1500 byte packet buffer, but
    /// wishes to send a 250 byte packet, the upper layer should slice the
    /// SubSlice down to its first 250 bytes before passing it down:
    ///
    /// ```rust,ignore
    /// let buffer = static_init!([u8; 1500], [0; 1500]);
    /// let s = SubSliceMut::new(buffer);
    /// s.slice(0..250);
    /// network.send(s);
    /// ```
    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.active_range.end - self.active_range.start,
        };

        let new_start = self.active_range.start + start;
        let new_end = new_start + (end - start);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<'a, T, I> Index<I> for SubSliceMut<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}

impl<'a, T, I> IndexMut<I> for SubSliceMut<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.internal[self.active_range.clone()][idx]
    }
}

impl<'a, T> SubSlice<'a, T> {
    /// Create a SubSlice from a passed reference to a raw buffer.
    pub fn new(buffer: &'a [T]) -> Self {
        let len = buffer.len();
        SubSlice {
            internal: buffer,
            active_range: 0..len,
        }
    }

    fn active_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Retrieve the raw buffer used to create the SubSlice. Consumes the
    /// SubSlice.
    pub fn take(self) -> &'a [T] {
        self.internal
    }

    /// Resets the SubSlice to its full size, making the entire buffer
    /// accessible again.
    ///
    /// This should only be called by layer that created the SubSlice, and not
    /// layers that were passed a SubSlice. Layers which are using a SubSlice
    /// should treat the SubSlice as a traditional Rust slice and not consider
    /// any additional size to the underlying buffer.
    ///
    /// Most commonly, this is called once a sliced leasable buffer is returned
    /// through a callback.
    pub fn reset(&mut self) {
        self.active_range = 0..self.internal.len();
    }

    /// Returns the length of the currently accessible portion of the SubSlice.
    pub fn len(&self) -> usize {
        self.active_slice().len()
    }

    /// Returns a pointer to the currently accessible portion of the SubSlice.
    pub fn as_ptr(&self) -> *const T {
        self.active_slice().as_ptr()
    }

    /// Returns a slice of the currently accessible portion of the
    /// LeasableBuffer.
    pub fn as_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Returns `true` if the LeasableBuffer is sliced internally.
    ///
    /// This is a useful check when switching between code that uses
    /// LeasableBuffers and code that uses traditional slice-and-length. Since
    /// slice-and-length _only_ supports using the entire buffer it is not valid
    /// to try to use a sliced LeasableBuffer.
    pub fn is_sliced(&self) -> bool {
        self.internal.len() != self.len()
    }

    /// Reduces the range of the SubSlice that is accessible.
    ///
    /// This should be called whenever a layer wishes to pass only a portion of
    /// a larger buffer to another layer.
    ///
    /// For example, if the application layer has a 1500 byte packet buffer, but
    /// wishes to send a 250 byte packet, the upper layer should slice the
    /// SubSlice down to its first 250 bytes before passing it down:
    ///
    /// ```rust,ignore
    /// let buffer = unsafe {
    ///    core::slice::from_raw_parts(&_ptr_in_flash as *const u8, 1500)
    /// };
    /// let s = SubSlice::new(buffer);
    /// s.slice(0..250);
    /// network.send(s);
    /// ```
    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.active_range.end - self.active_range.start,
        };

        let new_start = self.active_range.start + start;
        let new_end = new_start + (end - start);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<'a, T, I> Index<I> for SubSlice<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}
