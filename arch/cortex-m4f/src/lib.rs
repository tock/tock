//! Shared implementations for ARM Cortex-M4F MCUs.

#![crate_name = "cortexm4f"]
#![crate_type = "rlib"]
#![no_std]
#![feature(asm)]

pub mod scb;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m4.
pub use cortexm4::support;

pub use cortexm4::generic_isr;
pub use cortexm4::hard_fault_handler;
pub use cortexm4::mpu;
pub use cortexm4::nvic;
pub use cortexm4::print_cortexm4_state as print_cortexm4f_state;
pub use cortexm4::svc_handler;
pub use cortexm4::syscall;
pub use cortexm4::systick;
pub use cortexm4::systick_handler;
pub use cortexm4::unhandled_interrupt;
