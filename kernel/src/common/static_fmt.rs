//! A macro static_fmt! that provides a small amount of static storage
//! for formatting a string.  The value of the macro has type `&'static str`.
//! Each invocation overwrites what was written previously.

use core;
use core::fmt::{Write, Result};

const STORAGE_SIZE: usize = 500;

#[derive(Copy, Clone)]
pub struct StaticCursor {
    buf: &'static [u8],
}

impl StaticCursor {
    pub fn new() -> Self {
        StaticCursor{
            buf: b""
        }
    }
}

impl StaticCursor {
    pub fn as_str(&self) -> &'static str {
        unsafe {
            core::str::from_utf8_unchecked(self.buf)
        }
    }
}

impl Write for StaticCursor {
    fn write_str(&mut self, s: &str) -> Result {
        unsafe {
            static mut BUF: [u8; STORAGE_SIZE] = [b'X'; STORAGE_SIZE];

            let mut len = self.buf.len();

            let sb = s.as_bytes();
            if len + sb.len() > STORAGE_SIZE {
                panic!("static_fmt: overflow");
            }

            let buf = &mut BUF[ len .. ];
            for (i, b) in sb.iter().enumerate() {
                buf[i] = *b;
            }
            len += sb.len();

            self.buf = &BUF[ .. len ];
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! static_fmt {
    ($fmt:expr, $($arg:tt)+) => ({
        use core::fmt::Write;

        let mut d = $crate::common::static_fmt::StaticCursor::new();
        write!(d, $fmt, $($arg)+).unwrap();
        d.as_str()
    });
}
