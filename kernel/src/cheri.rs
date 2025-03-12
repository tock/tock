// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

//! CHERI helpers for capabilities and inline asm for mostly CHERI-unaware rustc.
//! This still requires a rustc compiled with a CHERI-aware llvm.

#[cfg(target_feature = "xcheri")]
use crate::debug;
#[cfg(target_feature = "xcheri")]
use core::arch::asm;
#[cfg(target_feature = "xcheri")]
use core::fmt::Debug;
#[cfg(target_feature = "xcheri")]
use core::fmt::{Formatter, LowerHex, UpperHex};
use core::mem;
#[cfg(target_feature = "xcheri")]
use core::ops::AddAssign;

#[cfg(target_feature = "xcheri")]
pub const CPTR_ALIGN: usize = 2 * mem::size_of::<usize>();
#[cfg(not(target_feature = "xcheri"))]
pub const CPTR_ALIGN: usize = mem::size_of::<usize>();

#[cfg(target_pointer_width = "64")]
#[repr(align(16))]
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CptrAlign();

#[cfg(not(target_pointer_width = "64"))]
#[repr(align(8))]
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CptrAlign();

/// On CHERI this is meant to be a the same as C/C++'s __capability void*
/// On non-CHERI this is just a usize.
/// Just use *mut () if you want a non-capability in hybrid mode
// TODO: Remove me when there is compiler support
#[cfg(target_feature = "xcheri")]
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct cptr {
    // FIXME: There is nothing stopping the compiler from using two usize moves which are not
    // tag preserving, apart from using a capability move being more efficient.
    // CHERI memcpy does understand this rule.
    as_ints: [usize; 2],
    align: CptrAlign,
}

#[cfg(target_feature = "xcheri")]
impl Debug for cptr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_cap(f)
    }
}

#[cfg(not(target_feature = "xcheri"))]
#[allow(non_camel_case_types)]
pub type cptr = usize;

#[cfg(target_feature = "xcheri")]
macro_rules! inplace_cheri_asm {
    ($self : expr, $code : literal, $($body : tt)*) => {
        unsafe {
            asm!(   "lc ct0, 0({sptr})",
                    $code,
                    "sc ct0, 0({sptr})",
                sptr = in(reg) $self,
                $($body)*,
                out("t0") _,
                options(preserves_flags, nostack)
                );
        }
    }
}

pub mod cheri_perms {
    pub const GLOBAL: usize = 1 << 0;
    pub const EXECUTE: usize = 1 << 1;
    pub const LOAD: usize = 1 << 2;
    pub const STORE: usize = 1 << 3;
    pub const LOAD_CAP: usize = 1 << 4;
    pub const STORE_CAP: usize = 1 << 5;
    pub const STORE_CAP_LOCAL: usize = 1 << 6;
    pub const SEAL: usize = 1 << 7;
    pub const CINVOKE: usize = 1 << 8;
    pub const UNSEAL: usize = 1 << 9;
    pub const ACCESS_SYS: usize = 1 << 10;
    pub const SET_CID: usize = 1 << 11;

    pub const DEFAULT_RWX: usize =
        EXECUTE | LOAD | STORE | LOAD_CAP | STORE_CAP | GLOBAL | STORE_CAP_LOCAL;
    pub const DEFAULT_RW: usize = LOAD | STORE | LOAD_CAP | STORE_CAP | GLOBAL | STORE_CAP_LOCAL;
    pub const DEFAULT_RX: usize = EXECUTE | LOAD | LOAD_CAP | GLOBAL | STORE_CAP_LOCAL;
    pub const DEFAULT_R: usize = LOAD | LOAD_CAP | GLOBAL | STORE_CAP_LOCAL;
}

#[cfg(target_feature = "xcheri")]
macro_rules! cheri_get_asm {
    ($self : expr, $op : literal) => {
        unsafe {
            let res : usize;
            asm!(   "lc ct0, 0({sptr})",
                    concat!($op, " {res}, ct0"),
                sptr = in(reg) $self,
                res = out(reg) res,
                out("t0") _,
                options(preserves_flags, pure, readonly, nostack),
                );
            res
        }
    }
}

pub trait CPtrOps {
    fn as_ptr(&self) -> *const ();

    fn is_valid_for_operation(&self, _length: usize, _perms: usize) -> bool {
        true
    }

    fn as_ptr_checked(&self, length: usize, perms: usize) -> *const () {
        if self.is_valid_for_operation(length, perms) {
            self.as_ptr()
        } else {
            core::ptr::null()
        }
    }

    fn set_addr_from_ddc(&mut self, _addr: usize);
    fn set_addr_from_pcc(&mut self, _addr: usize);

    fn set_addr_from_ddc_restricted(&mut self, addr: usize, base: usize, len: usize, perms: usize) {
        self.set_addr_from_ddc(base);
        // Justification for why this is not exact:
        // cheri_mpu.rs ensures that the true range of DDC (rounded_app_brk) will not cross the
        // the kernel break.
        self.set_bounds(len);
        self.set_addr(addr);
        self.and_perms(perms);
    }

    fn set_addr_from_pcc_restricted(&mut self, addr: usize, base: usize, len: usize) {
        self.set_addr_from_pcc(base);
        // This is not exact for the same reason.
        self.set_bounds(len);
        self.set_addr(addr);
        self.and_perms(cheri_perms::DEFAULT_RX);
    }

    fn set_addr(&mut self, _addr: usize);
    fn as_mut_usize(&mut self) -> &mut usize;
    // cptr can be treated like an Option<NonNull<()>>
    fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&Self) -> U,
    {
        if self.as_ptr() as usize == 0usize {
            default
        } else {
            f(self)
        }
    }
    fn set_bounds(&mut self, _length: usize) {}
    fn set_bounds_exact(&mut self, _length: usize) {}
    fn and_perms(&mut self, _perms: usize) {}
    fn seal_entry(&mut self) {}
    fn set_flags(&mut self, _flags: usize) {}
}

pub fn null() -> cptr {
    // On non-cheri this is a useless convervstion
    #[allow(clippy::useless_conversion)]
    0usize.into()
}

pub const TYPE_BITS_START: usize = 27;
pub const TYPE_BITS_LEN: usize = 18;

#[cfg(target_feature = "xcheri")]
impl cptr {
    pub fn fmt_cap(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:#018X} (b:{:#018X},t:{:#018X},v:{},p:{:2X}",
            self.as_ptr() as usize,
            self.get_base(),
            self.get_top_unclamped(),
            self.get_tag(),
            self.get_perms(),
        )
    }

    pub fn get_offset(&self) -> usize {
        cheri_get_asm!(self, "cgetoffset")
    }
    pub fn get_len(&self) -> usize {
        cheri_get_asm!(self, "cgetlen")
    }
    pub fn get_perms(&self) -> usize {
        cheri_get_asm!(self, "cgetperm")
    }
    pub fn get_tag(&self) -> usize {
        cheri_get_asm!(self, "cgettag")
    }
    pub fn get_type(&self) -> usize {
        cheri_get_asm!(self, "cgettype")
    }
    pub fn get_base(&self) -> usize {
        cheri_get_asm!(self, "cgetbase")
    }

    /// Slighter more efficient than doing get_base() and get_tag() separately
    #[inline]
    pub fn get_base_and_tag(&self) -> (usize, bool) {
        unsafe {
            let base: usize;
            let tag: usize;
            asm!(
                "lc ct0, 0({sptr})",
                "cgetbase {base}, ct0",
                "cgettag {tag}, ct0",
                sptr = in(reg) self,
                base = out(reg) base,
                tag = out(reg) tag,
                out("t0") _,
                options(preserves_flags, pure, readonly, nostack),
            );
            (base, tag != 0)
        }
    }

    #[inline]
    pub fn invalidate_shared(shared: &core::cell::Cell<Self>) {
        unsafe {
            asm!(
                "lw t0, 0({sptr})",
                "sw t0, 0({sptr})",
                sptr = in(reg) (shared as *const core::cell::Cell<Self>),
                out("t0") _,
                options(preserves_flags, nostack),
            )
        }
    }

    pub fn get_top_unclamped(&self) -> usize {
        self.get_base() + self.get_len()
    }
    pub fn get_high(&self) -> usize {
        self.as_ints[1]
    }
}

#[cfg(target_feature = "xcheri")]
impl CPtrOps for cptr {
    fn as_ptr(&self) -> *const () {
        usize::from(self) as *const ()
    }

    fn is_valid_for_operation(&self, length: usize, perms: usize) -> bool {
        let coffset: usize = self.get_offset();
        let clen: usize = self.get_len();
        let cperms: usize = self.get_perms();
        let tag: usize = self.get_tag();
        let ctype: usize = self.get_type();

        // Must be tagged
        let mut checks_pass = tag != 0;

        // Must be unsealed
        checks_pass &= ctype == !0usize;

        // Have all specified permissions
        checks_pass &= (cperms & perms) == perms;

        // Now check length. We want to check that [coffset, coffset + length] fits (non-strictly)
        // within [0, clen]

        // First, check offset is in [0, clen], i.e., the capability is in its bounds.
        // NOTE: If the offset is negative, this will still be false as we do an unsigned comparison
        // NOTE: cgetlen is saturating, not truncating.
        checks_pass &= coffset <= clen;

        // Second,  Check offset + length   (the end of the offset the user is asking us to access)
        //          <=
        //          clen.                   (the largest end the capability would allow)
        // NOTE: the user controls length and so can overflow the calculation offset + length
        // However, as we have already checked offset is in the range [0, clen] we can use
        // this arrangement:
        checks_pass &= length <= clen - coffset;

        if !checks_pass && length != usize::MAX && length != 1 {
            debug!(
                "Capability {:?} not valid for operation of length {}. perms: {:x}.",
                self, length, perms
            );
        }

        checks_pass
    }

    fn set_addr_from_ddc(&mut self, _addr: usize) {
        unsafe {
            asm!(   "cspecialr ct0, ddc",
                "csetaddr ct0, ct0, {val}",
                "sc ct0, 0({sptr})",
            sptr = in(reg) self,
            val = in(reg) _addr,
            out("t0") _
            );
        }
    }

    fn set_addr_from_pcc(&mut self, _addr: usize) {
        unsafe {
            asm!(   "cspecialr ct0, pcc",
                "csetaddr ct0, ct0, {val}",
                "sc ct0, 0({sptr})",
            sptr = in(reg) self,
            val = in(reg) _addr,
            out("t0") _
            );
        }
    }

    fn set_addr(&mut self, _addr: usize) {
        inplace_cheri_asm!(self, "csetaddr ct0, ct0, {val}", val = in(reg) _addr)
    }

    fn as_mut_usize(&mut self) -> &mut usize {
        return &mut self.as_ints[0];
    }

    fn set_bounds(&mut self, length: usize) {
        inplace_cheri_asm!(self, "csetbounds ct0, ct0, {val}", val = in(reg) length)
    }

    fn set_bounds_exact(&mut self, length: usize) {
        inplace_cheri_asm!(self, "csetboundsexact ct0, ct0, {val}", val = in(reg) length)
    }

    fn and_perms(&mut self, perms: usize) {
        inplace_cheri_asm!(self, "candperm ct0, ct0, {val}", val = in(reg) perms)
    }

    fn set_flags(&mut self, flags: usize) {
        inplace_cheri_asm!(self, "csetflags ct0, ct0, {val}", val = in(reg) flags)
    }
}

#[cfg(target_feature = "xcheri")]
impl Clone for cptr {
    fn clone(&self) -> Self {
        let mut x: cptr = cptr {
            as_ints: [0, 0],
            align: CptrAlign(),
        };
        x.clone_from(self);
        x
    }

    // The compiler is still getting moves wrong for capabilities
    // This version gets it right, I might make the type not copy,
    // and then use this everywhere

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            asm!(   "lc ct0, 0({src})",
                "sc ct0, 0({dst})",
            src = in(reg) source,
            dst = in(reg) self,
            out("t0") _
            );
        }
    }
}

#[cfg(not(target_feature = "xcheri"))]
impl CPtrOps for usize {
    fn as_ptr(&self) -> *const () {
        *self as *const ()
    }

    fn set_addr_from_ddc(&mut self, _addr: usize) {
        *self = _addr;
    }
    fn set_addr_from_pcc(&mut self, _addr: usize) {
        *self = _addr;
    }

    fn set_addr(&mut self, _addr: usize) {
        *self = _addr;
    }

    fn as_mut_usize(&mut self) -> &mut usize {
        self
    }
}

#[cfg(target_feature = "xcheri")]
impl AddAssign<usize> for cptr {
    fn add_assign(&mut self, rhs: usize) {
        inplace_cheri_asm!(self, "cincoffset ct0, ct0, {val}", val = in(reg) rhs)
    }
}

// For printing the address as hex
#[cfg(target_feature = "xcheri")]
impl UpperHex for cptr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_cap(f)
    }
}

#[cfg(target_feature = "xcheri")]
impl LowerHex for cptr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_cap(f)
    }
}

// A provenance free cast
#[cfg(target_feature = "xcheri")]
impl From<usize> for cptr {
    fn from(val: usize) -> Self {
        let mut res: cptr = cptr::default();
        res.as_ints[0] = val;
        res
    }
}

// Cast back to usize
#[cfg(target_feature = "xcheri")]
impl From<cptr> for usize {
    fn from(ptr: cptr) -> Self {
        ptr.as_ints[0]
    }
}
#[cfg(target_feature = "xcheri")]
impl From<&cptr> for usize {
    fn from(ptr: &cptr) -> Self {
        ptr.as_ints[0]
    }
}

// Trace on / off on QEMU
pub fn trace_on() {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        core::arch::asm!("slti zero, zero, 0x1b")
    }
}

pub fn trace_off() {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        core::arch::asm!("slti zero, zero, 0x1e")
    }
}

// Macros to help asm. Might just move these to easm

/// Gives the name of the pointer-width register
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! ptrreg {
    (zero) => {"cnull"};
    ($($targ: expr)?) => {concat!("c", $($targ)?)}
}

#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! ptrreg {
    (zero) => {"zero"};
    ($($targ: expr)?) => {concat!("", $($targ)?)}
}

#[macro_export]
macro_rules! ptrreg_non_zero {
    (zero) => {compile_error!("CHERI compiler silently treats writing zero to cap csrs as a no-op")};
    ($($targ: expr)?) => {ptrreg!($($targ)?)}
}

/// Gives the name of the pointer-width register (by number)
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! ptrregn {
    ($($targ: expr)?) => {concat!("c", $($targ)?)}
}

#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! ptrregn {
    ($($targ: expr)?) => {concat!("x", $($targ)?)}
}

// Loads an XLEN size register
#[cfg(target_arch = "riscv32")]
#[macro_export]
macro_rules! ldx {
    () => {
        "lw "
    };
}
#[cfg(not(target_arch = "riscv32"))]
#[macro_export]
macro_rules! ldx {
    () => {
        "ld "
    };
}

/// Stores a XLEN size register
#[cfg(target_arch = "riscv32")]
#[macro_export]
macro_rules! stx {
    () => {
        "sw "
    };
}
/// Stores a XLEN size register
#[cfg(not(target_arch = "riscv32"))]
#[macro_export]
macro_rules! stx {
    () => {
        "sd "
    };
}

/// Loads a pointer-sized register
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! ldptr {
    () => {
        "lc "
    };
}
/// Loads a pointer-sized register
#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! ldptr {
    () => {
        $crate::ldx!()
    };
}

/// Stores a pointer-sized register
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! stptr {
    () => {
        "sc "
    };
}

/// Stores a pointer-sized register
#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! stptr {
    () => {
        stx!()
    };
}

/// Does csr or cspecial depending on platform
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! csr_ptr {
    () => {
        "cspecial"
    };
}
/// Does csr or cspecial depending on platform
#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! csr_ptr {
    () => {
        "csr"
    };
}

#[macro_export]
macro_rules! csr_op {
    {$REG : tt <- $SRC : tt} =>
        {concat!(csr_ptr!(), "w", " ", $REG, ptrreg!(), ", ", $crate::ptrreg_non_zero!($SRC))};
    {$REG : tt -> $DST : tt} =>
        {concat!(csr_ptr!(), "r", " ", ptrreg!($DST), ", ", $REG, ptrreg!())};
    {$DST : tt <- $REG : tt <- $SRC : tt} =>
        {concat!(csr_ptr!(), "rw", " ", ptrreg!($DST), ", ", $REG, ptrreg!(), ", ", $crate::ptrreg_non_zero!($SRC))};
}

/// Is there already an assembly level symbol for this?
/// Expands to a string constant 0 or 1
#[cfg(target_feature = "xcheri")]
#[macro_export]
macro_rules! is_cheri {
    () => {
        "1"
    };
}

#[cfg(not(target_feature = "xcheri"))]
#[macro_export]
macro_rules! is_cheri {
    () => {
        "0"
    };
}

/// CRAM: Returns a mask (all ones in the top end) that can be used to round bounds and lengths
/// such that they can be represented. Both length, bottom, and top must be rounded using the mask.
///
/// Increasing length by rounding it up is guaranteed not to change the alignment requirement.
/// Increasing length by any more may change the alignment requirement.
pub fn cram(_length: usize) -> usize {
    let result: usize;

    // SAFETY: cram is always safe
    #[cfg(target_feature = "xcheri")]
    unsafe {
        asm!(
        "cram {result}, {input}",
        input = in(reg) _length,
        result = out(reg) result,
        options(pure, nomem, preserves_flags, nostack),
        )
    }
    #[cfg(not(target_feature = "xcheri"))]
    {
        result = !0;
    }

    result
}
