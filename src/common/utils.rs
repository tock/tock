use core::array::FixedSizeArray;
use core::mem::uninitialized;

/// Initializes a fixed size array of type Option<T>.
///
/// This is equivilant to typing out 
pub fn init_nones<T, A: FixedSizeArray<Option<T>>>() -> A {
    let mut res : A = unsafe { uninitialized() };
    for elm in res.as_mut_slice().iter_mut() {
        *elm = None;
    }
    res
}

