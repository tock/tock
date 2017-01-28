

use callback::{AppId, Callback};
use core::fmt::{Arguments, Result, Write, write};
use core::nonzero::NonZero;
use core::str;
use driver::Driver;
use mem::AppSlice;
use returncode::ReturnCode;

pub const APPID_IDX: usize = 255;

pub struct DebugWriter {
    driver: Option<&'static Driver>,
    pub container: Option<*mut u8>,
    output_buffer: [u8; 1024],
    output_head: usize,
    output_tail: usize,
}

static mut DEBUG_WRITER: DebugWriter = DebugWriter {
    driver: None,
    container: None,
    output_buffer: [0; 1024],
    output_head: 0,
    output_tail: 0,
};

pub unsafe fn assign_console_driver<T>(driver: Option<&'static Driver>, container: &mut T) {
    let ptr: *mut u8 = ::core::mem::transmute(container);
    DEBUG_WRITER.driver = driver;
    DEBUG_WRITER.container = Some(ptr);
}

pub unsafe fn get_container<T>() -> *mut T {
    match DEBUG_WRITER.container {
        Some(container) => ::core::mem::transmute(container),
        None => panic!("Request for unallocated kernel container"),
    }
}

impl DebugWriter {
    fn publish_str(&mut self) {
        unsafe {
            match self.driver {
                Some(driver) => {
                    /*
                    let s = match str::from_utf8(&DEBUG_WRITER.output_buffer) {
                        Ok(v) => v,
                        Err(e) => panic!("Not uf8 {}", e),
                    };
                    panic!("s: {}", s);
                    */
                    let slice = AppSlice::new(self.output_buffer.as_mut_ptr(),
                                              self.output_head - self.output_tail,
                                              AppId::kernel_new(APPID_IDX));
                    if driver.allow(AppId::kernel_new(APPID_IDX), 1, slice) != ReturnCode::SUCCESS {
                        panic!("Debug print allow fail");
                    }
                    if driver.subscribe(1, KERNEL_CONSOLE_CALLBACK) != ReturnCode::SUCCESS {
                        panic!("Debug print subscribe fail");
                    }
                }
                None => {
                    panic!("Platform has not yet configured kernel debug interface");
                }
            }
        }
    }
    fn callback() {
        unimplemented!();
    }
}

//XXX http://stackoverflow.com/questions/28116147
// I think this is benign and needed because NonZero's assuming threading in an
// inappropriate way?
unsafe impl Sync for Callback {}

pub static KERNEL_CONSOLE_CALLBACK: Callback = Callback {
    app_id: AppId::kernel_new(APPID_IDX),
    appdata: 0,
    fn_ptr: unsafe { NonZero::new(DebugWriter::callback as *mut ()) },
};

impl Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> Result {
        unsafe {
            let bytes = s.as_bytes();
            if (DEBUG_WRITER.output_buffer.len() - DEBUG_WRITER.output_head) < bytes.len() {
                panic!("Debug buffer full");
            }
            let start = DEBUG_WRITER.output_head;
            let end = DEBUG_WRITER.output_head + bytes.len();
            for (dst, src) in DEBUG_WRITER.output_buffer[start..end].iter_mut().zip(bytes.iter()) {
                *dst = *src;
            }
            DEBUG_WRITER.output_head += bytes.len();
            Ok(())
        }
    }
}

pub unsafe fn begin_debug_fmt(args: Arguments, file_line: &(&'static str, u32)) {
    let writer = &mut DEBUG_WRITER;
    let (file, line) = *file_line;
    let _ = writer.write_fmt(format_args!("TOCK_DEBUG: {}:{}: ", file, line));
    let _ = write(writer, args);
    let _ = writer.write_str("\r\n");
    writer.publish_str();
}

pub unsafe fn begin_debug(msg: &str, file_line: &(&'static str, u32)) {
    let writer = &mut DEBUG_WRITER;
    let (file, line) = *file_line;
    let _ = writer.write_fmt(format_args!("TOCK_DEBUG: {}:{}: ", file, line));
    let _ = writer.write_fmt(format_args!("{}\r\n", msg));
    writer.publish_str();
}

macro_rules! debug {
    () => ({
        // Allow an empty debug!() to print the location when hit
        debug!("")
    });
    ($msg:expr) => ({
        $crate::debug::begin_debug($msg, {
            // TODO: Maybe make opposite choice of panic!, no `static`, more
            // runtime code for less static data
            static _FILE_LINE: (&'static str, u32) = (file!(), line!());
            &_FILE_LINE
        })
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::debug::begin_debug_fmt(format_args!($fmt, $($arg)+), {
            static _FILE_LINE: (&'static str, u32) = (file!(), line!());
            &_FILE_LINE
        })
    });
}

pub trait Debug {
    fn write(&self, buf: &'static mut [u8], len: usize);
}

#[cfg(debug = "true")]
impl Default for Debug {
    fn write(&self, buf: &'static mut [u8], len: usize) {
        panic!("No registered kernel debug printer. Thrown printing {:?}",
               buf);
    }
}
