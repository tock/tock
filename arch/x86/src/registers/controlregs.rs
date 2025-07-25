// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;

#[cfg(target_arch = "x86")]
use core::arch::asm;

register_bitfields![u32,
    pub CR0[
        CR0_PROTECTED_MODE OFFSET(0) NUMBITS(1),
        CR0_MONITOR_COPROCESSOR OFFSET(1) NUMBITS(1),
        CR0_EMULATE_COPROCESSOR OFFSET(2) NUMBITS(1),
        CR0_TASK_SWITCHED OFFSET(3) NUMBITS(1),
        CR0_EXTENSION_TYPE OFFSET(4) NUMBITS(1),
        CR0_NUMERIC_ERROR OFFSET(5) NUMBITS(11),
        CR0_WRITE_PROTECT OFFSET(16) NUMBITS(2),
        CR0_ALIGNMENT_MASK OFFSET(18) NUMBITS(11),
        CR0_NOT_WRITE_THROUGH OFFSET(29) NUMBITS(1),
        CR0_CACHE_DISABLE OFFSET(30) NUMBITS(1),
        CR0_ENABLE_PAGING OFFSET(31) NUMBITS(1),
    ],
    pub CR4[
        CR4_ENABLE_VME OFFSET(0) NUMBITS(1),
        CR4_VIRTUAL_INTERRUPTS OFFSET(1) NUMBITS(1),
        CR4_TIME_STAMP_DISABLE OFFSET(2) NUMBITS(1),
        CR4_DEBUGGING_EXTENSIONS OFFSET(3) NUMBITS(1),
        CR4_ENABLE_PSE OFFSET(4) NUMBITS(1),
        CR4_ENABLE_PAE OFFSET(5) NUMBITS(1),
        CR4_ENABLE_MACHINE_CHECK OFFSET(6) NUMBITS(1),
        CR4_ENABLE_GLOBAL_PAGE OFFSET(7) NUMBITS(1),
        CR4_ENABLE_PPMC OFFSET(8) NUMBITS(1),
        CR4_ENABLE_SSE OFFSET(9) NUMBITS(1),
        CR4_UNMASKED_SSE OFFSET(10) NUMBITS(1),
        CR4_ENABLE_UMIP OFFSET(11) NUMBITS(1),
        CR4_ENABLE_LA57 OFFSET(12) NUMBITS(1),
        CR4_ENABLE_VMX OFFSET(13) NUMBITS(1),
        CR4_ENABLE_SMX OFFSET(14) NUMBITS(2),
        CR4_ENABLE_FSGSBASE OFFSET(16) NUMBITS(1),
        CR4_ENABLE_PCID OFFSET(17) NUMBITS(1),
        CR4_ENABLE_OS_XSAV OFFSET(18) NUMBITS(2),
        CR4_ENABLE_SMEP OFFSET(20) NUMBITS(1),
        CR4_ENABLE_SMAP OFFSET(21) NUMBITS(1),
        CR4_ENABLE_PROTECTION_KEY OFFSET(22) NUMBITS(1),
    ],
];

pub type Cr0 = LocalRegisterCopy<u32, CR0::Register>;
pub type Cr4 = LocalRegisterCopy<u32, CR4::Register>;

/// Read cr0
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr0() -> Cr0 {
    let ret: u32;
    unsafe {
        asm!("mov %cr0, {0}", out(reg) ret, options(att_syntax));
    }
    LocalRegisterCopy::new(ret)
}

/// Write cr0.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr0_write(val: Cr0) {
    unsafe {
        asm!("mov {0}, %cr0", in(reg) val.get(), options(att_syntax));
    }
}

/// Contains various flags to control operations in protected mode.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr4() -> Cr4 {
    let ret: u32;
    unsafe {
        asm!("mov %cr4, {0}", out(reg) ret, options(att_syntax));
    }
    LocalRegisterCopy::new(ret)
}

/// Write cr4.
///
/// # Example
///
/// ```no_run
/// use x86::registers::controlregs::*;
/// unsafe {
///   let cr4 = cr4();
///   let cr4 = cr4 | Cr4::CR4_ENABLE_PSE;
///   cr4_write(cr4);
/// }
/// ```
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr4_write(val: Cr4) {
    unsafe {
        asm!("mov {0}, %cr4", in(reg) val.get(), options(att_syntax));
    }
}

/// Contains page-table root pointer.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr3() -> u64 {
    let ret: usize;
    unsafe {
        asm!("mov %cr3, {0}", out(reg) ret, options(att_syntax));
    }
    ret as u64
}

/// Switch page-table PML4 pointer.
///
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn cr3_write(val: u64) {
    unsafe {
        asm!("mov {0}, %cr3", in(reg) val as usize, options(att_syntax));
    }
}

// For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr0() -> Cr0 {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr0_write(_val: Cr0) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr4() -> Cr4 {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr4_write(_val: Cr4) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr3() -> u64 {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn cr3_write(_val: u64) {
    unimplemented!()
}
