use core::clone::Clone;
use core::cmp::Eq;
use core::cmp::Ord;
use core::cmp::PartialEq;
use core::cmp::PartialOrd;
use core::convert::From;
use core::fmt::Debug;
use core::marker::Copy;
use core::ops::Rem;
use core::ops::{Deref, DerefMut};
use core::option::Option;
use core::option::Option::Some;
use core::prelude::rust_2021::derive;
use core::ptr::NonNull;
use core::todo;
use core::unimplemented;
use flux_rs::{refined_by, sig};

use crate::Pair;

#[flux_rs::opaque]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[refined_by(ptr: int)]
pub struct FluxPtr {
    inner: *mut u8,
}

#[flux_rs::trusted]
impl From<usize> for FluxPtr {
    #[flux_rs::sig(fn (value: usize) -> FluxPtr[value])]
    #[flux_rs::trusted]
    fn from(value: usize) -> Self {
        FluxPtr {
            inner: value as *mut u8,
        }
    }
}

impl From<NonNull<u8>> for FluxPtr {
    #[flux_rs::sig(fn (value: NonNull<u8>) -> FluxPtr[value])]
    #[flux_rs::trusted]
    fn from(value: NonNull<u8>) -> Self {
        FluxPtr {
            inner: value.as_ptr(),
        }
    }
}

#[flux_rs::trusted]
impl From<*mut u8> for FluxPtr {
    fn from(value: *mut u8) -> Self {
        FluxPtr {
            inner: value as *mut u8,
        }
    }
}

#[flux_rs::trusted]
impl From<FluxPtr> for NonNull<u8> {
    fn from(value: FluxPtr) -> NonNull<u8> {
        unsafe { NonNull::new_unchecked(value.inner) }
    }
}

// Support cast from FluxPtr to u32
impl From<FluxPtr> for u32 {
    fn from(ptr: FluxPtr) -> u32 {
        ptr.as_u32()
    }
}

// convert FluxPtr to *const u8
#[flux_rs::trusted]
impl From<FluxPtr> for u8 {
    fn from(ptr: FluxPtr) -> u8 {
        ptr.inner as u8
    }
}
// FluxPtr to usize
impl From<FluxPtr> for usize {
    #[flux_rs::sig(fn (ptr: FluxPtr) -> usize[ptr])]
    fn from(ptr: FluxPtr) -> usize {
        ptr.as_usize()
    }
}

// Implement Rem trait for FluxPtr
#[flux_rs::trusted]
impl Rem<usize> for FluxPtr {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        (self.inner as usize) % rhs
    }
}

// implement implement `AddAssign<usize>`` for FluxPtr
#[flux_rs::trusted]
impl core::ops::AddAssign<usize> for FluxPtr {
    fn add_assign(&mut self, rhs: usize) {
        *self = FluxPtr {
            inner: (self.inner as usize + rhs) as *mut u8,
        }
    }
}

impl core::cmp::Ord for FluxPtr {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_usize().cmp(&other.as_usize())
    }
}

// VTOCK-TODO: fill in these functions with obvious implementations
impl FluxPtr {
    #[flux_rs::trusted]
    #[sig(fn(self: Self[@lhs], rhs: usize) -> Self[if lhs + rhs <= usize::MAX { lhs + rhs } else { lhs + rhs - usize::MAX }])]
    pub const fn wrapping_add(self, count: usize) -> FluxPtr {
        Self {
            inner: self.inner.wrapping_add(count),
        }
    }

    #[flux_rs::trusted]
    #[sig(fn(self: Self[@lhs], rhs: usize) -> Self[if lhs >= rhs { lhs - rhs } else { - (lhs - rhs) }])]
    pub const fn wrapping_sub(self, count: usize) -> FluxPtr {
        Self {
            inner: self.inner.wrapping_sub(count),
        }
    }

    #[flux_rs::trusted]
    #[sig(fn(self: Self[@n]) -> bool[n == 0] )]
    pub fn is_null(self) -> bool {
        self.inner.is_null()
    }

    #[flux_rs::trusted]
    #[sig(fn(self: Self[@n]) -> usize[n])]
    pub fn as_usize(self) -> usize {
        self.inner as usize
    }

    #[flux_rs::trusted]
    #[sig(fn(self: Self[@n]) -> u32[n])]
    pub fn as_u32(self) -> u32 {
        self.inner as u32
    }

    #[sig(fn() -> Self[0])]
    pub const fn null() -> Self {
        unimplemented!()
    }

    #[sig(fn() -> Self[0])]
    pub const fn null_mut() -> Self {
        unimplemented!()
    }

    // VTOCK-TODO: Add precondition that input isn't zero
    #[sig(fn(Self[@ptr]) -> NonNull<u8>[ptr])]
    pub fn as_nonnull(self) -> NonNull<u8> {
        unimplemented!()
    }

    /// # Safety
    /// the size of u8 is 1 so this is equivalent to self + count
    /// must not overflow the address space
    #[flux_rs::trusted]
    #[flux_rs::sig(fn (Self[@s], { isize[@count] | s + count >= 0 && s + count <= usize::MAX }) -> Self[s + count])]
    pub const unsafe fn offset(self, count: isize) -> Self {
        Self {
            inner: self.inner.offset(count),
        }
    }

    #[flux_rs::trusted]
    pub const fn wrapping_offset(self, count: isize) -> Self {
        Self {
            inner: self.inner.wrapping_offset(count),
        }
    }

    /// # Safety
    #[flux_rs::trusted]
    #[flux_rs::sig(fn (Self[@s], { usize[@count] | s + count <= usize::MAX }) -> Self[s + count])]
    pub const unsafe fn add(self, count: usize) -> Self {
        Self {
            inner: self.inner.add(count),
        }
    }

    #[flux_rs::trusted]
    pub fn unsafe_as_ptr(self) -> *mut u8 {
        self.inner
    }
}

#[flux_rs::trusted]
impl PartialOrd for FluxPtr {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }

    // Provided methods
    #[sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs < rhs])]
    fn lt(&self, _other: &Self) -> bool {
        todo!()
    }
    #[sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs <= rhs])]
    fn le(&self, _other: &Self) -> bool {
        todo!()
    }
    #[sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs > rhs])]
    fn gt(&self, _other: &Self) -> bool {
        todo!()
    }
    #[sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs >= rhs])]
    fn ge(&self, _other: &Self) -> bool {
        todo!()
    }
}

pub type FluxPtrU8 = FluxPtr;

pub type FluxPtrU8Mut = FluxPtr;

pub trait FluxPtrExt {
    fn as_fluxptr(&self) -> FluxPtr;
}

impl<T> FluxPtrExt for &[T] {
    fn as_fluxptr(&self) -> FluxPtr {
        FluxPtr::from(self.as_ptr() as *mut u8)
    }
}

impl<T> FluxPtrExt for &mut [T] {
    fn as_fluxptr(&self) -> FluxPtr {
        FluxPtr::from(self.as_ptr() as *mut u8)
    }
}

impl<T> FluxPtrExt for NonNull<T> {
    fn as_fluxptr(&self) -> FluxPtr {
        FluxPtr::from(self.as_ptr() as *mut u8)
    }
}

impl FluxPtrExt for usize {
    fn as_fluxptr(&self) -> FluxPtr {
        FluxPtr::from(*self)
    }
}

impl Deref for FluxPtr {
    type Target = u8;

    #[flux_rs::trusted]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl DerefMut for FluxPtr {
    #[flux_rs::trusted]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

flux_rs::defs! {
    fn flash_before_ram(fst: SlicesToRaw, snd: SlicesToRaw) -> bool {
        fst.start + fst.len < snd.start
    }
}

#[flux_rs::refined_by(start: int, len: int)]
pub struct SlicesToRaw {
    #[field(FluxPtr[start])]
    pub start: FluxPtr,
    #[field(usize[len])]
    pub len: usize,
}

// TRUSTED: From Rust aliasing rules + the fact that we trust flash < ram in the address space
#[flux_rs::trusted]
#[flux_rs::sig(fn (&[u8][@l1], &mut [u8][@l2]) -> Pair<SlicesToRaw, SlicesToRaw>{p: flash_before_ram(p.fst, p.snd) })]
pub fn mem_slices_to_raw_ptrs(flash: &[u8], ram: &mut [u8]) -> Pair<SlicesToRaw, SlicesToRaw> {
    Pair {
        fst: SlicesToRaw {
            start: flash.as_fluxptr(),
            len: flash.len(),
        },
        snd: SlicesToRaw {
            start: ram.as_fluxptr(),
            len: ram.len(),
        },
    }
}

#[flux_rs::trusted]
#[flux_rs::sig(fn (_, usize[@len]) -> &mut [T][len])]
pub fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    unsafe { core::slice::from_raw_parts_mut(data, len) }
}

#[flux_rs::trusted]
#[flux_rs::sig(fn ({usize[@x] | x < isize::MAX}) -> isize[x])]
pub fn usize_into_isize(x: usize) -> isize {
    x as isize
}

#[flux_rs::trusted]
#[flux_rs::sig(fn ({isize[@x] | x >= 0}) -> usize[x])]
pub fn isize_into_usize(x: isize) -> usize {
    x as usize
}
