//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core_intrinsics,const_fn,fixed_size_array)]
#![no_std]

extern crate support;

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

#[macro_export]
macro_rules! interrupt_handler {
    ($name: ident, $nvic: ident $(, $body: expr)*) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        #[allow(unused_imports)]
        pub unsafe extern fn $name() {
            use common::Queue;
            use chip;

            $({
                $body
            })*

            let nvic = nvic::NvicIdx::$nvic;
            nvic::disable(nvic);
            chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nvic);
        }
    }
}

