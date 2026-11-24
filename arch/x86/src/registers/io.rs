// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! I/O port functionality.

#[cfg(target_arch = "x86")]
use core::arch::asm;
use core::marker::PhantomData;
use tock_registers_v0::{Address, Bus, BusRead, BusWrite};

/// An x86 I/O port.
///
/// This is a `tock_registers::register_map` bus.
#[derive(Clone, Copy)]
pub struct Port {
    // A list of common ports indicates the first DMA controller often starts at
    // address 0. If we made this a NonZeroU16, then we'd need a separate
    // PortNullable type to access that controller. However, because there
    // aren't a large number of devices at nonzero port numbers, it's probably
    // not worth the effort to maintain two separate port types, so this Port
    // can represent address 0.
    number: u16,
    // Makes Port !Sync. This isn't required for soundness, but helps to prevent
    // misuse (for similar reasons to why raw pointers are !Sync).
    _phantom: PhantomData<*mut ()>,
}

impl Port {
    /// Returns a new Port with the given number.
    ///
    /// # Panics
    /// Panics if the target is not x86.
    #[cfg_attr(not(target_arch = "x86"), track_caller, expect(unused))]
    pub const fn new(number: u16) -> Self {
        #[cfg(not(target_arch = "x86"))]
        panic!("Port requires an x86 system");
        Self {
            number,
            _phantom: PhantomData,
        }
    }
}

impl Address for Port {
    unsafe fn byte_add(self, offset: usize) -> Self {
        Port {
            // byte_add's safety preconditions allow us to use `unsafe` to tell
            // the compiler that the cast and addition will never overflow.
            // However, that's probably not worth the risk so this uses safe
            // operations.
            number: self.number + offset as u16,
            _phantom: PhantomData,
        }
    }
}

/// # Safety
///
/// `in_asm` and `out_asm` must be the correct assembly expressions for
/// accessing a port of width `value`.
macro_rules! unsafe_bus_impls {
    ($value:ty, $number: ident, $val:ident, $in_asm:expr, $out_asm:expr) => {
        // Safety: Ports are generally spaced 1 apart regardless of their width.
        unsafe impl Bus<$value> for Port {
            const PADDED_SIZE: usize = 1;
        }

        impl BusRead<$value> for Port {
            #[cfg_attr(not(target_arch = "x86"), expect(unused))]
            unsafe fn read(self) -> $value {
                #[cfg(not(target_arch = "x86"))]
                panic!("Port requires an x86 system");
                let $number = self.number;
                let $val: $value;
                // Safety: The macro call guaranteed this assembly is correct.
                #[cfg(target_arch = "x86")]
                unsafe {
                    $in_asm
                }
                $val
            }
        }

        impl BusWrite<$value> for Port {
            #[cfg_attr(not(target_arch = "x86"), expect(unused))]
            unsafe fn write(self, $val: $value) {
                #[cfg(not(target_arch = "x86"))]
                panic!("Port requires an x86 system");
                let $number = self.number;
                // Safety: The macro call guaranteed this assembly is correct.
                #[cfg(target_arch = "x86")]
                unsafe {
                    $out_asm
                }
            }
        }
    };
}

// Safety: These assembly operations are all correct.
unsafe_bus_impls!(
    u8,
    number,
    val,
    asm!("inb %dx, %al", in("dx") number, out("al") val, options(att_syntax)),
    asm!("outb %al, %dx", in("al") val, in("dx") number, options(att_syntax))
);
unsafe_bus_impls!(
    u16,
    number,
    val,
    asm!("inw %dx, %ax", in("dx") number, out("ax") val, options(att_syntax)),
    asm!("outw %ax, %dx", in("ax") val, in("dx") number, options(att_syntax))
);
unsafe_bus_impls!(
    u32,
    number,
    val,
    asm!("inl %dx, %eax", out("eax") val, in("dx") number, options(att_syntax)),
    asm!("outl %eax, %dx", in("eax") val, in("dx") number, options(att_syntax))
);
// This omits implementations for `usize` and pointers because ports generally
// use 16-bit operations for addresses, which doesn't match qemu_i486_virt's
// 32-bit target_pointer_width.

// Safety: Port does not expose safe operations to access registers on its own.
// Instead, it is used through two types:
// 1. The register accessor structs, which are always !Send + !Sync
// 2. RegisterSender<>, which is Send if the bus is Send but which guarantees
//    that only one thread can access the registers at a time. Making Port Send
//    enables RegisterSender<> to work with it.
unsafe impl Send for Port {}

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

#[cfg(not(target_arch = "x86"))]
pub unsafe fn outb(_port: u16, _val: u8) {
    unimplemented!()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe fn inb(_port: u16) -> u8 {
    unimplemented!()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe fn outw(_port: u16, _val: u16) {
    unimplemented!()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe fn inw(_port: u16) -> u16 {
    unimplemented!()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe fn outl(_port: u16, _val: u32) {
    unimplemented!()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe fn inl(_port: u16) -> u32 {
    unimplemented!()
}
