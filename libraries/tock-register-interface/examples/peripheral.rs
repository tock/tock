use tock_registers::{peripheral, read};

peripheral! {
    foo {
        0x00 => ctrl: u32 { read<> }
    }
}

fn main() {}
