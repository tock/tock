//! An example of using the `register_structs!` macro.

use tock_registers::register_structs;

register_structs! {
    Foo {
        (0x00 => ro_single: ReadOnly<u32, ()>),
        (0x04 => ro_array: [ReadOnly<u32, ()>; 2]),
        (0x0c => rw_single: ReadWrite<u32, ()>),
        (0x10 => rw_array: [ReadWrite<u32, ()>; 3]),
        (0x1c => wo_single: WriteOnly<u32, ()>),
        //(0x20 => _padding),
        (0x24 => wo_array: [WriteOnly<u32, ()>; 4]),
        //(0x34 => @END),
    }
}

fn main() {}
