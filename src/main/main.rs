#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate support;
extern crate hil;
extern crate platform;

#[no_mangle]
pub extern fn main() {
    let platform = unsafe {
        platform::init()
    };
    loop {
        unsafe {
            platform.service_pending_interrupts();
        }
        unsafe {
            support::atomic(|| {
                if !platform.has_pending_interrupts() {
                    support::wfi();
                }
            })
        };
    }
}

