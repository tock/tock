use super::syscalls::{command, subscribe};

pub fn enable_tmp006() {
    command(2, 0, 0);
}

pub fn subscribe_temperature(f: fn(i16)) {
    subscribe(2, 0, f as usize);
}

