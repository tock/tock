#![feature(const_fn)]
use posix::syscall;

pub mod chip;
pub mod console;
pub mod deferred_call_tasks;

pub unsafe fn init() {
    syscall::init();
}
