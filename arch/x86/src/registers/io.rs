// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! I/O port functionality.

#[cfg(target_arch = "x86")]
use core::arch::asm;
use tock_registers::{Address, Bus, BusRead, BusWrite};

#[derive(Clone, Copy)]
pub struct Port(pub u16);

impl Address for Port {
    unsafe fn byte_add(self, offset: usize) -> Port {
        Port(self.0 + offset as u16)
    }
}

macro_rules! bus_impls {
    ($value:ty, $size:literal, $in:ident, $out:ident) => {
        impl Bus<$value> for Port {
            const PADDED_SIZE: usize = $size;
        }
        impl BusRead<$value> for Port {
            unsafe fn read(self) -> $value {
                #[cfg(target_arch = "x86")]
                unsafe {
                    $in(self.0) as $value
                }
                #[cfg(not(target_arch = "x86"))]
                unimplemented!()
            }
        }
        impl BusWrite<$value> for Port {
            unsafe fn write(self, val: $value) {
                #[cfg(target_arch = "x86")]
                unsafe {
                    $out(self.0, val as _)
                }
                #[cfg(not(target_arch = "x86"))]
                {
                    let _ = val;
                    unimplemented!()
                }
            }
        }
    };
}

bus_impls!(u8, 1, inb, outb);
bus_impls!(u16, 2, inw, outw);
bus_impls!(u32, 4, inl, outl);
bus_impls!(usize, 4, inl, outl);

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
