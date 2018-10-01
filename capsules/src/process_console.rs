//! Provides implements a text console over the UART that allows
//! someone to control which processes are running.
//!
//! Protocol
//! --------
//!
//! This module provides a simple text-based console to inspect and control
//! which processes are running. The console has five commands:
//!  - 'help' prints the available commands and arguments
//!  - 'list' lists the current processes with their IDs and running state
//!  - 'stop n' stops the process with ID n
//!  - 'start n' starts the stopped process with ID n
//!  - 'restart n' restarts the process with ID n, rebooting it
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
use kernel::common::cells::TakeCell;
use kernel::hil::uart::{self, Client, UART};
use kernel::ReturnCode;

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000001;

pub static mut WRITE_BUF: [u8; 64] = [0; 64];
pub static mut READ_BUF: [u8; 16] = [0; 16];
pub static mut COMMAND_BUF: [u8; 16] = [0; 16];


pub struct ProcessConsole<'a, U: UART> {
    uart: &'a U,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    baud_rate: u32,
    command_buffer: TakeCell<'static, [u8]>,
    command_index: Cell<usize>,
    running: Cell<bool>,
}

impl<U: UART> ProcessConsole<'a, U> {
    pub fn new(
        uart: &'a U,
        baud_rate: u32,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
    ) -> ProcessConsole<'a, U> {
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
        debug!("ProcessConsole::start");
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

    // Compare if the first len bytes of str1 and str2 are the same
    fn compare(&self, str1: &[u8], str2: &[u8], len: usize) -> bool {
        let min_len = cmp::min(str1.len(), str2.len());
        let scan_len = cmp::min(len, min_len);
        for i in 0..scan_len {
            if str1[i] != str2[i] {
                return false; // Strings differ
            } else if str1[i] == 0 {
                return true; // Reached end of string
            }
        }
        return false; // Reached end of array
    }

    // Process the command in the command buffer and clear the buffer.
    fn read_command(&self) {
        self.command_buffer.map(|command| {
            debug!("Read command: {:?}", command);
            command[0] = 0;
        });
        self.command_index.set(0);
    }
}

impl<U: UART> Client for ProcessConsole<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_buffer.replace(buffer);

    }

    fn receive_complete(&self, read_buf: &'static mut [u8], rx_len: usize, error: uart::Error) {
        let mut execute = false;
        if error == uart::Error::CommandComplete {
            debug!("pc read: {} {}", read_buf[0], read_buf[0] as char);
            match rx_len {
                0 => debug!("ProcessConsole had read of 0 bytes"),
                1 => {
                    self.command_buffer.map(|command| {
                        let index = self.command_index.get() as usize;
                        if read_buf[0] == ('\n' as u8) ||
                            read_buf[0] == ('\r' as u8) {
                                execute = true;
                            } else if index < command.len() - 1{
                                command[index] = read_buf[0];
                                self.command_index.set(index + 1);
                                debug!("command[{}]: {} {}", index, read_buf[0], read_buf[0] as char);
                                command[index + 1] = 0;
                            }
                    });
                },
                _ => debug!("ProcessConsole issues reads of 1 byte, but receive_complete was length {}", rx_len),
            };
        }
        self.rx_in_progress.set(true);
        self.uart.receive(read_buf, 1);
        if execute {
            self.read_command();
        }
    }
}
