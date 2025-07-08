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
use core::fmt::{write, Arguments, Result, Write};
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use core::str;

use crate::capabilities::SetDebugWriterCapability;
use crate::collections::queue::Queue;
use crate::collections::ring_buffer::RingBuffer;
use crate::hil;
use crate::platform::chip::Chip;
use crate::process::Process;
use crate::process::ProcessPrinter;
use crate::processbuffer::ReadableProcessSlice;
use crate::utilities::binary_write::BinaryToWriteWrapper;
use crate::utilities::cells::NumericCellExt;
use crate::utilities::cells::{MapCell, TakeCell};
use crate::ErrorCode;

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
    processes: &'static [Option<&'static dyn Process>],
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
    processes: &'static [Option<&'static dyn Process>],
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
pub unsafe fn panic_process_info<PP: ProcessPrinter, W: Write>(
    procs: &'static [Option<&'static dyn Process>],
    process_printer: &'static Option<&'static PP>,
    writer: &mut W,
) {
    process_printer.map(|printer| {
        // print data about each process
        let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
        for proc in procs {
            proc.map(|process| {
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

/// Wrapper type that we need a mutable reference to for the
/// [`core::fmt::Write`] interface.
pub struct DebugWriterWrapper {
    dw: MapCell<&'static DebugWriter>,
}

/// Main type that we share with the UART provider and this debug module.
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

/// Static variable that holds the kernel's reference to the debug tool.
///
/// This is needed so the `debug!()` macros have a reference to the object to
/// use.
static mut DEBUG_WRITER: Option<&'static mut DebugWriterWrapper> = None;

unsafe fn try_get_debug_writer() -> Option<&'static mut DebugWriterWrapper> {
    (*addr_of_mut!(DEBUG_WRITER)).as_deref_mut()
}

unsafe fn get_debug_writer() -> &'static mut DebugWriterWrapper {
    try_get_debug_writer().unwrap() // Unwrap fail = Must call `set_debug_writer_wrapper` in board initialization.
}

/// Function used by board main.rs to set a reference to the writer.
pub fn set_debug_writer_wrapper<C: SetDebugWriterCapability>(
    debug_writer: &'static mut DebugWriterWrapper,
    _cap: C,
) {
    unsafe {
        DEBUG_WRITER = Some(debug_writer);
    }
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
            uart,
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
    /// mechanism as possible, returning the number written.
    fn publish_bytes(&self) -> usize {
        // Can only publish if we have the output_buffer. If we don't that is
        // fine, we will do it when the transmit done callback happens.
        self.internal_buffer.map_or(0, |ring_buffer| {
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
                    if let Err((_err, buf)) = self.uart.transmit_buffer(out_buffer, count) {
                        self.output_buffer.put(Some(buf));
                    } else {
                        self.output_buffer.put(None);
                    }
                }
                count
            } else {
                0
            }
        })
    }

    fn extract(&self) -> Option<&mut RingBuffer<'static, u8>> {
        self.internal_buffer.take()
    }

    fn available_len(&self) -> usize {
        self.internal_buffer.map_or(0, |rb| rb.available_len())
    }
}

impl hil::uart::TransmitClient for DebugWriter {
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: core::result::Result<(), ErrorCode>,
    ) {
        // Replace this buffer since we are done with it.
        self.output_buffer.replace(buffer);

        if self.internal_buffer.map_or(false, |buf| buf.has_elements()) {
            // Buffer not empty, go around again
            self.publish_bytes();
        }
    }
    fn transmitted_word(&self, _rcode: core::result::Result<(), ErrorCode>) {}
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

    fn publish_bytes(&self) -> usize {
        self.dw.map_or(0, |dw| dw.publish_bytes())
    }

    fn extract(&self) -> Option<&mut RingBuffer<'static, u8>> {
        self.dw.map_or(None, |dw| dw.extract())
    }

    fn available_len(&self) -> usize {
        const FULL_MSG: &[u8] = b"\n*** DEBUG BUFFER FULL ***\n";
        self.dw
            .map_or(0, |dw| dw.available_len().saturating_sub(FULL_MSG.len()))
    }
}

impl IoWrite for DebugWriterWrapper {
    fn write(&mut self, bytes: &[u8]) -> usize {
        const FULL_MSG: &[u8] = b"\n*** DEBUG BUFFER FULL ***\n";
        self.dw.map_or(0, |dw| {
            dw.internal_buffer.map_or(0, |ring_buffer| {
                let available_len_for_msg =
                    ring_buffer.available_len().saturating_sub(FULL_MSG.len());

                if available_len_for_msg >= bytes.len() {
                    for &b in bytes {
                        ring_buffer.enqueue(b);
                    }
                    bytes.len()
                } else {
                    for &b in &bytes[..available_len_for_msg] {
                        ring_buffer.enqueue(b);
                    }
                    // When the buffer is close to full, print a warning and drop the current
                    // string.
                    for &b in FULL_MSG {
                        ring_buffer.enqueue(b);
                    }
                    available_len_for_msg
                }
            })
        })
    }
}

impl Write for DebugWriterWrapper {
    fn write_str(&mut self, s: &str) -> Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

/// Write a debug message without a trailing newline.
pub fn debug_print(args: Arguments) {
    let writer = unsafe { get_debug_writer() };

    let _ = write(writer, args);
    writer.publish_bytes();
}

/// Write a debug message with a trailing newline.
pub fn debug_println(args: Arguments) {
    let writer = unsafe { get_debug_writer() };

    let _ = write(writer, args);
    let _ = writer.write_str("\r\n");
    writer.publish_bytes();
}

/// Write a [`ReadableProcessSlice`] to the debug output.
pub fn debug_slice(slice: &ReadableProcessSlice) -> usize {
    let writer = unsafe { get_debug_writer() };
    let mut total = 0;
    for b in slice.iter() {
        let buf: [u8; 1] = [b.get(); 1];
        let count = writer.write(&buf);
        if count > 0 {
            total += count;
        } else {
            break;
        }
    }
    writer.publish_bytes();
    total
}

/// Return how many bytes are remaining in the internal debug buffer.
pub fn debug_available_len() -> usize {
    let writer = unsafe { get_debug_writer() };
    writer.available_len()
}

fn write_header(writer: &mut DebugWriterWrapper, (file, line): &(&'static str, u32)) -> Result {
    writer.increment_count();
    let count = writer.get_count();
    writer.write_fmt(format_args!("TOCK_DEBUG({}): {}:{}: ", count, file, line))
}

/// Write a debug message with file and line information without a trailing
/// newline.
pub fn debug_verbose_print(args: Arguments, file_line: &(&'static str, u32)) {
    let writer = unsafe { get_debug_writer() };

    let _ = write_header(writer, file_line);
    let _ = write(writer, args);
    writer.publish_bytes();
}

/// Write a debug message with file and line information with a trailing
/// newline.
pub fn debug_verbose_println(args: Arguments, file_line: &(&'static str, u32)) {
    let writer = unsafe { get_debug_writer() };

    let _ = write_header(writer, file_line);
    let _ = write(writer, args);
    let _ = writer.write_str("\r\n");
    writer.publish_bytes();
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
    if let Some(debug_writer) = try_get_debug_writer() {
        if let Some(ring_buffer) = debug_writer.extract() {
            if ring_buffer.has_elements() {
                let _ = writer.write_str(
                    "\r\n---| Debug buffer not empty. Flushing. May repeat some of last message(s):\r\n",
                );

                writer.write_ring_buffer(ring_buffer);
            }
        }
    } else {
        let _ = writer.write_str(
            "\r\n---| Global debug writer not registered.\
             \r\n     Call `set_debug_writer_wrapper` in board initialization.\r\n",
        );
    }
}
