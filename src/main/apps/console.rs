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
    unsafe {
        subscribe(0, 1, write_done as usize,
                  mem::transmute(buf.bx.unwrap().raw()));
    }
}

pub fn puts(string: &str) {
    let bstr = String::new(string);
    allow(0, 1, string as *const str as *mut (), string.len());
    unsafe {
        subscribe(0, 1, write_done as usize,
                  mem::transmute(bstr.bx.unwrap().raw()));
    }
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

