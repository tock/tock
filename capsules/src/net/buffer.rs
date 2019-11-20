//! Defines a Buffer type which can be used to pass a section of a larger
//! buffer but still get the entire buffer back in a callback
//!
//! Author: Amit Levy

use core::ops::{Bound, Range, RangeBounds};
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

pub struct Buffer<'a, T> {
    internal: &'a mut [T],
    active_range: Range<usize>,
}

impl<'a, T> Buffer<'a, T> {
    pub fn new(buffer: &'a mut [T]) -> Self {
        let len = buffer.len();
        Buffer {
            internal: buffer,
            active_range: 0..len,
        }
    }

    pub fn take(self) -> &'a mut [T] {
        self.internal
    }

    pub fn reset(&mut self) {
        self.active_range = 0..self.internal.len();
    }

    fn active_slice(&self) -> &[T] {
        &self.internal[self.active_range.clone()]
    }

    pub fn len(&self) -> usize {
        self.active_slice().len()
    }

    pub fn as_ptr(&self) -> *const T {
        self.active_slice().as_ptr()
    }

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

impl<'a, T, I> Index<I> for Buffer<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    fn index(&self, idx: I) -> &Self::Output {
        &self.internal[self.active_range.clone()][idx]
    }
}

impl<'a, T, I> IndexMut<I> for Buffer<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.internal[self.active_range.clone()][idx]
    }
}

/*
#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn create_and_recover() {
        let mut internal = ['a', 'b', 'c', 'd'];
        let original_base_addr = internal.as_ptr();

        let mut buffer = Buffer::new(&mut internal);
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_ptr(), original_base_addr);

        assert_eq!(buffer[2], 'c');

        buffer[3] = 'z';

        let result = buffer.take();
        assert_eq!(original_base_addr, result.as_ptr());

        assert_eq!(result[3], 'z');
    }

    #[test]
    fn slice_and_reset() {
        let mut internal = ['a', 'b', 'c', 'd'];
        let original_base_addr = internal.as_ptr();

        let mut buffer = Buffer::new(&mut internal);

        buffer.slice(1..3);

        assert_eq!(buffer.as_ptr(), unsafe { original_base_addr.offset(1) });
        assert_eq!(buffer.len(), 2);
        assert_eq!((buffer[0], buffer[1]), ('b', 'c'));

        buffer.reset();

        assert_eq!(buffer.as_ptr(), original_base_addr);
        assert_eq!(buffer.len(), 4);
        assert_eq!((buffer[0], buffer[1]), ('a', 'b'));
    }

    #[test]
    fn double_slice_and_reset() {
        let mut internal = ['0', 'a', 'b', 'c', 'd', 'e', 'f'];
        let original_base_addr = internal.as_ptr();

        let mut buffer = Buffer::new(&mut internal);

        buffer.slice(1..5);
        buffer.slice(1..3);

        assert_eq!(buffer.as_ptr(), unsafe { original_base_addr.offset(2) });
        assert_eq!(buffer.len(), 2);
        assert_eq!((buffer[0], buffer[1]), ('b', 'c'));

        buffer.reset();

        assert_eq!(buffer.as_ptr(), original_base_addr);
        assert_eq!(buffer.len(), 7);
        assert_eq!((buffer[0], buffer[1]), ('0', 'a'));
    }
} */
