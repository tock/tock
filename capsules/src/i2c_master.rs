//! Driver for an I2C Master interface.

use enum_primitive::enum_from_primitive;
use kernel::common::cells::{MapCell, OptionalCell, TakeCell};
use kernel::hil::i2c;
use kernel::{
    AppId, Callback, CommandReturn, Driver, ErrorCode, Grant, Read, ReadWrite, ReadWriteAppSlice,
};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::I2cMaster as usize;

#[derive(GrantDefault)]
pub struct App {
    #[subscribe_num = 1]
    callback: Callback,
    slice: ReadWriteAppSlice,
}

pub static mut BUF: [u8; 64] = [0; 64];

struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    app_id: AppId,
    /// The total amount to transmit
    read_len: OptionalCell<usize>,
}

pub struct I2CMasterDriver<'a, I: 'a + i2c::I2CMaster> {
    i2c: &'a I,
    buf: TakeCell<'static, [u8]>,
    tx: MapCell<Transaction>,
    apps: Grant<App>,
}

impl<'a, I: 'a + i2c::I2CMaster> I2CMasterDriver<'a, I> {
    pub fn new(i2c: &'a I, buf: &'static mut [u8], apps: Grant<App>) -> I2CMasterDriver<'a, I> {
        I2CMasterDriver {
            i2c,
            buf: TakeCell::new(buf),
            tx: MapCell::empty(),
            apps,
        }
    }

    fn operation(&self, app_id: AppId, app: &mut App, command: Cmd, addr: u8, wlen: u8, rlen: u8) {
        // TODO(alevy) this function used to try and return ReturnCodes, but would always return
        // ENOSUPPORT and all call-sites simply ignore the return value. Nonetheless, some error
        // handling is probably useful. Comments inline where there used to be non-success results.
        self.apps
            .enter(app_id, |_, _| {
                // TODO(alevy): if app.slice.map doesn't have a slice, we would have returned
                // EINVAL here. I.e., the driver is attempting an operation without sharing memory.
                app.slice.map_or((), |app_buffer| {
                    self.buf.take().map(|buffer| {
                        buffer[..(wlen as usize)].copy_from_slice(&app_buffer[..(wlen as usize)]);

                        let read_len: OptionalCell<usize>;
                        if rlen == 0 {
                            read_len = OptionalCell::empty();
                        } else {
                            read_len = OptionalCell::new(rlen as usize);
                        }
                        self.tx.put(Transaction { app_id, read_len });

                        match command {
                            Cmd::Ping => (), // Unexpected, shouldn't get here (was ReturnCode::EINVAL)
                            Cmd::Write => self.i2c.write(addr, buffer, wlen),
                            Cmd::Read => self.i2c.read(addr, buffer, rlen),
                            Cmd::WriteRead => self.i2c.write_read(addr, buffer, wlen, rlen),
                        }
                    });
                    // TODO(alevy): if buf.take() returned None, the I2C hadn't returned the
                    // buffer. This shouldn't happen and previous this returned ENOMEM
                })
            })
            .expect("Appid does not map to app");
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

impl<'a, I: 'a + i2c::I2CMaster> Driver for I2CMasterDriver<'a, I> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: buffer for command
    fn allow_readwrite(
        &self,
        appid: AppId,
        allow_num: usize,
        mut slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        let res = match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    core::mem::swap(&mut app.slice, &mut slice);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = match subscribe_num {
            1 /* write_read_done */ => {
                self.apps.enter(app_id, |app, _| {
                    core::mem::swap(&mut app.callback, &mut callback);
                }).map_err(ErrorCode::from)
            },
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(callback),
            Err(e) => Err((callback, e)),
        }
    }

    /// Initiate transfers
    fn command(&self, cmd_num: usize, arg1: usize, arg2: usize, appid: AppId) -> CommandReturn {
        if let Some(cmd) = Cmd::from_usize(cmd_num) {
            match cmd {
                Cmd::Ping => CommandReturn::success(),
                Cmd::Write => self
                    .apps
                    .enter(appid, |app, _| {
                        let addr = arg1 as u8;
                        let write_len = arg2;
                        self.operation(appid, app, Cmd::Write, addr, write_len as u8, 0);
                        CommandReturn::success()
                    })
                    .unwrap_or_else(|err| err.into()),
                Cmd::Read => self
                    .apps
                    .enter(appid, |app, _| {
                        let addr = arg1 as u8;
                        let read_len = arg2;
                        self.operation(appid, app, Cmd::Read, addr, 0, read_len as u8);
                        CommandReturn::success()
                    })
                    .unwrap_or_else(|err| err.into()),
                Cmd::WriteRead => {
                    let addr = arg1 as u8;
                    let write_len = arg1 >> 8; // can extend to 24 bit write length
                    let read_len = arg2; // can extend to 32 bit read length
                    self.apps
                        .enter(appid, |app, _| {
                            self.operation(
                                appid,
                                app,
                                Cmd::WriteRead,
                                addr,
                                write_len as u8,
                                read_len as u8,
                            );
                            CommandReturn::success()
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }
        } else {
            CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }
}

impl<'a, I: 'a + i2c::I2CMaster> i2c::I2CHwMasterClient for I2CMasterDriver<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        self.tx.take().map(|tx| {
            self.apps.enter(tx.app_id, |app, _| {
                if let Some(read_len) = tx.read_len.take() {
                    app.slice.mut_map_or((), |app_buffer| {
                        app_buffer[..read_len].copy_from_slice(&buffer[..read_len]);
                    });
                }

                // signal to driver that tx complete
                app.callback.schedule(0, 0, 0);
            })
        });

        //recover buffer
        self.buf.put(Some(buffer));
    }
}
