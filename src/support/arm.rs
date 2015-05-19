use core::fmt::Arguments;
use core::intrinsics::*;

#[cfg(not(test))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
  unsafe { asm!("nop" :::: "volatile"); }
}

#[cfg(test)]
/// NOP instruction (mock)
pub fn nop() {
}

#[cfg(not(test))]
#[inline(always)]
/// WFI instruction
pub fn wfi() {
    unsafe { asm!("wfi" :::: "volatile"); }
}

#[cfg(test)]
/// WFI instruction (mock)
pub fn wfi() {
}

#[cfg(not(test))]
#[lang="stack_exhausted"]
pub extern fn stack_exhausted() {}

#[cfg(not(test))]
#[lang="eh_personality"]
pub extern fn eh_personality() {}

#[cfg(not(test))]
#[lang="begin_unwind"]
pub extern fn begin_unwind() {}

#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_unwind(_fmt: &Arguments,
    _file_line: &(&'static str, usize)) -> ! {
  loop { }
}

#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub unsafe extern fn __aeabi_unwind_cpp_pr0() {
  abort();
}

#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub unsafe extern fn __aeabi_unwind_cpp_pr1() {
  abort();
}

