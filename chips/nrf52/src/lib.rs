#![feature(asm,concat_idents,const_fn,const_cell_new)]
#![no_std]

#![crate_name = "nrf52"]
#![crate_type = "rlib"]
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;
extern crate nrf5x;

#[macro_use]
extern crate bitfield;

extern "C" {
    pub fn init();
}

mod peripheral_registers;

pub mod chip;
pub use chip::NRF52;
pub mod crt1;
pub mod nvmc;
pub mod radio;
pub mod uart;
pub mod uicr;
pub mod spi;
