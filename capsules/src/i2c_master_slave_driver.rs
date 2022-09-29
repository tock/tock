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

use kernel::hil;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};

pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [0; 256];
pub static mut BUFFER3: [u8; 256] = [0; 256];

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::I2cMasterSlave as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const MASTER_TX: usize = 0;
    pub const SLAVE_TX: usize = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 3;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const MASTER_RX: usize = 1;
    pub const SLAVE_RX: usize = 3;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 4;
}

#[derive(Default)]
pub struct App;

#[derive(Clone, Copy, PartialEq)]
enum MasterAction {
    Read(u8),
    Write,
    WriteRead(u8),
}

pub struct I2CMasterSlaveDriver<'a> {
    i2c: &'a dyn hil::i2c::I2CMasterSlave,
    listening: Cell<bool>,
    master_action: Cell<MasterAction>, // Whether we issued a write or read as master
    master_buffer: TakeCell<'static, [u8]>,
    slave_buffer1: TakeCell<'static, [u8]>,
    slave_buffer2: TakeCell<'static, [u8]>,
    app: OptionalCell<ProcessId>,
    apps: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl<'a> I2CMasterSlaveDriver<'a> {
    pub fn new(
        i2c: &'a dyn hil::i2c::I2CMasterSlave,
        master_buffer: &'static mut [u8],
        slave_buffer1: &'static mut [u8],
        slave_buffer2: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<1>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> I2CMasterSlaveDriver<'a> {
        I2CMasterSlaveDriver {
            i2c,
            listening: Cell::new(false),
            master_action: Cell::new(MasterAction::Write),
            master_buffer: TakeCell::new(master_buffer),
            slave_buffer1: TakeCell::new(slave_buffer1),
            slave_buffer2: TakeCell::new(slave_buffer2),
            app: OptionalCell::empty(),
            apps: grant,
        }
    }
}

impl hil::i2c::I2CHwMasterClient for I2CMasterSlaveDriver<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), hil::i2c::Error>) {
        // Map I2C error to a number we can pass back to the application
        let status = kernel::errorcode::into_statuscode(match status {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        });

        // Signal the application layer. Need to copy read in bytes if this
        // was a read call.
        match self.master_action.get() {
            MasterAction::Write => {
                self.master_buffer.replace(buffer);

                self.app.map(|app| {
                    let _ = self.apps.enter(*app, |_, kernel_data| {
                        kernel_data.schedule_upcall(0, (0, status, 0)).ok();
                    });
                });
            }

            MasterAction::Read(read_len) => {
                self.app.map(|app| {
                    let _ = self.apps.enter(*app, |_, kernel_data| {
                        // Because this (somewhat incorrectly) doesn't report
                        // back how many bytes were read, the result of mut_enter
                        // is ignored. Note that this requires userspace to keep
                        // track of this information, and if read_len is longer
                        // than the buffer could lead to array overrun errors in
                        // userspace. The I2C syscall API should pass back lengths.
                        // -pal 3/5/21
                        kernel_data
                            .get_readwrite_processbuffer(rw_allow::MASTER_RX)
                            .and_then(|master_rx| {
                                master_rx.mut_enter(move |app_buffer| {
                                    let len = cmp::min(app_buffer.len(), read_len as usize);

                                    for (i, c) in buffer[0..len].iter().enumerate() {
                                        app_buffer[i].set(*c);
                                    }

                                    self.master_buffer.replace(buffer);
                                    0
                                })
                            })
                            .unwrap_or(0);
                        kernel_data.schedule_upcall(0, (1, status, 0)).ok();
                    });
                });
            }

            MasterAction::WriteRead(read_len) => {
                self.app.map(|app| {
                    let _ = self.apps.enter(*app, |_, kernel_data| {
                        // Because this (somewhat incorrectly) doesn't report
                        // back how many bytes were read, the result of mut_enter
                        // is ignored. Note that this requires userspace to keep
                        // track of this information, and if read_len is longer
                        // than the buffer could lead to array overrun errors in
                        // userspace. The I2C syscall API should pass back lengths.
                        // -pal 3/5/21
                        kernel_data
                            .get_readwrite_processbuffer(rw_allow::MASTER_RX)
                            .and_then(|master_rx| {
                                master_rx.mut_enter(move |app_buffer| {
                                    let len = cmp::min(app_buffer.len(), read_len as usize);
                                    app_buffer[..len].copy_from_slice(&buffer[..len]);
                                    self.master_buffer.replace(buffer);
                                    0
                                })
                            })
                            .unwrap_or(0);
                        kernel_data.schedule_upcall(0, (7, status, 0)).ok();
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

impl hil::i2c::I2CHwSlaveClient for I2CMasterSlaveDriver<'_> {
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
                    let _ = self.apps.enter(*app, |_, kernel_data| {
                        kernel_data
                            .get_readwrite_processbuffer(rw_allow::SLAVE_RX)
                            .and_then(|slave_rx| {
                                slave_rx.mut_enter(move |app_rx| {
                                    // Check bounds for write length
                                    // Because this (somewhat incorrectly) doesn't report
                                    // back how many bytes were read, the result of mut_map_or
                                    // is ignored. Note that this requires userspace to keep
                                    // track of this information, and if read_len is longer
                                    // than the buffer could lead to array overrun errors in
                                    // userspace. The I2C syscall API should pass back lengths.
                                    // -pal 3/5/21
                                    let buf_len = cmp::min(app_rx.len(), buffer.len());
                                    let read_len = cmp::min(buf_len, length as usize);

                                    for (i, c) in buffer[0..read_len].iter_mut().enumerate() {
                                        app_rx[i].set(*c);
                                    }

                                    self.slave_buffer1.replace(buffer);
                                    0
                                })
                            })
                            .unwrap_or(0);

                        kernel_data.schedule_upcall(0, (3, length as usize, 0)).ok();
                    });
                });
            }

            hil::i2c::SlaveTransmissionType::Read => {
                self.slave_buffer2.replace(buffer);

                // Notify the app that the read finished
                self.app.map(|app| {
                    let _ = self.apps.enter(*app, |_, kernel_data| {
                        kernel_data.schedule_upcall(0, (4, length as usize, 0)).ok();
                    });
                });
            }
        }
    }

    fn read_expected(&self) {
        // Pass this up to the client. Not much we can do until the application
        // has setup a buffer to read from.
        self.app.map(|app| {
            let _ = self.apps.enter(*app, |_, kernel_data| {
                // Ask the app to setup a read buffer. The app must call
                // command 3 after it has setup the shared read buffer with
                // the correct bytes.
                kernel_data.schedule_upcall(0, (2, 0, 0)).ok();
            });
        });
    }

    fn write_expected(&self) {
        // Don't expect this to occur. We will typically have a buffer waiting
        // to receive bytes because this module has a buffer and may as well
        // just let the hardware layer have it. But, if it does happen
        // we can respond.
        self.slave_buffer1.take().map(|buffer| {
            // TODO verify errors
            let _ = hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
        });
    }
}

impl SyscallDriver for I2CMasterSlaveDriver<'_> {
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally
            return CommandReturn::success();
        }
        // Check if this non-virtualized driver is already in use by
        // some (alive) process
        let match_or_empty_or_nonexistant = self.app.map_or(true, |current_process| {
            self.apps
                .enter(*current_process, |_, _| current_process == &process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.app.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }
        let app = process_id;

        match command_num {
            // Do a write to another I2C device
            1 => {
                let address = (data & 0xFFFF) as u8;
                let len = (data >> 16) & 0xFFFF;

                // No need to check error on enter() -- we entered successfully
                // above, so grant is allocated, and the app can't disappear
                // while we are in the kernel.
                let _ = self.apps.enter(app, |_, kernel_data| {
                    kernel_data
                        .get_readonly_processbuffer(ro_allow::MASTER_TX)
                        .and_then(|master_tx| {
                            master_tx.enter(|app_tx| {
                                // Because this (somewhat incorrectly) doesn't report
                                // back how many bytes are being written, the result of mut_map_or
                                // is ignored. Note that this does not provide useful feedback
                                // to user space if a write is longer than the buffer.
                                // The I2C syscall API should pass back lengths.
                                // -pal 3/5/21
                                self.master_buffer.take().map(|kernel_tx| {
                                    // Check bounds for write length
                                    let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                                    let write_len = cmp::min(buf_len, len);

                                    for (i, c) in kernel_tx[0..write_len].iter_mut().enumerate() {
                                        *c = app_tx[i].get();
                                    }

                                    self.master_action.set(MasterAction::Write);

                                    hil::i2c::I2CMaster::enable(self.i2c);
                                    // TODO verify errors
                                    let _ = hil::i2c::I2CMaster::write(
                                        self.i2c,
                                        address,
                                        kernel_tx,
                                        write_len as u8,
                                    );
                                });
                                0
                            })
                        })
                        .unwrap_or(0);
                });

                CommandReturn::success()
            }

            // Do a read to another I2C device
            2 => {
                let address = (data & 0xFFFF) as u8;
                let len = (data >> 16) & 0xFFFF;

                let _ = self.apps.enter(app, |_, kernel_data| {
                    // Because this (somewhat incorrectly) doesn't report
                    // back how many bytes are being read, the result of mut_map_or
                    // is ignored. Note that this does not provide useful feedback
                    // to user space if a write is longer than the buffer.
                    // The I2C syscall API should pass back lengths.
                    // -pal 3/5/21
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::MASTER_RX)
                        .and_then(|master_rx| {
                            master_rx.enter(|app_rx| {
                                self.master_buffer.take().map(|kernel_tx| {
                                    // Check bounds for write length
                                    let buf_len = cmp::min(app_rx.len(), kernel_tx.len());
                                    let read_len = cmp::min(buf_len, len);

                                    for (i, c) in kernel_tx[0..read_len].iter_mut().enumerate() {
                                        *c = app_rx[i].get();
                                    }

                                    self.master_action.set(MasterAction::Read(read_len as u8));

                                    hil::i2c::I2CMaster::enable(self.i2c);
                                    // TODO verify errors
                                    let _ = hil::i2c::I2CMaster::read(
                                        self.i2c,
                                        address,
                                        kernel_tx,
                                        read_len as u8,
                                    );
                                });
                                0
                            })
                        })
                        .unwrap_or(0);
                });

                CommandReturn::success()
            }

            // Listen for messages to this device as a slave.
            3 => {
                // We can always handle a write since this module has a buffer.
                // .map will handle if we have already done this.
                self.slave_buffer1.take().map(|buffer| {
                    // TODO verify errors
                    let _ = hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
                });

                // Actually get things going
                hil::i2c::I2CSlave::enable(self.i2c);
                hil::i2c::I2CSlave::listen(self.i2c);

                // Note that we have enabled listening, so that if we switch
                // to Master mode to send a message we can go back to listening.
                self.listening.set(true);
                CommandReturn::success()
            }

            // Prepare for a read from another Master by passing what's
            // in the shared slice to the lower level I2C hardware driver.
            4 => {
                let _ = self.apps.enter(app, |_, kernel_data| {
                    // Because this (somewhat incorrectly) doesn't report
                    // back how many bytes are being read, the result of mut_map_or
                    // is ignored. Note that this does not provide useful feedback
                    // to user space if a write is longer than the buffer.
                    // The I2C syscall API should pass back lengths.
                    // -pal 3/5/21
                    kernel_data
                        .get_readonly_processbuffer(ro_allow::SLAVE_TX)
                        .and_then(|slave_tx| {
                            slave_tx.enter(|app_tx| {
                                self.slave_buffer2.take().map(|kernel_tx| {
                                    // Check bounds for write length
                                    let len = data;
                                    let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                                    let read_len = cmp::min(buf_len, len);

                                    for (i, c) in kernel_tx[0..read_len].iter_mut().enumerate() {
                                        *c = app_tx[i].get();
                                    }

                                    // TODO verify errors
                                    let _ = hil::i2c::I2CSlave::read_send(
                                        self.i2c,
                                        kernel_tx,
                                        read_len as u8,
                                    );
                                });
                                0
                            })
                        })
                        .unwrap_or(0);
                });

                CommandReturn::success()
            }

            // Stop listening for messages as an I2C slave
            5 => {
                hil::i2c::I2CSlave::disable(self.i2c);

                // We are no longer listening for I2C messages from a different
                // master device.
                self.listening.set(false);
                CommandReturn::success()
            }

            // Setup this device's slave address.
            6 => {
                let address = data as u8;
                // We do not count the R/W bit as part of the address, so the
                // valid range is 0x00-0x7f
                if address > 0x7f {
                    return CommandReturn::failure(ErrorCode::INVAL);
                }
                // TODO verify errors
                let _ = hil::i2c::I2CSlave::set_address(self.i2c, address);
                CommandReturn::success()
            }

            // Perform write-to then read-from a slave device.
            // Uses tx buffer for both read and write.
            7 => {
                let address = (data & 0xFF) as u8;
                let read_len = (data >> 8) & 0xFF;
                let write_len = (data >> 16) & 0xFF;
                let _ = self.apps.enter(app, |_, kernel_data| {
                    // Because this (somewhat incorrectly) doesn't report
                    // back how many bytes are being read/read, the result of mut_map_or
                    // is ignored. Note that this does not provide useful feedback
                    // to user space if a write is longer than the buffer.
                    // The I2C syscall API should pass back lengths.
                    // -pal 3/5/21
                    let _ = kernel_data
                        .get_readonly_processbuffer(ro_allow::MASTER_TX)
                        .and_then(|master_tx| {
                            master_tx.enter(|app_tx| {
                                self.master_buffer.take().map(|kernel_tx| {
                                    // Check bounds for write length
                                    let buf_len = cmp::min(app_tx.len(), kernel_tx.len());
                                    let write_len = cmp::min(buf_len, write_len);
                                    let read_len = cmp::min(buf_len, read_len);
                                    app_tx[..write_len].copy_to_slice(&mut kernel_tx[..write_len]);
                                    self.master_action
                                        .set(MasterAction::WriteRead(read_len as u8));
                                    hil::i2c::I2CMaster::enable(self.i2c);
                                    // TODO verify errors
                                    let _ = hil::i2c::I2CMaster::write_read(
                                        self.i2c,
                                        address,
                                        kernel_tx,
                                        write_len as u8,
                                        read_len as u8,
                                    );
                                });
                            })
                        });
                });
                CommandReturn::success()
            }

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
