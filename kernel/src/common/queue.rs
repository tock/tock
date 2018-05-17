//! Interface for queue structure.

pub trait Queue<T> {
    fn has_elements(&self) -> bool;
    fn is_full(&self) -> bool;
    fn len(&self) -> usize;
    fn enqueue(&mut self, val: T) -> bool;
    fn dequeue(&mut self) -> Option<T>;

    /// Remove all elements from the ring buffer.
    fn empty(&mut self);
}
