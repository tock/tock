#![allow(unused)]
use core::slice::{self, Iter, SliceIndex};

// #[flux_rs::extern_spec]
// #[generics(Self as base, T as base)]
// #[assoc(fn in_bounds(idx: Self, v: T) -> bool)]
// trait SliceIndex<T>
// where
//     T: ?Sized,
// {
// }

// #[flux_rs::extern_spec]
// #[assoc(fn in_bounds(idx: int, len: int) -> bool {idx < len} )]
// impl<T> SliceIndex<[T]> for usize {}

#[flux_rs::extern_spec]
impl<T> [T] {
    #[flux_rs::sig(fn(&[T][@len]) -> usize[len])]
    fn len(v: &[T]) -> usize;

    #[flux_rs::sig(fn(&[T][@len]) -> Iter<T>[0, len])]
    fn iter(v: &[T]) -> Iter<'_, T>;

    // #[flux_rs::generics(I as base)]
    // #[flux_rs::sig(fn(&[T][@len], I[@idx]) -> Option<_>[<I as SliceIndex<[T]>>::in_bounds(idx, len)])]
    // fn get(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>;
}
