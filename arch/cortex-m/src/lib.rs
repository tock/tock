//! Generic support for all Cortex-M platforms.

#![crate_name = "cortexm"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, lang_items)]
#![no_std]

pub mod nvic;
pub mod scb;
pub mod support;
pub mod syscall;
pub mod systick;
