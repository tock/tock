use core::fmt::Arguments;
use kernel::hil::gpio::Pin;
use cc26xx;

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(
    _args: Arguments,
    _file: &'static str,
    _line: usize,
) -> ! {
    let led0 = &cc26xx::gpio::PORT[6]; // Red led
    let led1 = &cc26xx::gpio::PORT[7]; // Green led

    led0.make_output();
    led1.make_output();
    loop {
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..100000 {
            led0.set();
            led1.set();
        }
        for _ in 0..1000000 {
            led0.clear();
            led1.clear();
        }
        for _ in 0..500000 {
            led0.set();
            led1.set();
        }
    }
}
