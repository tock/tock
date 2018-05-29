//! Generic support for all Cortex-M platforms.

#![crate_name = "cortexm"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, naked_functions, lang_items)]
#![no_std]

pub mod support;
