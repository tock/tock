use core::fmt::Arguments;

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(
    _args: Arguments,
    _file: &'static str,
    _line: usize,
) -> ! {
    loop {}
}
