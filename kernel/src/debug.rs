//! Support for in-kernel debugging.
//!
//! For printing, this module uses an internal buffer to write the strings into.
//! If you are writing and the buffer fills up, you can make the size of
//! `output_buffer` larger.
//!
//! Before debug interfaces can be used, the board file must assign them hardware:
//!
//! ```rust
//! kernel::debug::assign_gpios(
//!     Some(&sam4l::gpio::PA[13]),
//!     Some(&sam4l::gpio::PA[15]),
//!     None,
//!     );
//!
//! let kc = static_init!(
//!     capsules::console::App,
//!     capsules::console::App::default());
//! kernel::debug::assign_console_driver(Some(hail.console), kc);
//! ```
//!
//! Example
//! -------
//!
//! ```rust
//! debug!("Yes the code gets here with value {}", i);
//! debug_verbose!("got here"); // includes message count, file, and line
//! debug_gpio!(0, toggle); // Toggles the first debug GPIO
//! ```
//!
//! ```
//! Yes the code gets here with value 42
//! TOCK_DEBUG(0): /tock/capsules/src/sensys.rs:24: got here
//! ```

use callback::{AppId, Callback};
use core::cmp::min;
use core::fmt::{write, Arguments, Result, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::{slice, str};
use driver::Driver;
use hil;
use mem::AppSlice;
use process;
use returncode::ReturnCode;

///////////////////////////////////////////////////////////////////
// panic! support routines

/// Tock default panic routine.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic<L: hil::led::Led, W: Write>(
    leds: &mut [&mut L],
    writer: &mut W,
    panic_info: &PanicInfo,
    nop: &Fn(),
) -> ! {
    panic_begin(nop);
    panic_banner(writer, panic_info);
    // Flush debug buffer if needed
    flush(writer);
    panic_process_info(writer);
    panic_blink_forever(leds)
}

/// Generic panic entry.
///
/// This opaque method should always be called at the beginning of a board's
/// panic method to allow hooks for any core kernel cleanups that may be
/// appropriate.
pub unsafe fn panic_begin(nop: &Fn()) {
    // Let any outstanding uart DMA's finish
    for _ in 0..200000 {
        nop();
    }
}

/// Lightweight prints about the current panic and kernel version.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_banner<W: Write>(writer: &mut W, panic_info: &PanicInfo) {
    if let Some(location) = panic_info.location() {
        let _ = writer.write_fmt(format_args!(
            "\r\n\nKernel panic at {}:{}:\r\n\t\"",
            location.file(),
            location.line()
        ));
    } else {
        let _ = writer.write_fmt(format_args!("\r\n\nKernel panic:\r\n\t\""));
    }
    if let Some(args) = panic_info.message() {
        let _ = write(writer, *args);
    }
    let _ = writer.write_str("\"\r\n");

    // Print version of the kernel
    let _ = writer.write_fmt(format_args!(
        "\tKernel version {}\r\n",
        env!("TOCK_KERNEL_VERSION")
    ));
}

/// More detailed prints about all processes.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_process_info<W: Write>(writer: &mut W) {
    // Print fault status once
    let procs = &mut process::PROCS;
    if !procs.is_empty() {
        procs[0].as_mut().map(|process| {
            process.fault_str(writer);
        });
    }

    // print data about each process
    let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
    for idx in 0..procs.len() {
        procs[idx].as_mut().map(|process| {
            process.statistics_str(writer);
        });
    }
}

/// Blinks a recognizable pattern forever.
///
/// If a multi-color LED is used for the panic pattern, it is
/// advised to turn off other LEDs before calling this method.
///
/// Generally, boards should blink red during panic if possible,
/// otherwise choose the 'first' or most prominent LED. Some
/// boards may find it appropriate to blink multiple LEDs (e.g.
/// one on the top and one on the bottom), thus this method
/// accepts an array, however most will only need one.
pub fn panic_blink_forever<L: hil::led::Led>(leds: &mut [&mut L]) -> ! {
    leds.iter_mut().for_each(|led| led.init());
    loop {
        for _ in 0..1000000 {
            leds.iter_mut().for_each(|led| led.on());
        }
        for _ in 0..100000 {
            leds.iter_mut().for_each(|led| led.off());
        }
        for _ in 0..1000000 {
            leds.iter_mut().for_each(|led| led.on());
        }
        for _ in 0..500000 {
            leds.iter_mut().for_each(|led| led.off());
        }
    }
}

// panic! support routines
///////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////
// debug_gpio! support

pub static mut DEBUG_GPIOS: (
    Option<&'static hil::gpio::Pin>,
    Option<&'static hil::gpio::Pin>,
    Option<&'static hil::gpio::Pin>,
) = (None, None, None);

pub unsafe fn assign_gpios(
    gpio0: Option<&'static hil::gpio::Pin>,
    gpio1: Option<&'static hil::gpio::Pin>,
    gpio2: Option<&'static hil::gpio::Pin>,
) {
    DEBUG_GPIOS.0 = gpio0;
    DEBUG_GPIOS.1 = gpio1;
    DEBUG_GPIOS.2 = gpio2;
}

/// In-kernel gpio debugging, accepts any GPIO HIL method
#[macro_export]
macro_rules! debug_gpio {
    ($i:tt, $method:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            $crate::debug::DEBUG_GPIOS.$i.map(|g| g.$method());
        }
    }};
}

///////////////////////////////////////////////////////////////////
// debug! and debug_verbose! support

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
    output_head: 0,       // first valid index in output_buffer
    output_tail: 0,       // one past last valid index (wraps to 0)
    output_active_len: 0, // how big is the current transaction?
    count: 0,             // how many debug! calls
};

pub unsafe fn assign_console_driver<T>(driver: Option<&'static Driver>, grant: &mut T) {
    let ptr: *mut u8 = grant as *mut T as *mut u8;
    DEBUG_WRITER.driver = driver;
    DEBUG_WRITER.grant = Some(ptr);
}

pub unsafe fn get_grant<T>() -> *mut T {
    match DEBUG_WRITER.grant {
        Some(grant) => grant as *mut T,
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
            for (dst, src) in DEBUG_WRITER.output_buffer[start..end]
                .iter_mut()
                .zip(bytes.iter())
            {
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

                    let slice = AppSlice::new(
                        self.output_buffer.as_mut_ptr().offset(start as isize),
                        end - start,
                        AppId::kernel_new(APPID_IDX),
                    );
                    let slice_len = slice.len();
                    if driver.allow(AppId::kernel_new(APPID_IDX), 1, Some(slice))
                        != ReturnCode::SUCCESS
                    {
                        panic!("Debug print allow fail");
                    }
                    write_volatile(&mut DEBUG_WRITER.output_active_len, slice_len);
                    if driver.subscribe(
                        1,
                        Some(KERNEL_CONSOLE_CALLBACK),
                        AppId::kernel_new(APPID_IDX),
                    ) != ReturnCode::SUCCESS
                    {
                        panic!("Debug print subscribe fail");
                    }
                    if driver.command(1, slice_len, 0, AppId::kernel_new(APPID_IDX))
                        != ReturnCode::SUCCESS
                    {
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
            panic!(
                "active {} bytes_written {} count {}",
                active, bytes_written, count
            );
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

static KERNEL_CONSOLE_CALLBACK: Callback =
    Callback::kernel_new(AppId::kernel_new(APPID_IDX), DebugWriter::callback);

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
            &bytes[written..]
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
                panic!(
                    "Debug buffer full. Head {} tail {} len {} active {} remaining {}",
                    head,
                    tail,
                    len,
                    active,
                    remaining_bytes.len()
                );
            }
            if remaining_bytes.len() > end - start {
                let active = unsafe { read_volatile(&DEBUG_WRITER.output_active_len) };
                panic!(
                    "Debug buffer out of room. Head {} tail {} len {} active {} remaining {}",
                    head,
                    tail,
                    len,
                    active,
                    remaining_bytes.len()
                );
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

pub fn begin_debug_fmt(args: Arguments) {
    unsafe {
        let writer = &mut DEBUG_WRITER;
        let _ = write(writer, args);
        let _ = writer.write_str("\n");
        writer.publish_str();
    }
}

pub fn begin_debug_verbose_fmt(args: Arguments, file_line: &(&'static str, u32)) {
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

/// In-kernel `println()` debugging.
#[macro_export]
macro_rules! debug {
    () => ({
        // Allow an empty debug!() to print the location when hit
        debug!("")
    });
    ($msg:expr) => ({
        $crate::debug::begin_debug_fmt(format_args!($msg))
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::debug::begin_debug_fmt(format_args!($fmt, $($arg)+))
    });
}

/// In-kernel `println()` debugging with filename and line numbers.
#[macro_export]
macro_rules! debug_verbose {
    () => ({
        // Allow an empty debug_verbose!() to print the location when hit
        debug_verbose!("")
    });
    ($msg:expr) => ({
        $crate::debug::begin_debug_verbose_fmt(format_args!($msg), {
            // TODO: Maybe make opposite choice of panic!, no `static`, more
            // runtime code for less static data
            static _FILE_LINE: (&'static str, u32) = (file!(), line!());
            &_FILE_LINE
        })
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::debug::begin_debug_verbose_fmt(format_args!($fmt, $($arg)+), {
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
        panic!(
            "No registered kernel debug printer. Thrown printing {:?}",
            buf
        );
    }
}

pub unsafe fn flush<W: Write>(writer: &mut W) {
    let debug_head = read_volatile(&DEBUG_WRITER.output_head);
    let mut debug_tail = read_volatile(&DEBUG_WRITER.output_tail);
    let mut debug_buffer = DEBUG_WRITER.output_buffer;
    if debug_head != debug_tail {
        let _ = writer.write_str(
            "\r\n---| Debug buffer not empty. Flushing. May repeat some of last message(s):\r\n",
        );

        if debug_tail > debug_head {
            let start = debug_buffer.as_mut_ptr().offset(debug_tail as isize);
            let len = debug_buffer.len();
            let slice = slice::from_raw_parts(start, len);
            let s = str::from_utf8_unchecked(slice);
            let _ = writer.write_str(s);
            debug_tail = 0;
        }
        if debug_tail != debug_head {
            let start = debug_buffer.as_mut_ptr().offset(debug_tail as isize);
            let len = debug_head - debug_tail;
            let slice = slice::from_raw_parts(start, len);
            let s = str::from_utf8_unchecked(slice);
            let _ = writer.write_str(s);
        }
    }
}
