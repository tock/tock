//! An example of using the `registers!` macro.

use tock_registers::registers;

registers! {
    foo {
        (0x00 => ro_single: ReadOnly<u32, ()>),
        (0x04 => ro_array: [ReadOnly<u32, ()>; 2]),
        (0x0c => rw_single: ReadWrite<u32, ()>),
        (0x10 => rw_array: [ReadWrite<u32, ()>; 3]),
        (0x1c => wo_single: WriteOnly<u32, ()>),
        (0x20 => wo_array: [WriteOnly<u32, ()>; 4]),
    }
}

fn main() {}
