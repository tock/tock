#[macro_export]
macro_rules! peripheral {
    {$($tokens:tt)*} => {
        tock_registers_derive::peripheral!{$crate $($tokens)*}
    }
}

pub mod read {
    use crate::*;
    pub trait Access<const REL_ADDR: usize>: ValueAt<REL_ADDR> {
    }
}

pub struct Register<const REL_ADDR: usize, Accessor> {
    pub accessor: Accessor,
}

pub trait ValueAt<const REL_ADDR: usize> {
    type Value;
}
