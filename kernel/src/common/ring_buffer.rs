//! Implementation of a ring buffer.

use common::queue;
use core::ptr::read_volatile;

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
        unsafe {
            let head = read_volatile(&self.head);
            let tail = read_volatile(&self.tail);
            head != tail
        }
    }

    fn is_full(&self) -> bool {
        unsafe { read_volatile(&self.head) == ((read_volatile(&self.tail) + 1) % self.ring.len()) }
    }

    fn len(&self) -> usize {
        let head = unsafe { read_volatile(&self.head) };
        let tail = unsafe { read_volatile(&self.tail) };

        if tail > head {
            tail - head
        } else if tail < head {
            (self.ring.len() - head) + tail
        } else {
            // head equals tail, length is zero
            0
        }
    }

    fn enqueue(&mut self, val: T) -> bool {
        unsafe {
            let head = read_volatile(&self.head);
            if ((self.tail + 1) % self.ring.len()) == head {
                // Incrementing tail will overwrite head
                return false;
            } else {
                self.ring[self.tail] = val;
                self.tail = (self.tail + 1) % self.ring.len();
                return true;
            }
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
