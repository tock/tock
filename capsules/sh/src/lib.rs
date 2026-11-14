// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Just a basic semihosting interface for debugging
// Not intended to be merged to master

#![no_std]

#[macro_export]
macro_rules! p {
    ($($arg:tt)*) => {
        semihosting::println!("\x1b[0;37m{}\x1b[0m", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! s {
    ($($arg:tt)*) => {
        semihosting::print!("\x1b[0;37m{}\x1b[0m", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! v {
    ($($arg:tt)*) => {
        semihosting::println!("\x1b[0;35m{}\x1b[0m", format_args!($($arg)*));
    };
}

pub fn reg(name: &str, addr: u32) {
    unsafe {
        let value = *(addr as *const u32);
        p!("{name} [0x{addr:08x}] = 0x{value:08x}");
    }
}
