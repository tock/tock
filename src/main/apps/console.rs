use core::mem;
use super::boxed::Box;
use super::syscalls::*;
use super::string::String;

const WRITE_DONE_TOKEN : isize = 0xdeadbeef;

fn write_done(_: usize, _: usize, _: usize, _todrop: Box<&'static [u8]>) -> isize {
    WRITE_DONE_TOKEN
}

macro_rules! print {
    ($str:expr) => (::apps::console::puts($str));
    ($fmt:expr, $($arg:tt)*) => (::apps::console::print(format_args!($fmt, $($arg)*)));
}


pub fn print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    let mut buf = String::new("");
    let _ = buf.write_fmt(args);
    allow(0, 1, buf.as_str() as *const str as *mut (), buf.len());
    let str_ptr = unsafe { mem::transmute(buf.bx.as_mut().unwrap().raw()) };
    mem::forget(buf);
    subscribe(0, 1, write_done as usize, str_ptr);
    while wait() != WRITE_DONE_TOKEN {}
}

pub fn puts(string: &str) {
    let mut buf = String::new(string);
    allow(0, 1, buf.as_str() as *const str as *mut (), string.len());
    let str_ptr = unsafe { mem::transmute(buf.bx.as_mut().unwrap().raw()) };
    mem::forget(buf);
    subscribe(0, 1, write_done as usize, str_ptr);
    while wait() != WRITE_DONE_TOKEN {}
}

pub fn putc(c: u8) {
    command(0, 0, c as usize);
}

pub fn subscribe_read_line(buf: *mut u8, len: usize,
                           f: fn(usize, *mut u8)) -> isize {
    let res =  allow(0, 0, buf as *mut (), len);
    if res < 0 {
        res
    } else {
        subscribe(0, 0, f as usize, 0)
    }
}

