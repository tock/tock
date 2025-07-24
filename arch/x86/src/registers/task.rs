// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Helpers to program the task state segment.
//! See Intel 3a, Chapter 7

pub use super::segmentation;

#[cfg(target_arch = "x86")]
use core::arch::asm;

/// Returns the current value of the task register.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn tr() -> segmentation::SegmentSelector {
    let segment: u16;
    unsafe {
        asm!("str {0:x}",
            out(reg) segment,
            options(att_syntax, nostack, nomem, preserves_flags));
    }
    segmentation::SegmentSelector::from_raw(segment)
}

/// Loads the task register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_tr(sel: segmentation::SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}",
            in(reg) sel.bits(),
            options(att_syntax, nostack, nomem, preserves_flags));
    }
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn tr() -> segmentation::SegmentSelector {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_tr(_sel: segmentation::SegmentSelector) {
    unimplemented!()
}
