#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core,concat_idents,no_std)]
#![no_std]
#[allow(dead_code)]

extern crate core;
extern crate common;
extern crate hil;

pub fn volatile_load<T>(item: &T) -> T {
    unsafe {
        core::intrinsics::volatile_load(item)
    }
}

pub fn volatile_store<T>(item: &mut T, val: T) {
    unsafe {
        core::intrinsics::volatile_store(item, val)
    }
}

macro_rules! volatile {
    ($item:expr) => ({
        ::volatile_load(&$item)
    });

    ($item:ident = $value:expr) => ({
        ::volatile_store(&mut $item, $value)
    });

    ($item:ident |= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) | $value)
    });

    ($item:ident &= $value:expr) => ({
        ::volatile_store(&mut $item, ::volatile_load(&$item) & $value)
    });
}

pub mod ast;
pub mod adc;
pub mod gpio;
pub mod nvic;
pub mod usart;
pub mod pm;
pub mod chip;
pub mod i2c;
