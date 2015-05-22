pub trait GPIOPin {
    fn enable_output(&mut self);
    fn set(&mut self);
    fn clear(&mut self);
    fn toggle(&mut self);
    fn read(&self) -> bool;
}
