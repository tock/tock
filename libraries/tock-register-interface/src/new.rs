use core::marker::PhantomData;

#[macro_export]
macro_rules! peripheral {
    {$($tokens:tt)*} => {
        tock_registers_derive::peripheral!{$crate $($tokens)*}
    }
}

pub mod read {
    use crate::*;
    pub trait At<const REL_ADDR: usize>: ValueAt<REL_ADDR> {}
    pub trait Has<const REL_ADDR: usize>: ValueAt<REL_ADDR> {}
}

pub struct Register<const REL_ADDR: usize, Peripheral, Accessor> {
    pub accessor: Accessor,
    _phantom: PhantomData<Peripheral>,
}

pub trait ValueAt<const REL_ADDR: usize> {
    type Value;
}
