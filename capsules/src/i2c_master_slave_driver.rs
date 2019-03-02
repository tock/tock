//! Provides both an I2C Master and I2C Slave interface to userspace.
//!
//! By calling `listen` this module will wait for I2C messages
//! send to it by other masters on the I2C bus. If this device wants to
//! transmit as an I2C master, this module will put the I2C hardware in master
//! mode, transmit the read/write, then go back to listening (if listening
//! was enabled).
//!
//! This capsule must sit directly above the I2C HIL interface (and not
//! on top of the mux) because there is no way to mux the slave (it can't
//! listen on more than one address) and because the application may want
//! to be able to talk to any I2C address.

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{MapCell, TakeCell};
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Driver, Shared};

pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [0; 256];
pub static mut BUFFER3: [u8; 256] = [0; 256];

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::I2C_MASTER_SLAVE as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    master_tx_buffer: Option<AppSlice<Shared, u8>>,
    master_rx_buffer: Option<AppSlice<Shared, u8>>,
    slave_tx_buffer: Option<AppSlice<Shared, u8>>,
    slave_rx_buffer: Option<AppSlice<Shared, u8>>,
}

#[derive(Clone, Copy, PartialEq)]
enum MasterAction {
    Read(u8),
    Write,
    WriteRead(u8),
}

pub struct I2CMasterSlaveDriver<'a> {
    i2c: &'a hil::i2c::I2CMasterSlave,
    listening: Cell<bool>,
    master_action: Cell<MasterAction>, // Whether we issued a write or read as master
    master_buffer: TakeCell<'static, [u8]>,
    slave_buffer1: TakeCell<'static, [u8]>,
    slave_buffer2: TakeCell<'static, [u8]>,
    app: MapCell<App>,
}

impl I2CMasterSlaveDriver<'a> {
    pub fn new(
        i2c: &'a hil::i2c::I2CMasterSlave,
        master_buffer: &'static mut [u8],
        slave_buffer1: &'static mut [u8],
        slave_buffer2: &'static mut [u8],
    ) -> I2CMasterSlaveDriver<'a> {
        I2CMasterSlaveDriver {
            i2c: i2c,
            listening: Cell::new(false),
            master_action: Cell::new(MasterAction::Write),
            master_buffer: TakeCell::new(master_buffer),
            slave_buffer1: TakeCell::new(slave_buffer1),
            slave_buffer2: TakeCell::new(slave_buffer2),
            app: MapCell::new(App::default()),
        }
    }
}

impl hil::i2c::I2CHwMasterClient for I2CMasterSlaveDriver<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {
        // Map I2C error to a number we can pass back to the application
        let err: isize = match error {
            hil::i2c::Error::AddressNak => -1,
            hil::i2c::Error::DataNak => -2,
            hil::i2c::Error::ArbitrationLost => -3,
            hil::i2c::Error::Overrun => -4,
            hil::i2c::Error::CommandComplete => 0,
        };

        // Signal the application layer. Need to copy read in bytes if this
        // was a read call.
        match self.master_action.get() {
            MasterAction::Write => {
                self.master_buffer.replace(buffer);

                self.app.map(|app| {
                    app.callback.map(|mut cb| {
                        cb.schedule(0, err as usize, 0);
                    });
                });
            }

            MasterAction::Read(read_len) => {
                self.app.map(|app| {
                    app.master_rx_buffer.as_mut().map(move |app_buffer| {
                        let len = cmp::min(app_buffer.len(), read_len as usize);

                        let d = &mut app_buffer.as_mut()[0..(len as usize)];
                        for (i, c) in buffer[0..len].iter().enumerate() {
                            d[i] = *c;
                        }

                        self.master_buffer.replace(buffer);
                    });

                    app.callback.map(|mut cb| {
                        cb.schedule(1, err as usize, 0);
                    });
                });
            }

            MasterAction::WriteRead(read_len) => {
                self.app.map(|app| {
                    app.master_tx_buffer.as_mut().map(move |app_buffer| {
                        let len = cmp::min(app_buffer.len(), read_len as usize);
                        app_buffer.as_mut()[..len].copy_from_slice(&buffer[..len]);
                        self.master_buffer.replace(buffer);
                    });
                    app.callback.map(|mut cb| {
                        cb.schedule(7, err as usize, 0);
                    });
                });
            }
        }

        // Check to see if we were listening as an I2C slave and should re-enable
        // that mode.
        if self.listening.get() {
            hil::i2c::I2CSlave::enable(self.i2c);
            hil::i2c::I2CSlave::listen(self.i2c);
        }
    }
}

impl hil::i2c::I2CHwSlaveClient for I2CMasterSlaveDriver<'a> {
    fn command_complete(
        &self,
        buffer: &'static mut [u8],
        length: u8,
        transmission_type: hil::i2c::SlaveTransmissionType,
    ) {
        // Need to know if read or write
        //   - on write, copy bytes to app slice and do callback
        //     then pass buffer back to hw driver
        //   - on read, just signal upper layer and replace the read buffer
        //     in this driver

        match transmission_type {
            hil::i2c::SlaveTransmissionType::Write => {
                self.app.map(|app| {
                    app.slave_rx_buffer.as_mut().map(move |app_rx| {
                        // Check bounds for write length
                        let buf_len = cmp::min(app_rx.len(), buffer.len());
                        let read_len = cmp::min(buf_len, length as usize);

                        let d = &mut app_rx.as_mut()[0..read_len];
                        for (i, c) in buffer[0..read_len].iter_mut().enumerate() {
                            d[i] = *c;
                        }

                        self.slave_buffer1.replace(buffer);
                    });

                    app.callback.map(|mut cb| {
                        cb.schedule(3, length as usize, 0);
                    });
                });
            }

            hil::i2c::SlaveTransmissionType::Read => {
                self.slave_buffer2.replace(buffer);

                // Notify the app that the read finished
                self.app.map(|app| {
                    app.callback.map(|mut cb| {
                        cb.schedule(4, length as usize, 0);
                    });
                });
            }
        }
    }

    fn read_expected(&self) {
        // Pass this up to the client. Not much we can do until the application
        // has setup a buffer to read from.
        self.app.map(|app| {
            app.callback.map(|mut cb| {
                // Ask the app to setup a read buffer. The app must call
                // command 3 after it has setup the shared read buffer with
                // the correct bytes.
                cb.schedule(2, 0, 0);
            });
        });
    }

    fn write_expected(&self) {
        // Don't expect this to occur. We will typically have a buffer waiting
        // to receive bytes because this module has a buffer and may as well
        // just let the hardware layer have it. But, if it does happen
        // we can respond.
        self.slave_buffer1.take().map(|buffer| {
            hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
        });
    }
}

impl Driver for I2CMasterSlaveDriver<'a> {
    fn allow(
        &self,
        _appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Pass in a buffer for transmitting a `write` to another
            // I2C device.
            0 => {
                self.app.map(|app| {
                    app.master_tx_buffer = slice;
                });
                ReturnCode::SUCCESS
            }
            // Pass in a buffer for doing a read from another I2C device.
            1 => {
                self.app.map(|app| {
                    app.master_rx_buffer = slice;
                });
                ReturnCode::SUCCESS
            }
            // Pass in a buffer for handling a read issued by another I2C master.
            2 => {
                self.app.map(|app| {
                    app.slave_tx_buffer = slice;
                });
                ReturnCode::SUCCESS
            }
            // Pass in a buffer for handling a write issued by another I2C master.
            3 => {
                self.app.map(|app| {
                    app.slave_rx_buffer = slice;
                });
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.app.map(|app| {
                    app.callback = callback;
                });
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,

            // Do a write to another I2C device
            1 => {
                let address = (data & 0xFFFF) as u8;
                let len = (data >> 16) & 0xFFFF;

                self.app.map(|app| {
                    app.master_tx_buffer.as_mut().map(|app_tx| {
                        self.master_buffer.take().map(|kernel_tx| {
                            // Check bounds for write length
                            let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                            let write_len = cmp::min(buf_len, len);

                            let d = &mut app_tx.as_mut()[0..write_len];
                            for (i, c) in kernel_tx[0..write_len].iter_mut().enumerate() {
                                *c = d[i];
                            }

                            self.master_action.set(MasterAction::Write);

                            hil::i2c::I2CMaster::enable(self.i2c);
                            hil::i2c::I2CMaster::write(self.i2c,
                                                       address,
                                                       kernel_tx,
                                                       write_len as u8);
                        });
                    });
                });

                ReturnCode::SUCCESS
            }

            // Do a read to another I2C device
            2 => {
                let address = (data & 0xFFFF) as u8;
                let len = (data >> 16) & 0xFFFF;

                self.app.map(|app| {
                    app.master_rx_buffer.as_mut().map(|app_rx| {
                        self.master_buffer.take().map(|kernel_tx| {
                            // Check bounds for write length
                            let buf_len = cmp::min(app_rx.len(), kernel_tx.len());
                            let read_len = cmp::min(buf_len, len);

                            let d = &mut app_rx.as_mut()[0..read_len];
                            for (i, c) in kernel_tx[0..read_len].iter_mut().enumerate() {
                                *c = d[i];
                            }

                            self.master_action.set(MasterAction::Read(read_len as u8));

                            hil::i2c::I2CMaster::enable(self.i2c);
                            hil::i2c::I2CMaster::read(self.i2c, address, kernel_tx, read_len as u8);
                        });
                    });
                });

                ReturnCode::SUCCESS
            }

            // Listen for messages to this device as a slave.
            3 => {
                // We can always handle a write since this module has a buffer.
                // .map will handle if we have already done this.
                self.slave_buffer1.take().map(|buffer| {
                    hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
                });

                // Actually get things going
                hil::i2c::I2CSlave::enable(self.i2c);
                hil::i2c::I2CSlave::listen(self.i2c);

                // Note that we have enabled listening, so that if we switch
                // to Master mode to send a message we can go back to listening.
                self.listening.set(true);
                ReturnCode::SUCCESS
            }

            // Prepare for a read from another Master by passing what's
            // in the shared slice to the lower level I2C hardware driver.
            4 => {
                self.app.map(|app| {
                    app.slave_tx_buffer.as_mut().map(|app_tx| {
                        self.slave_buffer2.take().map(|kernel_tx| {
                            // Check bounds for write length
                            let len = data;
                            let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                            let read_len = cmp::min(buf_len, len);

                            let d = &mut app_tx.as_mut()[0..read_len];
                            for (i, c) in kernel_tx[0..read_len].iter_mut().enumerate() {
                                *c = d[i];
                            }

                            hil::i2c::I2CSlave::read_send(self.i2c, kernel_tx, read_len as u8);
                        });
                    });
                });

                ReturnCode::SUCCESS
            }

            // Stop listening for messages as an I2C slave
            5 => {
                hil::i2c::I2CSlave::disable(self.i2c);

                // We are no longer listening for I2C messages from a different
                // master device.
                self.listening.set(false);
                ReturnCode::SUCCESS
            }

            // Setup this device's slave address.
            6 => {
                let address = data as u8;
                // We do not count the R/W bit as part of the address, so the
                // valid range is 0x00-0x7f
                if address > 0x7f {
                    return ReturnCode::EINVAL;
                }
                hil::i2c::I2CSlave::set_address(self.i2c, address);
                ReturnCode::SUCCESS
            }

            // Perform write-to then read-from a slave device.
            // Uses tx buffer for both read and write.
            7 => {
                let address = (data & 0xFF) as u8;
                let read_len = (data >> 8) & 0xFF;
                let write_len = (data >> 16) & 0xFF;
                self.app.map(|app| {
                    app.master_tx_buffer.as_mut().map(|app_tx| {
                        self.master_buffer.take().map(|kernel_tx| {
                            // Check bounds for write length
                            let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                            let write_len = cmp::min(buf_len, write_len);
                            let read_len = cmp::min(buf_len, read_len);
                            kernel_tx[..write_len].copy_from_slice(&app_tx.as_ref()[..write_len]);
                            self.master_action.set(MasterAction::WriteRead(read_len as u8));
                            hil::i2c::I2CMaster::enable(self.i2c);
                            hil::i2c::I2CMaster::write_read(self.i2c,
                                                            address,
                                                            kernel_tx,
                                                            write_len as u8,
                                                            read_len as u8);
                        });
                    });
                });
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
