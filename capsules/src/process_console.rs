//! Provides implements a text console over the UART that allows
//! someone to control which processes are running.
//!
//! Protocol
//! --------
//!
//! This module provides a simple text-based console to inspect and control
//! which processes are running. The console has five commands:
//!  - 'help' prints the available commands and arguments
//!  - 'status' prints the current system status
//!  - 'list' lists the current processes with their IDs and running state
//!  - 'stop n' stops the process with ID n
//!  - 'start n' starts the stopped process with ID n
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait.
//!
//! ```rust
//! let console = static_init!(
//!     ProcessConsole<usart::USART>,
//!     ProcessConsole::new(&usart::USART0,
//!                  115200,
//!                  &mut console::WRITE_BUF,
//!                  &mut console::READ_BUF,
//!                  &mut console::COMMAND_BUF);
//! hil::uart::UART::set_client(&usart::USART0, console);
//! ```

use core::cell::Cell;
use core::cmp;
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::common::cells::TakeCell;
use kernel::hil::uart::{self, Client, UART};
use kernel::introspection::KernelInfo;
use kernel::Kernel;
use kernel::ReturnCode;

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000001;

pub static mut WRITE_BUF: [u8; 64] = [0; 64];
pub static mut READ_BUF: [u8; 16] = [0; 16];
pub static mut COMMAND_BUF: [u8; 16] = [0; 16];

pub struct ProcessConsole<'a, U: UART, C: ProcessManagementCapability> {
    uart: &'a U,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    baud_rate: u32,
    command_buffer: TakeCell<'static, [u8]>,
    command_index: Cell<usize>,
    running: Cell<bool>,
    kernel: &'static Kernel,
    capability: C,
}

impl<U: UART, C: ProcessManagementCapability> ProcessConsole<'a, U, C> {
    pub fn new(
        uart: &'a U,
        baud_rate: u32,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
        kernel: &'static Kernel,
        capability: C,
    ) -> ProcessConsole<'a, U, C> {
        ProcessConsole {
            uart: uart,
            tx_in_progress: Cell::new(false),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            baud_rate: baud_rate,
            command_buffer: TakeCell::new(cmd_buffer),
            command_index: Cell::new(0),
            running: Cell::new(false),
            kernel: kernel,
            capability: capability,
        }
    }

    pub fn initialize(&self) {
        self.uart.configure(uart::UARTParameters {
            baud_rate: self.baud_rate,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }

    pub fn start(&self) -> ReturnCode {
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                self.uart.receive(buffer, 1);
                self.running.set(true);
                debug!("Starting process console");
            });
        }
        ReturnCode::SUCCESS
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
                            debug!("Welcome to the process console.");
                            debug!("Valid commands are: help status list stop start");
                        } else if clean_str.starts_with("start") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel.process_each_capability(
                                    &self.capability,
                                    |_i, proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.resume();
                                            debug!("Process {} resumed.", name);
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
                                            debug!("Process {} stopped", proc_name);
                                        }
                                    },
                                );
                            });
                        } else if clean_str.starts_with("list") {
                            debug!(" PID    Name\tSlices  Syscalls  Dropped Callbacks    State");
                            self.kernel
                                .process_each_capability(&self.capability, |i, proc| {
                                    let pname = proc.get_process_name();
                                    debug!(
                                        "  {:02}\t{}\t{:6}{:10}{:19}  {:?}",
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
                            debug!(
                                "Total processes: {}",
                                info.number_loaded_processes(&self.capability)
                            );
                            debug!(
                                "Active processes: {}",
                                info.number_active_processes(&self.capability)
                            );
                            debug!(
                                "Timeslice expirations: {}",
                                info.timeslice_expirations(&self.capability)
                            );
                        } else {
                            debug!("Valid commands are: help status list stop start");
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
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                buffer[0] = byte;
                self.uart.transmit(buffer, 1);
            });
            ReturnCode::SUCCESS
        }
    }

    fn write_bytes(&self, bytes: &[u8]) -> ReturnCode {
        if self.tx_in_progress.get() {
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                let len = cmp::min(bytes.len(), buffer.len());
                for i in 0..len {
                    buffer[i] = bytes[i];
                }
                self.uart.transmit(buffer, len);
            });
            ReturnCode::SUCCESS
        }
    }
}

impl<U: UART, C: ProcessManagementCapability> Client for ProcessConsole<'a, U, C> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.set(false);
    }

    fn receive_complete(&self, read_buf: &'static mut [u8], rx_len: usize, error: uart::Error) {
        let mut execute = false;
        if error == uart::Error::CommandComplete {
            match rx_len {
                0 => debug!("ProcessConsole had read of 0 bytes"),
                1 => {
                    self.command_buffer.map(|command| {
                        let index = self.command_index.get() as usize;
                        if read_buf[0] == ('\n' as u8) || read_buf[0] == ('\r' as u8) {
                            execute = true;
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
        self.uart.receive(read_buf, 1);
        if execute {
            self.read_command();
        }
    }
}
