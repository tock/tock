//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core_intrinsics,const_fn,fixed_size_array)]
#![no_std]

pub mod ring_buffer;
pub mod queue;
pub mod utils;
pub mod take_cell;
pub mod volatile_cell;
pub mod list;
pub mod math;

pub use queue::Queue;
pub use ring_buffer::RingBuffer;
pub use volatile_cell::VolatileCell;
pub use list::{List, ListLink, ListNode};

