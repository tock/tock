#![allow(unused)]
use std::slice::Iter;

#[flux_rs::extern_spec]
impl<T> [T] {
    #[flux_rs::sig(fn(&[T][@n]) -> usize[n])]
    fn len(v: &[T]) -> usize;

    #[flux_rs::sig(fn(&[T][@n]) -> Iter<T>[0, n])]
    fn iter(v: &[T]) -> Iter<'_, T>;

    // slice::from_raw_parts_mut
}
