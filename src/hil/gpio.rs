pub enum InputMode {
    PullUp,
    PullDown
}

pub enum InterruptMode {
    Change,
    RisingEdge,
    FallingEdge
}

pub trait GPIOPin {
    fn enable_output(&self);
    fn enable_input(&self, mode: InputMode);
    fn disable(&self);
    fn set(&self);
    fn clear(&self);
    fn toggle(&self);
    fn read(&self) -> bool;
    fn enable_interrupt(&self, identifier: usize, mode: InterruptMode);
}

pub trait Client {
    fn fired(&self, identifier: usize);
}

