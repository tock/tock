#![feature(const_fn)]
use posix_x86_64::syscall;

pub mod chip;
pub mod console;
pub mod deferred_call_tasks;

pub unsafe fn init() {
    syscall::init();
}
