//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core_slice_ext,core_intrinsics,no_std)]
#![no_std]

extern crate support;

pub mod shared;
pub mod ring_buffer;
pub mod queue;

pub use queue::Queue;
pub use ring_buffer::RingBuffer;
pub use shared::Shared;

