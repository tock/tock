//! Tock Register Interface
//!
//!

#![feature(const_fn)]
#![no_std]

#[macro_use]
pub mod macros;

pub mod registers;

#[derive(Clone, Copy)]
pub struct BaseAddress {
    base_address: usize,
}

impl BaseAddress {
    pub const unsafe fn new(base_address: usize) -> BaseAddress {
        BaseAddress { base_address }
    }

    pub const fn offset(&self, offset: usize) -> usize {
        self.base_address + offset
    }
}
