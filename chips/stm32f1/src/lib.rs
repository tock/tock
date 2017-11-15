#![feature(asm,concat_idents,const_fn,const_cell_new,core_intrinsics)]
#![no_std]

#![crate_name = "stm32f1"]
#![crate_type = "rlib"]

extern crate cortexm3;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate stm32;

extern "C" {
    pub fn init();
}

pub mod chip;
pub mod crt1;
