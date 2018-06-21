//! Implementation of a ring buffer.

use common::queue;

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
            ring: ring,
        }
    }
}

impl<'a, T: Copy> queue::Queue<T> for RingBuffer<'a, T> {
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
        if ((self.tail + 1) % self.ring.len()) == self.head {
            // Incrementing tail will overwrite head
            return false;
        } else {
            self.ring[self.tail] = val;
            self.tail = (self.tail + 1) % self.ring.len();
            return true;
        }
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
}
