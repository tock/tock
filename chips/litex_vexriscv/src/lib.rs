//! LiteX SoCs based around a VexRiscv CPU

#![no_std]
#![crate_name = "litex_vexriscv"]
#![crate_type = "rlib"]

pub use litex::{event_manager, gpio, led_controller, liteeth, litex_registers, timer, uart};

pub mod chip;
pub mod interrupt_controller;
