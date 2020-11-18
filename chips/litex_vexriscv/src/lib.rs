//! LiteX SoCs based around a VexRiscv CPU

#![feature(asm, llvm_asm, const_fn, const_panic, naked_functions)]
#![no_std]
#![crate_name = "litex_vexriscv"]
#![crate_type = "rlib"]

pub use litex::{event_manager, led_controller, liteeth, litex_registers, timer, uart};

pub mod chip;
pub mod plic;
