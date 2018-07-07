//! "Architecture"-specific methods for running on native (host) machine.

#![crate_name = "tock_native_arch"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, naked_functions)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio)]
extern crate kernel;

//pub mod mpu;
//pub mod nvic;
//pub mod scb;
//pub mod systick;

/// NOP instruction (mock)
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn systick_handler() {}

#[cfg(target_os = "none")]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    unimplemented!("systick");
}

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn generic_isr() {}

#[cfg(target_os = "none")]
#[naked]
/// All ISRs are caught by this handler which disables the NVIC and switches to the kernel.
pub unsafe extern "C" fn generic_isr() {
    unimplemented!("generic_isr");
}

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn svc_handler() {}

#[cfg(target_os = "none")]
#[naked]
pub unsafe extern "C" fn svc_handler() {
    unimplemented!("svc_handler");
}

#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_got: *const u8,
) -> *mut u8 {
    unimplemented!("switch_to_user");
}

#[cfg(target_os = "none")]
#[no_mangle]
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const u8,
    process_regs: &mut [usize; 8],
) -> *mut u8 {
    unimplemented!("switch_to_user");
}
