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
//!
//! ### `list` Command Fields:
//!
//! - `PID`: The identifier for the process. This can change if the process
//!   restarts.
//! - `Name`: The process name.
//! - `Quanta`: How many times this process has exceeded its alloted time
//!   quanta.
//! - `Syscalls`: The number of system calls the process has made to the kernel.
//! - `Dropped Callbacks`: How many callbacks were dropped for this process
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
//! pconsole.initialize();
//! pconsole.start();
//! ```
//!
//! Buffer use and output
//! ---------------------
//! `ProcessConsole` does not use its own write buffer for output:
//! it uses the debug!() buffer, so as not to repeat all of its buffering and
//! to maintain a correct ordering with debug!() calls. The write buffer of
//! `ProcessConsole` is used solely for echoing what someone types.
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
//! PID    Name    Quanta  Syscalls  Dropped Callbacks  Restarts    State  Grants
//! 00     blink        0       113                  0         0  Yielded    1/12
//! 01     c_hello      0         8                  0         0  Yielded    3/12
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
use core::fmt::{write, Result, Write};
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::common::cells::TakeCell;
use kernel::debug;
use kernel::hil::uart;
use kernel::introspection::KernelInfo;
use kernel::procs::ProcessType;
use kernel::Kernel;
use kernel::ReturnCode;

// Since writes are character echoes, we do not need more than 4 bytes:
// the longest write is 3 bytes for a backspace (backspace, space, backspace).
pub static mut WRITE_BUF: [u8; 10000] = [0; 10000];
pub static mut QUEUE_BUF: [u8; 10000] = [0; 10000];
pub static mut SIZE: usize = 0;
// Since reads are byte-by-byte, to properly echo what's typed,
// we can use a very small read buffer.
pub static mut READ_BUF: [u8; 4] = [0; 4];
// Commands can be up to 32 bytes long: since commands themselves are 4-5
// characters, limiting arguments to 25 bytes or so seems fine for now.
pub static mut COMMAND_BUF: [u8; 32] = [0; 32];
//const BUF_LEN :usize = 500;
//static mut STATIC_BUF: [u8; BUF_LEN] = [0; BUF_LEN];

pub struct ProcessConsole<'a, C: ProcessManagementCapability> {
    uart: &'a dyn uart::UartData<'a>,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    queue_buffer: TakeCell<'static, [u8]>,
    queue_size: Cell<usize>,
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
    kernel: &'static Kernel,
    capability: C,
}

pub struct ConsoleWriter {
    buf: [u8; 2000],
    size: usize,
}
impl ConsoleWriter {
    pub fn new() -> ConsoleWriter {
        ConsoleWriter {
            buf: [0; 2000],
            size: 0,
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }
}
impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> Result {
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
        capability: C,
    ) -> ProcessConsole<'a, C> {
        ProcessConsole {
            uart: uart,
            tx_in_progress: Cell::new(false),
            tx_buffer: TakeCell::new(tx_buffer),
            queue_buffer: TakeCell::new(queue_buffer),
            queue_size: Cell::new(0),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
            command_index: Cell::new(0),
            running: Cell::new(false),
            execute: Cell::new(false),
            kernel: kernel,
            capability: capability,
        }
    }

    pub fn start(&self) -> ReturnCode {
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                self.uart.receive_buffer(buffer, 1);
                self.running.set(true);
                //debug!("Starting process console");
            });
        }
        ReturnCode::SUCCESS
    }

    pub fn print_process_memory_map(&self, process: &dyn ProcessType) {
        // Flash
        let flash_end = process.flash_end() as usize;
        let flash_start = process.flash_start() as usize;
        let flash_protected_size = process.flash_protected() as usize;
        let flash_app_start = process.flash_non_protected_start() as usize;
        let flash_app_size = flash_end - flash_app_start;

        // SRAM addresses
        let sram_end = process.mem_end() as usize;
        let sram_grant_start = process.kernel_memory_break() as usize;
        let sram_heap_end = process.app_memory_break() as usize;
        let sram_heap_start: Option<usize> = process.get_app_heap_start();
        let sram_stack_start: Option<usize> = process.get_app_stack_start();
        let sram_stack_bottom: Option<usize> = process.get_app_stack_end();
        let sram_start = process.mem_start() as usize;

        // SRAM sizes
        let sram_grant_size = sram_end - sram_grant_start;
        let sram_grant_allocated = sram_end - sram_grant_start;

        // application statistics
        // let events_queued = process.tasks.map_or(0, |tasks| tasks.len());
        // let syscall_count = process.debug.map_or(0, |debug| debug.syscall_count);
        // let last_syscall = process.debug.map(|debug| debug.last_syscall);
        // let dropped_callback_count = process.debug.map_or(0, |debug| debug.dropped_callback_count);
        // let restart_count = process.restart_count.get();
        let mut w = ConsoleWriter::new();
        // let _ = write(&mut w,format_args!(

        //     "\
        //      𝐀𝐩𝐩: {}   -   [{:?}]\
        //      \r\n Events Queued: {}   Syscall Count: {}   Dropped Callback Count: {}\
        //      \r\n Restart Count: {}\r\n",
        //     process.process_name,
        //     process.state.get(),
        //     events_queued,
        //     syscall_count,
        //     dropped_callback_count,
        //     restart_count,
        // ));
        // self.write_bytes(&(w.buf)[..w.size]);
        // w.clear();

        // let _ = match last_syscall {
        //     Some(syscall) => write(&mut w,format_args!(" Last Syscall: {:?}\r\n", syscall)),
        //     None => write(&mut w,format_args!(" Last Syscall: None\r\n")),
        // };
        // self.write_bytes(&(w.buf)[..w.size]);
        // w.clear();
        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n\
             \r\n ╔═══════════╤══════════════════════════════════════════╗\
             \r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
             \r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
             \r\n             │ ▼ Grant      {:6} | {:6}{}\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┼───────────────────────────────────────────",
                sram_end,
                sram_grant_size,
                sram_grant_allocated,
                exceeded_check(sram_grant_size, sram_grant_allocated),
                sram_grant_start,
                sram_heap_end,
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();

        match sram_heap_start {
            Some(sram_heap_start) => {
                let sram_heap_size = sram_heap_end - sram_heap_start;
                let sram_heap_allocated = sram_grant_start - sram_heap_start;

                let _ = write(
                    &mut w,
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
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
            None => {
                let _ = write(
                    &mut w,
                    format_args!(
                        "\
                     \r\n             │ ▲ Heap            ? |      ?               S\
                     \r\n  ?????????? ┼─────────────────────────────────────────── R",
                    ),
                );
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
        }

        match (sram_heap_start, sram_stack_start) {
            (Some(sram_heap_start), Some(sram_stack_start)) => {
                let sram_data_size = sram_heap_start - sram_stack_start;
                let sram_data_allocated = sram_data_size as usize;

                let _ = write(
                    &mut w,
                    format_args!(
                        "\
                     \r\n             │ Data         {:6} | {:6}               A",
                        sram_data_size, sram_data_allocated,
                    ),
                );
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
            _ => {
                let _ = write(
                    &mut w,
                    format_args!(
                        "\
                     \r\n             │ Data              ? |      ?               A",
                    ),
                );
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
        }

        match (sram_stack_start, sram_stack_bottom) {
            (Some(sram_stack_start), Some(sram_stack_bottom)) => {
                let sram_stack_size = sram_stack_start - sram_stack_bottom;
                let sram_stack_allocated = sram_stack_start - sram_start;

                let _ = write(
                    &mut w,
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
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
            _ => {
                let _ = write(
                    &mut w,
                    format_args!(
                        "\
                     \r\n  ?????????? ┼─────────────────────────────────────────── M\
                     \r\n             │ ▼ Stack           ? |      ?",
                    ),
                );
                self.write_bytes(&(w.buf)[..w.size]);
                w.clear();
            }
        }

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┴───────────────────────────────────────────\
             \r\n             .....\
             \r\n  {:#010X} ┬─────────────────────────────────────────── F\
             \r\n             │ App Flash    {:6}                        L\
             \r\n  {:#010X} ┼─────────────────────────────────────────── A\
             \r\n             │ Protected    {:6}                        S\
             \r\n  {:#010X} ┴─────────────────────────────────────────── H\
             \r\n",
                sram_stack_bottom.unwrap_or(0),
                sram_start,
                flash_end,
                flash_app_size,
                flash_app_start,
                flash_protected_size,
                flash_start
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();
    }
    pub fn print_kernel_memory_map(&self, kernel_info: KernelInfo) {
        let stack_start: usize = kernel_info.get_kernel_stack_start() as usize;
        let stack_bottom: usize = kernel_info.get_kernel_stack_end() as usize;
        let text_start: usize = kernel_info.get_kernel_text_start() as usize;
        //let text_bottom: usize = kernel_info.get_kernel_text_end() as usize;
        let rodata_start: usize = kernel_info.get_kernel_rodata_start() as usize;
        let rodata_bottom: usize = kernel_info.get_kernel_rodata_end() as usize;
        //let init_start: usize = kernel_info.get_kernel_init_start() as usize;
        let init_bottom: usize = kernel_info.get_kernel_init_end() as usize;
        //let bss_start: usize = kernel_info.get_kernel_bss_start() as usize;
        let bss_bottom: usize = kernel_info.get_kernel_bss_end() as usize;

        let mut w = ConsoleWriter::new();

        let rodata_size = rodata_bottom - rodata_start;

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n\
             \r\n ╔═══════════╤══════════════════════════════╗\
             \r\n ║  Address  │ Region Name    Used (bytes)  ║\
             \r\n ╚{:#010X}═╪══════════════════════════════╝\
             \r\n             │   RoData     {:6}",
                rodata_bottom, rodata_size
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();

        let text_size = rodata_start - text_start;

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n  {:#010X} ┼───────────────────────────────\
             \r\n             │   Text       {:6}\
             \r\n  {:#010X} ┼───────────────────────────────",
             rodata_start, text_size, text_start
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();

        let bss_size = bss_bottom - init_bottom;

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n             .....\
             \r\n  {:#010X} ┼───────────────────────────────\
             \r\n             │   Bss        {:6}",
                bss_bottom, bss_size
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();

        let init_size = init_bottom - stack_bottom;

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n  {:#010X} ┼───────────────────────────────\
             \r\n             │   Init       {:6}",
                init_bottom, init_size
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();

        let stack_size = stack_bottom - stack_start;

        let _ = write(
            &mut w,
            format_args!(
                "\
             \r\n  {:#010X} ┼───────────────────────────────\
             \r\n             │ ▼ Stack      {:6}\
             \r\n  {:#010X} ┼───────────────────────────────\
             \r\n",
                stack_bottom, stack_size,stack_start
            ),
        );
        self.write_bytes(&(w.buf)[..w.size]);
        w.clear();
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
            //debug!("Command: {}-{} {:?}", start, terminator, command);
            // A command is valid only if it starts inside the buffer,
            // ends before the beginning of the buffer, and ends after
            // it starts.
            if terminator > 0 {
                let cmd_str = str::from_utf8(&command[0..terminator]);

                match cmd_str {
                    Ok(s) => {
                        let clean_str = s.trim();

                        if clean_str.starts_with("help") {
                            // unsafe{
                            //     let buf = b"hello_";
                            //     STATIC_BUF[..].copy_from_slice(&buf[..]);
                            //     // STATIC_PANIC_BUF[..max].copy_from_slice(&buf[..max]);
                            //     // let static_buf = &mut STATIC_PANIC_BUF;
                            //     let static_buf = &mut STATIC_BUF;
                            //     self.uart.transmit_buffer(static_buf, 6);
                            // }
                            self.write_bytes(b"Welcome to the process console.\n");
                            self.write_bytes(b"Valid commands are: help status list stop start fault process kernel\n");
                        } else if clean_str.starts_with("start") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.resume();
                                            let mut w = ConsoleWriter::new();
                                            let _ = write(&mut w,format_args!("Process {} resumed.\n", name));

                                            self.write_bytes(&(w.buf)[..w.size]);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("stop") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.stop();
                                            let mut w = ConsoleWriter::new();
                                            let _ = write(&mut w,format_args!("Process {} stopped\n", proc_name));

                                            self.write_bytes(&(w.buf)[..w.size]);
                                            //debug!("Process {} stopped", proc_name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("fault") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.set_fault_state();
                                            let mut w = ConsoleWriter::new();
                                            let _ = write(&mut w,format_args!("Process {} now faulted\n", proc_name));

                                            self.write_bytes(&(w.buf)[..w.size]);
                                            //debug!("Process {} now faulted", proc_name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("list") {
                            self.write_bytes(b" PID    Name                Quanta  Syscalls  Dropped Callbacks  Restarts    State  Grants\n");
                            self.kernel
                                .process_each_capability(&self.capability, |proc| {
                                    let info: KernelInfo = KernelInfo::new(self.kernel);

                                    let pname = proc.get_process_name();
                                    let appid = proc.appid();
                                    let (grants_used, grants_total) = info.number_app_grant_uses(appid, &self.capability);
                                    let mut w = ConsoleWriter::new();
                                    let _ = write(&mut w,format_args!(
                                        "  {:?}\t{:<20}{:6}{:10}{:19}{:10}  {:?}{:5}/{}\n",
                                        appid,
                                        pname,
                                        proc.debug_timeslice_expiration_count(),
                                        proc.debug_syscall_count(),
                                        proc.debug_dropped_callback_count(),
                                        proc.get_restart_count(),
                                        proc.get_state(),
                                        grants_used,
                                        grants_total));

                                    self.write_bytes(&(w.buf)[..w.size]);
                                });
                        } else if clean_str.starts_with("status") {
                            let info: KernelInfo = KernelInfo::new(self.kernel);
                            let mut w = ConsoleWriter::new();
                            let _ = write(&mut w,format_args!(
                                "Total processes: {}\n",
                                info.number_loaded_processes(&self.capability)));
                            self.write_bytes(&(w.buf)[..w.size]);
                            // debug!(
                            //     "Total processes: {}",
                            //     info.number_loaded_processes(&self.capability)
                            // );
                            w.clear();
                            let _ = write(&mut w,format_args!(
                                "Active processes: {}\n",
                                info.number_active_processes(&self.capability)));
                            self.write_bytes(&(w.buf)[..w.size]);
                            // debug!(
                            //     "Active processes: {}",
                            //     info.number_active_processes(&self.capability)
                            // );
                            w.clear();
                            let _ = write(&mut w,format_args!(
                                "Timeslice expirations: {}\n",
                                info.timeslice_expirations(&self.capability)));
                            self.write_bytes(&(w.buf)[..w.size]);
                            // debug!(
                            //     "Timeslice expirations: {}",
                            //     info.timeslice_expirations(&self.capability)
                            // );
                        } else if clean_str.starts_with("process"){
                            // let writer = debug::get_debug_writer();
                            // self.kernel.process_each_capability(
                            //     &self.capability, 
                            //     |proc| {
                            //         self.print_memory_map(proc);
                            // });
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            self.print_process_memory_map(proc);
                                        }
                                    },
                                );
                            });
                        }else if clean_str.starts_with("kernel"){
                            let kernel_info = KernelInfo::new(self.kernel);
                            self.print_kernel_memory_map(kernel_info);
                            // let mut w = ConsoleWriter::new();
                            // let _ = write(&mut w,format_args!(
                            //     "{}\n",
                            //     kernel_info.get_kernel_stack_start()));
                            // self.write_bytes(&(w.buf)[..w.size]);

                            // debug::panic_begin(Fn());
                            // let _ = writer.write_fmt(format_args!(
                            //     "\tKernel version {}\r\n",
                            //     option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown")
                            // ));
                            // //Flush debug buffer if needed
                            // debug::flush(writer);
                            // debug::panic_cpu_state(chip, writer);
                            // debug::panic_process_info(processes, writer);
                        } else {
                            debug!("Valid commands are: help status list stop start fault process kernel");
                        }
                    }
                    Err(_e) => debug!("Invalid command: {:?}", command),
                }
            }
        });
        self.command_buffer.map(|command| {
            command[0] = 0;
        });
        self.command_index.set(0);
    }

    fn write_byte(&self, byte: u8) -> ReturnCode {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                buf[self.queue_size.get()] = byte;
                self.queue_size.set(self.queue_size.get() + 1);
            });
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                buffer[0] = byte;
                self.uart.transmit_buffer(buffer, 1);
            });
            ReturnCode::SUCCESS
        }
    }

    fn write_bytes(&self, bytes: &[u8]) -> ReturnCode {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                let size = self.queue_size.get();
                let len = cmp::min(bytes.len(), buf.len() - size);
                (&mut buf[size..size + len]).copy_from_slice(&bytes[..len]);
                self.queue_size.set(size + len);
            });
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                let len = cmp::min(bytes.len(), buffer.len());
                // Copy elements of `bytes` into `buffer`
                (&mut buffer[..len]).copy_from_slice(&bytes[..len]);
                self.uart.transmit_buffer(buffer, len);
            });
            ReturnCode::SUCCESS
        }
    }
}

impl<'a, C: ProcessManagementCapability> uart::TransmitClient for ProcessConsole<'a, C> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.set(false);
        self.queue_buffer.map(|buf| {
            let len = self.queue_size.get();
            if len != 0 {
                self.write_bytes(&buf[..len]);
            }
            //self.uart.transmit_buffer(buf, len);
            self.queue_size.set(0);
        });

        // Check if we just received and echoed a newline character, and
        // therefore need to process the received message.
        if self.execute.get() {
            self.execute.set(false);
            self.read_command();
        }
    }
}
impl<'a, C: ProcessManagementCapability> uart::ReceiveClient for ProcessConsole<'a, C> {
    fn received_buffer(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
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
                            self.write_bytes(&['\r' as u8, '\n' as u8]);
                        } else if read_buf[0] == ('\x08' as u8) && index > 0 {
                            // Backspace, echo and remove last byte
                            // Note echo is '\b \b' to erase
                            self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
                            command[index - 1] = '\0' as u8;
                            self.command_index.set(index - 1);
                        } else if index < (command.len() - 1) && read_buf[0] < 128 {
                            // For some reason, sometimes reads return > 127 but no error,
                            // which causes utf-8 decoding failure, so check byte is < 128. -pal

                            // Echo the byte and store it
                            self.write_byte(read_buf[0]);
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
        self.uart.receive_buffer(read_buf, 1);
    }
}
