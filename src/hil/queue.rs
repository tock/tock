// A fixed-size ring buffer
pub trait Queue<T> {
    fn has_elements(&self) -> bool;
    fn is_full(&self) -> bool;
    fn enqueue(&mut self, val: T) -> bool; 
    fn dequeue(&mut self) -> T;
}
