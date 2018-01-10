use tests::{blink, alarm};

pub fn print_test() {
    alarm::loop_500ms(|| {
        println!("Hello World!"); 
        blink::led_toggle(); 
    });
}

pub fn panic_test() {
    panic!("This is a kernel panic.");
}

pub fn debug_test() {
    alarm::loop_500ms(|| {
        debug!("This is a kernel debug message."); 
        blink::led_toggle(); 
    });
}
