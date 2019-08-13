use enum_primitive::enum_from_primitive;
use kernel::common::cells::{MapCell, OptionalCell, TakeCell};
use kernel::hil::i2c;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::I2cMaster as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    slice: Option<AppSlice<Shared, u8>>,
}

pub static mut BUF: [u8; 64] = [0; 64];

struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    app_id: AppId,
    /// The total amount to transmit
    read_len: OptionalCell<usize>,
}

pub struct I2CMasterDriver<I: 'static + i2c::I2CMaster> {
    i2c: &'static I,
    buf: TakeCell<'static, [u8]>,
    tx: MapCell<Transaction>,
    apps: Grant<App>,
}

impl<I: 'static + i2c::I2CMaster> I2CMasterDriver<I> {
    pub fn new(i2c: &'static I, buf: &'static mut [u8], apps: Grant<App>) -> I2CMasterDriver<I> {
        I2CMasterDriver {
            i2c,
            buf: TakeCell::new(buf),
            tx: MapCell::empty(),
            apps,
        }
    }

    fn operation(
        &self,
        app_id: AppId,
        app: &mut App,
        command: Cmd,
        addr: u8,
        wlen: u8,
        rlen: u8,
    ) -> ReturnCode {
        self.apps
            .enter(app_id, |_, _| {
                if let Some(app_buffer) = app.slice.take() {
                    self.buf.take().map(|buffer| {
                        for n in 0..wlen as usize {
                            buffer[n] = app_buffer.as_ref()[n];
                        }

                        let read_len: OptionalCell<usize>;
                        if rlen == 0 {
                            read_len = OptionalCell::empty();
                        } else {
                            read_len = OptionalCell::new(rlen as usize);
                        }
                        self.tx.put(Transaction { app_id, read_len });
                        app.slice = Some(app_buffer);

                        match command {
                            Cmd::Ping => return ReturnCode::EINVAL,
                            Cmd::Write => self.i2c.write(addr, buffer, wlen),
                            Cmd::Read => self.i2c.read(addr, buffer, rlen),
                            Cmd::WriteRead => self.i2c.write_read(addr, buffer, wlen, rlen),
                        }
                        ReturnCode::SUCCESS
                    });
                    // buffer has not been returned by I2C
                    // i2c_master.rs should not allow us to get here
                    return ReturnCode::ENOMEM;
                } else {
                    // AppDriver is attempting operation
                    // but has not granted memory
                    return ReturnCode::EINVAL;
                }
            })
            .expect("Appid does not map to app");
        ReturnCode::ENOSUPPORT
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

impl<I: i2c::I2CMaster> Driver for I2CMasterDriver<I> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: buffer for command
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    app.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
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
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            1 /* write_read_done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Initiate transfers
    fn command(&self, cmd_num: usize, arg1: usize, arg2: usize, appid: AppId) -> ReturnCode {
        if let Some(cmd) = Cmd::from_usize(cmd_num) {
            match cmd {
                Cmd::Ping => ReturnCode::SUCCESS,
                Cmd::Write => self
                    .apps
                    .enter(appid, |app, _| {
                        let addr = arg1 as u8;
                        let write_len = arg2;
                        self.operation(appid, app, Cmd::Write, addr, write_len as u8, 0);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into()),
                Cmd::Read => self
                    .apps
                    .enter(appid, |app, _| {
                        let addr = arg1 as u8;
                        let read_len = arg2;
                        self.operation(appid, app, Cmd::Read, addr, 0, read_len as u8);
                        ReturnCode::SUCCESS
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
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }
        } else {
            ReturnCode::ENOSUPPORT
        }
    }
}

impl<I: i2c::I2CMaster> i2c::I2CHwMasterClient for I2CMasterDriver<I> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        self.tx.take().map(|tx| {
            self.apps.enter(tx.app_id, |app, _| {
                if let Some(read_len) = tx.read_len.take() {
                    if let Some(mut app_buffer) = app.slice.take() {
                        for n in 0..read_len {
                            app_buffer.as_mut()[n] = buffer[n];
                        }
                    } else {
                        // app has requested read but we have no buffer
                        // should not arrive here
                    }
                }

                // signal to driver that tx complete
                app.callback.map(|mut cb| {
                    cb.schedule(0, 0, 0);
                });
            })
        });

        //recover buffer
        self.buf.put(Some(buffer));
    }
}
