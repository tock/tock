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
