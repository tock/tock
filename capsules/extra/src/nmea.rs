// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to NMEA sentences.

use core::cell::Cell;
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::TakeCell;
use kernel::{ErrorCode, ProcessId};

/// Client for receiving NMEA sentences
pub trait NmeaClient {
    /// Called when a full NMEA sentence has been found, or if an error
    /// occurs.
    ///
    /// The possible errors are:
    /// - `BUSY`: Indicates that the hardware is busy with an existing
    ///           operation or initialisation/calibration.
    /// - `NOACK`: Failed to correctly communicate over communication protocol.
    /// - `NOSUPPORT`: Indicates that the received NMEA message was not valid UTF-8 data.
    fn callback(&self, buffer: &'static mut [u8], len: usize, status: Result<(), ErrorCode>);
}

/// A basic interface for a Nmea sentence reader
pub trait NmeaDevice<'a> {
    /// Set the client
    fn set_client(&self, client: &'a dyn NmeaClient);

    /// Read a full NMEA sentence, including the first `$`.
    /// As NMEA sentences are variable lengths this will return
    /// the first sentence that fits inside `buffer`. If a sentence
    /// doesn't fit it will be silently dropped. This allows callers to
    /// use smaller buffers if they are only interested in certain data
    /// types.
    ///
    /// When the sentence is read the `NmeaClient` callback will be called.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing
    ///           operation or initialisation/calibration.
    /// - `NOACK`: Failed to correctly communicate over communication protocol.
    fn read_sentence(
        &self,
        buffer: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Write a full NMEA sentence of `length` to the device.
    ///
    /// When the sentence is written the `NmeaClient` callback will be called.
    ///
    /// This function might return the following errors:
    /// - `BUSY`: Indicates that the hardware is busy with an existing
    ///           operation or initialisation/calibration.
    /// - `NOACK`: Failed to correctly communicate over communication protocol.
    fn write_sentence(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Nmea as usize;

#[derive(Clone, Copy, PartialEq, Default)]
enum Operation {
    #[default]
    None,
    Read,
    Write,
}

#[derive(Default)]
pub struct App {
    operation: Operation,
}

pub struct Nmea<'a> {
    driver: &'a dyn NmeaDevice<'a>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<1>, AllowRwCount<1>>,
    operation: Cell<Operation>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Nmea<'a> {
    pub fn new(
        driver: &'a dyn NmeaDevice<'a>,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<1>, AllowRwCount<1>>,
        buffer: &'static mut [u8],
    ) -> Nmea<'a> {
        Nmea {
            driver,
            apps: grant,
            operation: Cell::new(Operation::None),
            buffer: TakeCell::new(buffer),
        }
    }
}

impl NmeaClient for Nmea<'_> {
    fn callback(&self, buffer: &'static mut [u8], len: usize, status: Result<(), ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, kernel_data| {
                if app.operation == Operation::Read {
                    app.operation = Operation::None;

                    if status.is_err() {
                        kernel_data
                            .schedule_upcall(0, (into_statuscode(Err(ErrorCode::FAIL)), 0, 0))
                            .ok();
                    } else {
                        let _ = kernel_data.get_readwrite_processbuffer(0).and_then(|dest| {
                            dest.mut_enter(|dest| {
                                let copy_len = dest.len().min(len);

                                dest[0..copy_len].copy_from_slice(&buffer[0..copy_len]);
                            })
                        });

                        kernel_data
                            .schedule_upcall(0, (into_statuscode(Ok(())), len, 0))
                            .ok();
                    }
                } else if app.operation == Operation::Write {
                    app.operation = Operation::None;

                    // Only a single application can write at a time
                    if status.is_err() {
                        kernel_data
                            .schedule_upcall(0, (into_statuscode(Err(ErrorCode::FAIL)), 0, 0))
                            .ok();
                    } else {
                        kernel_data
                            .schedule_upcall(0, (into_statuscode(Ok(())), len, 0))
                            .ok();
                    }
                }
            });
        }

        self.buffer.replace(buffer);
        self.operation.set(Operation::None);
    }
}

impl SyscallDriver for Nmea<'_> {
    fn command(
        &self,
        command_num: usize,
        length: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check whether the driver exists!!
            0 => CommandReturn::success(),

            // Read a sentence
            // This is virtulised across all applications, so multiple
            // applications can read data.
            // Read/write operations can not occur in parrallel though.
            // A write can change GNSS settings so we need to ensure
            // that completes before we try to read again.
            1 => {
                if self.operation.get() == Operation::Write {
                    // A write is ongoing, we need to let that finish
                    return CommandReturn::failure(ErrorCode::BUSY);
                }

                self.apps
                    .enter(processid, |app, _| {
                        app.operation = Operation::Read;
                        self.operation.set(Operation::Read);

                        // If the buffer is already in use we return success
                        // and the app will be notified when the curren read
                        // operation completes
                        self.buffer.take().map_or(CommandReturn::success(), |buf| {
                            match self.driver.read_sentence(buf) {
                                Ok(()) => CommandReturn::success(),
                                Err((e, buffer)) => {
                                    self.buffer.replace(buffer);
                                    app.operation = Operation::None;
                                    CommandReturn::failure(e)
                                }
                            }
                        })
                    })
                    .unwrap_or_else(|err| CommandReturn::failure(err.into()))
            }

            // Write sentence
            // Only a single write operation can occur at a time.
            2 => {
                if self.operation.get() == Operation::Read {
                    // A read is ongoing, we need to let that finish
                    return CommandReturn::failure(ErrorCode::BUSY);
                }

                self.apps
                    .enter(processid, |app, kernel_data| {
                        self.buffer
                            .take()
                            .map_or(CommandReturn::failure(ErrorCode::BUSY), |buf| {
                                app.operation = Operation::Write;
                                self.operation.set(Operation::Write);

                                let _ = kernel_data.get_readonly_processbuffer(0).and_then(
                                    |sentence| {
                                        sentence.enter(|src| {
                                            let len = core::cmp::min(buf.len(), length);

                                            src[..len].copy_to_slice(&mut buf[..len]);
                                        })
                                    },
                                );

                                let len = core::cmp::min(buf.len(), length);

                                match self.driver.write_sentence(buf, len) {
                                    Ok(()) => CommandReturn::success(),
                                    Err((e, buffer)) => {
                                        self.buffer.replace(buffer);
                                        app.operation = Operation::None;
                                        CommandReturn::failure(e)
                                    }
                                }
                            })
                    })
                    .unwrap_or_else(|err| CommandReturn::failure(err.into()))
            }

            _ => CommandReturn::failure(ErrorCode::SIZE),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
