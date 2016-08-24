// A fixed-size ring buffer
pub trait Queue<T> {
    fn is_empty(&self) -> bool;
    fn enqueue(&mut self, val: T) -> bool;
    fn dequeue(&mut self) -> T;
}

pub const IQ_SIZE: usize = 100;

#[allow(dead_code)]
pub struct InterruptQueue {
    ring: [usize; IQ_SIZE],
    head: usize,
    tail: usize,
}

#[allow(dead_code)]
impl InterruptQueue {
    pub fn new() -> InterruptQueue {
        InterruptQueue {
            head: 0,
            tail: 0,
            ring: [0; IQ_SIZE],
        }
    }
}

#[allow(dead_code)]
impl Queue<usize> for InterruptQueue {
    fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    fn enqueue(&mut self, val: usize) -> bool {
        if ((self.tail + 1) % IQ_SIZE) == self.head {
            // Incrementing tail will overwrite head
            return false;
        } else {
            self.ring[self.tail] = val;
            self.tail = (self.tail + 1) % IQ_SIZE;
            return true;
        }
    }

    fn dequeue(&mut self) -> usize {
        let val: usize = self.ring[self.head];
        if !self.is_empty() {
            self.head = (self.head + 1) % IQ_SIZE;
        }
        val
    }
}
