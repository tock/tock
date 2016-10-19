#![no_std]
#![no_main]
#![feature(const_fn,lang_items)]

extern crate kernel;
extern crate sam4l;

mod io;

#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    panic!("foo");
}
