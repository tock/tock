//! Implements a text console over the UART that allows
//! a terminal to inspect and control userspace processes.
//!
//! Protocol
//! --------
//!
//! This module provides a simple text-based console to inspect and control
//! which processes are running. The console has five commands:
//!  - 'help' prints the available commands and arguments
//!  - 'status' prints the current system status
//!  - 'list' lists the current processes with their IDs and running state
//!  - 'stop n' stops the process with name n
//!  - 'start n' starts the stopped process with name n
//!  - 'fault n' forces the process with name n into a fault state
//!  - 'panic' causes the kernel to run the panic handler
//!  - 'process n' prints the memory map of process with name n
//!  - 'kernel' prints the kernel memory map
//!
//! ### `list` Command Fields:
//!
//! - `PID`: The identifier for the process. This can change if the process
//!   restarts.
//! - `Name`: The process name.
//! - `Quanta`: How many times this process has exceeded its alloted time
//!   quanta.
//! - `Syscalls`: The number of system calls the process has made to the kernel.
//! - `Dropped Upcalls`: How many upcalls were dropped for this process
//!   because the queue was full.
//! - `Restarts`: How many times this process has crashed and been restarted by
//!   the kernel.
//! - `State`: The state the process is in.
//! - `Grants`: The number of grants that have been initialized for the process
//!   out of the total number of grants defined by the kernel.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait. This code
//! connects a `ProcessConsole` directly up to USART0:
//!
//! ```rust
//! # use kernel::{capabilities, hil, static_init};
//! # use capsules::process_console::ProcessConsole;
//!
//! pub struct Capability;
//! unsafe impl capabilities::ProcessManagementCapability for Capability {}
//!
//! let pconsole = static_init!(
//!     ProcessConsole<usart::USART>,
//!     ProcessConsole::new(&usart::USART0,
//!                  115200,
//!                  &mut console::WRITE_BUF,
//!                  &mut console::READ_BUF,
//!                  &mut console::COMMAND_BUF,
//!                  kernel,
//!                  Capability));
//! hil::uart::UART::set_client(&usart::USART0, pconsole);
//!
//! pconsole.start();
//! ```
//!
//! Using ProcessConsole
//! --------------------
//!
//! With this capsule properly added to a board's `main.rs` and that kernel
//! loaded to the board, make sure there is a serial connection to the board.
//! Likely, this just means connecting a USB cable from a computer to the board.
//! Next, establish a serial console connection to the board. An easy way to do
//! this is to run:
//!
//! ```shell
//! $ tockloader listen
//! ```
//!
//! With that console open, you can issue commands. For example, to see all of
//! the processes on the board, use `list`:
//!
//! ```text
//! $ tockloader listen
//! Using "/dev/cu.usbserial-c098e513000c - Hail IoT Module - TockOS"
//!
//! Listening for serial output.
//! ProcessConsole::start
//! Starting process console
//! Initialization complete. Entering main loop
//! Hello World!
//! list
//! PID    Name    Quanta  Syscalls  Dropped Upcalls  Restarts    State  Grants
//! 00     blink        0       113                0         0  Yielded    1/12
//! 01     c_hello      0         8                0         0  Yielded    3/12
//! ```
//!
//! To get a general view of the system, use the status command:
//!
//! ```text
//! status
//! Total processes: 2
//! Active processes: 2
//! Timeslice expirations: 0
//! ```
//!
//! and you can control processes with the `start` and `stop` commands:
//!
//! ```text
//! stop blink
//! Process blink stopped
//! ```

use core::cell::Cell;
use core::cmp;
use core::fmt;
use core::fmt::write;
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::utilities::cells::TakeCell;
use kernel::ProcessId;

use kernel::debug;
use kernel::hil::uart;
use kernel::introspection::KernelInfo;
use kernel::ErrorCode;
use kernel::Kernel;

/// Buffer to hold outgoing data that is passed to the UART hardware.
pub static mut WRITE_BUF: [u8; 500] = [0; 500];
/// Buffer responses are initially held in until copied to the TX buffer and
/// transmitted.
pub static mut QUEUE_BUF: [u8; 300] = [0; 300];
/// Since reads are byte-by-byte, to properly echo what's typed,
/// we can use a very small read buffer.
pub static mut READ_BUF: [u8; 4] = [0; 4];
/// Commands can be up to 32 bytes long: since commands themselves are 4-5
/// characters, limiting arguments to 25 bytes or so seems fine for now.
pub static mut COMMAND_BUF: [u8; 32] = [0; 32];

/// States used for state machine to allow printing large strings asynchronously
/// across multiple calls. This reduces the size of the buffer needed to print
/// each section of the debug message.
#[derive(PartialEq, Eq, Copy, Clone)]
enum WriterState {
    Empty,
    KernelStart,
    KernelBss,
    KernelInit,
    KernelStack,
    KernelRoData,
    KernelText,
    ProcessStart { process_id: ProcessId },
    ProcessGrant { process_id: ProcessId },
    ProcessHeap { process_id: ProcessId },
    ProcessHeapUnused { process_id: ProcessId },
    ProcessData { process_id: ProcessId },
    ProcessStack { process_id: ProcessId },
    ProcessStackUnused { process_id: ProcessId },
    ProcessFlash { process_id: ProcessId },
    ProcessProtected { process_id: ProcessId },
    List { index: isize, total: isize },
}

impl Default for WriterState {
    fn default() -> Self {
        WriterState::Empty
    }
}

/// Data structure to hold addresses about how the kernel is stored in memory on
/// the chip.
///
/// All "end" addresses are the memory addresses immediately following the end
/// of the memory region.
pub struct KernelAddresses {
    pub stack_start: *const u8,
    pub stack_end: *const u8,
    pub text_start: *const u8,
    pub text_end: *const u8,
    pub read_only_data_start: *const u8,
    pub relocations_start: *const u8,
    pub relocations_end: *const u8,
    pub bss_start: *const u8,
    pub bss_end: *const u8,
}

pub struct ProcessConsole<'a, C: ProcessManagementCapability> {
    uart: &'a dyn uart::UartData<'a>,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    queue_buffer: TakeCell<'static, [u8]>,
    queue_size: Cell<usize>,
    writer_state: Cell<WriterState>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    command_index: Cell<usize>,

    /// Flag to mark that the process console is active and has called receive
    /// from the underlying UART.
    running: Cell<bool>,

    /// Internal flag that the process console should parse the command it just
    /// received after finishing echoing the last newline character.
    execute: Cell<bool>,

    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,

    /// Memory addresses of where the kernel is placed in memory on chip.
    kernel_addresses: KernelAddresses,

    /// This capsule needs to use potentially dangerous APIs related to
    /// processes, and requires a capability to access those APIs.
    capability: C,
}

pub struct ConsoleWriter {
    buf: [u8; 500],
    size: usize,
}
impl ConsoleWriter {
    pub fn new() -> ConsoleWriter {
        ConsoleWriter {
            buf: [0; 500],
            size: 0,
        }
    }
    pub fn clear(&mut self) {
        self.size = 0;
    }
}
impl fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let curr = (s).as_bytes().len();
        self.buf[self.size..self.size + curr].copy_from_slice(&(s).as_bytes()[..]);
        self.size += curr;
        Ok(())
    }
}

fn exceeded_check(size: usize, allocated: usize) -> &'static str {
    if size > allocated {
        " EXCEEDED!"
    } else {
        "          "
    }
}

impl<'a, C: ProcessManagementCapability> ProcessConsole<'a, C> {
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        queue_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
        kernel: &'static Kernel,
        kernel_addresses: KernelAddresses,
        capability: C,
    ) -> ProcessConsole<'a, C> {
        ProcessConsole {
            uart: uart,
            tx_in_progress: Cell::new(false),
            tx_buffer: TakeCell::new(tx_buffer),
            queue_buffer: TakeCell::new(queue_buffer),
            queue_size: Cell::new(0),
            writer_state: Cell::new(WriterState::Empty),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
            command_index: Cell::new(0),
            running: Cell::new(false),
            execute: Cell::new(false),
            kernel: kernel,
            kernel_addresses: kernel_addresses,
            capability: capability,
        }
    }

    /// Start the process console listening for user commands.
    pub fn start(&self) -> Result<(), ErrorCode> {
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                let _ = self.uart.receive_buffer(buffer, 1);
                self.running.set(true);
            });
        }
        Ok(())
    }

    /// Print base information about the kernel version installed and the help
    /// message.
    pub fn display_welcome(&self) {
        // Start if not already started.
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                let _ = self.uart.receive_buffer(buffer, 1);
                self.running.set(true);
            });
        }

        // Display pconsole info.
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "Kernel version: {}\r\n",
                option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown")
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);

        let _ = self.write_bytes(b"Welcome to the process console.\n");
        let _ = self
            .write_bytes(b"Valid commands are: help status list stop start fault process kernel\n");
    }

    /// Simple state machine helper function that identifies the next state for
    /// printing log debug messages.
    fn next_state(&self, state: WriterState) -> WriterState {
        match state {
            WriterState::KernelStart => WriterState::KernelBss,
            WriterState::KernelBss => WriterState::KernelInit,
            WriterState::KernelInit => WriterState::KernelStack,
            WriterState::KernelStack => WriterState::KernelRoData,
            WriterState::KernelRoData => WriterState::KernelText,
            WriterState::KernelText => WriterState::Empty,
            WriterState::ProcessStart { process_id } => WriterState::ProcessGrant { process_id },
            WriterState::ProcessGrant { process_id } => {
                WriterState::ProcessHeapUnused { process_id }
            }
            WriterState::ProcessHeapUnused { process_id } => {
                WriterState::ProcessHeap { process_id }
            }
            WriterState::ProcessHeap { process_id } => WriterState::ProcessData { process_id },
            WriterState::ProcessData { process_id } => WriterState::ProcessStack { process_id },
            WriterState::ProcessStack { process_id } => {
                WriterState::ProcessStackUnused { process_id }
            }
            WriterState::ProcessStackUnused { process_id } => {
                WriterState::ProcessFlash { process_id }
            }
            WriterState::ProcessFlash { process_id } => {
                WriterState::ProcessProtected { process_id }
            }
            WriterState::ProcessProtected { process_id: _ } => WriterState::Empty,
            WriterState::List { index, total } => {
                // Next state just increments index, unless we are at end in
                // which next state is just the empty state.
                if index + 1 == total {
                    WriterState::Empty
                } else {
                    WriterState::List {
                        index: index + 1,
                        total,
                    }
                }
            }
            WriterState::Empty => WriterState::Empty,
        }
    }

    // These `print_process_()` functions are split out from the main state
    // machine because of an incompatibility with rustfmt. Rustfmt cannot handle
    // long lines inside of match statements, so we use individual functions to
    // avoid the issue.

    fn print_process_heap_populated(
        &self,
        sram_heap_size: usize,
        sram_heap_allocated: usize,
        sram_heap_start: usize,
    ) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
            \r\n             │ ▲ Heap       {:6} | {:6}{}     S\
            \r\n  {:#010X} ┼─────────────────────────────────────────── R",
                sram_heap_size,
                sram_heap_allocated,
                exceeded_check(sram_heap_size, sram_heap_allocated),
                sram_heap_start,
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_heap_unknown(&self) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
            \r\n             │ ▲ Heap            ? |      ?               S\
            \r\n  ?????????? ┼─────────────────────────────────────────── R",
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_data_populated(&self, sram_data_size: usize, sram_data_allocated: usize) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
                \r\n             │ Data         {:6} | {:6}               A",
                sram_data_size, sram_data_allocated,
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_data_unknown(&self) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
                \r\n             │ Data              ? |      ?               A",
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_stack_populated(
        &self,
        sram_stack_start: usize,
        sram_stack_size: usize,
        sram_stack_allocated: usize,
    ) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
            \r\n  {:#010X} ┼─────────────────────────────────────────── M\
            \r\n             │ ▼ Stack      {:6} | {:6}{}",
                sram_stack_start,
                sram_stack_size,
                sram_stack_allocated,
                exceeded_check(sram_stack_size, sram_stack_allocated),
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_stack_unknown(&self) {
        let mut console_writer = ConsoleWriter::new();

        let _ = write(
            &mut console_writer,
            format_args!(
                "\
            \r\n  ?????????? ┼─────────────────────────────────────────── M\
            \r\n             │ ▼ Stack           ? |      ?",
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_flash(&self, flash_end: usize, flash_app_size: usize) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
                \r\n  {:#010X} ┬─────────────────────────────────────────── F\
                \r\n             │ App Flash    {:6}                        L",
                flash_end, flash_app_size
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    fn print_process_protected(
        &self,
        flash_app_start: usize,
        flash_protected_size: usize,
        flash_start: usize,
    ) {
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "\
                \r\n  {:#010X} ┼─────────────────────────────────────────── A\
                \r\n             │ Protected    {:6}                        S\
                \r\n  {:#010X} ┴─────────────────────────────────────────── H\
                \r\n",
                flash_app_start, flash_protected_size, flash_start
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
    }

    /// Create the debug message for each state in the state machine.
    fn create_state_buffer(&self, state: WriterState) {
        match state {
            WriterState::KernelBss => {
                let mut console_writer = ConsoleWriter::new();

                let bss_start = self.kernel_addresses.bss_start as usize;
                let bss_end = self.kernel_addresses.bss_end as usize;
                let bss_size = bss_end - bss_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\r\n ╔═══════════╤══════════════════════════════╗\
                    \r\n ║  Address  │ Region Name    Used (bytes)  ║\
                    \r\n ╚{:#010X}═╪══════════════════════════════╝\
                    \r\n             │   Bss        {:6}",
                        bss_end, bss_size
                    ),
                );

                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelInit => {
                let mut console_writer = ConsoleWriter::new();

                let relocate_start = self.kernel_addresses.relocations_start as usize;
                let relocate_end = self.kernel_addresses.relocations_end as usize;
                let relocate_size = relocate_end - relocate_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                    \r\n  {:#010X} ┼─────────────────────────────── S\
                    \r\n             │   Relocate   {:6}            R",
                        relocate_end, relocate_size
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelStack => {
                let mut console_writer = ConsoleWriter::new();

                let stack_start = self.kernel_addresses.stack_start as usize;
                let stack_end = self.kernel_addresses.stack_end as usize;
                let stack_size = stack_end - stack_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                    \r\n  {:#010X} ┼─────────────────────────────── A\
                    \r\n             │ ▼ Stack      {:6}            M\
                    \r\n  {:#010X} ┼───────────────────────────────",
                        stack_end, stack_size, stack_start
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelRoData => {
                let mut console_writer = ConsoleWriter::new();

                let rodata_start = self.kernel_addresses.read_only_data_start as usize;
                let text_end = self.kernel_addresses.text_end as usize;
                let rodata_size = text_end - rodata_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                        \r\n             .....\
                     \r\n  {:#010X} ┼─────────────────────────────── F\
                     \r\n             │   RoData     {:6}            L",
                        text_end, rodata_size
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelText => {
                let mut console_writer = ConsoleWriter::new();

                let code_start = self.kernel_addresses.text_start as usize;
                let code_end = self.kernel_addresses.read_only_data_start as usize;
                let code_size = code_end - code_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                     \r\n  {:#010X} ┼─────────────────────────────── A\
                     \r\n             │   Code       {:6}            S\
                     \r\n  {:#010X} ┼─────────────────────────────── H\
                     \r\n",
                        code_end, code_size, code_start
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::ProcessGrant { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let mut console_writer = ConsoleWriter::new();
                            // SRAM addresses
                            let sram_end = process.mem_end() as usize;
                            let sram_grant_start = process.kernel_memory_break() as usize;

                            // SRAM sizes
                            let sram_grant_size = sram_end - sram_grant_start;
                            let sram_grant_allocated = sram_end - sram_grant_start;

                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "\r\n ╔═══════════╤══════════════════════════════════════════╗\
                                    \r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
                                    \r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
                                    \r\n             │ ▼ Grant      {:6} | {:6}{}",
                                    sram_end,
                                    sram_grant_size,
                                    sram_grant_allocated,
                                    exceeded_check(sram_grant_size, sram_grant_allocated),
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        }
                    });
            }
            WriterState::ProcessHeapUnused { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let mut console_writer = ConsoleWriter::new();
                            // SRAM addresses
                            let sram_grant_start = process.kernel_memory_break() as usize;
                            let sram_heap_end = process.app_memory_break() as usize;

                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "\
                                    \r\n  {:#010X} ┼───────────────────────────────────────────\
                                    \r\n             │ Unused\
                                    \r\n  {:#010X} ┼───────────────────────────────────────────",
                                    sram_grant_start, sram_heap_end,
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        }
                    });
            }
            WriterState::ProcessHeap { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let sram_grant_start = process.kernel_memory_break() as usize;
                            let sram_heap_end = process.app_memory_break() as usize;
                            let sram_heap_start = process.debug_heap_start();

                            if let Some(sram_heap_start) = sram_heap_start {
                                let sram_heap_size =
                                    sram_heap_end as usize - sram_heap_start as usize;
                                let sram_heap_allocated =
                                    sram_grant_start as usize - sram_heap_start as usize;
                                self.print_process_heap_populated(
                                    sram_heap_size,
                                    sram_heap_allocated,
                                    sram_heap_start as usize,
                                );
                            } else {
                                self.print_process_heap_unknown();
                            }
                        }
                    });
            }
            WriterState::ProcessData { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let sram_heap_start = process.debug_heap_start();
                            let sram_stack_start = process.debug_stack_start();

                            if let (Some(sram_heap_start), Some(sram_stack_start)) =
                                (sram_heap_start, sram_stack_start)
                            {
                                let sram_data_size =
                                    sram_heap_start as usize - sram_stack_start as usize;
                                let sram_data_allocated = sram_data_size as usize;

                                self.print_process_data_populated(
                                    sram_data_size,
                                    sram_data_allocated,
                                );
                            } else {
                                self.print_process_data_unknown();
                            }
                        }
                    });
            }
            WriterState::ProcessStack { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let sram_stack_start = process.debug_stack_start();
                            let sram_stack_bottom = process.debug_stack_end();
                            let sram_start = process.mem_start() as usize;

                            if let (Some(sram_stack_start), Some(sram_stack_bottom)) =
                                (sram_stack_start, sram_stack_bottom)
                            {
                                let sram_stack_size =
                                    sram_stack_start as usize - sram_stack_bottom as usize;
                                let sram_stack_allocated =
                                    sram_stack_start as usize - sram_start as usize;

                                self.print_process_stack_populated(
                                    sram_stack_start as usize,
                                    sram_stack_size,
                                    sram_stack_allocated,
                                );
                            } else {
                                self.print_process_stack_unknown();
                            }
                        }
                    });
            }

            WriterState::ProcessStackUnused { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let mut console_writer = ConsoleWriter::new();

                            let sram_stack_bottom =
                                process.debug_stack_end().map_or_else(|| 0, |p| p as usize);
                            let sram_start = process.mem_start() as usize;

                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "\
                                    \r\n  {:#010X} ┼───────────────────────────────────────────\
                                    \r\n             │ Unused\
                                    \r\n  {:#010X} ┴───────────────────────────────────────────\
                                    \r\n             .....",
                                    sram_stack_bottom, sram_start
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        }
                    });
            }
            WriterState::ProcessFlash { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            // Flash
                            let flash_end = process.flash_end() as usize;
                            let flash_app_start = process.flash_non_protected_start() as usize;
                            let flash_app_size = flash_end - flash_app_start;

                            self.print_process_flash(flash_end, flash_app_size);
                        }
                    });
            }
            WriterState::ProcessProtected { process_id } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            // Flash
                            let flash_start = process.flash_start() as usize;
                            let flash_app_start = process.flash_non_protected_start() as usize;
                            let flash_protected_size = flash_app_start - flash_start;

                            self.print_process_protected(
                                flash_app_start,
                                flash_protected_size,
                                flash_start,
                            );
                        }
                    });
            }
            WriterState::List { index, total: _ } => {
                let mut local_index = -1;
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        local_index += 1;
                        if local_index == index {
                            let info: KernelInfo = KernelInfo::new(self.kernel);

                            let pname = process.get_process_name();
                            let process_id = process.processid();
                            let (grants_used, grants_total) =
                                info.number_app_grant_uses(process_id, &self.capability);
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "  {:?}\t{:<20}{:6}{:10}{:19}{:10}  {:?}{:5}/{}\n",
                                    process_id,
                                    pname,
                                    process.debug_timeslice_expiration_count(),
                                    process.debug_syscall_count(),
                                    process.debug_dropped_upcall_count(),
                                    process.get_restart_count(),
                                    process.get_state(),
                                    grants_used,
                                    grants_total
                                ),
                            );

                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        }
                    });
            }
            _ => {}
        }
    }

    // Process the command in the command buffer and clear the buffer.
    fn read_command(&self) {
        self.command_buffer.map(|command| {
            let mut terminator = 0;
            let len = command.len();
            for i in 0..len {
                if command[i] == 0 {
                    terminator = i;
                    break;
                }
            }

            // A command is valid only if it starts inside the buffer,
            // ends before the beginning of the buffer, and ends after
            // it starts.
            if terminator > 0 {
                let cmd_str = str::from_utf8(&command[0..terminator]);

                match cmd_str {
                    Ok(s) => {
                        let clean_str = s.trim();

                        if clean_str.starts_with("help") {
                            let _ = self.write_bytes(b"Welcome to the process console.\n");
                            let _ = self.write_bytes(b"Valid commands are: ");
                            let _ = self
                                .write_bytes(b"help status list stop start fault process kernel\n");
                        } else if clean_str.starts_with("start") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.resume();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} resumed.\n", name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("stop") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.stop();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} stopped\n", proc_name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("fault") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.set_fault_state();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} now faulted\n", proc_name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("list") {
                            let _ = self.write_bytes(b" PID    Name                Quanta  ");
                            let _ = self.write_bytes(b"Syscalls  Dropped Callbacks  ");
                            let _ = self.write_bytes(b"Restarts    State  Grants\n");

                            // Count the number of current processes.
                            let mut count = 0;
                            self.kernel.process_each_capability(&self.capability, |_| {
                                count += 1;
                            });

                            // Start the state machine to print each separately.
                            self.write_state(WriterState::List {
                                index: -1,
                                total: count,
                            });
                        } else if clean_str.starts_with("status") {
                            let info: KernelInfo = KernelInfo::new(self.kernel);
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Total processes: {}\n",
                                    info.number_loaded_processes(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Active processes: {}\n",
                                    info.number_active_processes(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Timeslice expirations: {}\n",
                                    info.timeslice_expirations(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        } else if clean_str.starts_with("process") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                // If two processes have the same name, only
                                // print the first one we find.
                                let mut found = false;
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        if found {
                                            return;
                                        }
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            //prints process memory by moving the writer to the start state
                                            self.write_state(WriterState::ProcessStart {
                                                process_id: proc.processid(),
                                            });
                                            found = true;
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("kernel") {
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Kernel version: {}\r\n",
                                    option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown")
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();

                            // Prints kernel memory by moving the writer to the
                            // start state.
                            self.writer_state.replace(WriterState::KernelStart);
                        } else {
                            let _ = self.write_bytes(b"Valid commands are: ");
                            let _ = self
                                .write_bytes(b"help status list stop start fault process kernel\n");
                        }
                    }
                    Err(_e) => {
                        let mut console_writer = ConsoleWriter::new();
                        let _ = write(
                            &mut console_writer,
                            format_args!("Invalid command: {:?}", command),
                        );
                        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                    }
                }
            }
        });
        self.command_buffer.map(|command| {
            command[0] = 0;
        });
        self.command_index.set(0);
    }

    /// Start or iterate the state machine for an asynchronous write operation
    /// spread across multiple callback cycles.
    fn write_state(&self, state: WriterState) {
        self.writer_state.replace(self.next_state(state));
        self.create_state_buffer(self.writer_state.get());
    }

    fn write_byte(&self, byte: u8) -> Result<(), ErrorCode> {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                buf[self.queue_size.get()] = byte;
                self.queue_size.set(self.queue_size.get() + 1);
            });
            Err(ErrorCode::BUSY)
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                buffer[0] = byte;
                let _ = self.uart.transmit_buffer(buffer, 1);
            });
            Ok(())
        }
    }

    fn write_bytes(&self, bytes: &[u8]) -> Result<(), ErrorCode> {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                let size = self.queue_size.get();
                let len = cmp::min(bytes.len(), buf.len() - size);
                (&mut buf[size..size + len]).copy_from_slice(&bytes[..len]);
                self.queue_size.set(size + len);
            });
            Err(ErrorCode::BUSY)
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                let len = cmp::min(bytes.len(), buffer.len());
                // Copy elements of `bytes` into `buffer`
                (&mut buffer[..len]).copy_from_slice(&bytes[..len]);
                let _ = self.uart.transmit_buffer(buffer, len);
            });
            Ok(())
        }
    }

    /// If there is anything in the queue, copy it to the TX buffer and send
    /// it to the UART.
    ///
    /// Returns Ok(usize) with the number of bytes sent from the queue. If Ok(0)
    /// is returned, nothing was sent and the UART is free.
    fn handle_queue(&self) -> Result<usize, ErrorCode> {
        if self.tx_in_progress.get() {
            // This shouldn't happen because we should only try to handle the
            // queue when nothing else is happening, but still have the check
            // for safety.
            return Err(ErrorCode::BUSY);
        }

        self.queue_buffer.map_or(Err(ErrorCode::FAIL), |qbuf| {
            let qlen = self.queue_size.get();

            if qlen > 0 {
                self.tx_buffer.take().map_or(Err(ErrorCode::FAIL), |txbuf| {
                    let txlen = cmp::min(qlen, txbuf.len());

                    // Copy elements of the queue into the TX buffer.
                    (&mut txbuf[..txlen]).copy_from_slice(&qbuf[..txlen]);

                    // TODO: If the queue needs to print over multiple TX
                    // buffers, we need to shift the remaining contents of the
                    // queue back to index 0.
                    // if qlen > txlen {
                    //     (&mut qbuf[txlen..qlen]).copy_from_slice(&qbuf[txlen..qlen]);
                    // }

                    // Mark that we sent at least some of the queue.
                    let remaining = qlen - txlen;
                    self.queue_size.set(remaining);

                    self.tx_in_progress.set(true);
                    let _ = self.uart.transmit_buffer(txbuf, txlen);
                    Ok(txlen)
                })
            } else {
                // Queue was empty, nothing to do.
                Ok(0)
            }
        })
    }
}

impl<'a, C: ProcessManagementCapability> uart::TransmitClient for ProcessConsole<'a, C> {
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        // Reset state now that we no longer have an active transmission on the
        // UART.
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.set(false);

        // Check if we have anything queued up. If we do, let the queue
        // empty.
        let ret = self.handle_queue();
        if ret.ok() == Some(0) || ret.is_err() {
            // The queue was empty or we couldn't print the queue.

            let current_state = self.writer_state.get();
            if current_state != WriterState::Empty {
                self.write_state(current_state);
                return;
            }

            // Check if we just received and echoed a newline character, and
            // therefore need to process the received message.
            if self.execute.get() {
                self.execute.set(false);
                self.read_command();
            }
        }
    }
}
impl<'a, C: ProcessManagementCapability> uart::ReceiveClient for ProcessConsole<'a, C> {
    fn received_buffer(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        if error == uart::Error::None {
            match rx_len {
                0 => debug!("ProcessConsole had read of 0 bytes"),
                1 => {
                    self.command_buffer.map(|command| {
                        let index = self.command_index.get() as usize;
                        if read_buf[0] == ('\n' as u8) || read_buf[0] == ('\r' as u8) {
                            self.execute.set(true);
                            let _ = self.write_bytes(&['\r' as u8, '\n' as u8]);
                        } else if read_buf[0] == ('\x08' as u8) && index > 0 {
                            // Backspace, echo and remove last byte
                            // Note echo is '\b \b' to erase
                            let _ = self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
                            command[index - 1] = '\0' as u8;
                            self.command_index.set(index - 1);
                        } else if index < (command.len() - 1) && read_buf[0] < 128 {
                            // For some reason, sometimes reads return > 127 but no error,
                            // which causes utf-8 decoding failure, so check byte is < 128. -pal

                            // Echo the byte and store it
                            let _ = self.write_byte(read_buf[0]);
                            command[index] = read_buf[0];
                            self.command_index.set(index + 1);
                            command[index + 1] = 0;
                        }
                    });
                }
                _ => debug!(
                    "ProcessConsole issues reads of 1 byte, but receive_complete was length {}",
                    rx_len
                ),
            };
        }
        self.rx_in_progress.set(true);
        let _ = self.uart.receive_buffer(read_buf, 1);
    }
}
