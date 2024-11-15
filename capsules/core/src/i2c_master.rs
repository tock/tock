// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for an I2C Master interface.

use enum_primitive::enum_from_primitive;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::i2c;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::I2cMaster as usize;

/// Ids for read-write allow buffers
mod rw_allow {
    pub const BUFFER: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

#[derive(Default)]
pub struct App;

pub const BUFFER_LENGTH: usize = 64;

struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    processid: ProcessId,
    /// The total amount to transmit
    read_len: OptionalCell<usize>,
}

pub struct I2CMasterDriver<'a, I: i2c::I2CMultiDevice> {
    i2c: &'a I,
    buf: TakeCell<'static, [u8]>,
    tx: MapCell<Transaction>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
}

impl<'a, I: i2c::I2CMultiDevice> I2CMasterDriver<'a, I> {
    pub fn new(
        i2c: &'a I,
        buf: &'static mut [u8],
        apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    ) -> I2CMasterDriver<'a, I> {
        I2CMasterDriver {
            i2c,
            buf: TakeCell::new(buf),
            tx: MapCell::empty(),
            apps,
        }
    }

    fn operation(
        &self,
        processid: ProcessId,
        kernel_data: &GrantKernelData,
        command: Cmd,
        addr: u8,
        wlen: usize,
        rlen: usize,
    ) -> Result<(), ErrorCode> {
        kernel_data
            .get_readwrite_processbuffer(rw_allow::BUFFER)
            .and_then(|buffer| {
                buffer.enter(|app_buffer| {
                    self.buf.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                        app_buffer[..wlen].copy_to_slice(&mut buffer[..wlen]);

                        let read_len = if rlen == 0 {
                            OptionalCell::empty()
                        } else {
                            OptionalCell::new(rlen)
                        };
                        self.tx.put(Transaction {
                            processid,
                            read_len,
                        });

                        let res = match command {
                            Cmd::Ping => {
                                self.buf.put(Some(buffer));
                                return Err(ErrorCode::INVAL);
                            }
                            Cmd::Write => {
                                self.i2c.enable();
                                self.i2c.set_address(addr);
                                self.i2c.write(buffer, wlen)
                            }
                            Cmd::Read => {
                                self.i2c.enable();
                                self.i2c.set_address(addr);
                                self.i2c.read(buffer, rlen)
                            }
                            Cmd::WriteRead => {
                                self.i2c.enable();
                                self.i2c.set_address(addr);
                                self.i2c.write_read(buffer, wlen, rlen)
                            }
                        };
                        match res {
                            Ok(()) => Ok(()),
                            Err((error, data)) => {
                                self.buf.put(Some(data));
                                Err(error.into())
                            }
                        }
                    })
                })
            })
            .unwrap_or(Err(ErrorCode::INVAL))
    }
}

use enum_primitive::cast::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Cmd {
    Ping = 0,
    Write = 1,
    Read = 2,
    WriteRead = 3,
}
}

impl<'a, I: i2c::I2CMultiDevice> SyscallDriver for I2CMasterDriver<'a, I> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: buffer for command

    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `0`: Write buffer completed callback

    /// Initiate transfers
    fn command(
        &self,
        cmd_num: usize,
        arg1: usize,
        arg2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        if let Some(cmd) = Cmd::from_usize(cmd_num) {
            match cmd {
                Cmd::Ping => CommandReturn::success(),
                Cmd::Write => self
                    .apps
                    .enter(processid, |_, kernel_data| {
                        let addr = arg1 as u8;
                        let write_len = arg2;
                        self.operation(processid, kernel_data, Cmd::Write, addr, write_len, 0)
                            .into()
                    })
                    .unwrap_or_else(|err| err.into()),
                Cmd::Read => self
                    .apps
                    .enter(processid, |_, kernel_data| {
                        let addr = arg1 as u8;
                        let read_len = arg2;
                        self.operation(processid, kernel_data, Cmd::Read, addr, 0, read_len)
                            .into()
                    })
                    .unwrap_or_else(|err| err.into()),
                Cmd::WriteRead => {
                    let addr = arg1 as u8;
                    let write_len = arg1 >> 8; // can extend to 24 bit write length
                    let read_len = arg2; // can extend to 32 bit read length
                    self.apps
                        .enter(processid, |_, kernel_data| {
                            self.operation(
                                processid,
                                kernel_data,
                                Cmd::WriteRead,
                                addr,
                                write_len,
                                read_len,
                            )
                            .into()
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }
        } else {
            CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, I: i2c::I2CMultiDevice> i2c::I2CClient for I2CMasterDriver<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        self.tx.take().map(|tx| {
            self.apps.enter(tx.processid, |_, kernel_data| {
                if let Some(read_len) = tx.read_len.take() {
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::BUFFER)
                        .and_then(|app_buffer| {
                            app_buffer.mut_enter(|app_buffer| {
                                app_buffer[..read_len].copy_from_slice(&buffer[..read_len]);
                            })
                        });
                }

                // signal to driver that tx complete
                kernel_data
                    .schedule_upcall(
                        0,
                        (
                            kernel::errorcode::into_statuscode(status.map_err(|e| e.into())),
                            0,
                            0,
                        ),
                    )
                    .ok();
            })
        });

        self.i2c.disable();

        //recover buffer
        self.buf.put(Some(buffer));
    }
}
