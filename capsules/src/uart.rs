//! Provides userspace with access to a serial interface.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait.
//!
//! ```rust
//! let uart_driver = static_init!(
//!     UartDriver<usart::USART>,
//!     UartDriver::new(&usart::USART0,
//!                  115200,
//!                  &mut UartDriver::WRITE_BUF,
//!                  &mut UartDriver::READ_BUF,
//!                  kernel::Grant::create()));
//! hil::uart::UART::set_client(&usart::USART0, uart_driver);
//! ```
//!
//! Usage
//! -----
//!
//! The user must perform three steps in order to write a buffer:
//!
//! ```c
//! // (Optional) Set a callback to be invoked when the buffer has been written
//! subscribe(UartDriver_DRIVER_NUM, 1, my_callback);
//! // Share the buffer from userspace with the driver
//! allow(UartDriver_DRIVER_NUM, buffer, buffer_len_in_bytes);
//! // Initiate the transaction
//! command(UartDriver_DRIVER_NUM, 1, len_to_write_in_bytes)
//! ```
//!
//! When the buffer has been written successfully, the buffer is released from
//! the driver. Successive writes must call `allow` each time a buffer is to be
//! written.

use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000001;

#[derive(Default)]
pub struct App {
    write_uart: usize,
    write_callback: Option<Callback>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,

    read_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    read_len: usize,
}

pub static mut WRITE_BUF0: [u8; 64] = [0; 64];
pub static mut READ_BUF0: [u8; 64] = [0; 64];

pub static mut WRITE_BUF1: [u8; 64] = [0; 64];
pub static mut READ_BUF1: [u8; 64] = [0; 64];

pub struct UartDriver<'a, U: 'static + hil::uart::UART> {
    uarts: &'a mut [&'a mut Uart<'a, U>],
    apps: [Grant<App>; 2],
}

impl<'a, U: 'static + hil::uart::UART> UartDriver<'a, U> {
    pub fn new(
        uarts: &'a mut [&'static mut Uart<'a, U>],
        apps: [Grant<App>; 2]
        ) -> UartDriver<'a, U> {
        UartDriver { uarts, apps }
    }

    pub fn initialize(&mut self) {
        for (i, uart) in self.uarts.iter_mut().enumerate() {
            uart.index = i;
        }
    }


    pub fn transmit_complete(&self, uart_index: usize, buffer: &'static mut [u8], _error: hil::uart::Error){
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.uarts[uart_index].tx_buffer.replace(buffer);
        self.uarts[uart_index].tx_in_progress.take().map(|appid| {
            self.apps[uart_index].enter(appid, |app, _| {
                match self.uarts[uart_index].send_continue(appid, app) {
                    Ok(more_to_send) => {
                        if !more_to_send {
                            // Go ahead and signal the application
                            let written = app.write_len;
                            app.write_len = 0;
                            app.write_callback.map(|mut cb| {
                                cb.schedule(written, uart_index, 0);
                            });
                        }
                    }
                    Err(return_code) => {
                        // XXX This shouldn't ever happen?
                        app.write_len = 0;
                        app.write_remaining = 0;
                        app.pending_write = false;
                        let r0 = isize::from(return_code) as usize;
                        app.write_callback.map(|mut cb| {
                            cb.schedule(r0, uart_index, 0);
                        });
                    }
                }
            })
        });

        // If we are not printing more from the current AppSlice,
        // see if any other applications have pending messages.
        if self.uarts[uart_index].tx_in_progress.is_none() {
            for cntr in self.apps[uart_index].iter() {
                let started_tx = cntr.enter(|app, _| {
                    if app.pending_write {
                        app.pending_write = false;
                        match self.uarts[uart_index].send_continue(app.appid(), app) {
                            Ok(more_to_send) => more_to_send,
                            Err(return_code) => {
                                // XXX This shouldn't ever happen?
                                app.write_len = 0;
                                app.write_remaining = 0;
                                app.pending_write = false;
                                let r0 = isize::from(return_code) as usize;
                                app.write_callback.map(|mut cb| {
                                    cb.schedule(r0, uart_index, 0);
                                });
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

    pub fn receive_complete(&self, uart_index: usize, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.uarts[uart_index].rx_in_progress
                .take()
                .map(|appid| {
                    self.apps[uart_index]
                        .enter(appid, |app, _| {
                            app.read_callback.map(|mut cb| {
                                // An iterator over the returned buffer yielding only the first `rx_len`
                                // bytes
                                let rx_buffer = buffer.iter().take(rx_len);
                                match error {
                                    hil::uart::Error::CommandComplete | hil::uart::Error::Aborted => {
                                        // Receive some bytes, signal error type and return bytes to process buffer
                                        if let Some(mut app_buffer) = app.read_buffer.take() {
                                            for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
                                                *a = *b;
                                            }
                                            let rettype = if error == hil::uart::Error::CommandComplete
                                            {
                                                ReturnCode::SUCCESS
                                            } else {
                                                ReturnCode::ECANCEL
                                            };
                                            debug!("scheduled cb");
                                            cb.schedule(From::from(rettype), rx_len, uart_index);
                                        } else {
                                            // Oops, no app buffer
                                            cb.schedule(From::from(ReturnCode::EINVAL), uart_index, 0);
                                        }
                                    }
                                    _ => {
                                        // Some UART error occurred
                                        cb.schedule(From::from(ReturnCode::FAIL), uart_index, 0);
                                    }
                                }
                            });
                        }).unwrap_or_default();
                }).unwrap_or_default();

            // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
            self.uarts[uart_index].rx_buffer.replace(buffer);
    }

}

pub struct Uart<'a, U: 'static + hil::uart::UART> {
    parent: Option<&'static UartDriver<'static, U>>,
    hw: Option<&'a U>,
    index: usize,
    tx_in_progress: OptionalCell<AppId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: OptionalCell<AppId>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<U: 'static + hil::uart::UART> Uart<'a, U> {
    pub const fn new(
        index: usize,

    ) -> Uart<'a, U> {
        Uart {
            parent: None,
            hw: None,
            index: index,
            tx_in_progress: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
        }
    }

    pub fn initialize(&mut self, uart: &'a U, tx_buffer: &'static mut [u8],rx_buffer: &'static mut [u8], parent: &'static UartDriver<'a, U>){
        self.hw = Some(uart);
        self.tx_buffer = TakeCell::new(tx_buffer);
        self.rx_buffer = TakeCell::new(rx_buffer);
        self.parent = Some(parent)
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
            app.write_buffer
                .take()
                .map_or(Err(ReturnCode::ERESERVE), |slice| {
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
        if self.tx_in_progress.is_none() {
            self.tx_in_progress.set(app_id);
            self.tx_buffer.take().map(|buffer| {
                let mut transaction_len = app.write_remaining;
                for (i, c) in slice.as_ref()[slice.len() - app.write_remaining..slice.len()]
                    .iter()
                    .enumerate()
                {
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
                if let Some(hw) = self.hw {
                    hw.transmit(buffer, transaction_len);
                }
            });
        } else {
            app.pending_write = true;
            app.write_buffer = Some(slice);
        }
    }

    /// Internal helper function for starting a receive operation
    fn receive_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        if self.rx_buffer.is_none() {
            // For now, we tolerate only one concurrent receive operation on this UartDriver.
            // Competing apps will have to retry until success.
            return ReturnCode::EBUSY;
        }

        match app.read_buffer {
            Some(ref slice) => {
                let read_len = cmp::min(len, slice.len());
                if read_len > self.rx_buffer.map_or(0, |buf| buf.len()) {
                    // For simplicity, impose a small maximum receive length
                    // instead of doing incremental reads
                    ReturnCode::EINVAL
                } else {
                    // Note: We have ensured above that rx_buffer is present
                    app.read_len = read_len;
                    self.rx_buffer.take().map(|buffer| {
                        self.rx_in_progress.set(app_id);
                        if let Some(hw) = self.hw {
                            hw.receive(buffer, app.read_len);
                        }
                    });
                    ReturnCode::SUCCESS
                }
            }
            None => {
                // Must supply read buffer before performing receive operation
                ReturnCode::EINVAL
            }
        }
    }

}

impl<'a, U: 'static + hil::uart::UART + hil::uart::Client> Driver for UartDriver<'a, U> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(&self, appid: AppId, arg2: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        let allow_num = arg2 as u16;
        let uart_num =  (arg2 >> 16) as usize;

        match allow_num {
            1 => self.apps[uart_num]
                .enter(appid, |app, _| {
                    app.write_buffer = slice;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into()),
            2 => self.apps[uart_num]
                .enter(appid, |app, _| {
                    app.read_buffer = slice;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(&self, arg1: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        let subscribe_num = arg1 as u16;
        let uart_num =  (arg1 >> 16) as usize;

        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.apps[uart_num]
                .enter(app_id, |app, _| {
                    app.write_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.apps[uart_num]
                .enter(app_id, |app, _| {
                    app.read_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback)
    ///        what has been received so far.
    fn command(&self, arg0: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        let cmd_num = arg0 as u16;
        let uart_num =  (arg0 >> 16) as usize;

        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* putstr */ => {
                let len = arg1;
                self.apps[uart_num]
                .enter(appid, |app, _| {
                    self.uarts[uart_num].send_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr */ => {
                let len = arg1;
                self.apps[uart_num].enter(appid, |app, _| {
                    self.uarts[uart_num].receive_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            3 /* abort rx */ => {
                self.apps[uart_num]
                .enter(appid, |app, _| {
                    if let Some(hw) = self.uarts[app.write_uart].hw {
                        hw.abort_receive();
                    }
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}


impl<U: hil::uart::UART> hil::uart::Client for Uart<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], error: hil::uart::Error) {
        if let Some(parent) = self.parent {
            parent.transmit_complete(self.index, buffer, error);
        }
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        if let Some(parent) = self.parent {
            parent.receive_complete(self.index, buffer, rx_len, error);
        }
    }
}
