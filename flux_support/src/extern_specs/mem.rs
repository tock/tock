// alignment of data types must be at least 0:
// https://doc.rust-lang.org/reference/type-layout.html
#[flux_rs::extern_spec(core::mem)]
#[flux_rs::sig(fn<T>() -> usize{align: align > 0})]
fn align_of<T>() -> usize;
