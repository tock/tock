
/// Define a function for an interrupt and enqueue the interrupt on the global
/// queue.
//  TODO: Common functionality. Maybe a better place for this?
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
