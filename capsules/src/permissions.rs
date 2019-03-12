//! Basic capsule that is the beginning of a hardware permissions capsule

#![forbid(unsafe_code)]
#![allow(dead_code, unused_imports, unused_variables)]

// // unused but in the book tutorial??
// #![no_std]
// extern crate kernel;
// #[macro_use(debug)]
#[allow(unused_imports)]

use kernel::debug;

pub struct Permissions<> {
}

impl<> Permissions<> {
    pub fn new() -> Permissions<> {
        Permissions {
        }
    }

    pub fn start(&self) {
        debug!("Hello from the preferences capsule!");
    }
}
