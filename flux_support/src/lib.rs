mod flux_register_interface;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
pub use flux_register_interface::*;

#[allow(dead_code)]
#[flux::sig(fn(x: bool[true]))]
pub fn assert(_x: bool) {}

#[flux::sig(fn(b:bool) ensures b)]
pub fn assume(b: bool) {
    if !b {
        panic!("assume fails")
    }
}

#[flux::opaque]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[flux::refined_by(ptr: int)]
pub struct FluxPtr {
    _inner: *mut u8,
}

// VTOCK-TODO: fill in these functions with obvious implementations
impl FluxPtr {
    pub const fn wrapping_add(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    pub const fn wrapping_sub(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    pub fn is_null(self) -> bool {
        unimplemented!()
    }

    pub fn as_usize(self) -> usize {
        unimplemented!()
    }

    pub fn as_u32(self) -> u32 {
        unimplemented!()
    }

    pub const fn null() -> Self {
        unimplemented!()
    }

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

#[flux::alias(type FluxPtrU8[n: int] = FluxPtr[n])]
pub type FluxPtrU8 = FluxPtr;
#[flux::alias(type FluxPtrU8Mut[n: int] = FluxPtr[n])]
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
