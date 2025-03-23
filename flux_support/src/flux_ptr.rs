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

#[flux_rs::trusted]
impl From<*mut u8> for FluxPtr {
    fn from(value: *mut u8) -> Self {
        FluxPtr {
            inner: value as *mut u8,
        }
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
    #[sig(fn(self: Self[@lhs], rhs: usize) -> Self[if lhs + rhs <= usize::MAX { lhs + rhs } else { lhs + rhs - usize::MAX }])]
    pub const fn wrapping_add(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    #[sig(fn(self: Self[@lhs], rhs: usize) -> Self[if lhs - rhs >= 0 { lhs - rhs } else { - (lhs - rhs) }])]
    pub const fn wrapping_sub(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    #[sig(fn(self: Self[@n]) -> bool[n == 0] )]
    pub fn is_null(self) -> bool {
        unimplemented!()
    }

    #[sig(fn(self: Self[@n]) -> usize[n])]
    pub fn as_usize(self) -> usize {
        unimplemented!()
    }

    pub fn as_u32(self) -> u32 {
        unimplemented!()
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
    pub const unsafe fn offset(self, _count: isize) -> Self {
        unimplemented!()
    }

    /// # Safety
    pub const unsafe fn add(self, _count: usize) -> Self {
        unimplemented!()
    }

    pub fn unsafe_as_ptr(self) -> *mut u8 {
        unimplemented!()
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
        unimplemented!()
    }
}

impl<T> FluxPtrExt for &mut [T] {
    fn as_fluxptr(&self) -> FluxPtr {
        unimplemented!()
    }
}

impl<T> FluxPtrExt for NonNull<T> {
    fn as_fluxptr(&self) -> FluxPtr {
        unimplemented!()
    }
}

impl FluxPtrExt for usize {
    fn as_fluxptr(&self) -> FluxPtr {
        unimplemented!()
    }
}

impl Deref for FluxPtr {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}

impl DerefMut for FluxPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unimplemented!()
    }
}
