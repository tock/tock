use core::ops::{BitOr,BitAnd};

pub fn volatile_load<T>(item: &T) -> T {
    unsafe {
        ::core::intrinsics::volatile_load(item)
    }
}

pub fn volatile_store<T>(item: &mut T, val: T) {
    unsafe {
        ::core::intrinsics::volatile_store(item, val)
    }
}

pub fn volatile_transform<F, T>(item: &mut T, f: F) where F: FnOnce(T) -> T {
    let x = volatile_load(item);
    volatile_store(item, f(x))
}

#[allow(dead_code)]
pub fn volatile_bitwise_or<T: BitOr<Output = T>>(item: &mut T, val: T) {
    volatile_transform(item, |t| { t | val });
}

#[allow(dead_code)]
pub fn volatile_bitwise_and<T: BitAnd<Output = T>>(item: &mut T, val: T) {
    volatile_transform(item, |t| { t & val });
}

