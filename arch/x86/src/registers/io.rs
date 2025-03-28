// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! I/O port functionality.

#[cfg(target_arch = "x86")]
use core::arch::asm;

/// Write 8 bits to port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("outb %al, %dx", in("al") val, in("dx") port, options(att_syntax));
    }
}

/// Read 8 bits from port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        asm!("inb %dx, %al", in("dx") port, out("al") ret, options(att_syntax));
    }
    ret
}

/// Write 16 bits to port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    unsafe {
        asm!("outw %ax, %dx", in("ax") val, in("dx") port, options(att_syntax));
    }
}

/// Read 16 bits from port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let ret: u16;
    unsafe {
        asm!("inw %dx, %ax", in("dx") port, out("ax") ret, options(att_syntax));
    }
    ret
}

/// Write 32 bits to port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    unsafe {
        asm!("outl %eax, %dx", in("eax") val, in("dx") port, options(att_syntax));
    }
}

/// Read 32 bits from port
///
/// # Safety
/// Needs IO privileges.
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let ret: u32;
    unsafe {
        asm!("inl %dx, %eax", out("eax") ret, in("dx") port, options(att_syntax));
    }
    ret
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn outb(_port: u16, _val: u8) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn inb(_port: u16) -> u8 {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn outw(_port: u16, _val: u16) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn inw(_port: u16) -> u16 {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn outl(_port: u16, _val: u32) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn inl(_port: u16) -> u32 {
    unimplemented!()
}
