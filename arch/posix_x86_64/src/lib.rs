//! Generic support for POSIX x64 platforms (used for testing).
//!
//! This is used only for testing and running Tock as POSIX user space process.
//! Due to the fact that this crate is used only for testing,
//! it does depend on std.

#![crate_name = "posix_x86_64"]
#![crate_type = "rlib"]
#![feature(llvm_asm)]

use core::fmt::Write;
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use std::ptr;

pub mod nvic;
pub mod support;
pub mod syscall;
pub mod systick;

pub static mut FLASH_POSITION: usize = 0;
pub static mut ORIGINAL_FLASH_POSITION: usize = 0;

pub unsafe fn initialize_flash(flash: &[u8]) -> *mut u8 {
    // allocate flash
    let mem = mmap(
        ptr::null_mut(),
        flash.len(),
        ProtFlags::PROT_READ | ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC,
        MapFlags::MAP_ANON | MapFlags::MAP_PRIVATE,
        -1,
        0,
    )
    .unwrap();
    ORIGINAL_FLASH_POSITION = flash.as_ptr() as usize;
    FLASH_POSITION = mem as *mut u8 as usize;
    // copy flash
    ptr::copy_nonoverlapping(flash.as_ptr() as *const u8, mem as *mut u8, flash.len());
    mem as *mut u8
}

pub unsafe fn print_cpu_state(writer: &mut dyn Write) {
    let _ = writer.write_fmt(format_args!("\r\n---| Fault Status |---\r\n"));
}
