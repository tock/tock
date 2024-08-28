use flux_rs::extern_spec;

// VTOCK-TODO: merge with flux_support crate once we can import flux_specs from another crate
#[extern_spec]
impl<T> [T] {
    #[flux_rs::sig(fn(&[T][@n]) -> usize[n])]
    fn len(v: &[T]) -> usize;
}

#[extern_spec(core::ptr)]
#[flux_rs::refined_by(n: int)]
struct NonNull<T>;

// #[extern_spec(core::ops::Range)]
// #[flux::refined_by(start: int, end: int)]
// struct Range<Idx> {
//     pub start: Idx,
//     pub end: Idx,
// }
