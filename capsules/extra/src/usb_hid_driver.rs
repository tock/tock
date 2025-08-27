// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to USB HID devices with a simple syscall
//! interface.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::usb_hid;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Ids for read-write allow buffers
mod rw_allow {
    pub const RECV: usize = 0;
    pub const SEND: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

#[derive(Default)]
pub struct App {}

pub struct UsbHidDriver<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> {
    usb: &'a U,

    app: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    processid: OptionalCell<ProcessId>,

    send_buffer: TakeCell<'static, [u8; 64]>,
    recv_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> UsbHidDriver<'a, U> {
    pub fn new(
        usb: &'a U,
        send_buffer: &'static mut [u8; 64],
        recv_buffer: &'static mut [u8; 64],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    ) -> UsbHidDriver<'a, U> {
        UsbHidDriver {
            usb,
            app: grant,
            processid: OptionalCell::empty(),
            send_buffer: TakeCell::new(send_buffer),
            recv_buffer: TakeCell::new(recv_buffer),
        }
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> usb_hid::UsbHid<'a, [u8; 64]> for UsbHidDriver<'a, U> {
    fn send_buffer(
        &'a self,
        send: &'static mut [u8; 64],
    ) -> Result<usize, (ErrorCode, &'static mut [u8; 64])> {
        self.usb.send_buffer(send)
    }

    fn send_cancel(&'a self) -> Result<&'static mut [u8; 64], ErrorCode> {
        self.usb.send_cancel()
    }

    fn receive_buffer(
        &'a self,
        recv: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 64])> {
        self.usb.receive_buffer(recv)
    }

    fn receive_cancel(&'a self) -> Result<&'static mut [u8; 64], ErrorCode> {
        self.usb.receive_cancel()
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> usb_hid::Client<'a, [u8; 64]> for UsbHidDriver<'a, U> {
    fn packet_received(
        &'a self,
        _result: Result<(), ErrorCode>,
        buffer: &'static mut [u8; 64],
        _endpoint: usize,
    ) {
        self.processid.map(|id| {
            let _ = self.app.enter(id, |_app, kernel_data| {
                let _ = kernel_data
                    .get_readwrite_processbuffer(rw_allow::RECV)
                    .and_then(|recv| {
                        recv.mut_enter(|dest| {
                            dest.copy_from_slice(buffer);
                        })
                    });

                let _ = kernel_data.schedule_upcall(0, (0, 0, 0));
            });
        });

        self.recv_buffer.replace(buffer);
    }

    fn packet_transmitted(
        &'a self,
        _result: Result<(), ErrorCode>,
        buffer: &'static mut [u8; 64],
        _endpoint: usize,
    ) {
        self.processid.map(|id| {
            let _ = self.app.enter(id, |_app, kernel_data| {
                let _ = kernel_data.schedule_upcall(0, (1, 0, 0));
            });
        });

        // Save our send buffer so we can use it later
        self.send_buffer.replace(buffer);
    }
}

impl<'a, U: usb_hid::UsbHid<'a, [u8; 64]>> SyscallDriver for UsbHidDriver<'a, U> {
    // Subscribe to UsbHidDriver events.
    //
    // ### `subscribe_num`
    //
    // - `0`: Subscribe to interrupts from HID events.
    //        The callback signature is `fn(direction: u32)`
    //        `fn(0)` indicates a packet was received
    //        `fn(1)` indicates a packet was transmitted

    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let can_access = self.processid.map_or_else(
            || {
                self.processid.set(processid);
                true
            },
            |owning_app| {
                // Check if we own the HID device
                owning_app == processid
            },
        );

        if !can_access {
            return CommandReturn::failure(ErrorCode::BUSY);
        }

        match command_num {
            0 => CommandReturn::success(),

            // Send data
            1 => self
                .app
                .enter(processid, |_, kernel_data| {
                    self.processid.set(processid);
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::SEND)
                        .and_then(|send| {
                            send.enter(|data| {
                                self.send_buffer.take().map_or(
                                    CommandReturn::failure(ErrorCode::BUSY),
                                    |buf| {
                                        // Copy the data into the static buffer
                                        data.copy_to_slice(buf);

                                        let _ = self.usb.send_buffer(buf);
                                        CommandReturn::success()
                                    },
                                )
                            })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::RESERVE))
                })
                .unwrap_or_else(|err| err.into()),

            // Allow receive
            2 => self
                .app
                .enter(processid, |_app, _| {
                    self.processid.set(processid);
                    if let Some(buf) = self.recv_buffer.take() {
                        match self.usb.receive_buffer(buf) {
                            Ok(()) => CommandReturn::success(),
                            Err((err, buffer)) => {
                                self.recv_buffer.replace(buffer);
                                CommandReturn::failure(err)
                            }
                        }
                    } else {
                        CommandReturn::failure(ErrorCode::BUSY)
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Cancel send
            3 => self
                .app
                .enter(processid, |_app, _| {
                    self.processid.set(processid);
                    match self.usb.receive_cancel() {
                        Ok(buf) => {
                            self.recv_buffer.replace(buf);
                            CommandReturn::success()
                        }
                        Err(err) => CommandReturn::failure(err),
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Cancel receive
            4 => self
                .app
                .enter(processid, |_app, _| {
                    self.processid.set(processid);
                    match self.usb.receive_cancel() {
                        Ok(buf) => {
                            self.recv_buffer.replace(buf);
                            CommandReturn::success()
                        }
                        Err(err) => CommandReturn::failure(err),
                    }
                })
                .unwrap_or_else(|err| err.into()),

            // Send or receive
            // This command has two parts.
            //    Part 1: Receive
            //            This will allow receives, the same as the Allow
            //            receive command above. If data is ready to receive
            //            the `packet_received()` callback will be called.
            //            When this happens the client callback will be
            //            scheduled and no send event will occur.
            //    Part 2: Send
            //            If no receive occurs we will be left in a start where
            //            future recieves will be allowed. This is the same
            //            outcome as calling the Allow receive command.
            //            As well as that we will then send the data in the
            //            send buffer.
            5 => self
                .app
                .enter(processid, |_app, kernel_data| {
                    // First we try to setup a receive. If there is already
                    // a receive we return `ErrorCode::ALREADY`. If the
                    // receive fails we return an error.
                    if let Some(buf) = self.recv_buffer.take() {
                        if let Err((err, buffer)) = self.usb.receive_buffer(buf) {
                            self.recv_buffer.replace(buffer);
                            return CommandReturn::failure(err);
                        }
                    } else {
                        return CommandReturn::failure(ErrorCode::ALREADY);
                    }

                    // If we were able to setup a read then next we do the
                    // transmit.
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::SEND)
                        .and_then(|send| {
                            send.enter(|data| {
                                self.send_buffer.take().map_or(
                                    CommandReturn::failure(ErrorCode::BUSY),
                                    |buf| {
                                        // Copy the data into the static buffer
                                        data.copy_to_slice(buf);

                                        let _ = self.usb.send_buffer(buf);
                                        CommandReturn::success()
                                    },
                                )
                            })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
                })
                .unwrap_or_else(|err| err.into()),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.app.enter(processid, |_, _| {})
    }
}
