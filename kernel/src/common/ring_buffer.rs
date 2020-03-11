//! Implementation of a ring buffer.

use crate::common::queue;

pub struct RingBuffer<'a, T: 'a> {
    ring: &'a mut [T],
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<'a, T> {
    pub fn new(ring: &'a mut [T]) -> RingBuffer<'a, T> {
        RingBuffer {
            head: 0,
            tail: 0,
            ring: ring,
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

impl<T: Copy> queue::Queue<T> for RingBuffer<'a, T> {
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
