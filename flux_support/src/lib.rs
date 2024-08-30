mod flux_register_interface;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
pub use flux_register_interface::*;
use flux_rs::{extern_spec, refined_by, sig};

#[allow(dead_code)]
#[sig(fn(x: bool[true]))]
pub const fn assert(_x: bool) {}

#[sig(fn(b:bool) ensures b)]
pub const fn assume(b: bool) {
    if !b {
        panic!("assume fails")
    }
}

#[flux_rs::opaque]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord)]
#[refined_by(ptr: int)]
pub struct FluxPtr {
    inner: *mut u8,
}

// VTOCK-TODO: fill in these functions with obvious implementations
impl FluxPtr {
    #[sig(fn(self: Self[@lhs], rhs: usize) -> Self{r: ((lhs + rhs <= usize::MAX) => r == lhs + rhs) && ((lhs + rhs > usize::MAX) => r == lhs + rhs - usize::MAX) })]
    pub const fn wrapping_add(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

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

    pub const unsafe fn offset(self, _count: isize) -> Self {
        unimplemented!()
    }

    pub const unsafe fn add(self, _count: usize) -> Self {
        unimplemented!()
    }

    pub fn unsafe_as_ptr(self) -> *mut u8 {
        unimplemented!()
    }
}

#[flux_rs::trusted]
impl PartialOrd for FluxPtr {
    fn partial_cmp(&self, _other: &Self) -> Option<core::cmp::Ordering> {
        todo!()
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

#[flux_rs::alias(type FluxPtrU8[n: int] = FluxPtr[n])]
pub type FluxPtrU8 = FluxPtr;
#[flux_rs::alias(type FluxPtrU8Mut[n: int] = FluxPtr[n])]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[refined_by(start: int, end: int)]
pub struct FluxRange {
    #[field(usize[start])]
    pub start: usize,
    #[field(usize[end])]
    pub end: usize,
}

#[sig(fn(lhs: usize, rhs: usize) -> usize {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_usize(lhs: usize, rhs: usize) -> usize {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

#[sig(fn(self: FluxPtr[@lhs], other: FluxPtr[@rhs]) -> FluxPtr {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_ptr(lhs: FluxPtr, rhs: FluxPtr) -> FluxPtr {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

#[extern_spec]
impl<T> [T] {
    #[sig(fn(&[T][@n]) -> usize[n])]
    fn len(v: &[T]) -> usize;
}

#[extern_spec(core::ptr)]
#[refined_by(n: int)]
struct NonNull<T>;
