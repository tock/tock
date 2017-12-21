use core::cell::Cell;

#[derive(Debug)]
pub struct CircularBuffer<T>
    where T: Copy
{
    bag: Cell<[Option<T>; 16]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    max_size: usize,
}


impl<T> CircularBuffer<T>
    where T: Copy
{
    pub fn new() -> Self {
        CircularBuffer {
            bag: Cell::new([None; 16]),
            head: Cell::new(0),
            tail: Cell::new(0),
            max_size: 16,
        }
    }

    pub fn enqueue(&self, x: Option<T>) {
        if self.is_full() {
            return;
        }

        let mut buf = self.bag.get();
        let tail = self.tail.get();
        buf[tail] = x;
        self.tail.set((self.tail.get() + 1) % self.max_size);
        self.bag.set(buf);
    }

    pub fn dequeue(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {

            let mut buf = self.bag.get();
            let mut head = self.head.get();

            let pop = buf[head];
            buf[head] = None;

            head = (head + 1) % self.max_size;

            self.head.set(head);
            self.bag.set(buf);
            pop
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.get() == self.tail.get()
    }


    pub fn is_full(&self) -> bool {
        (self.tail.get() + 1) % self.max_size == self.head.get()
    }
}
