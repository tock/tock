// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Functions and data-structures for working with descriptor tables.

use super::segmentation::SegmentSelector;

#[cfg(target_arch = "x86")]
use core::arch::asm;

use core::fmt;
use core::mem::size_of;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[repr(C, packed)]
pub struct DescriptorTablePointer<Entry> {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const Entry,
}

impl<T> Default for DescriptorTablePointer<T> {
    fn default() -> DescriptorTablePointer<T> {
        DescriptorTablePointer {
            limit: 0,
            base: core::ptr::null(),
        }
    }
}

impl<T> DescriptorTablePointer<T> {
    pub fn new(tbl: &T) -> Self {
        // GDT, LDT, and IDT all expect the limit to be set to "one less".
        // See Intel 3a, Section 3.5.1 "Segment Descriptor Tables" and
        // Section 6.10 "Interrupt Descriptor Table (IDT)".
        let len = size_of::<T>() - 1;
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: core::ptr::from_ref::<T>(tbl),
            limit: len as u16,
        }
    }

    pub fn new_from_slice(slice: &[T]) -> Self {
        // GDT, LDT, and IDT all expect the limit to be set to "one less".
        // See Intel 3a, Section 3.5.1 "Segment Descriptor Tables" and
        // Section 6.10 "Interrupt Descriptor Table (IDT)".
        let len = core::mem::size_of_val(slice) - 1;
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: slice.as_ptr(),
            limit: len as u16,
        }
    }
}

impl<T> fmt::Debug for DescriptorTablePointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DescriptorTablePointer ({} {:?})", { self.limit }, {
            self.base
        })
    }
}

/// Load the GDTR register with the specified base and limit.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn lgdt<T>(gdt: &DescriptorTablePointer<T>) {
    unsafe {
        asm!("lgdt ({0})", in(reg) gdt, options(att_syntax));
    }
}

/// Retrieve base and limit from the GDTR register.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn sgdt<T>(idt: &mut DescriptorTablePointer<T>) {
    unsafe {
        asm!("sgdt ({0})", in(reg) core::ptr::from_mut::<DescriptorTablePointer<T>>(idt), options(att_syntax));
    }
}

/// Loads the segment selector into the selector field of the local
/// descriptor table register (LDTR).
///
/// After the segment selector is loaded in the LDTR,
/// the processor uses the segment selector to locate
/// the segment descriptor for the LDT in the global
/// descriptor table (GDT).
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_ldtr(selector: SegmentSelector) {
    unsafe {
        asm!("lldt {0:x}", in(reg) selector.bits(), options(att_syntax));
    }
}

/// Returns the segment selector from the local descriptor table register (LDTR).
///
/// The returned segment selector points to the segment descriptor
/// (located in the GDT) for the current LDT.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn ldtr() -> SegmentSelector {
    let selector: u16;
    unsafe {
        asm!("sldt {0:x}", out(reg) selector, options(att_syntax));
    }
    SegmentSelector::from_raw(selector)
}

/// Load the IDTR register with the specified base and limit.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn lidt<T>(idt: &DescriptorTablePointer<T>) {
    unsafe {
        asm!("lidt ({0})", in(reg) idt, options(att_syntax));
    }
}

/// Retrieve base and limit from the IDTR register.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn sidt<T>(idt: &mut DescriptorTablePointer<T>) {
    unsafe {
        asm!("sidt ({0})", in(reg) core::ptr::from_mut::<DescriptorTablePointer<T>>(idt), options(att_syntax));
    }
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn lgdt<T>(_gdt: &DescriptorTablePointer<T>) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn sgdt<T>(_idt: &mut DescriptorTablePointer<T>) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_ldtr(_selector: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn ldtr() -> SegmentSelector {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn lidt<T>(_idt: &DescriptorTablePointer<T>) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn sidt<T>(_idt: &mut DescriptorTablePointer<T>) {
    unimplemented!()
}
