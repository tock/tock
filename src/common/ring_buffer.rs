use core::intrinsics::volatile_load;
use queue;

pub struct RingBuffer<'a, T: 'a> {
    ring: &'a mut [T],
    head: usize,
    tail: usize
}

impl<'a, T: Copy> RingBuffer<'a, T> {
    pub fn new(ring: &'a mut [T]) -> RingBuffer<'a, T> {
        RingBuffer {
            head: 0,
            tail: 0,
            ring: ring
        }
    }
}

impl<'a, T: Copy> queue::Queue<T> for RingBuffer<'a, T> {
    fn has_elements(&self) -> bool {
        unsafe {
            let head = volatile_load(&self.head);
            let tail = volatile_load(&self.tail);
            head != tail
        }
    }

    fn is_full(&self) -> bool {
        unsafe {
            volatile_load(&self.head) == ((volatile_load(&self.tail) + 1) % self.ring.len())
        }
    }

    fn enqueue(&mut self, val: T) -> bool {
        unsafe {
            let head = volatile_load(&self.head);
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
}
