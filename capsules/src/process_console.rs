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
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait. This code
//! connects a `ProcessConsole` directly up to USART0:
//!
//! ```rust
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
//!                  Capability);
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
//!     $ tockloader listen
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
//! PID    Name    Quanta  Syscalls  Dropped Callbacks    State
//! 00     blink        0       113                  0  Yielded
//! 01     c_hello      0         8                  0  Yielded
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
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::common::cells::TakeCell;
use kernel::debug;
use kernel::hil::uart;
use kernel::introspection::KernelInfo;
use kernel::Kernel;
use kernel::ReturnCode;
use kernel::console;

use crate::console_mux;

// Since writes are character echoes, we do not need more than 4 bytes:
// the longest write is 3 bytes for a backspace (backspace, space, backspace).
pub static mut WRITE_BUF: [u8; 256] = [0; 256];
// Since reads are byte-by-byte, to properly echo what's typed,
// we can use a very small read buffer.
pub static mut READ_BUF: [u8; 32] = [0; 32];
// Commands can be up to 32 bytes long: since commands themselves are 4-5
// characters, limiting arguments to 25 bytes or so seems fine for now.
pub static mut COMMAND_BUF: [u8; 32] = [0; 32];

pub struct ProcessConsole<'a, C: ProcessManagementCapability> {
    console_mux: &'a console::Console<'a>,
    writer: TakeCell<'static, console_mux::ConsoleWriter>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    running: Cell<bool>,
    kernel: &'static Kernel,
    capability: C,
}

impl<'a, C: ProcessManagementCapability> ProcessConsole<'a, C> {
    pub fn new(
        console_mux: &'a console::Console<'a>,
        writer: &'static mut console_mux::ConsoleWriter,
        // tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
        kernel: &'static Kernel,
        capability: C,
    ) -> ProcessConsole<'a, C> {
        ProcessConsole {
            console_mux: console_mux,
            writer: TakeCell::new(writer),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
            running: Cell::new(false),
            kernel: kernel,
            capability: capability,
        }
    }

    pub fn start(&self) -> ReturnCode {
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                self.console_mux.receive_message(buffer);
                self.running.set(true);
            });
        }
        ReturnCode::SUCCESS
    }

    fn send(&self) {
        self.writer.map(|writer| {
            let (buffer, tx_len) = writer.get_tx_buffer();
            buffer.map(|tx_buffer| {
                self.console_mux.transmit_message(tx_buffer, tx_len, None);
            });
        });
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
                            console_write!(self.writer, "Welcome to the process console.\n");
                            console_write!(self.writer, "Valid commands are: help status list stop start");
                        } else if clean_str.starts_with("start") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |_i, proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.resume();
                                            console_write!(self.writer, "Process {} resumed.\n", name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("stop") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |_i, proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.stop();
                                            console_write!(self.writer, "Process {} stopped\n", proc_name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("fault") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |_i, proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.set_fault_state();
                                            console_write!(self.writer, "Process {} now faulted\n", proc_name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("list") {
                            console_write!(self.writer, " PID    Name                Quanta  Syscalls  Dropped Callbacks    State\n");
                            self.kernel
                                .process_each_capability(&self.capability, |i, proc| {
                                    let pname = proc.get_process_name();
                                    console_write!(self.writer,
                                        "  {:02}\t{:<20}{:6}{:10}{:19}  {:?}\n",
                                        i,
                                        pname,
                                        proc.debug_timeslice_expiration_count(),
                                        proc.debug_syscall_count(),
                                        proc.debug_dropped_callback_count(),
                                        proc.get_state()
                                    );
                                });
                        } else if clean_str.starts_with("status") {
                            let info: KernelInfo = KernelInfo::new(self.kernel);
                            console_write!(self.writer,
                                "Total processes: {}\n",
                                info.number_loaded_processes(&self.capability)
                            );
                            console_write!(self.writer,
                                "Active processes: {}\n",
                                info.number_active_processes(&self.capability)
                            );
                            console_write!(self.writer,
                                "Timeslice expirations: {}\n",
                                info.timeslice_expirations(&self.capability)
                            );
                        } else {
                            console_write!(self.writer, "Valid commands are: help status list stop start fault");
                        }
                    }
                    Err(_e) => {
                        console_write!(self.writer, "Invalid command: {:?}", command);
                    }
                }
                self.send();
            }
        });
        self.command_buffer.map(|command| {
            command[0] = 0;
        });
    }
}

impl<'a, C: ProcessManagementCapability> console::ConsoleClient for ProcessConsole<'a, C> {
    fn transmitted_message(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        self.writer.map(move |writer| {
            writer.set_tx_buffer(buffer);
        });
    }

    fn received_message(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
    ) {
        self.command_buffer.map(|command| {
            for (a, b) in command.iter_mut().zip(read_buf.as_ref()) {
                *a = *b;
            }
        });
        self.console_mux.receive_message(read_buf);
        self.read_command();
    }
}
