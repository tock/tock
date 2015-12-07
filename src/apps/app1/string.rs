use core::fmt::{self, Write};
use super::boxed::{Box, uninitialized_box_slice};

pub struct String { bx: Box<&'static mut [u8]> }

impl String {
    pub fn new(x: &str) -> String {
        if x.len() == 0 {
            return String { bx: Box::new(&mut []) };
        }
        unsafe {
            let mut slice = uninitialized_box_slice(x.len());
            for (i,c) in x.bytes().enumerate() {
                slice[i] = c;
            }
            String { bx: slice }
        }
    }

    pub fn len(&self) -> usize {
        self.bx.len()
    }

    pub fn as_str(&self) -> &str {
        use core::mem;
        use core::raw::Repr;

        unsafe {
            mem::transmute((*self.bx.raw()).repr())
        }
    }
}

impl Write for String {
    fn write_char(&mut self, c: char) -> fmt::Result {
        use core::slice::bytes::copy_memory;
        let charlen = c.len_utf8();
        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + charlen) };
        
        copy_memory(self.as_str().as_bytes(), &mut newbox[..oldlen]);

        match charlen {
            1 => newbox[oldlen] = c as u8,
            _ => {
                c.encode_utf8(&mut newbox[oldlen..]);
            }
        }

        self.bx = newbox;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        use core::slice::bytes::copy_memory;

        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + s.len()) };

        copy_memory(self.as_str().as_bytes(), &mut newbox[..oldlen]);
        copy_memory(s.as_bytes(), &mut newbox[oldlen..]);

        self.bx = newbox;

        Ok(())
    }
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl<T: fmt::Display + ?Sized> ToString for T {
    fn to_string(&self) -> String {
        use core::fmt::Write;
        let mut buf = String::new("");
        let _ = buf.write_fmt(format_args!("{}", self));
        buf
    }
}

