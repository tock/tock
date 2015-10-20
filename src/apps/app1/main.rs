#![crate_name = "app1rs"]
#![feature(core_str_ext,core_slice_ext,const_fn,no_std,raw,core_char_ext,unique,slice_bytes)]
#![no_std]
#![no_main]

extern crate support;

mod boxed;
#[macro_use] mod console;
mod gpio;
mod syscalls;
mod string;

mod app;

static mut app : *mut app::App = 0 as *mut app::App;

#[link_section = ".text"]
pub extern "C" fn _start(mem_start: *mut u8, mem_size: usize) {
    use core::mem::size_of;

    let myapp = unsafe {
        app = mem_start as *mut app::App;
        &mut *app
    };
    let appsize = size_of::<app::App>();
    myapp.memory = boxed::BoxMgr::new(mem_start, mem_size, appsize);

    app::init();

    loop {
        syscalls::wait();
    }
}

