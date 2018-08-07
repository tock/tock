//! Support for in-kernel debugging.
//!
//! For printing, this module uses an internal buffer to write the strings into.
//! If you are writing and the buffer fills up, you can make the size of
//! `output_buffer` larger.
//!
//! Before debug interfaces can be used, the board file must assign them hardware:
//!
//! ```ignore
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
//! ```no_run
//! # #[macro_use] extern crate kernel;
//! # fn main() {
//! # let i = 42;
//! debug!("Yes the code gets here with value {}", i);
//! debug_verbose!("got here"); // includes message count, file, and line
//! debug_gpio!(0, toggle); // Toggles the first debug GPIO
//! # }
//! ```
//!
//! ```text
//! Yes the code gets here with value 42
//! TOCK_DEBUG(0): /tock/capsules/src/sensys.rs:24: got here
//! ```

use core::cell::Cell;
use core::cmp::{self, min};
use core::fmt::{write, Arguments, Result, Write};
use core::panic::PanicInfo;
use core::ptr;
use core::slice;
use core::str;

use common::cells::NumericCellExt;
use common::cells::{MapCell, TakeCell};
use hil;
use process::Process;

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
    processes: &'static [Option<&'static Process<'static>>],
) -> ! {
    panic_begin(nop);
    panic_banner(writer, panic_info);
    // Flush debug buffer if needed
    flush(writer);
    panic_process_info(processes, writer);
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
pub unsafe fn panic_process_info<W: Write>(
    procs: &'static [Option<&'static Process<'static>>],
    writer: &mut W,
) {
    // Print fault status once
    if !procs.is_empty() {
        procs[0].as_ref().map(|process| {
            process.fault_str(writer);
        });
    }

    // print data about each process
    let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
    for idx in 0..procs.len() {
        procs[idx].as_ref().map(|process| {
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

/// Wrapper type that we need a mutable reference to for the core::fmt::Write
/// interface.
pub struct DebugWriterWrapper {
    dw: MapCell<&'static DebugWriter>,
}

/// Main type that we need an immutable reference to so we can share it with
/// the UART provider and this debug module.
pub struct DebugWriter {
    // What provides the actual writing mechanism.
    uart: &'static hil::uart::UART,
    // The buffer that is passed to the writing mechanism.
    output_buffer: TakeCell<'static, [u8]>,
    // An internal buffer that is used to hold debug!() calls as they come in.
    internal_buffer: TakeCell<'static, [u8]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    // How many bytes are being written on the current publish_str call.
    active_len: Cell<usize>,
    // Number of debug!() calls.
    count: Cell<usize>,
}

/// Static variable that holds the kernel's reference to the debug tool. This is
/// needed so the debug!() macros have a reference to the object to use.
static mut DEBUG_WRITER: Option<&'static mut DebugWriterWrapper> = None;

pub static mut OUTPUT_BUF: [u8; 64] = [0; 64];
pub static mut INTERNAL_BUF: [u8; 1024] = [0; 1024];

pub unsafe fn get_debug_writer() -> &'static mut DebugWriterWrapper {
    match ptr::read(&DEBUG_WRITER) {
        Some(x) => x,
        None => panic!("Must call `set_debug_writer_wrapper` in board initialization."),
    }
}

/// Function used by board main.rs to set a reference to the writer.
pub unsafe fn set_debug_writer_wrapper(debug_writer: &'static mut DebugWriterWrapper) {
    DEBUG_WRITER = Some(debug_writer);
}

impl DebugWriterWrapper {
    pub fn new(dw: &'static DebugWriter) -> DebugWriterWrapper {
        DebugWriterWrapper {
            dw: MapCell::new(dw),
        }
    }
}

impl DebugWriter {
    pub fn new(
        uart: &'static hil::uart::UART,
        out_buffer: &'static mut [u8],
        internal_buffer: &'static mut [u8],
    ) -> DebugWriter {
        DebugWriter {
            uart: uart,
            output_buffer: TakeCell::new(out_buffer),
            internal_buffer: TakeCell::new(internal_buffer),
            head: Cell::new(0),       // first valid index in output_buffer
            tail: Cell::new(0),       // one past last valid index (wraps to 0)
            active_len: Cell::new(0), // how big is the current transaction?
            count: Cell::new(0),      // how many debug! calls
        }
    }

    fn increment_count(&self) {
        self.count.increment();
    }

    fn get_count(&self) -> usize {
        self.count.get()
    }

    /// Convenience method that writes (end-start) bytes from bytes into the
    /// internal debug buffer.
    fn write_buffer(&self, start: usize, end: usize, bytes: &[u8]) {
        if end < start {
            panic!("wb bounds: start {} end {} bytes {:?}", start, end, bytes);
        }
        self.internal_buffer.map(|in_buffer| {
            for (dst, src) in in_buffer[start..end].iter_mut().zip(bytes.iter()) {
                *dst = *src;
            }
        });
    }

    /// Write as many of the bytes from the internal_buffer to the output
    /// mechanism as possible.
    fn publish_str(&self) {
        // Can only publish if we have the output_buffer. If we don't that is
        // fine, we will do it when the transmit done callback happens.
        self.output_buffer.take().map(|out_buffer| {
            let head = self.head.get();
            let tail = self.tail.get();
            let len = self
                .internal_buffer
                .map_or(0, |internal_buffer| internal_buffer.len());

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

            // Check that we aren't writing a segment larger than the output
            // buffer.
            let real_end = start + cmp::min(end - start, out_buffer.len());

            self.internal_buffer.map(|internal_buffer| {
                for (dst, src) in out_buffer
                    .iter_mut()
                    .zip(internal_buffer[start..real_end].iter())
                {
                    *dst = *src;
                }
            });

            // Set the outgoing length
            let out_len = real_end - start;
            self.active_len.set(out_len);

            // Transmit the data in the output buffer.
            self.uart.transmit(out_buffer, out_len);
        });
    }

    fn extract(&self) -> Option<(usize, usize, &mut [u8])> {
        self.internal_buffer
            .take()
            .map(|buf| (self.head.get(), self.tail.get(), buf))
    }
}

impl hil::uart::Client for DebugWriter {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: hil::uart::Error) {
        // Replace this buffer since we are done with it.
        self.output_buffer.replace(buffer);

        let written_length = self.active_len.get();
        self.active_len.set(0);
        let len = self
            .internal_buffer
            .map_or(0, |internal_buffer| internal_buffer.len());
        let head = self.head.get();
        let mut tail = self.tail.get();

        // Increment the tail with how many bytes were written to the output
        // mechanism, and wrap if needed.
        tail += written_length;
        if tail > len {
            tail = tail - len;
        }

        if head == tail {
            // Empty. As an optimization, reset the head and tail pointers to 0
            // to maximize the buffer length available before fragmentation
            self.head.set(0);
            self.tail.set(0);
        } else {
            // Buffer not empty, go around again
            self.tail.set(tail);
            self.publish_str();
        }
    }

    fn receive_complete(
        &self,
        _buffer: &'static mut [u8],
        _rx_len: usize,
        _error: hil::uart::Error,
    ) {
    }
}

/// Pass through functions.
impl DebugWriterWrapper {
    fn increment_count(&self) {
        self.dw.map(|dw| {
            dw.increment_count();
        });
    }

    fn get_count(&self) -> usize {
        self.dw.map_or(0, |dw| dw.get_count())
    }

    fn publish_str(&self) {
        self.dw.map(|dw| {
            dw.publish_str();
        });
    }

    fn extract(&self) -> Option<(usize, usize, &mut [u8])> {
        self.dw.map_or(None, |dw| dw.extract())
    }
}

impl Write for DebugWriterWrapper {
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

        self.dw.map(|dw| {
            let mut head = dw.head.get();
            let tail = dw.tail.get();
            let len = dw.internal_buffer.map_or(0, |buffer| buffer.len());

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
                    dw.write_buffer(start, end, bytes);
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
                    let active = dw.active_len.get();
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
                    let active = dw.active_len.get();
                    panic!(
                        "Debug buffer out of room. Head {} tail {} len {} active {} remaining {}",
                        head,
                        tail,
                        len,
                        active,
                        remaining_bytes.len()
                    );
                }
                dw.write_buffer(start, end, remaining_bytes);
                let written = min(end - start, remaining_bytes.len());

                // head cannot wrap here
                head += written;
            }

            dw.head.set(head);
        });

        Ok(())
    }
}

pub fn begin_debug_fmt(args: Arguments) {
    unsafe {
        let writer = get_debug_writer();
        let _ = write(writer, args);
        let _ = writer.write_str("\n");
        writer.publish_str();
    }
}

pub fn begin_debug_verbose_fmt(args: Arguments, file_line: &(&'static str, u32)) {
    unsafe {
        let writer = get_debug_writer();

        writer.increment_count();
        let count = writer.get_count();

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
    let debug_writer = get_debug_writer();

    if let Some((head, mut tail, buffer)) = debug_writer.extract() {
        if head != tail {
            let _ = writer.write_str(
                "\r\n---| Debug buffer not empty. Flushing. May repeat some of last message(s):\r\n",
            );

            if tail > head {
                let start = buffer.as_mut_ptr().offset(tail as isize);
                let len = buffer.len();
                let slice = slice::from_raw_parts(start, len);
                let s = str::from_utf8_unchecked(slice);
                let _ = writer.write_str(s);
                tail = 0;
            }
            if tail != head {
                let start = buffer.as_mut_ptr().offset(tail as isize);
                let len = head - tail;
                let slice = slice::from_raw_parts(start, len);
                let s = str::from_utf8_unchecked(slice);
                let _ = writer.write_str(s);
            }
        }
    }
}
