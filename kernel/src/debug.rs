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
//! # use kernel::{debug, debug_gpio, debug_verbose};
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
use core::fmt::{write, Arguments, Result, Write};
use core::panic::PanicInfo;
use core::ptr;
use core::str;

use crate::common::cells::NumericCellExt;
use crate::common::cells::{MapCell, TakeCell};
use crate::common::queue::Queue;
use crate::common::ring_buffer::RingBuffer;
use crate::hil;
use crate::process::ProcessType;
use crate::Chip;
use crate::ReturnCode;

/// This trait is similar to std::io::Write in that it takes bytes instead of a string (contrary to
/// core::fmt::Write), but io::Write isn't available in no_std (due to std::io::Error not being
/// available).
///
/// Also, in our use cases, writes are infaillible, so the write function just doesn't return
/// anything.
///
/// See also the tracking issue: https://github.com/rust-lang/rfcs/issues/2262
pub trait IoWrite {
    fn write(&mut self, buf: &[u8]);
}

///////////////////////////////////////////////////////////////////
// panic! support routines

/// Tock default panic routine.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic<L: hil::led::Led, W: Write + IoWrite, C: Chip>(
    leds: &mut [&mut L],
    writer: &mut W,
    panic_info: &PanicInfo,
    nop: &dyn Fn(),
    processes: &'static [Option<&'static dyn ProcessType>],
    chip: &'static Option<&'static C>,
) -> ! {
    panic_begin(nop);
    panic_banner(writer, panic_info);
    // Flush debug buffer if needed
    flush(writer);
    panic_cpu_state(chip, writer);
    panic_process_info(processes, writer);
    panic_blink_forever(leds)
}

/// Generic panic entry.
///
/// This opaque method should always be called at the beginning of a board's
/// panic method to allow hooks for any core kernel cleanups that may be
/// appropriate.
pub unsafe fn panic_begin(nop: &dyn Fn()) {
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
        option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown")
    ));
}

/// Print current machine (CPU) state.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_cpu_state<W: Write, C: Chip>(
    chip: &'static Option<&'static C>,
    writer: &mut W,
) {
    chip.map(|c| {
        c.print_state(writer);
    });
}

/// More detailed prints about all processes.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_process_info<W: Write>(
    procs: &'static [Option<&'static dyn ProcessType>],
    writer: &mut W,
) {
    // print data about each process
    let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
    for idx in 0..procs.len() {
        procs[idx].as_ref().map(|process| {
            process.print_full_process(writer);
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
    Option<&'static dyn hil::gpio::Pin>,
    Option<&'static dyn hil::gpio::Pin>,
    Option<&'static dyn hil::gpio::Pin>,
) = (None, None, None);

pub unsafe fn assign_gpios(
    gpio0: Option<&'static dyn hil::gpio::Pin>,
    gpio1: Option<&'static dyn hil::gpio::Pin>,
    gpio2: Option<&'static dyn hil::gpio::Pin>,
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
    uart: &'static dyn hil::uart::Transmit<'static>,
    // The buffer that is passed to the writing mechanism.
    output_buffer: TakeCell<'static, [u8]>,
    // An internal buffer that is used to hold debug!() calls as they come in.
    internal_buffer: TakeCell<'static, RingBuffer<'static, u8>>,
    // Number of debug!() calls.
    count: Cell<usize>,
}

/// Static variable that holds the kernel's reference to the debug tool. This is
/// needed so the debug!() macros have a reference to the object to use.
static mut DEBUG_WRITER: Option<&'static mut DebugWriterWrapper> = None;

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
        uart: &'static dyn hil::uart::Transmit,
        out_buffer: &'static mut [u8],
        internal_buffer: &'static mut RingBuffer<'static, u8>,
    ) -> DebugWriter {
        DebugWriter {
            uart: uart,
            output_buffer: TakeCell::new(out_buffer),
            internal_buffer: TakeCell::new(internal_buffer),
            count: Cell::new(0), // how many debug! calls
        }
    }

    fn increment_count(&self) {
        self.count.increment();
    }

    fn get_count(&self) -> usize {
        self.count.get()
    }

    /// Write as many of the bytes from the internal_buffer to the output
    /// mechanism as possible.
    fn publish_str(&self) {
        // Can only publish if we have the output_buffer. If we don't that is
        // fine, we will do it when the transmit done callback happens.
        self.internal_buffer.map(|ring_buffer| {
            if let Some(out_buffer) = self.output_buffer.take() {
                let mut count = 0;

                for dst in out_buffer.iter_mut() {
                    match ring_buffer.dequeue() {
                        Some(src) => {
                            *dst = src;
                            count += 1;
                        }
                        None => {
                            break;
                        }
                    }
                }

                if count != 0 {
                    // Transmit the data in the output buffer.
                    let (_rval, opt) = self.uart.transmit_buffer(out_buffer, count);
                    self.output_buffer.put(opt);
                }
            }
        });
    }

    fn extract(&self) -> Option<&mut RingBuffer<'static, u8>> {
        self.internal_buffer.take()
    }
}

impl hil::uart::TransmitClient for DebugWriter {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        // Replace this buffer since we are done with it.
        self.output_buffer.replace(buffer);

        if self.internal_buffer.map_or(false, |buf| buf.has_elements()) {
            // Buffer not empty, go around again
            self.publish_str();
        }
    }
    fn transmitted_word(&self, _rcode: ReturnCode) {}
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

    fn extract(&self) -> Option<&mut RingBuffer<'static, u8>> {
        self.dw.map_or(None, |dw| dw.extract())
    }
}

impl Write for DebugWriterWrapper {
    fn write_str(&mut self, s: &str) -> Result {
        const FULL_MSG: &[u8] = b"\n*** DEBUG BUFFER FULL ***\n";
        self.dw.map(|dw| {
            dw.internal_buffer.map(|ring_buffer| {
                let bytes = s.as_bytes();

                let available_len_for_msg =
                    ring_buffer.available_len().saturating_sub(FULL_MSG.len());

                if available_len_for_msg >= bytes.len() {
                    for &b in bytes {
                        ring_buffer.enqueue(b);
                    }
                } else {
                    for &b in &bytes[..available_len_for_msg] {
                        ring_buffer.enqueue(b);
                    }
                    // When the buffer is close to full, print a warning and drop the current
                    // string.
                    for &b in FULL_MSG {
                        ring_buffer.enqueue(b);
                    }
                }
            });
        });

        Ok(())
    }
}

pub fn begin_debug_fmt(args: Arguments) {
    unsafe {
        let writer = get_debug_writer();
        let _ = write(writer, args);
        let _ = writer.write_str("\r\n");
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
        let _ = writer.write_str("\r\n");
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

pub unsafe fn flush<W: Write + IoWrite>(writer: &mut W) {
    let debug_writer = get_debug_writer();

    if let Some(ring_buffer) = debug_writer.extract() {
        if ring_buffer.has_elements() {
            let _ = writer.write_str(
                "\r\n---| Debug buffer not empty. Flushing. May repeat some of last message(s):\r\n",
            );

            let (left, right) = ring_buffer.as_slices();
            if let Some(slice) = left {
                writer.write(slice);
            }
            if let Some(slice) = right {
                writer.write(slice);
            }
        }
    }
}
