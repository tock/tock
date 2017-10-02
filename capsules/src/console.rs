//! Provides userspace with access to a serial interface.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait.
//!
//! ``rust
//! let console = static_init!(
//!     Console<usart::USART>,
//!     Console::new(&usart::USART0,
//!                  115200,
//!                  &mut console::WRITE_BUF,
//!                  kernel::Grant::create()));
//! hil::uart::UART::set_client(&usart::USART0, console);
//! ```
//!
//! Usage
//! -----
//!
//! Currently, only writing buffers to the serial device is implemented.
//!
//! The user must perform three steps in order to write a buffer:
//!
//! ```c
//! // (Optional) Set a callback to be invoked when the buffer has been written
//! subscribe(CONSOLE_DRIVER_NUM, 1, my_callback);
//! // Share the buffer from userspace with the driver
//! allow(CONSOLE_DRIVER_NUM, buffer, buffer_len_in_bytes);
//! // Initiate the transaction
//! command(CONSOLE_DRIVER_NUM, 1, len_to_write_in_bytes)
//! ```
//!
//! When the buffer has been written successfully, the buffer is released from
//! the driver. Successive writes must call `allow` each time a buffer is to be
//! written.

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Grant, Callback, Shared, Driver, ReturnCode};
use kernel::common::take_cell::TakeCell;
use kernel::hil::uart::{self, UART, Client};
use kernel::process::Error;

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000001;

pub struct App {
    write_callback: Option<Callback>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,
}

impl Default for App {
    fn default() -> App {
        App {
            write_callback: None,
            write_buffer: None,
            write_len: 0,
            write_remaining: 0,
            pending_write: false,
        }
    }
}

pub static mut WRITE_BUF: [u8; 64] = [0; 64];

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Grant<App>,
    in_progress: Cell<Option<AppId>>,
    tx_buffer: TakeCell<'static, [u8]>,
    baud_rate: u32,
}

impl<'a, U: UART> Console<'a, U> {
    pub fn new(uart: &'a U,
               baud_rate: u32,
               tx_buffer: &'static mut [u8],
               grant: Grant<App>)
               -> Console<'a, U> {
        Console {
            uart: uart,
            apps: grant,
            in_progress: Cell::new(None),
            tx_buffer: TakeCell::new(tx_buffer),
            baud_rate: baud_rate,
        }
    }

    pub fn initialize(&self) {
        self.uart.init(uart::UARTParams {
            baud_rate: self.baud_rate,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }

    /// Internal helper function for setting up a new send transaction
    fn send_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        match app.write_buffer.take() {
            Some(slice) => {
                app.write_len = cmp::min(len, slice.len());
                app.write_remaining = app.write_len;
                self.send(app_id, app, slice);
                ReturnCode::SUCCESS
            }
            None => ReturnCode::EBUSY,
        }
    }

    /// Internal helper function for continuing a previously set up transaction
    /// Returns true if this send is still active, or false if it has completed
    fn send_continue(&self, app_id: AppId, app: &mut App) -> Result<bool, ReturnCode> {
        if app.write_remaining > 0 {
            app.write_buffer.take().map_or(Err(ReturnCode::ERESERVE), |slice| {
                self.send(app_id, app, slice);
                Ok(true)
            })
        } else {
            Ok(false)
        }
    }

    /// Internal helper function for sending data for an existing transaction.
    /// Cannot fail. If can't send now, it will schedule for sending later.
    fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {
        if self.in_progress.get().is_none() {
            self.in_progress.set(Some(app_id));
            self.tx_buffer.take().map(|buffer| {
                let mut transaction_len = app.write_remaining;
                for (i, c) in slice.as_ref()[slice.len() - app.write_remaining..slice.len()]
                    .iter()
                    .enumerate() {
                    if buffer.len() <= i {
                        break;
                    }
                    buffer[i] = *c;
                }

                // Check if everything we wanted to print
                // fit in the buffer.
                if app.write_remaining > buffer.len() {
                    transaction_len = buffer.len();
                    app.write_remaining -= buffer.len();
                    app.write_buffer = Some(slice);
                } else {
                    app.write_remaining = 0;
                }

                self.uart.transmit(buffer, transaction_len);
            });
        } else {
            app.pending_write = true;
            app.write_buffer = Some(slice);
        }
    }
}

impl<'a, U: UART> Driver for Console<'a, U> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.write_buffer = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    app.write_callback = Some(callback);
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| {
                    match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    }
                })
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Prints a buffer passed through `allow` up to the length passed in
    ///        `arg1`
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* putstr */ => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.send_new(appid, app, len)
                }).unwrap_or_else(|err| {
                    match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    }
                })
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_buffer.replace(buffer);
        self.in_progress.get().map(|appid| {
            self.in_progress.set(None);
            self.apps.enter(appid, |app, _| {
                match self.send_continue(appid, app) {
                    Ok(more_to_send) => {
                        if !more_to_send {
                            // Go ahead and signal the application
                            let written = app.write_len;
                            app.write_len = 0;
                            app.write_callback.map(|mut cb| { cb.schedule(written, 0, 0); });
                        }
                    }
                    Err(return_code) => {
                        // XXX This shouldn't ever happen?
                        app.write_len = 0;
                        app.write_remaining = 0;
                        app.pending_write = false;
                        let r0 = isize::from(return_code) as usize;
                        app.write_callback.map(|mut cb| { cb.schedule(r0, 0, 0); });
                    }
                }
            })
        });

        // If we are not printing more from the current AppSlice,
        // see if any other applications have pending messages.
        if self.in_progress.get().is_none() {
            for cntr in self.apps.iter() {
                let started_tx = cntr.enter(|app, _| {
                    if app.pending_write {
                        app.pending_write = false;
                        match self.send_continue(app.appid(), app) {
                            Ok(more_to_send) => more_to_send,
                            Err(return_code) => {
                                // XXX This shouldn't ever happen?
                                app.write_len = 0;
                                app.write_remaining = 0;
                                app.pending_write = false;
                                let r0 = isize::from(return_code) as usize;
                                app.write_callback.map(|mut cb| { cb.schedule(r0, 0, 0); });
                                false
                            }
                        }
                    } else {
                        false
                    }
                });
                if started_tx {
                    break;
                }
            }
        }
    }

    fn receive_complete(&self,
                        _rx_buffer: &'static mut [u8],
                        _rx_len: usize,
                        _error: uart::Error) {
        // this is currently unimplemented for console
    }
}
