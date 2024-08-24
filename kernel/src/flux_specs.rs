// VTOCK-TODO: merge with flux_support crate once we can import flux_specs from another crate
#[flux_rs::extern_spec]
impl<T> [T] {
    #[flux_rs::sig(fn(&[T][@n]) -> usize[n])]
    fn len(v: &[T]) -> usize;
}


