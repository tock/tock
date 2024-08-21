use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

use crate::utilities::cells::TakeCell;
use crate::collections::sync_queue;

pub struct AtomicRingBuffer<'a, T: 'a> {
    ring: UnsafeCell<&'a mut [T]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl<'a, T> Send for AtomicRingBuffer<'a, T> {}
unsafe impl<'a, T> Sync for AtomicRingBuffer<'a, T> {}

enum Failure {
    Full,
    Busy,
    Empty,
}


impl<'a, T> AtomicRingBuffer<'a, T> {
    pub fn new(ring: &'a mut [T]) -> AtomicRingBuffer<'a, T> {
        AtomicRingBuffer {
            ring: UnsafeCell::new(ring),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    // Safety: length of the backing buffer is immutable
    fn capacity(&self) -> usize {
        unsafe { (&*self.ring.get()).len() }
    }

    fn is_available(&self) -> Option<usize> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (head != (tail + 1) % self.capacity())
            .then(|| tail)
    }

    fn try_reserve(&self) -> Result<usize, Failure> {
        if let Some(next_available) = self.is_available() {
            if let Ok(_) =
                self.tail.compare_exchange(next_available, (next_available + 1) % self.capacity(), Ordering::Release, Ordering::Relaxed) {
                Ok(next_available)
            } else {
                Err(Failure::Busy)
            }
        } else {
            Err(Failure::Full)
        }
    }

    fn reserve(&self) -> Result<usize, Failure> {
        loop {
            match self.try_reserve() {
                ok @ Ok(_) => break ok,
                err @ Err(Failure::Full) => break err,
                _ => (),
            }
        }
    }

    fn peek(&self) -> Option<usize> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (head != tail).then(|| head)
    }
}

impl<'a, T: Copy> AtomicRingBuffer<'a, T> {
    fn try_release(&self) -> Result<T, Failure> {
        if let Some(next) = self.peek() {
            let value = unsafe {
                // must be accessed before compare_exchange
                (&mut *self.ring.get())[next]
            };
            if let Ok(_) = self.head.compare_exchange(next, (next + 1) % self.capacity(), Ordering::Release, Ordering::Relaxed) {
                Ok(value)
            } else {
                Err(Failure::Busy)
            }
        } else {
            Err(Failure::Empty)
        }
    }

    fn release(&self) -> Result<T, Failure> {
        loop {
            match self.try_release() {
                ok @ Ok(_) => break ok,
                err @ Err(Failure::Empty) => break err,
                _ => (),
            }
        }
    }
}

impl<'a, T: Copy> sync_queue::SyncQueue<T> for AtomicRingBuffer<'a, T> {
    fn has_elements(&self) -> bool {
        self.peek().is_some()
    }

    fn is_full(&self) -> bool {
        self.is_available().is_none()
    }

    fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if tail > head {
            tail - head
        } else if tail < head {
            (self.capacity() - head) + tail
        } else {
            // head equals tail, length is zero
            0
        }
    }

    fn enqueue(&self, val: T) -> bool {
        if let Ok(next_available) = self.reserve() {
            unsafe {
                (&mut *self.ring.get())[next_available] = val;
            }
            true
        } else {
            false
        }
    }

    fn push(&self, val: T) -> Option<T> {
        todo!()
    }

    fn dequeue(&self) -> Option<T> {
        self.release().ok()
    }

    fn empty(&self) {
        let mut head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Acquire);
        while let Err(new_tail) = self.tail.compare_exchange(tail, head, Ordering::Release, Ordering::Acquire) {
            head = self.head.load(Ordering::Acquire);
            tail = new_tail;
        }
    }

    fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool {
        todo!()
    }
}



#[cfg(test)]
mod test {
    use super::super::sync_queue::SyncQueue;
    use super::AtomicRingBuffer;

    #[test]
    fn test_enqueue_dequeue() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = AtomicRingBuffer::new(&mut ring);

        for _ in 0..2 * LEN {
            assert!(buf.enqueue(42));
            assert_eq!(buf.len(), 1);
            assert!(buf.has_elements());

            assert_eq!(buf.dequeue(), Some(42));
            assert_eq!(buf.len(), 0);
            assert!(!buf.has_elements());
        }
    }

    // #[test]
    fn test_push() {
        const LEN: usize = 10;
        const MAX: usize = 100;
        let mut ring = [0; LEN + 1];
        let mut buf = AtomicRingBuffer::new(&mut ring);

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
    fn enqueue_iota(buf: &mut AtomicRingBuffer<usize>, len: usize) {
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
    fn dequeue_iota(buf: &mut AtomicRingBuffer<usize>, len: usize) {
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
    fn move_head(buf: &mut AtomicRingBuffer<usize>, count: usize) {
        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);

        for _ in 0..count {
            assert!(buf.enqueue(0));
            assert_eq!(buf.dequeue(), Some(0));
        }

        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);
    }

    // #[test]
    fn test_fill_once() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = AtomicRingBuffer::new(&mut ring);

        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);

        enqueue_iota(&mut buf, LEN);
        dequeue_iota(&mut buf, LEN);
    }

    // #[test]
    fn test_refill() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = AtomicRingBuffer::new(&mut ring);

        for _ in 0..10 {
            enqueue_iota(&mut buf, LEN);
            dequeue_iota(&mut buf, LEN);
        }
    }

    // #[test]
    fn test_retain() {
        const LEN: usize = 10;
        let mut ring = [0; LEN];
        let mut buf = AtomicRingBuffer::new(&mut ring);

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
