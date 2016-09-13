use core::ops::{BitOr, BitAnd};

pub fn read_volatile<T>(item: &T) -> T {
    unsafe { ::core::ptr::read_volatile(item) }
}

pub fn write_volatile<T>(item: &mut T, val: T) {
    unsafe { ::core::ptr::write_volatile(item, val) }
}

pub fn transform_volatile<F, T>(item: &mut T, f: F)
    where F: FnOnce(T) -> T
{
    let x = read_volatile(item);
    write_volatile(item, f(x))
}

#[allow(dead_code)]
pub fn volatile_bitwise_or<T: BitOr<Output = T>>(item: &mut T, val: T) {
    transform_volatile(item, |t| t | val);
}

#[allow(dead_code)]
pub fn volatile_bitwise_and<T: BitAnd<Output = T>>(item: &mut T, val: T) {
    transform_volatile(item, |t| t & val);
}

#[macro_export]
macro_rules! interrupt_handler {
    ($name: ident, $nvic: ident $(, $body: expr)*) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        #[allow(unused_imports)]
        pub unsafe extern fn $name() {
            use kernel::common::Queue;
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
