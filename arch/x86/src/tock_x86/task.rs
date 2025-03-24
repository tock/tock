//! Helpers to program the task state segment.

//! See Intel 3a, Chapter 7


pub use super::segmentation;

use core::arch::asm;


/// Returns the current value of the task register.

///

/// # Safety

/// Needs CPL 0.

pub unsafe fn tr() -> segmentation::SegmentSelector {

    let segment: u16;
    unsafe{
        asm!("str {0:x}",
            out(reg) segment,
            options(att_syntax, nostack, nomem, preserves_flags));
    }
    segmentation::SegmentSelector::from_raw(segment)

}


/// Loads the task register.

///

/// # Safety

/// Needs CPL 0.

pub unsafe fn load_tr(sel: segmentation::SegmentSelector) {
    unsafe{
        asm!("ltr {0:x}",
            in(reg) sel.bits(),
            options(att_syntax, nostack, nomem, preserves_flags));
    }
}