use tock_registers::{peripheral, read};

peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write }
        0x04 => received: u8 { read }
    }
}

fn main() {}
