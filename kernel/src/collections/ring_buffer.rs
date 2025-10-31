// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of a ring buffer.

use crate::collections::queue;

pub struct RingBuffer<'a, T: 'a> {
    ring: &'a mut [T],
    head: usize,
    tail: usize,
}

impl<'a, T: Copy> RingBuffer<'a, T> {
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
    fn has_elements(&self) -> bool {
        self.head != self.tail
    }

    fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }

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

    fn empty(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

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

// ===== Flux code ========
// Below is a basic specification that checks:
// 1. There are no division-by-zero errors
// 2. There are no array bounds violations
// 3. Some common-sense invariants (e.g., indexes into the RingBuffer are always valid)
//
// If any of these properties could be violated either in the RingBuffer implementation or how it
// is used anywhere in Tock, Flux would raise an error.
#[cfg(feature = "flux")]
mod flux_specs {
    // Prelude: Here we provide some specifications for methods/types in the core library.
    // This allows Flux to make use of these specs for proving useful things about RingBuffer.
    // Generally, these specs are per-project---if we verified many modules in Tock,
    // there would only be one centralized set of these specs for all of Tock.
    #[flux_rs::extern_spec]
    impl<T> [T] {
        // Need to tell Flux what slice.len() does
        #[flux_rs::sig(fn(&[T][@len]) -> usize[len])]
        fn len(v: &[T]) -> usize;

        // Need to tell Flux what slice.split_at_mut() does (for our tests)
        #[flux_rs::sig(fn(&mut [T][@len], usize[@mid]) -> (&mut [T][mid], &mut [T][len - mid]))]
        fn split_at_mut(v: &mut [T], mid: usize) -> (&mut [T], &mut [T]);
    }

    // Need to tell Flux what an Option<T> is:
    // Here, we refine Option<T> with a bool, denoting whether it is `Some` or `None`
    #[flux_rs::extern_spec]
    #[flux_rs::refined_by(b: bool)]
    enum Option<T> {
        #[variant(Option<T>[false])]
        None,
        #[variant({T} -> Option<T>[true])]
        Some(T),
    }

    // ======= RingBuffer spec ===========
    #[flux::specs {
        mod collections {
            mod ring_buffer {
                // Specify well-formedness for RingBuffer<T>
                //
                // Flux will raise an error if
                //  1) any RingBuffer implementation violates these rules,
                //  2) any Tock code attempts to create a RingBuffer that
                //     violates these rules.
                // At the bottom of this spec, we have included some example
                // test code that raises Flux errors for both of these cases.
                #[refined_by(ring_len: int, hd: int, tl: int)]
                struct RingBuffer<T> {
                    ring: {&mut [T][ring_len] | ring_len > 1},
                    head: {usize[hd] | hd < ring_len},
                    tail: {usize[tl] | tl < ring_len},
                }

                impl RingBuffer<T> {
                    // Example of a function-level spec.
                    //
                    // It has a precondition that provided slice is length > 1,
                    // so every time RingBuffer::new() is called (throughout
                    // Tock), Flux ensures the provided slice has length > 1.
                    //
                    // It also has a postcondition that the output RingBuffer
                    // has a head and tail of zero.  Flux will check the
                    // implementation of `new` to ensure this is true.
                    //
                    // Design note: This contract has the strongest possible
                    // postcondition (head == 0 and tail == 0), but we could
                    // also make the postcondition something "weaker" like
                    // `result.head == result.tail`.  Weaker vs stronger
                    // contracts is a design decision: it is easier to prove
                    // that a weak contract holds in the implementation, but it
                    // lets you prove less in the rest of Tock (e.g., if there
                    // was code that was only safe if head/tail was 0 after it
                    // called new, we could prove its safety only with the
                    // stronger contract).
                    fn new({&mut [T][@ring_len] | ring_len > 1}) -> RingBuffer<T>[ring_len, 0, 0];
                }
            }

            mod queue {
                impl Queue<T> for collections::ring_buffer::RingBuffer<T> {
                  // Simple function-level contract.
                  // See `new` example above for a discussion of contract design.
                    fn empty(self: &mut RingBuffer<T>[@old])
                        ensures self: RingBuffer<T>[old.ring_len, 0, 0];

                    // These specs of the form:
                    //   `(self: RingBuffer) ensures RingBuffer`
                    // are present to compensate for a current technical
                    // limitation of Flux, and should be gone in the near
                    // future.
                    fn enqueue(self: &mut RingBuffer<T>, val: T) -> bool
                        ensures self: RingBuffer<T>;

                    fn push(self: &mut RingBuffer<T>, val: T) -> Option<T>
                        ensures self: RingBuffer<T>;

                    fn dequeue(self: &mut RingBuffer<T>) -> Option<T>
                        ensures self: RingBuffer<T>;

                    fn remove_first_matching<F>(self: &mut RingBuffer<T>, _) -> Option<T>
                        ensures self: RingBuffer<T>;

                    fn retain<F>(self: &mut RingBuffer<T>, _)
                        ensures self: RingBuffer<T>;

                    fn empty(self: &mut RingBuffer<T>[@old])
                        ensures self: RingBuffer<T>[old.ring_len, 0, 0];
                }
            }

            mod list {
                impl Iterator for collections::list::ListIterator<T> {
                    fn next(self: &mut ListIterator<T>) -> Option<&T>
                        ensures self: ListIterator<T>;
                }
            }
        }
    }]
    const _: () = ();

    // ========= Flux tests ==============
    // These functions will fail verification, and demonstrate the sorts of
    // errors that our RingBuffer spec protects us against. Specifically:
    // 1. Bad usage of the RingBuffer API that leads to kernel panics
    // 2. Bad implementation of the RingBuffer API that leads to kernel panics

    use crate::collections::queue::Queue;
    use crate::collections::ring_buffer::RingBuffer;

    // 1. Flux will prevent Tock from using the RingBuffer API incorrectly,
    // leading to panics
    #[allow(dead_code)]
    #[flux_rs::should_fail]
    fn bad_split_into_ringbuffers<'a, T: Copy>(
        buf: &'a mut [T],
        output_len: usize,
    ) -> (RingBuffer<'a, T>, RingBuffer<'a, T>) {
        // If `output_len` is `0` or `buf.len()`, then `output_ringbuf`
        // or `internal_ringbuf` will have a length of `0`.
        // This is bad, because if the `len` of a RingBuffer is `0`,
        // then functions like `is_full` will panic.
        let (output_buf, internal_buf) = buf.split_at_mut(output_len);
        let output_ringbuf = RingBuffer::new(output_buf);
        let internal_ringbuf = RingBuffer::new(internal_buf);
        (output_ringbuf, internal_ringbuf)
    }

    // 2. Here, Flux will prevent RingBuffer implementations from panicing via
    // out-of-bounds memory access or divide by zero.
    #[allow(dead_code)]
    #[flux_rs::should_fail]
    #[flux_rs::spec(fn bad_enqueue(self: &mut RingBuffer<T>, val: T) -> bool
                        ensures self: RingBuffer<T>)]
    fn bad_enqueue<T: Copy>(rb: &mut RingBuffer<T>, val: T) -> bool {
        // This function will not panic as long as rb.tail < rb.ring.len().
        // However, for this to be true, every RingBuffer method needs to
        // maintain this invariant (which is encoded in our RingBuffer spec).
        // This function does not maintain this invariant, as it increases
        // tail without checking ring.len(), and will throw an error.
        if rb.is_full() {
            false
        } else {
            rb.ring[rb.tail] = val;
            // CORRECT: rb.tail = (rb.tail + 1) % rb.ring.len();
            rb.tail = rb.tail + 1;
            true
        }
    }
}
