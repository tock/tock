use core::mem;
use super::syscalls::*;
use super::string::String;

pub fn write_done() {

}

pub fn print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    let mut buf = String::new("");
    let _ = buf.write_fmt(args);
    allow(0, 1, buf.as_str() as *const str as *mut (), buf.len());
    let str_ptr = unsafe { mem::transmute(buf.bx.unwrap().raw()) };
    subscribe(0, 1, write_done as usize, str_ptr);
}

pub fn puts(string: &str) {
    let buf = String::new(string);
    allow(0, 1, buf.as_str() as *const str as *mut (), string.len());
    let str_ptr = unsafe { mem::transmute(buf.bx.unwrap().raw()) };
    subscribe(0, 1, write_done as usize, str_ptr);
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

