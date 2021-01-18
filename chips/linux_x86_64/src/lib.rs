//! POSIX x64_86 simulation of a chip
//!
//! As this will be used only for debugging, this crate depends on std and
//! has external dependencies

#![feature(const_fn)]
use posix_x86_64::syscall;

pub mod chip;
pub mod console;
pub mod deferred_call_tasks;

pub unsafe fn init() {
    syscall::init();
}
