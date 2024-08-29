mod flux_register_interface;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
pub use flux_register_interface::*;

#[allow(dead_code)]
#[flux_rs::sig(fn(x: bool[true]))]
pub const fn assert(_x: bool) {}

#[flux_rs::sig(fn(b:bool) ensures b)]
pub const fn assume(b: bool) {
    if !b {
        panic!("assume fails")
    }
}

// #[flux_rs::extern_spec]
// #[flux_rs::refined_by(val: int)]
// #[flux_rs::invariant(val == -1 || val == 0 || val == 1)]
// enum Ordering {
//     #[flux::variant(Ordering[-1])]
//     Less,
//     #[flux::variant(Ordering[0])]
//     Equal,
//     #[flux::variant(Ordering[1])]
//     Greater,
// }

#[flux_rs::opaque]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord)]
#[flux_rs::refined_by(ptr: int)]
pub struct FluxPtr {
    inner: *mut u8,
}

// VTOCK-TODO: fill in these functions with obvious implementations
impl FluxPtr {
    #[flux_rs::sig(fn(self: Self[@lhs], rhs: usize) -> Self{r: ((lhs + rhs <= usize::MAX) => r == lhs + rhs) && ((lhs + rhs > usize::MAX) => r == lhs + rhs - usize::MAX) })]
    pub const fn wrapping_add(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    pub const fn wrapping_sub(self, _count: usize) -> FluxPtr {
        unimplemented!()
    }

    #[flux_rs::sig(fn(self: Self[@n]) -> bool[n == 0] )]
    pub fn is_null(self) -> bool {
        unimplemented!()
    }

    #[flux_rs::sig(fn(self: Self[@n]) -> usize[n])]
    pub fn as_usize(self) -> usize {
        unimplemented!()
    }

    pub fn as_u32(self) -> u32 {
        unimplemented!()
    }

    #[flux_rs::sig(fn() -> Self[0])]
    pub const fn null() -> Self {
        unimplemented!()
    }

    #[flux_rs::sig(fn() -> Self[0])]
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
    fn partial_cmp(&self, _other: &Self) -> Option<core::cmp::Ordering>{todo!()}

    // Provided methods
    #[flux_rs::sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs < rhs])]
    fn lt(&self, _other: &Self) -> bool { todo!() }
    #[flux_rs::sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs <= rhs])]
    fn le(&self, _other: &Self) -> bool { todo!() }
    #[flux_rs::sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs > rhs])]
    fn gt(&self, _other: &Self) -> bool { todo!() }
    #[flux_rs::sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> bool[lhs >= rhs])]
    fn ge(&self, _other: &Self) -> bool { todo!() }
}

// #[flux_rs::trusted]
// impl Ord for FluxPtr {
//     #[flux_rs::sig(fn(self: &Self[@lhs], other: &Self[@rhs]) -> Ordering {
//         order: order == -1 => lhs < rhs &&
//                order == 0 => lhs == rhs && 
//                order == 1 => lhs > rhs
//     })]
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.inner.cmp(&other.inner)
//     }
// }


// #[flux_rs::sig(
//     fn(self: &Wrapper[@lhs], other: &Wrapper[@rhs]) -> Ordering{val: val == -1 => lhs < rhs && 
//                                                                      val == 0 => lhs == rhs && 
//                                                                      val == 1 => lhs > rhs}
// )]

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
#[flux_rs::refined_by(start: int, end: int)]
pub struct FluxRange {
    #[field(usize[start])]
    pub start: usize,
    #[field(usize[end])]
    pub end: usize,
}

// #[extern_spec]
// impl<T> [T] {
//     #[flux_rs::sig(fn(&[T][@n]) -> usize[n])]
//     fn len(v: &[T]) -> usize;

//     #[flux_rs::sig(fn(&[T][@n]) -> bool[n == 0])]
//     fn is_empty(v: &[T]) -> bool;
// }

// #[flux_rs::extern_spec(core::ops::range)]
// #[flux_rs::refined_by(lo: int, hi: int)]
// struct Range;

#[flux_rs::sig(fn(lhs: usize, rhs: usize) -> usize {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_usize(lhs: usize, rhs: usize) -> usize {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

#[flux_rs::sig(fn(self: FluxPtr[@lhs], other: FluxPtr[@rhs]) -> FluxPtr {r: (lhs >= rhs => r == lhs) && (rhs > lhs => r == rhs)})]
pub fn max_ptr(lhs: FluxPtr, rhs: FluxPtr) -> FluxPtr {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

