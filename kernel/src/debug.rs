//! Provides a `debug!` macro for in-kernel debugging.
//!
//! This module uses an internal buffer to write the strings into. If you are
//! writing and the buffer fills up, you can make the size of `output_buffer`
//! larger.
//!
//! Example
//! -------
//!
//! ```rust
//! debug!("Yes the code gets here with value {}", i);
//! ```

use callback::{AppId, Callback};
use core::cmp::min;
use core::fmt::{Arguments, Result, Write, write};
use core::ptr::{read_volatile, write_volatile};
use core::str;
use driver::Driver;
use mem::AppSlice;
use returncode::ReturnCode;

pub const APPID_IDX: usize = 255;
const BUF_SIZE: usize = 1024;

pub struct DebugWriter {
    driver: Option<&'static Driver>,
    pub grant: Option<*mut u8>,
    output_buffer: [u8; BUF_SIZE],
    output_head: usize,
    output_tail: usize,
    output_active_len: usize,
    count: usize,
}

static mut DEBUG_WRITER: DebugWriter = DebugWriter {
    driver: None,
    grant: None,
    output_buffer: [0; BUF_SIZE],
    output_head: 0, // ........ first valid index in output_buffer
    output_tail: 0, // ........ one past last valid index (wraps to 0)
    output_active_len: 0, //... how big is the current transaction?
    count: 0, // .............. how many debug! calls
};

pub unsafe fn assign_console_driver<T>(driver: Option<&'static Driver>, grant: &mut T) {
    let ptr: *mut u8 = ::core::mem::transmute(grant);
    DEBUG_WRITER.driver = driver;
    DEBUG_WRITER.grant = Some(ptr);
}

pub unsafe fn get_grant<T>() -> *mut T {
    match DEBUG_WRITER.grant {
        Some(grant) => ::core::mem::transmute(grant),
        None => panic!("Request for unallocated kernel grant"),
    }
}

impl DebugWriter {
    /// Convenience method that writes (end-start) bytes from bytes into the debug buffer
    fn write_buffer(start: usize, end: usize, bytes: &[u8]) {
        unsafe {
            if end < start {
                panic!("wb bounds: start {} end {} bytes {:?}", start, end, bytes);
            }
            for (dst, src) in DEBUG_WRITER.output_buffer[start..end].iter_mut().zip(bytes.iter()) {
                *dst = *src;
            }
        }
    }

    fn publish_str(&mut self) {
        unsafe {
            if read_volatile(&self.output_active_len) != 0 {
                // Cannot publish now, there is already an outstanding request
                // the callback will call publish_str again to finish
                return;
            }

            match self.driver {
                Some(driver) => {
                    /*
                    let s = match str::from_utf8(&DEBUG_WRITER.output_buffer) {
                        Ok(v) => v,
                        Err(e) => panic!("Not uf8 {}", e),
                    };
                    panic!("s: {}", s);
                    */
                    let head = read_volatile(&self.output_head);
                    let tail = read_volatile(&self.output_tail);
                    let len = self.output_buffer.len();

                    // Want to write everything from tail inclusive to head
                    // exclusive
                    let (start, end) = if tail > head {
                        // Need to pass subscribe a contiguous buffer, so first
                        // write from tail to end of buffer. The completion
                        // callback will see that the buffer's not empty and
                        // call again to write the rest (tail will be 0)
                        let start = tail;
                        let end = len;
                        (start, end)
                    } else if tail < head {
                        let start = tail;
                        let end = head;
                        (start, end)
                    } else {
                        panic!("Consistency error: publish empty buffer?")
                    };

                    let slice =
                        AppSlice::new(self.output_buffer.as_mut_ptr().offset(start as isize),
                                      end - start,
                                      AppId::kernel_new(APPID_IDX));
                    let slice_len = slice.len();
                    if driver.allow(AppId::kernel_new(APPID_IDX), 1, slice) != ReturnCode::SUCCESS {
                        panic!("Debug print allow fail");
                    }
                    write_volatile(&mut DEBUG_WRITER.output_active_len, slice_len);
                    if driver.subscribe(1, KERNEL_CONSOLE_CALLBACK) != ReturnCode::SUCCESS {
                        panic!("Debug print subscribe fail");
                    }
                    if driver.command(1, slice_len, 0, AppId::kernel_new(APPID_IDX)) !=
                       ReturnCode::SUCCESS {
                        panic!("Debug print command fail");
                    }
                }
                None => {
                    panic!("Platform has not yet configured kernel debug interface");
                }
            }
        }
    }
    fn callback(bytes_written: usize, _: usize, _: usize, _: usize) {
        let active = unsafe { read_volatile(&DEBUG_WRITER.output_active_len) };
        if active != bytes_written {
            let count = unsafe { read_volatile(&DEBUG_WRITER.count) };
            panic!("active {} bytes_written {} count {}",
                   active,
                   bytes_written,
                   count);
        }
        let len = unsafe { DEBUG_WRITER.output_buffer.len() };
        let head = unsafe { read_volatile(&DEBUG_WRITER.output_head) };
        let mut tail = unsafe { read_volatile(&DEBUG_WRITER.output_tail) };
        tail = tail + bytes_written;
        if tail > len {
            tail = tail - len;
        }

        if head == tail {
            // Empty. As an optimization, reset the head and tail pointers to 0
            // to maximize the buffer length available before fragmentation
            unsafe {
                write_volatile(&mut DEBUG_WRITER.output_active_len, 0);
                write_volatile(&mut DEBUG_WRITER.output_head, 0);
                write_volatile(&mut DEBUG_WRITER.output_tail, 0);
            }
        } else {
            // Buffer not empty, go around again
            unsafe {
                write_volatile(&mut DEBUG_WRITER.output_active_len, 0);
                write_volatile(&mut DEBUG_WRITER.output_tail, tail);
                DEBUG_WRITER.publish_str();
            }
        }
    }
}

//XXX http://stackoverflow.com/questions/28116147
// I think this is benign and needed because NonZero's assuming threading in an
// inappropriate way?
unsafe impl Sync for Callback {}

static KERNEL_CONSOLE_CALLBACK: Callback = Callback::kernel_new(AppId::kernel_new(APPID_IDX),
                                                                DebugWriter::callback);

impl Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> Result {
        // Circular buffer.
        //
        // Note, we don't use the kernel's RingBuffer here because we want
        // slightly different semantics. Specifically, we need to be able
        // to take *contiguous* slices of the buffer and pass them around,
        // we're also okay with fragmenting if we're inserting a slice over
        // the internal wraparound, but we need to handle that case manually
        //
        //  - head points to the index of the first valid place to write
        //  - tail points one past the index of the last open place to write
        //  -> head == tail implies buffer is empty
        //  -> there's no "full/empty" bit, so the effective buffer size is -1

        let mut head = unsafe { read_volatile(&DEBUG_WRITER.output_head) };
        let tail = unsafe { read_volatile(&DEBUG_WRITER.output_tail) };
        let len = unsafe { DEBUG_WRITER.output_buffer.len() };

        let remaining_bytes = if head >= tail {
            let bytes = s.as_bytes();

            // First write from current head to end of buffer in memory
            let mut backside_len = len - head;
            if tail == 0 {
                // Handle special case where tail has just wrapped to 0,
                // so we can't let the head point to 0 as well
                backside_len -= 1;
            }

            let written = if backside_len != 0 {
                let start = head;
                let end = head + backside_len;
                DebugWriter::write_buffer(start, end, bytes);
                min(end - start, bytes.len())
            } else {
                0
            };

            // Advance and possibly wrap the head
            head += written;
            if head == len {
                head = 0;
            }

            let remaining_bytes = &bytes[written..];

            remaining_bytes
        } else {
            s.as_bytes()
        };

        // At this point, either
        //  o head < tail
        //  o head = len-1, tail = 0 (buffer full edge case)
        //  o there are no more bytes to write

        if remaining_bytes.len() != 0 {
            // Now write from the head up to tail
            let start = head;
            let end = tail;
            if (tail == 0) && (head == len - 1) {
                let active = unsafe { read_volatile(&DEBUG_WRITER.output_active_len) };
                panic!("Debug buffer full. Head {} tail {} len {} active {} remaining {}",
                       head,
                       tail,
                       len,
                       active,
                       remaining_bytes.len());
            }
            if remaining_bytes.len() > end - start {
                let active = unsafe { read_volatile(&DEBUG_WRITER.output_active_len) };
                panic!("Debug buffer out of room. Head {} tail {} len {} active {} remaining {}",
                       head,
                       tail,
                       len,
                       active,
                       remaining_bytes.len());
            }
            DebugWriter::write_buffer(start, end, remaining_bytes);
            let written = min(end - start, remaining_bytes.len());

            // head cannot wrap here
            head += written;
        }

        unsafe {
            write_volatile(&mut DEBUG_WRITER.output_head, head);
        }

        Ok(())
    }
}

pub fn begin_debug_fmt(args: Arguments, file_line: &(&'static str, u32)) {
    unsafe {
        let count = read_volatile(&DEBUG_WRITER.count);
        write_volatile(&mut DEBUG_WRITER.count, count + 1);

        let writer = &mut DEBUG_WRITER;
        let (file, line) = *file_line;
        let _ = writer.write_fmt(format_args!("TOCK_DEBUG({}): {}:{}: ", count, file, line));
        let _ = write(writer, args);
        let _ = writer.write_str("\n");
        writer.publish_str();
    }
}

pub fn begin_debug(msg: &str, file_line: &(&'static str, u32)) {
    unsafe {
        let count = read_volatile(&DEBUG_WRITER.count);
        write_volatile(&mut DEBUG_WRITER.count, count + 1);

        let writer = &mut DEBUG_WRITER;
        let (file, line) = *file_line;
        let _ = writer.write_fmt(format_args!("TOCK_DEBUG({}): {}:{}: ", count, file, line));
        let _ = writer.write_fmt(format_args!("{}\n", msg));
        writer.publish_str();
    }
}

/// In-kernel `printf()` debugging.
#[macro_export]
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
