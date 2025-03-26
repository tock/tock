// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Data structures and functions used by 32-bit mode.

pub mod eflags;
pub mod paging;
// pub mod segmentation;
pub mod task;

#[cfg(target_arch = "x86")]
use core::arch::asm;

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn stack_jmp(stack: *mut (), ip: *const ()) -> ! {
    unsafe {
        asm!("movl {0}, %esp; jmp {1}", in(reg) stack, in(reg) ip, options(att_syntax));
    }

    unreachable!()
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn stack_jmp(_stack: *mut (), _ip: *const ()) -> ! {
    unimplemented!()
}
