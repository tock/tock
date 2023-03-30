// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Defines a LeasableBuffer type which can be used to pass a section of a larger
//! buffer but still get the entire buffer back in a callback
//!
//! Author: Amit Levy
//!
//! Usage
//! -----
//!
//! `slice()` is used to set the portion of the `LeasableBuffer` that is accessbile.
//! `reset()` makes the entire `LeasableBuffer` accessible again.
//!  Typically, `slice()` will be called prior to passing the buffer down to lower layers,
//!  and `reset()` will be called once the `LeasableBuffer` is returned via a callback
//!
//!  ```rust
//! # use kernel::utilities::leasable_buffer::LeasableBuffer;
//!
//! let mut internal = ['a', 'b', 'c', 'd'];
//! let original_base_addr = internal.as_ptr();
//!
//! let mut buffer = LeasableBuffer::new(&mut internal);
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

use core::ops::{Bound, Range, RangeBounds};
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

/// Leasable buffer which can be used to pass a section of a larger mutable
/// buffer but still get the entire buffer back in a callback
#[derive(Debug, PartialEq)]
pub struct LeasableMutableBuffer<'a, T> {
    internal: &'a mut [T],
    active_range: Range<usize>,
}

/// Leasable Buffer which can be used to pass a section of a larger buffer but still
/// get the entire buffer back in a callback
#[derive(Debug, PartialEq)]
pub struct LeasableBuffer<'a, T> {
    internal: &'a [T],
    active_range: Range<usize>,
}

pub enum LeasableBufferDynamic<'a, T> {
    Immutable(LeasableBuffer<'a, T>),
    Mutable(LeasableMutableBuffer<'a, T>),
}

impl<'a, T> LeasableBufferDynamic<'a, T> {
    pub fn reset(&mut self) {
        match *self {
            LeasableBufferDynamic::Immutable(ref mut buf) => buf.reset(),
            LeasableBufferDynamic::Mutable(ref mut buf) => buf.reset(),
        }
    }

    /// Returns the length of the currently accessible portion of the LeasableBuffer
    pub fn len(&self) -> usize {
        match *self {
            LeasableBufferDynamic::Immutable(ref buf) => buf.len(),
            LeasableBufferDynamic::Mutable(ref buf) => buf.len(),
        }
    }

    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        match *self {
            LeasableBufferDynamic::Immutable(ref mut buf) => buf.slice(range),
            LeasableBufferDynamic::Mutable(ref mut buf) => buf.slice(range),
        }
    }
}

impl<'a, T, I> Index<I> for LeasableBufferDynamic<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        match *self {
            LeasableBufferDynamic::Immutable(ref buf) => &buf[idx],
            LeasableBufferDynamic::Mutable(ref buf) => &buf[idx],
        }
    }
}

impl<'a, T> LeasableMutableBuffer<'a, T> {
    /// Create a leasable buffer from a passed reference to a raw buffer
    pub fn new(buffer: &'a mut [T]) -> Self {
        let len = buffer.len();
        LeasableMutableBuffer {
            internal: buffer,
            active_range: 0..len,
        }
    }

    /// Retrieve the raw buffer used to create the LeasableBuffer. Consumes
    /// the LeasableBuffer.
    pub fn take(self) -> &'a mut [T] {
        self.internal
    }

    /// Resets the LeasableBuffer to its full size, making the entire buffer
    /// accessible again. Typically this would be called once a sliced
    /// LeasableBuffer is returned through a callback
    pub fn reset(&mut self) {
        self.active_range = 0..self.internal.len();
    }

    fn active_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Returns the length of the currently accessible portion of the LeasableBuffer
    pub fn len(&self) -> usize {
        self.active_slice().len()
    }

    /// Returns a pointer to the currently accessible portion of the LeasableBuffer
    pub fn as_ptr(&self) -> *const T {
        self.active_slice().as_ptr()
    }

    /// Reduces the range of the LeasableBuffer that is accessible. This should be called
    /// whenever an upper layer wishes to pass only a portion of a larger buffer down to
    /// a lower layer. For example: if the application layer has a 1500 byte packet
    /// buffer, but wishes to send a 250 byte packet, the upper layer should slice the
    /// LeasableBuffer down to its first 250 bytes before passing it down.
    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.internal.len(),
        };

        let new_start = self.active_range.start + start;
        let new_end = new_start + (end - start);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<'a, T, I> Index<I> for LeasableMutableBuffer<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}

impl<'a, T, I> IndexMut<I> for LeasableMutableBuffer<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.internal[self.active_range.clone()][idx]
    }
}

impl<'a, T> LeasableBuffer<'a, T> {
    /// Create a leasable buffer from a passed reference to a raw buffer
    pub fn new(buffer: &'a [T]) -> Self {
        let len = buffer.len();
        LeasableBuffer {
            internal: buffer,
            active_range: 0..len,
        }
    }

    /// Retrieve the raw buffer used to create the LeasableBuffer. Consumes
    /// the LeasableBuffer.
    pub fn take(self) -> &'a [T] {
        self.internal
    }

    /// Resets the LeasableBuffer to its full size, making the entire buffer
    /// accessible again. Typically this would be called once a sliced
    /// LeasableBuffer is returned through a callback
    pub fn reset(&mut self) {
        self.active_range = 0..self.internal.len();
    }

    fn active_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Returns the length of the currently accessible portion of the LeasableBuffer
    pub fn len(&self) -> usize {
        self.active_slice().len()
    }

    /// Returns a pointer to the currently accessible portion of the LeasableBuffer
    pub fn as_ptr(&self) -> *const T {
        self.active_slice().as_ptr()
    }

    /// Reduces the range of the LeasableBuffer that is accessible. This should be called
    /// whenever an upper layer wishes to pass only a portion of a larger buffer down to
    /// a lower layer. For example: if the application layer has a 1500 byte packet
    /// buffer, but wishes to send a 250 byte packet, the upper layer should slice the
    /// LeasableBuffer down to its first 250 bytes before passing it down.
    pub fn slice<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => self.internal.len(),
        };

        let new_start = self.active_range.start + start;
        let new_end = new_start + (end - start);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<'a, T, I> Index<I> for LeasableBuffer<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}
