//! Data structures and functions used by 32-bit mode.

// pub mod eflags;
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
    loop {}
}
