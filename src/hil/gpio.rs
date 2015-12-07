pub trait GPIOPin {
    fn enable_output(&self);
    fn set(&self);
    fn clear(&self);
    fn toggle(&self);
    fn read(&self) -> bool;
}
