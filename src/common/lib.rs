//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core,no_std)]
#![no_std]

extern crate core;
extern crate support;

pub mod shared;
pub mod ring_buffer;
pub mod led;
pub mod adc;
pub mod queue;

pub use queue::Queue;
pub use ring_buffer::RingBuffer;
pub use shared::Shared;

