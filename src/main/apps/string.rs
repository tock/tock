use core::fmt::{self, Write};
use super::boxed::{Box, uninitialized_box_slice};

pub struct String { pub bx: Option<Box<&'static mut [u8]>> }

impl String {
    pub fn new(x: &str) -> String {
        if x.len() == 0 {
            return String { bx: None };
        }
        unsafe {
            let mut slice = uninitialized_box_slice(x.len());
            for (i,c) in x.bytes().enumerate() {
                slice[i] = c;
            }
            String { bx: Some(slice) }
        }
    }

    pub fn len(&self) -> usize {
        self.bx.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn as_str(&self) -> &str {
        use core::mem;
        use core::raw::Repr;

        match self.bx.as_ref() {
            Some(bx) => unsafe {
                mem::transmute((*bx.raw()).repr())
            },
            None => ""
        }
    }
}

impl Write for String {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let charlen = c.len_utf8();
        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + charlen) };
        
        self.bx.as_ref().map(|bx| {
            for (i, c) in bx.iter().enumerate() {
                newbox[i] = *c;
            }
        });

        match charlen {
            1 => newbox[oldlen] = c as u8,
            _ => {
                c.encode_utf8(&mut newbox[oldlen..]);
            }
        }

        self.bx = Some(newbox);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + s.len()) };

        self.bx.as_ref().map(|bx| {
            for (i, c) in bx.iter().enumerate() {
                newbox[i] = *c;
            }
        });

        for (i,c) in s.bytes().enumerate() {
            newbox[i + oldlen] = c;
        }

        self.bx = Some(newbox);

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

