#![crate_name = "sam4l"]
#![crate_type = "rlib"]
#![feature(asm,core_intrinsics,core_slice_ext,concat_idents,no_std,const_fn)]
#![no_std]

extern crate common;
extern crate hil;
extern crate process;

mod helpers;

pub mod chip;
pub mod ast;
pub mod dma;
pub mod i2c;
pub mod spi;
pub mod nvic;
pub mod pm;
pub mod gpio;
pub mod usart;
pub mod scif;
pub mod adc;
