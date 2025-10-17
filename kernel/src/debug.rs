// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Support for in-kernel debugging.
//!
//! For printing, this module uses an internal buffer to write the strings into.
//! If you are writing and the buffer fills up, you can make the size of
//! `output_buffer` larger.
//!
//! Before debug interfaces can be used, the board file must assign them
//! hardware:
//!
//! ```ignore
//! kernel::debug::assign_gpios(
//!     Some(&sam4l::gpio::PA[13]),
//!     Some(&sam4l::gpio::PA[15]),
//!     None,
//! );
//!
//! components::debug_writer::DebugWriterComponent::new(
//!     uart_mux,
//!     create_capability!(kernel::capabilities::SetDebugWriterCapability)
//! )
//! .finalize(components::debug_writer_component_static!());
//! ```
//!
//! An alternative to using the default `DebugWriterComponent`, which defaults
//! to sending over UART, is to implement a custom [`DebugWriter`] that can be
//! used for other types of output. For example a simple "endless" FIFO with
//! fixed "push" address:
//!
//! ```ignore
//! use kernel::debug::DebugWriter;
//!
//! pub struct SyncDebugWriter;
//!
//! impl DebugWriter for SyncDebugWriter {
//!    fn write(&self, buf: &[u8], _overflow: &[u8]) -> usize {
//!        let out_reg = 0x4000 as *mut u8; // Replace with the actual address of the FIFO
//!        for c in buf.iter() {
//!            unsafe { out_reg.write_volatile(*c) };
//!        }
//!        buf.len()
//!    }
//!
//!    fn available_len(&self) -> usize {
//!        usize::MAX
//!    }
//!
//!    fn to_write_len(&self) -> usize {
//!        0
//!    }
//!
//!    fn publish(&self) -> usize {
//!        0
//!    }
//!
//!    fn flush(&self, _writer: &mut dyn IoWrite) { }
//! ```
//! And instantiate it in the main board file:
//!
//! ```ignore
//! let debug_writer = static_init!(
//!     utils::SyncDebugWriter,
//!     utils::SyncDebugWriter
//! );
//!
//! kernel::debug::set_debug_writer_wrapper(
//!     static_init!(
//!         kernel::debug::DebugWriterWrapper,
//!         kernel::debug::DebugWriterWrapper::new(debug_writer)
//!     ),
//! );
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
//! debug_verbose!("got here"); // Includes message count, file, and line.
//!
//! debug_gpio!(0, toggle); // Toggles the first debug GPIO.
//!
//! # }
//! ```
//!
//! ```text
//! Yes the code gets here with value 42
//! TOCK_DEBUG(0): /tock/capsules/src/sensys.rs:24: got here
//! ```

use core::cell::Cell;
use core::fmt::{write, Arguments, Write};
use core::panic::PanicInfo;
use core::str;

use crate::capabilities::SetDebugWriterCapability;
use crate::collections::ring_buffer::RingBuffer;
use crate::hil;
use crate::platform::chip::Chip;
use crate::platform::chip::ThreadIdProvider;
use crate::process::ProcessPrinter;
use crate::process::ProcessSlot;
use crate::processbuffer::ReadableProcessSlice;
use crate::utilities::binary_write::BinaryToWriteWrapper;
use crate::utilities::cells::MapCell;
use crate::utilities::cells::NumericCellExt;
use crate::utilities::single_thread_value::SingleThreadValue;

/// Implementation of `std::io::Write` for `no_std`.
///
/// This takes bytes instead of a string (contrary to [`core::fmt::Write`]), but
/// we cannot use `std::io::Write' as it isn't available in `no_std` (due to
/// `std::io::Error` not being available).
///
/// Also, in our use cases, writes are infallible, so the write function cannot
/// return an `Err`, however it might not be able to write everything, so it
/// returns the number of bytes written.
///
/// See also the tracking issue:
/// <https://github.com/rust-lang/rfcs/issues/2262>.
pub trait IoWrite {
    fn write(&mut self, buf: &[u8]) -> usize;

    fn write_ring_buffer(&mut self, buf: &RingBuffer<'_, u8>) -> usize {
        let (left, right) = buf.as_slices();
        let mut total = 0;
        if let Some(slice) = left {
            total += self.write(slice);
        }
        if let Some(slice) = right {
            total += self.write(slice);
        }
        total
    }
}

///////////////////////////////////////////////////////////////////
// panic! support routines

/// Tock panic routine, without the infinite LED-blinking loop.
///
/// This is useful for boards which do not feature LEDs to blink or want to
/// implement their own behavior. This method returns after performing the panic
/// dump.
///
/// After this method returns, the system is no longer in a well-defined state.
/// Care must be taken on how one interacts with the system once this function
/// returns.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_print<W: Write + IoWrite, C: Chip, PP: ProcessPrinter>(
    writer: &mut W,
    panic_info: &PanicInfo,
    nop: &dyn Fn(),
    processes: &'static [ProcessSlot],
    chip: &'static Option<&'static C>,
    process_printer: &'static Option<&'static PP>,
) {
    panic_begin(nop);
    // Flush debug buffer if needed
    flush(writer);
    panic_banner(writer, panic_info);
    panic_cpu_state(chip, writer);

    // Some systems may enforce memory protection regions for the kernel, making
    // application memory inaccessible. However, printing process information
    // will attempt to access memory. If we are provided a chip reference,
    // attempt to disable userspace memory protection first:
    chip.map(|c| {
        use crate::platform::mpu::MPU;
        c.mpu().disable_app_mpu()
    });
    panic_process_info(processes, process_printer, writer);
}

/// Tock default panic routine.
///
/// **NOTE:** The supplied `writer` must be synchronous.
///
/// This will print a detailed debugging message and then loop forever while
/// blinking an LED in a recognizable pattern.
pub unsafe fn panic<L: hil::led::Led, W: Write + IoWrite, C: Chip, PP: ProcessPrinter>(
    leds: &mut [&L],
    writer: &mut W,
    panic_info: &PanicInfo,
    nop: &dyn Fn(),
    processes: &'static [ProcessSlot],
    chip: &'static Option<&'static C>,
    process_printer: &'static Option<&'static PP>,
) -> ! {
    // Call `panic_print` first which will print out the panic information and
    // return
    panic_print(writer, panic_info, nop, processes, chip, process_printer);

    // The system is no longer in a well-defined state, we cannot
    // allow this function to return
    //
    // Forever blink LEDs in an infinite loop
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
    let _ = writer.write_fmt(format_args!("\r\n{}\r\n", panic_info));

    // Print version of the kernel
    if crate::KERNEL_PRERELEASE_VERSION != 0 {
        let _ = writer.write_fmt(format_args!(
            "\tKernel version {}.{}.{}-dev{}\r\n",
            crate::KERNEL_MAJOR_VERSION,
            crate::KERNEL_MINOR_VERSION,
            crate::KERNEL_PATCH_VERSION,
            crate::KERNEL_PRERELEASE_VERSION,
        ));
    } else {
        let _ = writer.write_fmt(format_args!(
            "\tKernel version {}.{}.{}\r\n",
            crate::KERNEL_MAJOR_VERSION,
            crate::KERNEL_MINOR_VERSION,
            crate::KERNEL_PATCH_VERSION,
        ));
    }
}

/// Print current machine (CPU) state.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_cpu_state<W: Write, C: Chip>(
    chip: &'static Option<&'static C>,
    writer: &mut W,
) {
    C::print_state(*chip, writer);
}

/// More detailed prints about all processes.
///
/// **NOTE:** The supplied `writer` must be synchronous.
pub unsafe fn panic_process_info<PP: ProcessPrinter, W: Write>(
    processes: &'static [ProcessSlot],
    process_printer: &'static Option<&'static PP>,
    writer: &mut W,
) {
    process_printer.map(|printer| {
        // print data about each process
        let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
        for slot in processes {
            slot.proc.get().map(|process| {
                // Print the memory map and basic process info.
                //
                // Because we are using a synchronous printer we do not need to
                // worry about looping on the print function.
                printer.print_overview(process, &mut BinaryToWriteWrapper::new(writer), None);
                // Print all of the process details.
                process.print_full_process(writer);
            });
        }
    });
}

/// Blinks a recognizable pattern forever.
///
/// The LED will blink "sporadically" in a somewhat irregular pattern. This
/// should look different from a traditional blinking LED which typically blinks
/// with a consistent duty cycle. The panic blinking sequence is intentionally
/// unusual to make it easier to tell when a panic has occurred.
///
/// If a multi-color LED is used for the panic pattern, it is advised to turn
/// off other LEDs before calling this method.
///
/// Generally, boards should blink red during panic if possible, otherwise
/// choose the 'first' or most prominent LED. Some boards may find it
/// appropriate to blink multiple LEDs (e.g. one on the top and one on the
/// bottom), thus this method accepts an array, however most will only need one.
pub fn panic_blink_forever<L: hil::led::Led>(leds: &mut [&L]) -> ! {
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

/// Object to hold the assigned debugging GPIOs.
pub static mut DEBUG_GPIOS: (
    Option<&'static dyn hil::gpio::Pin>,
    Option<&'static dyn hil::gpio::Pin>,
    Option<&'static dyn hil::gpio::Pin>,
) = (None, None, None);

/// Map up to three GPIO pins to use for debugging.
pub unsafe fn assign_gpios(
    gpio0: Option<&'static dyn hil::gpio::Pin>,
    gpio1: Option<&'static dyn hil::gpio::Pin>,
    gpio2: Option<&'static dyn hil::gpio::Pin>,
) {
    DEBUG_GPIOS.0 = gpio0;
    DEBUG_GPIOS.1 = gpio1;
    DEBUG_GPIOS.2 = gpio2;
}

/// In-kernel gpio debugging that accepts any GPIO HIL method.
#[macro_export]
macro_rules! debug_gpio {
    ($i:tt, $method:ident $(,)?) => {{
        #[allow(unused_unsafe)]
        unsafe {
            $crate::debug::DEBUG_GPIOS.$i.map(|g| g.$method());
        }
    }};
}

///////////////////////////////////////////////////////////////////
// debug! and debug_verbose! support

/// A trait for writing debug output.
///
/// This can be used for example to implement asynchronous or synchronous
/// writers, and buffered or unbuffered writers. Various platforms may have
/// in-memory logs, memory mapped "endless" FIFOs, JTAG support for output,
/// etc.
pub trait DebugWriter {
    /// Write bytes to output with overflow notification.
    ///
    /// The `overflow` slice is used as a message to be appended to the end of
    /// the available buffer if it becomes full.
    fn write(&self, bytes: &[u8], overflow_message: &[u8]) -> usize;

    /// Available length of the internal buffer if limited.
    ///
    /// If the buffer can support a write of any size, it should lie and return
    /// `usize::MAX`.
    ///
    /// Across subsequent calls to this function, without invoking `write()` in
    /// between, this returned value may only increase, but never decrease.
    fn available_len(&self) -> usize;

    /// How many bytes are buffered and not yet written.
    fn to_write_len(&self) -> usize;

    /// Publish bytes from the internal buffer to the output.
    ///
    /// Returns how many bytes were written.
    fn publish(&self) -> usize;

    /// Flush any buffered bytes to the provided output writer.
    ///
    /// `flush()` should be used to write an buffered bytes to a new `writer`
    /// instead of the internal writer that `publish()` would use.
    fn flush(&self, writer: &mut dyn IoWrite);
}

/// Static variable that holds the kernel's reference to the debug tool.
///
/// This is needed so the `debug!()` macros have a reference to the object to
/// use.
static DEBUG_WRITER: SingleThreadValue<MapCell<&'static dyn DebugWriter>> =
    SingleThreadValue::new(MapCell::empty());

/// Static variable that holds how many times `debug!()` has been called.
///
/// This enables printing a verbose header message that enumerates independent
/// debug messages.
static DEBUG_WRITER_COUNT: SingleThreadValue<Cell<usize>> = SingleThreadValue::new(Cell::new(0));

/// Initialize the static debug writer.
///
/// This ensures it can safely be used as a global variable.
#[cfg(target_has_atomic = "ptr")]
pub fn initialize_debug_writer_wrapper<P: ThreadIdProvider>() {
    DEBUG_WRITER.bind_to_thread::<P>();
    DEBUG_WRITER_COUNT.bind_to_thread::<P>();
}

/// Initialize the static debug writer.
///
/// This ensures it can safely be used as a global variable.
///
/// # Safety
///
/// Callers of this function must ensure that this function is never called
/// concurrently with other calls to [`initialize_debug_writer_wrapper_unsafe`].
pub unsafe fn initialize_debug_writer_wrapper_unsafe<P: ThreadIdProvider>() {
    DEBUG_WRITER.bind_to_thread_unsafe::<P>();
    DEBUG_WRITER_COUNT.bind_to_thread_unsafe::<P>();
}

fn try_get_debug_writer<F, R>(closure: F) -> Option<R>
where
    F: FnOnce(&dyn DebugWriter) -> R,
{
    DEBUG_WRITER
        .get()
        .and_then(|dw| dw.map_or(None, |writer| Some(closure(*writer))))
}

/// Function used by board main.rs to set a reference to the writer.
pub fn set_debug_writer_wrapper<C: SetDebugWriterCapability>(
    debug_writer: &'static dyn DebugWriter,
    _cap: C,
) {
    DEBUG_WRITER.get().map(|dw| dw.replace(debug_writer));
}

impl Write for &dyn DebugWriter {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.write(s.as_bytes(), b"");
        Ok(())
    }
}

/// Write a debug message without a trailing newline.
pub fn debug_print(args: Arguments) {
    try_get_debug_writer(|mut writer| {
        let _ = write(&mut writer, args);
        writer.publish();
    });
}

/// Write a debug message with a trailing newline.
pub fn debug_println(args: Arguments) {
    try_get_debug_writer(|mut writer| {
        let _ = write(&mut writer, args);
        let _ = writer.write_str("\r\n");
        writer.publish();
    });
}

/// Write a [`ReadableProcessSlice`] to the debug output.
///
/// # Errors
///
/// Will return `Err` if it is not possible to write any output.
pub fn debug_slice(slice: &ReadableProcessSlice) -> Result<usize, ()> {
    try_get_debug_writer(|writer| {
        let mut total = 0;
        for b in slice.iter() {
            let buf: [u8; 1] = [b.get(); 1];
            let count = writer.write(&buf, b"");
            if count > 0 {
                total += count;
            } else {
                break;
            }
        }
        writer.publish();
        total
    })
    .ok_or(())
}

/// Return how many bytes are remaining in the internal debug buffer.
pub fn debug_available_len() -> usize {
    try_get_debug_writer(|writer| writer.available_len()).unwrap_or(0)
}

fn write_header(
    writer: &mut &dyn DebugWriter,
    (file, line): &(&'static str, u32),
) -> Result<(), core::fmt::Error> {
    let count = DEBUG_WRITER_COUNT.get().map_or(0, |count| {
        count.increment();
        count.get()
    });

    writer.write_fmt(format_args!("TOCK_DEBUG({}): {}:{}: ", count, file, line))
}

/// Write a debug message with file and line information without a trailing
/// newline.
pub fn debug_verbose_print(args: Arguments, file_line: &(&'static str, u32)) {
    try_get_debug_writer(|mut writer| {
        let _ = write_header(&mut writer, file_line);
        let _ = write(&mut writer, args);
        writer.publish();
    });
}

/// Write a debug message with file and line information with a trailing
/// newline.
pub fn debug_verbose_println(args: Arguments, file_line: &(&'static str, u32)) {
    try_get_debug_writer(|mut writer| {
        let _ = write_header(&mut writer, file_line);
        let _ = write(&mut writer, args);
        let _ = writer.write_str("\r\n");
        writer.publish();
    });
}

/// In-kernel `println()` debugging.
#[macro_export]
macro_rules! debug {
    () => ({
        // Allow an empty debug!() to print the location when hit
        debug!("")
    });
    ($msg:expr $(,)?) => ({
        $crate::debug::debug_println(format_args!($msg));
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::debug::debug_println(format_args!($fmt, $($arg)+));
    });
}

/// In-kernel `println()` debugging that can take a process slice.
#[macro_export]
macro_rules! debug_process_slice {
    ($msg:expr $(,)?) => {{
        $crate::debug::debug_slice($msg)
    }};
}

/// In-kernel `println()` debugging with filename and line numbers.
#[macro_export]
macro_rules! debug_verbose {
    () => ({
        // Allow an empty debug_verbose!() to print the location when hit
        debug_verbose!("")
    });
    ($msg:expr $(,)?) => ({
        $crate::debug::debug_verbose_println(format_args!($msg), {
            // TODO: Maybe make opposite choice of panic!, no `static`, more
            // runtime code for less static data
            static _FILE_LINE: (&'static str, u32) = (file!(), line!());
            &_FILE_LINE
        })
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::debug::debug_verbose_println(format_args!($fmt, $($arg)+), {
            static _FILE_LINE: (&'static str, u32) = (file!(), line!());
            &_FILE_LINE
        })
    });
}

/// Prints out the expression and its location, then returns it.
///
/// ```rust,ignore
/// let foo: u8 = debug_expr!(0xff);
/// // Prints [main.rs:2] 0xff = 255
/// ```
/// Taken straight from Rust `std::dbg`.
#[macro_export]
macro_rules! debug_expr {
    // NOTE: We cannot use `concat!` to make a static string as a format
    // argument of `eprintln!` because `file!` could contain a `{` or `$val`
    // expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::debug!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::debug!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::debug_expr!($val)),+,)
    };
}

/// Flush any stored messages to the output writer.
pub unsafe fn flush<W: Write + IoWrite>(writer: &mut W) {
    try_get_debug_writer(|debug_writer|{
        if debug_writer.to_write_len() > 0 {
            let _ = writer.write_str(
                    "\r\n---| Debug buffer not empty. Flushing. May repeat some of last message(s):\r\n",
                );
            debug_writer.flush(writer);
        }
    }).or_else(||{
        let _ = writer.write_str(
            "\r\n---| Global debug writer not registered.\
             \r\n     Call `set_debug_writer_wrapper` in board initialization.\r\n",
        );
        None
    });
}
