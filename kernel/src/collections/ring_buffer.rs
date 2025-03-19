// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of a ring buffer.

use crate::collections::queue;

#[flux_rs::refined_by(ring_len: int, hd: int, tl: int)]
#[flux_rs::invariant(ring_len > 1)]
pub struct RingBuffer<'a, T: 'a> {
    #[field({&mut [T][ring_len] | ring_len > 1})]
    ring: &'a mut [T],
    #[field({usize[hd] | hd < ring_len})]
    head: usize,
    #[field({usize[tl] | tl < ring_len})]
    tail: usize,
}

flux_rs::defs! {
    fn next_index(x:int, ring_len: int) -> int { (x + 1) % ring_len }
    fn empty(rb: RingBuffer) -> bool { rb.hd == rb.tl }
    fn full(rb: RingBuffer) -> bool { rb.hd == next_index(rb.tl, rb.ring_len) }
    fn next_hd(rb: RingBuffer) -> int { next_index(rb.hd, rb.ring_len) }
    fn next_tl(rb: RingBuffer) -> int { next_index(rb.tl, rb.ring_len) }
}

impl<'a, T: Copy> RingBuffer<'a, T> {
    #[flux_rs::sig(fn({&mut [T][@ring_len] | ring_len > 1}) -> RingBuffer<T>[ring_len, 0, 0])]
    pub fn new(ring: &'a mut [T]) -> RingBuffer<'a, T> {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Returns the number of elements that can be enqueued until the ring buffer is full.
    pub fn available_len(&self) -> usize {
        // The maximum capacity of the queue is ring.len - 1, because head == tail for the empty
        // queue.
        self.ring.len().saturating_sub(1 + queue::Queue::len(self))
    }

    /// Returns up to 2 slices that together form the contents of the ring buffer.
    ///
    /// Returns:
    /// - `(None, None)` if the buffer is empty.
    /// - `(Some(slice), None)` if the head is before the tail (therefore all the contents is
    /// contiguous).
    /// - `(Some(left), Some(right))` if the head is after the tail. In that case, the logical
    /// contents of the buffer is `[left, right].concat()` (although physically the "left" slice is
    /// stored after the "right" slice).
    pub fn as_slices(&'a self) -> (Option<&'a [T]>, Option<&'a [T]>) {
        if self.head < self.tail {
            (Some(&self.ring[self.head..self.tail]), None)
        } else if self.head > self.tail {
            let (left, right) = self.ring.split_at(self.head);
            (
                Some(right),
                if self.tail == 0 {
                    None
                } else {
                    Some(&left[..self.tail])
                },
            )
        } else {
            (None, None)
        }
    }
}

impl<T: Copy> queue::Queue<T> for RingBuffer<'_, T> {
    #[flux_rs::sig(fn(&RingBuffer<T>[@rb]) -> bool[!empty(rb)]) ]
    fn has_elements(&self) -> bool {
        self.head != self.tail
    }

    #[flux_rs::sig(fn(&RingBuffer<T>[@rb]) -> bool[full(rb)]) ]
    fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }

    #[flux_rs::sig(fn(&RingBuffer<T>[@rb]) -> usize{r: r < rb.ring_len}) ]
    fn len(&self) -> usize {
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            // head equals tail, length is zero
            0
        }
    }

    #[flux_rs::sig(
        fn(self: &strg RingBuffer<T>[@old], _) -> bool 
            ensures self: RingBuffer<T>{ new: 
                // either we're full and don't update
                (full(old) => new.tl == old.tl && new.hd == old.hd)
                &&
                // or tail is incremented
                (!full(old) => new.tl == next_tl(old) && new.hd == old.hd)
            }
    )]
    fn enqueue(&mut self, val: T) -> bool {
        if self.is_full() {
            // Incrementing tail will overwrite head
            false
        } else {
            self.ring[self.tail] = val;
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    #[flux_rs::sig(
        fn(self: &strg Self[@old], _) -> Option<T> 
            ensures self: Self{ new: 
                // the buffer is full so we dequeue and then enqueue 
                (full(old) => (new.hd == next_hd(old) && new.tl == next_tl(old)))
                &&
                // or we have space so we just enqueue
                (!full(old) => (new.tl == next_tl(old) && new.hd == old.hd))
            }
    )]
    fn push(&mut self, val: T) -> Option<T> {
        let result = if self.is_full() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        };

        self.ring[self.tail] = val;
        self.tail = (self.tail + 1) % self.ring.len();
        result
    }

    #[flux_rs::sig(
        fn(self: &strg RingBuffer<T>[@old]) -> Option<T> 
            ensures self: RingBuffer<T>{ new: 
                (empty(old) => (new == old))
                &&
                (!empty(old) => new.hd == next_hd(old))
             }
    )]
    fn dequeue(&mut self) -> Option<T> {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Removes the first element for which the provided closure returns `true`.
    ///
    /// This walks the ring buffer and, upon finding a matching element, removes
    /// it. It then shifts all subsequent elements forward (filling the hole
    /// created by removing the element).
    ///
    /// If an element was removed, this function returns it as `Some(elem)`.
    #[flux_rs::sig(
        fn(self: &strg Self, _) -> Option<_> ensures self: Self
    )]
    fn remove_first_matching<F>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        let len = self.ring.len();
        let mut slot = self.head;
        while slot != self.tail {
            if f(&self.ring[slot]) {
                // This is the desired element, remove it and return it
                let val = self.ring[slot];

                let mut next_slot = (slot + 1) % len;
                // Move everything past this element forward in the ring
                while next_slot != self.tail {
                    self.ring[slot] = self.ring[next_slot];
                    slot = next_slot;
                    next_slot = (next_slot + 1) % len;
                }
                self.tail = slot;
                return Some(val);
            }
            slot = (slot + 1) % len;
        }
        None
    }

    #[flux_rs::sig(
        fn(self: &strg RingBuffer<T>[@old]) ensures self: RingBuffer<T>[old.ring_len, 0, 0]
    )]
    fn empty(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    #[flux_rs::sig(
        fn(self: &strg RingBuffer<T>, _) ensures self: RingBuffer<T>
    )]
    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.ring.len();
        // Index over the elements before the retain operation.
        let mut src = self.head;
        // Index over the retained elements.
        let mut dst = self.head;

        while src != self.tail {
            if f(&self.ring[src]) {
                // When the predicate is true, move the current element to the
                // destination if needed, and increment the destination index.
                if src != dst {
                    self.ring[dst] = self.ring[src];
                }
                dst = (dst + 1) % len;
            }
            src = (src + 1) % len;
        }

        self.tail = dst;
    }
}

#[cfg(test)]
mod test {
    use super::super::queue::Queue;
    use super::RingBuffer;

    #[test]
    fn test_enqueue_dequeue() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = RingBuffer::new(&mut ring);

        for _ in 0..2 * LEN {
            assert!(buf.enqueue(42));
            assert_eq!(buf.len(), 1);
            assert!(buf.has_elements());

            assert_eq!(buf.dequeue(), Some(42));
            assert_eq!(buf.len(), 0);
            assert!(!buf.has_elements());
        }
    }

    #[test]
    fn test_push() {
        const LEN: usize = 10;
        const MAX: usize = 100;
        let mut ring = [0; LEN + 1];
        let mut buf = RingBuffer::new(&mut ring);

        for i in 0..LEN {
            assert_eq!(buf.len(), i);
            assert!(!buf.is_full());
            assert_eq!(buf.push(i), None);
            assert!(buf.has_elements());
        }

        for i in LEN..MAX {
            assert!(buf.is_full());
            assert_eq!(buf.push(i), Some(i - LEN));
        }

        for i in 0..LEN {
            assert!(buf.has_elements());
            assert_eq!(buf.len(), LEN - i);
            assert_eq!(buf.dequeue(), Some(MAX - LEN + i));
            assert!(!buf.is_full());
        }

        assert!(!buf.has_elements());
    }

    // Enqueue integers 1 <= n < len, checking that it succeeds and that the
    // queue is full at the end.
    // See std::iota in C++.
    fn enqueue_iota(buf: &mut RingBuffer<usize>, len: usize) {
        for i in 1..len {
            assert!(!buf.is_full());
            assert!(buf.enqueue(i));
            assert!(buf.has_elements());
            assert_eq!(buf.len(), i);
        }

        assert!(buf.is_full());
        assert!(!buf.enqueue(0));
        assert!(buf.has_elements());
    }

    // Dequeue all elements, expecting integers 1 <= n < len, checking that the
    // queue is empty at the end.
    // See std::iota in C++.
    fn dequeue_iota(buf: &mut RingBuffer<usize>, len: usize) {
        for i in 1..len {
            assert!(buf.has_elements());
            assert_eq!(buf.len(), len - i);
            assert_eq!(buf.dequeue(), Some(i));
            assert!(!buf.is_full());
        }

        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);
    }

    // Move the head by `count` elements, by enqueueing/dequeueing `count`
    // times an element.
    // This assumes an empty queue at the beginning, and yields an empty queue.
    fn move_head(buf: &mut RingBuffer<usize>, count: usize) {
        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);

        for _ in 0..count {
            assert!(buf.enqueue(0));
            assert_eq!(buf.dequeue(), Some(0));
        }

        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_fill_once() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = RingBuffer::new(&mut ring);

        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);

        enqueue_iota(&mut buf, LEN);
        dequeue_iota(&mut buf, LEN);
    }

    #[test]
    fn test_refill() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = RingBuffer::new(&mut ring);

        for _ in 0..10 {
            enqueue_iota(&mut buf, LEN);
            dequeue_iota(&mut buf, LEN);
        }
    }

    #[test]
    fn test_retain() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = RingBuffer::new(&mut ring);

        move_head(&mut buf, LEN - 2);
        enqueue_iota(&mut buf, LEN);

        buf.retain(|x| x % 2 == 1);
        assert_eq!(buf.len(), LEN / 2);

        assert_eq!(buf.dequeue(), Some(1));
        assert_eq!(buf.dequeue(), Some(3));
        assert_eq!(buf.dequeue(), Some(5));
        assert_eq!(buf.dequeue(), Some(7));
        assert_eq!(buf.dequeue(), Some(9));
        assert_eq!(buf.dequeue(), None);
    }
}
