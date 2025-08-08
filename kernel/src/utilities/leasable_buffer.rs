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
//! ```rust
//! # use kernel::utilities::leasable_buffer::SubSlice;
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

// Author: Amit Levy

use core::cmp::min;
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

impl<'a, T> From<&'a mut [T]> for SubSliceMut<'a, T> {
    fn from(internal: &'a mut [T]) -> Self {
        let active_range = 0..(internal.len());
        Self {
            internal,
            active_range,
        }
    }
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

impl<'a, T> From<&'a [T]> for SubSlice<'a, T> {
    fn from(internal: &'a [T]) -> Self {
        let active_range = 0..(internal.len());
        Self {
            internal,
            active_range,
        }
    }
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

impl<'a, T> From<&'a [T]> for SubSliceMutImmut<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self::Immutable(value.into())
    }
}

impl<'a, T> From<&'a mut [T]> for SubSliceMutImmut<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        Self::Mutable(value.into())
    }
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

    pub fn as_ptr(&self) -> *const T {
        match *self {
            SubSliceMutImmut::Immutable(ref buf) => buf.as_ptr(),
            SubSliceMutImmut::Mutable(ref buf) => buf.as_ptr(),
        }
    }

    pub fn map_mut(&mut self, f: impl Fn(&mut SubSliceMut<'a, T>)) {
        match self {
            SubSliceMutImmut::Immutable(_) => (),
            SubSliceMutImmut::Mutable(subslice) => f(subslice),
        }
    }
}

impl<T, I> Index<I> for SubSliceMutImmut<'_, T>
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
        self.as_slice().len()
    }

    /// Returns a const pointer to the currently accessible portion of the
    /// SubSlice.
    pub fn as_ptr(&self) -> *const T {
        self.as_slice().as_ptr()
    }

    /// Returns a mutable pointer to the currently accessible portion of the
    /// SubSlice.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.as_mut_slice().as_mut_ptr()
    }

    /// Returns the bounds of the currently accessible `SubSliceMut` window,
    /// relative to the underlying, internal buffer.
    ///
    /// This method can be used to re-construct a `SubSliceMut`, retaining its
    /// active window, after `take()`ing its internal buffer. This is useful for
    /// performing nested sub-slicing and interoperability with interfaces that take
    /// a `&[u8]` buffer with an offset and length.
    ///
    /// ## Example
    ///
    /// ```
    /// use kernel::utilities::leasable_buffer::SubSliceMut;
    /// let mut buffer: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    ///
    /// // Construct a SubSliceMut and activate a window:
    /// let mut original_subslicemut = SubSliceMut::new(&mut buffer[..]);
    /// original_subslicemut.slice(3..5);
    /// assert!(original_subslicemut.as_slice() == &[3, 4]);
    ///
    /// // Destruct the SubSliceMut, extracting its underlying buffer, but
    /// // remembering its active range:
    /// let remembered_range = original_subslicemut.active_range();
    /// let extracted_buffer = original_subslicemut.take();
    /// assert!(remembered_range == (3..5));
    ///
    /// // Construct a new SubSliceMut, over the original buffer, with identical
    /// // bounds:
    /// let mut reconstructed_subslicemut = SubSliceMut::new(extracted_buffer);
    /// reconstructed_subslicemut.slice(remembered_range);
    ///
    /// // The new, reconstructed SubSliceMut's window is identical to the
    /// // original one's:
    /// assert!(reconstructed_subslicemut.as_slice() == &[3, 4]);
    /// ```
    pub fn active_range(&self) -> Range<usize> {
        self.active_range.clone()
    }

    /// Returns a slice of the currently accessible portion of the
    /// LeasableBuffer.
    pub fn as_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    /// Returns a mutable slice of the currently accessible portion of
    /// the LeasableBuffer.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
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

        let new_start = min(self.active_range.start + start, self.active_range.end);
        let new_end = min(new_start + (end - start), self.active_range.end);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<T, I> Index<I> for SubSliceMut<'_, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}

impl<T, I> IndexMut<I> for SubSliceMut<'_, T>
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
        self.as_slice().len()
    }

    /// Returns a pointer to the currently accessible portion of the SubSlice.
    pub fn as_ptr(&self) -> *const T {
        self.as_slice().as_ptr()
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

    /// Returns the bounds of the currently accessible `SubSlice` window,
    /// relative to the underlying, internal buffer.
    ///
    /// This method can be used to re-construct a `SubSlice`, retaining its
    /// active window, after `take()`ing its internal buffer. This is useful for
    /// performing nested sub-slicing and interoperability with interfaces that take
    /// a `&[u8]` buffer with an offset and length.
    ///
    /// ## Example
    ///
    /// ```
    /// use kernel::utilities::leasable_buffer::SubSlice;
    /// let mut buffer: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    ///
    /// // Construct a SubSlice and activate a window:
    /// let mut original_subslicemut = SubSlice::new(&mut buffer[..]);
    /// original_subslicemut.slice(3..5);
    /// assert!(original_subslicemut.as_slice() == &[3, 4]);
    ///
    /// // Destruct the SubSlice, extracting its underlying buffer, but
    /// // remembering its active range:
    /// let remembered_range = original_subslicemut.active_range();
    /// let extracted_buffer = original_subslicemut.take();
    /// assert!(remembered_range == (3..5));
    ///
    /// // Construct a new SubSlice, over the original buffer, with identical
    /// // bounds:
    /// let mut reconstructed_subslicemut = SubSlice::new(extracted_buffer);
    /// reconstructed_subslicemut.slice(remembered_range);
    ///
    /// // The new, reconstructed SubSlice's window is identical to the
    /// // original one's:
    /// assert!(reconstructed_subslicemut.as_slice() == &[3, 4]);
    /// ```
    pub fn active_range(&self) -> Range<usize> {
        self.active_range.clone()
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
    ///    core::slice::from_raw_parts(core::ptr::addr_of!(_ptr_in_flash), 1500)
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

        let new_start = min(self.active_range.start + start, self.active_range.end);
        let new_end = min(new_start + (end - start), self.active_range.end);

        self.active_range = Range {
            start: new_start,
            end: new_end,
        };
    }
}

impl<T, I> Index<I> for SubSlice<'_, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}

#[cfg(test)]
mod test {

    use crate::utilities::leasable_buffer::SubSliceMut;
    use crate::utilities::leasable_buffer::SubSliceMutImmut;

    #[test]
    fn subslicemut_create() {
        let mut b: [u8; 100] = [0; 100];
        let s = SubSliceMut::new(&mut b);
        assert_eq!(s.len(), 100);
    }

    #[test]
    fn subslicemut_edit_middle() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(5..10);
        s[0] = 1;
        s.reset();
        assert_eq!(s.as_slice(), [0, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn subslicemut_double_slice() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(5..10);
        s.slice(2..5);
        s[0] = 2;
        s.reset();
        assert_eq!(s.as_slice(), [0, 0, 0, 0, 0, 0, 0, 2, 0, 0]);
    }

    #[test]
    fn subslicemut_double_slice_endopen() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(5..10);
        s.slice(3..);
        s[0] = 3;
        s.reset();
        assert_eq!(s.as_slice(), [0, 0, 0, 0, 0, 0, 0, 0, 3, 0]);
    }

    #[test]
    fn subslicemut_double_slice_beginningopen1() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(5..10);
        s.slice(..3);
        s[0] = 4;
        s.reset();
        assert_eq!(s.as_slice(), [0, 0, 0, 0, 0, 4, 0, 0, 0, 0]);
    }

    #[test]
    fn subslicemut_double_slice_beginningopen2() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(..5);
        s.slice(..3);
        s[0] = 5;
        s.reset();
        assert_eq!(s.as_slice(), [5, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn subslicemut_double_slice_beginningopen3() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(2..5);
        s.slice(..3);
        s[0] = 6;
        s.reset();
        assert_eq!(s.as_slice(), [0, 0, 6, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    #[should_panic]
    fn subslicemut_double_slice_panic1() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(2..5);
        s.slice(..3);
        s[3] = 1;
    }

    #[test]
    #[should_panic]
    fn subslicemut_double_slice_panic2() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(4..);
        s.slice(..3);
        s[3] = 1;
    }

    #[test]
    fn subslicemut_slice_nop() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(0..10);
        assert!(!s.is_sliced());
    }

    #[test]
    fn subslicemut_slice_empty() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(1..1);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn subslicemut_slice_down() {
        let mut b: [u8; 100] = [0; 100];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(0..50);
        assert_eq!(s.len(), 50);
    }

    #[test]
    fn subslicemut_slice_up() {
        let mut b: [u8; 100] = [0; 100];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(0..200);
        assert_eq!(s.len(), 100);
    }

    #[test]
    fn subslicemut_slice_up_ptr() {
        let mut b: [u8; 100] = [0; 100];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(0..200);
        assert_eq!(s.as_slice().len(), 100);
    }

    #[test]
    fn subslicemut_slice_outside() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(20..25);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn subslicemut_slice_beyond() {
        let mut b: [u8; 10] = [0; 10];
        let mut s = SubSliceMut::new(&mut b);
        s.slice(6..15);
        assert_eq!(s.len(), 4);
    }

    fn slice_len1<T>(mut s: SubSliceMutImmut<T>) {
        s.slice(4..8);
        s.slice(0..2);
        assert_eq!(s.len(), 2);
    }

    fn slice_len2<T>(mut s: SubSliceMutImmut<T>) {
        s.slice(4..8);
        s.slice(3..);
        assert_eq!(s.len(), 1);
    }

    fn slice_len3<T>(mut s: SubSliceMutImmut<T>) {
        s.slice(4..8);
        s.slice(..);
        assert_eq!(s.len(), 4);
    }

    fn slice_len4<T>(mut s: SubSliceMutImmut<T>) {
        s.slice(5..);
        s.slice(4..);
        assert_eq!(s.len(), 1);
    }

    fn slice_len5<T>(mut s: SubSliceMutImmut<T>) {
        s.slice(5..);
        s.slice(5..);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn subslicemut_slice_len1() {
        let mut b: [u8; 10] = [0; 10];
        slice_len1(b.as_mut().into())
    }

    #[test]
    fn subslicemut_slice_len2() {
        let mut b: [u8; 10] = [0; 10];
        slice_len2(b.as_mut().into())
    }

    #[test]
    fn subslicemut_slice_len3() {
        let mut b: [u8; 10] = [0; 10];
        slice_len3(b.as_mut().into())
    }

    #[test]
    fn subslicemut_slice_len4() {
        let mut b: [u8; 10] = [0; 10];
        slice_len4(b.as_mut().into())
    }

    #[test]
    fn subslicemut_slice_len5() {
        let mut b: [u8; 10] = [0; 10];
        slice_len5(b.as_mut().into())
    }

    #[test]
    fn subslice_slice_len1() {
        let b: [u8; 10] = [0; 10];
        slice_len1(b.as_ref().into())
    }

    #[test]
    fn subslice_slice_len2() {
        let b: [u8; 10] = [0; 10];
        slice_len2(b.as_ref().into())
    }

    #[test]
    fn subslice_slice_len3() {
        let b: [u8; 10] = [0; 10];
        slice_len3(b.as_ref().into())
    }

    #[test]
    fn subslice_slice_len4() {
        let b: [u8; 10] = [0; 10];
        slice_len4(b.as_ref().into())
    }

    #[test]
    fn subslice_slice_len5() {
        let b: [u8; 10] = [0; 10];
        slice_len5(b.as_ref().into())
    }

    fn slice_contents1(mut s: SubSliceMutImmut<u8>) {
        s.slice(4..8);
        s.slice(0..2);
        assert_eq!(s[0], 4);
        assert_eq!(s[1], 5);
    }

    fn slice_contents2(mut s: SubSliceMutImmut<u8>) {
        s.slice(2..);
        s.slice(5..);
        assert_eq!(s[0], 7);
        assert_eq!(s[1], 8);
        assert_eq!(s[2], 9);
    }

    #[test]
    fn subslicemut_slice_contents1() {
        let mut b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        slice_contents1(b.as_mut().into())
    }

    #[test]
    fn subslicemut_slice_contents2() {
        let mut b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        slice_contents2(b.as_mut().into())
    }

    #[test]
    fn subslice_slice_contents1() {
        let b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        slice_contents1(b.as_ref().into())
    }

    #[test]
    fn subslice_slice_contents2() {
        let b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        slice_contents2(b.as_ref().into())
    }

    fn reset_contents(mut s: SubSliceMutImmut<u8>) {
        s.slice(4..8);
        s.slice(0..2);
        s.reset();
        assert_eq!(s[0], 0);
        assert_eq!(s[1], 1);
        assert_eq!(s[2], 2);
        assert_eq!(s[3], 3);
        assert_eq!(s[4], 4);
        assert_eq!(s[5], 5);
        assert_eq!(s[6], 6);
        assert_eq!(s[7], 7);
        assert_eq!(s[8], 8);
        assert_eq!(s[9], 9);
    }

    #[test]
    fn subslicemut_reset_contents() {
        let mut b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        reset_contents(b.as_mut().into())
    }

    #[test]
    fn subslice_reset_contents() {
        let b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        reset_contents(b.as_ref().into())
    }

    fn reset_panic(mut s: SubSliceMutImmut<u8>) -> u8 {
        s.reset();
        s[s.len()]
    }

    #[test]
    #[should_panic]
    fn subslicemut_reset_panic() {
        let mut b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        reset_panic(b.as_mut().into());
    }

    #[test]
    #[should_panic]
    fn subslice_reset_panic() {
        let b: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        reset_panic(b.as_ref().into());
    }
}
