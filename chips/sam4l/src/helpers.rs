
/// Define a function for an interrupt and enqueue the interrupt on the global
/// queue.
#[macro_export]
macro_rules! interrupt_handler {
    ($name: ident, $nvic: ident $(, $body: expr)*) => {
    }
}
