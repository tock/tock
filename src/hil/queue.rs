// A fixed-size ring buffer
pub trait Queue<T> {
    fn is_empty(&self) -> bool;
    fn enqueue(&mut self, val: T) -> bool; 
    fn dequeue(&mut self) -> T;
}
