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

#[macro_export]
macro_rules! interrupt_handler {
    ($name: ident, $nvic: ident $(, $body: expr)*) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        #[allow(unused_imports)]
        pub unsafe extern fn $name() {
            use common::Queue;
            use chip;

            $({
                $body
            })*

            let nvic = nvic::NvicIdx::$nvic;
            nvic::disable(nvic);
            chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nvic);
        }
    }
}

