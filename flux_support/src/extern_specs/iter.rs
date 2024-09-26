#![allow(unused)]
use crate::assert;
use std::slice::Iter;

#[flux_rs::extern_spec(std::slice)]
#[flux_rs::refined_by(idx: int, len: int)]
struct Iter<'a, T>;

// #[flux_rs::extern_spec(std::iter)]
// #[flux_rs::refined_by(idx: int, inner: I)]
// struct Enumerate<I>;

// #[flux_rs::extern_spec(std::iter)]
// #[flux_rs::generics(Self as base)]
// #[flux_rs::assoc(fn done(self: Self) -> bool )]
// #[flux_rs::assoc(fn step(self: Self, other: Self) -> bool )]
// trait Iterator {
//     #[flux_rs::sig(fn(self: &strg Self[@curr_s]) -> Option<Self::Item>[!<Self as Iterator>::done(curr_s)] ensures self: Self{next_s: <Self as Iterator>::step(curr_s, next_s)})]
//     fn next(&mut self) -> Option<Self::Item>;

//     #[flux_rs::sig(fn(Self[@s]) -> Enumerate<Self>[0, s])]
//     fn enumerate(self) -> Enumerate<Self>
//     where
//         Self: Sized;
// }

// #[flux_rs::extern_spec(std::slice)]
// #[flux_rs::generics(T as base)]
// #[flux_rs::assoc(fn done(x: Iter<T>) -> bool { x.idx >= x.len })]
// #[flux_rs::assoc(fn step(x: Iter<T>, y: Iter<T>) -> bool { x.idx + 1 == y.idx && x.len == y.len})]
// impl<'a, T> Iterator for Iter<'a, T> {
//     #[flux_rs::sig(fn(self: &strg Iter<T>[@curr_s]) -> Option<_>[curr_s.idx < curr_s.len] ensures self: Iter<T>{next_s: curr_s.idx + 1 == next_s.idx && curr_s.len == next_s.len})]
//     fn next(&mut self) -> Option<&'a T>;
// }

// #[flux_rs::extern_spec(std::iter)]
// #[flux_rs::generics(I as base)]
// #[flux_rs::assoc(fn done(x: Enumerate<I>) -> bool { <I as Iterator>::done(x.inner)})]
// #[flux_rs::assoc(fn step(x: Enumerate<I>, y: Enumerate<I>) -> bool { <I as Iterator>::step(x.inner, y.inner)})]
// impl<I: Iterator> Iterator for Enumerate<I> {
//     // #[flux_rs::sig(fn(self: &strg Enumerate<I>[@curr_s]) -> Option<(usize[curr_s.idx], _)>[curr_s.idx < curr_s.len] ensures self: Enumerate<I>{next_s: curr_s.idx + 1 == next_s.idx && curr_s.len == next_s.len})]
//     #[flux_rs::sig(fn(self: &strg Enumerate<I>[@curr_s]) -> Option<(usize[curr_s.idx], _)>[!<I as Iterator>::done(curr_s.inner)]
//     ensures self: Enumerate<I>{next_s: curr_s.idx + 1 == next_s.idx && <I as Iterator>::step(curr_s.inner, next_s.inner)})]
//     fn next(&mut self) -> Option<(usize, <I as Iterator>::Item)>;
// }

// // Helper functions for inspecting indexes of `Iter`
// #[flux_rs::trusted]
// #[flux_rs::sig(fn(&Iter<T>[@idx, @len]) -> usize[len])]
// fn get_iter_len<'a, T>(iter: &Iter<'a, T>) -> usize {
//     unimplemented!()
// }

// #[flux_rs::trusted]
// #[flux_rs::sig(fn(&Iter<T>[@idx, @len]) -> usize[idx])]
// fn get_iter_idx<'a, T>(iter: &Iter<'a, T>) -> usize {
//     unimplemented!()
// }

// #[flux_rs::sig(fn(slice: &[u8]{n: n > 0}))]
// fn test_iter1(slice: &[u8]) {
//     let mut iter = slice.iter();
//     let next = iter.next();
//     assert(next.is_some());
// }

// // TODO: I should be able to prove r == n, but I can't
// #[flux_rs::sig(fn(slice: &[u8][@n]) -> usize{r: r >= n})]
// fn test_iter2(slice: &[u8]) -> usize {
//     let mut ctr = 0_usize;
//     let mut iter = slice.iter();
//     while let Some(_) = iter.next() {
//         assert(ctr < slice.len());
//         ctr += 1;
//     }
//     ctr
// }

// #[flux_rs::should_fail]
// #[flux_rs::sig(fn(slice: &[u8]{n: n > 0}))]
// fn test_iter1_neg(slice: &[u8]) {
//     assert(slice.len() > 0);
//     let mut iter = slice.iter();
//     let next = iter.next();
//     assert(next.is_some());
//     assert(iter.next().is_some());
// }

// #[flux_rs::sig(fn(slice: &[u8]{n: n > 1}))]
// fn test_enumerate1(slice: &[u8]) {
//     assert(slice.len() > 0);
//     let mut enumer = slice.iter().enumerate();

//     let next = enumer.next();
//     assert(next.is_some());
//     let (idx, _) = next.unwrap();
//     assert(idx == 0);

//     let next_next = enumer.next();
//     assert(next_next.is_some());
//     let (idx, _) = next_next.unwrap();
//     assert(idx == 1);
// }

// #[flux_rs::sig(fn(&[usize][1]) )]
// pub fn test_enumer2(slice: &[usize]) {
//     assert(slice.len() == 1);
//     let mut enumer = slice.iter().enumerate();

//     let next = enumer.next();
//     assert(next.is_some());

//     let next_next = enumer.next();
//     assert(next_next.is_none())
// }

// #[flux_rs::sig(fn(&[usize][@n]) )]
// pub fn test_enumer3(slice: &[usize]) {
//     let mut e = slice.iter().enumerate();
//     while let Some((idx, _)) = e.next() {
//         assert(idx < slice.len())
//     }
// }

// #[flux_rs::sig(fn(&[usize][@len]) -> Option<usize{r: r < len}> )]
// pub fn find_index_of_3(slice: &[usize]) -> Option<usize> {
//     let mut e = slice.iter().enumerate();
//     while let Some((idx, num)) = e.next() {
//         if num == &3 {
//             return Some(idx);
//         }
//     }
//     None
// }

// TODO: implement IntoIter so I can use these with `for` loops
