// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to a serial interface.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait.
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! # use capsules_core::console::Console;
//!
//! let console = static_init!(
//!     Console<usart::USART>,
//!     Console::new(&usart::USART0,
//!                  115200,
//!                  &mut console::WRITE_BUF,
//!                  &mut console::READ_BUF,
//!                  board_kernel.create_grant(&grant_cap)));
//! hil::uart::UART::set_client(&usart::USART0, console);
//! ```
//!
//! Usage
//! -----
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

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::uart;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Console as usize;

/// Default size for the read and write buffers used by the console.
/// Boards may pass different-size buffers if needed.
pub const DEFAULT_BUF_SIZE: usize = 64;

/// IDs for subscribed upcalls.
mod upcall {
    /// Write buffer completed callback
    pub const WRITE_DONE: usize = 1;
    /// Read buffer completed callback
    pub const READ_DONE: usize = 2;
    /// Number of upcalls. Even though we only use two, indexing starts at 0 so
    /// to be able to use indices 1 and 2 we need to specify three upcalls.
    pub const COUNT: u8 = 3;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Readonly buffer for write buffer
    ///
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const WRITE: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Writeable buffer for read buffer
    ///
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const READ: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

#[derive(Default)]
pub struct App {
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,
    read_len: usize,
}

pub struct Console<'a> {
    uart: &'a dyn uart::UartData<'a>,
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    tx_in_progress: OptionalCell<ProcessId>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: OptionalCell<ProcessId>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a> Console<'a> {
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Console<'a> {
        Console {
            uart,
            apps: grant,
            tx_in_progress: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    /// Internal helper function for setting up a new send transaction
    fn send_new(
        &self,
        processid: ProcessId,
        app: &mut App,
        kernel_data: &GrantKernelData,
        len: usize,
    ) -> Result<(), ErrorCode> {
        app.write_len = kernel_data
            .get_readonly_processbuffer(ro_allow::WRITE)
            .map_or(0, |write| write.len())
            .min(len);
        app.write_remaining = app.write_len;
        self.send(processid, app, kernel_data);
        Ok(())
    }

    /// Internal helper function for continuing a previously set up transaction.
    /// Returns `true` if this send is still active, or `false` if it has
    /// completed.
    fn send_continue(
        &self,
        processid: ProcessId,
        app: &mut App,
        kernel_data: &GrantKernelData,
    ) -> bool {
        if app.write_remaining > 0 {
            self.send(processid, app, kernel_data);

            // The send may have errored, meaning nothing is being transmitted.
            // In that case there is nothing pending and we return false. In the
            // common case, this will return true.
            self.tx_in_progress.is_some()
        } else {
            false
        }
    }

    /// Internal helper function for sending data for an existing transaction.
    /// Cannot fail. If can't send now, it will schedule for sending later.
    fn send(&self, processid: ProcessId, app: &mut App, kernel_data: &GrantKernelData) {
        if self.tx_in_progress.is_none() {
            self.tx_in_progress.set(processid);
            self.tx_buffer.take().map(|buffer| {
                let transaction_len = kernel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .and_then(|write| {
                        write.enter(|data| {
                            let remaining_data = match data
                                .get(app.write_len - app.write_remaining..app.write_len)
                            {
                                Some(remaining_data) => remaining_data,
                                None => {
                                    // A slice has changed under us and is now
                                    // smaller than what we need to write. Our
                                    // behavior in this case is documented as
                                    // undefined; the simplest thing we can do
                                    // that doesn't panic is to abort the write.
                                    // We update app.write_len so that the
                                    // number of bytes written (which is passed
                                    // to the write done upcall) is correct.
                                    app.write_len -= app.write_remaining;
                                    app.write_remaining = 0;
                                    return 0;
                                }
                            };
                            for (i, c) in remaining_data.iter().enumerate() {
                                if buffer.len() <= i {
                                    return i; // Short circuit on partial send
                                }
                                buffer[i] = c.get();
                            }
                            app.write_remaining
                        })
                    })
                    .unwrap_or(0);
                app.write_remaining -= transaction_len;
                match self.uart.transmit_buffer(buffer, transaction_len) {
                    Err((_e, tx_buffer)) => {
                        // The UART didn't start, so we will not get a transmit
                        // done callback. Need to signal the app now.
                        self.tx_buffer.replace(tx_buffer);
                        self.tx_in_progress.clear();

                        // Go ahead and signal the application
                        let written = app.write_len;
                        app.write_len = 0;
                        kernel_data.schedule_upcall(1, (written, 0, 0)).ok();
                    }
                    Ok(()) => {}
                }
            });
        } else {
            app.pending_write = true;
        }
    }

    /// Internal helper function for starting a receive operation
    fn receive_new(
        &self,
        processid: ProcessId,
        app: &mut App,
        kernel_data: &GrantKernelData,
        len: usize,
    ) -> Result<(), ErrorCode> {
        if self.rx_buffer.is_none() {
            // For now, we tolerate only one concurrent receive operation on this console.
            // Competing apps will have to retry until success.
            return Err(ErrorCode::BUSY);
        }

        let read_len = kernel_data
            .get_readwrite_processbuffer(rw_allow::READ)
            .map_or(0, |read| read.len())
            .min(len);
        if read_len > self.rx_buffer.map_or(0, |buf| buf.len()) {
            // For simplicity, impose a small maximum receive length
            // instead of doing incremental reads
            Err(ErrorCode::INVAL)
        } else {
            // Note: We have ensured above that rx_buffer is present
            app.read_len = read_len;
            self.rx_buffer
                .take()
                .map_or(Err(ErrorCode::INVAL), |buffer| {
                    self.rx_in_progress.set(processid);
                    if let Err((e, buf)) = self.uart.receive_buffer(buffer, app.read_len) {
                        self.rx_buffer.replace(buf);
                        return Err(e);
                    }
                    Ok(())
                })
        }
    }
}

impl SyscallDriver for Console<'_> {
    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length passed in
    ///   `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///   passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback) what
    ///   has been received so far.
    fn command(
        &self,
        cmd_num: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let res = self
            .apps
            .enter(processid, |app, kernel_data| {
                match cmd_num {
                    0 => Ok(()),
                    1 => {
                        // putstr
                        let len = arg1;
                        self.send_new(processid, app, kernel_data, len)
                    }
                    2 => {
                        // getnstr
                        let len = arg1;
                        self.receive_new(processid, app, kernel_data, len)
                    }
                    3 => {
                        // Abort RX
                        let _ = self.uart.receive_abort();
                        Ok(())
                    }
                    _ => Err(ErrorCode::NOSUPPORT),
                }
            })
            .map_err(ErrorCode::from);
        match res {
            Ok(Ok(())) => CommandReturn::success(),
            Ok(Err(e)) => CommandReturn::failure(e),
            Err(e) => CommandReturn::failure(e),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl uart::TransmitClient for Console<'_> {
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.take().map(|processid| {
            self.apps.enter(processid, |app, kernel_data| {
                match self.send_continue(processid, app, kernel_data) {
                    true => {
                        // Still more to send. Wait to notify the process.
                    }
                    false => {
                        // Go ahead and signal the application
                        let written = app.write_len;
                        app.write_len = 0;
                        kernel_data
                            .schedule_upcall(upcall::WRITE_DONE, (written, 0, 0))
                            .ok();
                    }
                }
            })
        });

        // If we are not printing more from the current AppSlice,
        // see if any other applications have pending messages.
        if self.tx_in_progress.is_none() {
            for cntr in self.apps.iter() {
                let processid = cntr.processid();
                let started_tx = cntr.enter(|app, kernel_data| {
                    if app.pending_write {
                        app.pending_write = false;
                        self.send_continue(processid, app, kernel_data)
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
}

impl uart::ReceiveClient for Console<'_> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        self.rx_in_progress
            .take()
            .map(|processid| {
                self.apps
                    .enter(processid, |_, kernel_data| {
                        // An iterator over the returned buffer yielding only the first `rx_len`
                        // bytes
                        let rx_buffer = buffer.iter().take(rx_len);
                        match error {
                            uart::Error::None | uart::Error::Aborted => {
                                // Receive some bytes, signal error type and return bytes to process buffer
                                let count = kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .and_then(|read| {
                                        read.mut_enter(|data| {
                                            let mut c = 0;
                                            for (a, b) in data.iter().zip(rx_buffer) {
                                                c += 1;
                                                a.set(*b);
                                            }
                                            c
                                        })
                                    })
                                    .unwrap_or(-1);

                                // Make sure we report the same number
                                // of bytes that we actually copied into
                                // the app's buffer. This is defensive:
                                // we shouldn't ever receive more bytes
                                // than will fit in the app buffer since
                                // we use the app_buffer's length when
                                // calling `receive()`. However, a buggy
                                // lower layer could return more bytes
                                // than we asked for, and we don't want
                                // to propagate that length error to
                                // userspace. However, we do return an
                                // error code so that userspace knows
                                // something went wrong.
                                //
                                // If count < 0 this means the buffer
                                // disappeared: return NOMEM.
                                let read_buffer_len = kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .map_or(0, |read| read.len());
                                let (ret, received_length) = if count < 0 {
                                    (Err(ErrorCode::NOMEM), 0)
                                } else if rx_len > read_buffer_len {
                                    // Return `SIZE` indicating that
                                    // some received bytes were dropped.
                                    // We report the length that we
                                    // actually copied into the buffer,
                                    // but also indicate that there was
                                    // an issue in the kernel with the
                                    // receive.
                                    (Err(ErrorCode::SIZE), read_buffer_len)
                                } else {
                                    // This is the normal and expected
                                    // case.
                                    (rcode, rx_len)
                                };

                                kernel_data
                                    .schedule_upcall(
                                        upcall::READ_DONE,
                                        (
                                            kernel::errorcode::into_statuscode(ret),
                                            received_length,
                                            0,
                                        ),
                                    )
                                    .ok();
                            }
                            _ => {
                                // Some UART error occurred
                                kernel_data
                                    .schedule_upcall(
                                        upcall::READ_DONE,
                                        (
                                            kernel::errorcode::into_statuscode(Err(
                                                ErrorCode::FAIL,
                                            )),
                                            0,
                                            0,
                                        ),
                                    )
                                    .ok();
                            }
                        }
                    })
                    .unwrap_or_default();
            })
            .unwrap_or_default();

        // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
        self.rx_buffer.replace(buffer);
    }
}
