//! Provides userspace applications with the ability to communicate over the SPI
//! bus.

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{MapCell, TakeCell};
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice, SpiSlaveClient, SpiSlaveDevice};
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Spi as usize;

// SPI operations are handled by coping into a kernel buffer for
// writes and copying out of a kernel buffer for reads.
//
// If the application buffer is larger than the kernel buffer,
// the driver issues multiple HAL operations. The len field
// of an application keeps track of the length of the desired
// operation, while the index variable keeps track of the
// index an ongoing operation is at in the buffers.

#[derive(Default)]
struct App<'ker> {
    callback: Option<Callback<'ker>>,
    app_read: Option<AppSlice<'ker, Shared, u8>>,
    app_write: Option<AppSlice<'ker, Shared, u8>>,
    len: usize,
    index: usize,
}

// Since we provide an additional callback in slave mode for
// when the chip is selected, we have added a "SlaveApp" struct
// that includes this new callback field.
#[derive(Default)]
struct SlaveApp<'ker> {
    callback: Option<Callback<'ker>>,
    selected_callback: Option<Callback<'ker>>,
    app_read: Option<AppSlice<'ker, Shared, u8>>,
    app_write: Option<AppSlice<'ker, Shared, u8>>,
    len: usize,
    index: usize,
}

pub struct Spi<'a, 'ker, S: SpiMasterDevice> {
    spi_master: &'a S,
    busy: Cell<bool>,
    app: MapCell<App<'ker>>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
}

pub struct SpiSlave<'a, 'ker, S: SpiSlaveDevice> {
    spi_slave: &'a S,
    busy: Cell<bool>,
    app: MapCell<SlaveApp<'ker>>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
}

impl<S: SpiMasterDevice> Spi<'a, 'ker, S> {
    pub fn new(spi_master: &'a S) -> Spi<'a, 'ker, S> {
        Spi {
            spi_master: spi_master,
            busy: Cell::new(false),
            app: MapCell::new(App::default()),
            kernel_len: Cell::new(0),
            kernel_read: TakeCell::empty(),
            kernel_write: TakeCell::empty(),
        }
    }

    pub fn config_buffers(&mut self, read: &'static mut [u8], write: &'static mut [u8]) {
        let len = cmp::min(read.len(), write.len());
        self.kernel_len.set(len);
        self.kernel_read.replace(read);
        self.kernel_write.replace(write);
    }

    // Assumes checks for busy/etc. already done
    // Updates app.index to be index + length of op
    fn do_next_read_write(&self, app: &mut App) {
        let start = app.index;
        let len = cmp::min(app.len - start, self.kernel_len.get());
        let end = start + len;
        app.index = end;

        self.kernel_write.map(|kwbuf| {
            app.app_write.as_mut().map(|src| {
                for (i, c) in src.as_ref()[start..end].iter().enumerate() {
                    kwbuf[i] = *c;
                }
            });
        });
        self.spi_master.read_write_bytes(
            self.kernel_write.take().unwrap(),
            self.kernel_read.take(),
            len,
        );
    }
}

impl<S: SpiMasterDevice> Driver<'ker> for Spi<'a, 'ker, S> {
    fn allow(
        &self,
        _appid: AppId<'ker>,
        allow_num: usize,
        slice: Option<AppSlice<'ker, Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Pass in a read buffer to receive bytes into.
            0 => {
                self.app.map(|app| {
                    app.app_read = slice;
                });
                ReturnCode::SUCCESS
            }
            // Pass in a write buffer to transmit bytes from.
            1 => {
                self.app.map(|app| {
                    app.app_write = slice;
                });
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback<'ker>>,
        _app_id: AppId<'ker>,
    ) -> ReturnCode {
        match subscribe_num {
            0 /* read_write */ => {
                self.app.map(|app| {
                    app.callback = callback;
                });
                ReturnCode::SUCCESS
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    // 2: read/write buffers
    //   - requires write buffer registered with allow
    //   - read buffer optional
    // 3: set chip select
    //   - selects which peripheral (CS line) the SPI should
    //     activate
    //   - valid values are 0-3 for SAM4L
    //   - invalid value will result in CS 0
    // 4: get chip select
    //   - returns current selected peripheral
    // 5: set rate on current peripheral
    //   - parameter in bps
    // 6: get rate on current peripheral
    //   - value in bps
    // 7: set clock phase on current peripheral
    //   - 0 is sample leading
    //   - non-zero is sample trailing
    // 8: get clock phase on current peripheral
    //   - 0 is sample leading
    //   - non-zero is sample trailing
    // 9: set clock polarity on current peripheral
    //   - 0 is idle low
    //   - non-zero is idle high
    // 10: get clock polarity on current peripheral
    //   - 0 is idle low
    //   - non-zero is idle high
    //
    // x: lock spi
    //   - if you perform an operation without the lock,
    //     it implicitly acquires the lock before the
    //     operation and releases it after
    //   - while an app holds the lock no other app can issue
    //     operations on SPI (they are buffered)
    // x+1: unlock spi
    //   - does nothing if lock not held
    //
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, _: AppId<'ker>) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // No longer supported, wrap inside a read_write_bytes
            1 /* read_write_byte */ => ReturnCode::ENOSUPPORT,
            2 /* read_write_bytes */ => {
                if self.busy.get() {
                    return ReturnCode::EBUSY;
                }
                self.app.map_or(ReturnCode::FAIL, |app| {
                    let mut mlen = 0;
                    app.app_write.as_mut().map(|w| {
                        mlen = w.len();
                    });
                    app.app_read.as_mut().map(|r| {
                        mlen = cmp::min(mlen, r.len());
                    });
                    if mlen >= arg1 {
                        app.len = arg1;
                        app.index = 0;
                        self.busy.set(true);
                        self.do_next_read_write(app);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL /* write buffer too small */
                    }
                })
            }
            3 /* set chip select */ => {
                // XXX: TODO: do nothing, for now, until we fix interface
                // so virtual instances can use multiple chip selects
                ReturnCode::ENOSUPPORT
            }
            4 /* get chip select */ => {
                // XXX: We don't really know what chip select is being used
                // since we can't set it. Return error until set chip select
                // works.
                ReturnCode::ENOSUPPORT
            }
            5 /* set baud rate */ => {
                self.spi_master.set_rate(arg1 as u32);
                ReturnCode::SUCCESS
            }
            6 /* get baud rate */ => {
                ReturnCode::SuccessWithValue { value: self.spi_master.get_rate() as usize }
            }
            7 /* set phase */ => {
                match arg1 {
                    0 => self.spi_master.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_master.set_phase(ClockPhase::SampleTrailing),
                };
                ReturnCode::SUCCESS
            }
            8 /* get phase */ => {
                ReturnCode::SuccessWithValue { value: self.spi_master.get_phase() as usize }
            }
            9 /* set polarity */ => {
                match arg1 {
                    0 => self.spi_master.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_master.set_polarity(ClockPolarity::IdleHigh),
                };
                ReturnCode::SUCCESS
            }
            10 /* get polarity */ => {
                ReturnCode::SuccessWithValue { value: self.spi_master.get_polarity() as usize }
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl<S: SpiMasterDevice> SpiMasterClient for Spi<'a, 'ker, S> {
    fn read_write_done(
        &self,
        writebuf: &'static mut [u8],
        readbuf: Option<&'static mut [u8]>,
        length: usize,
    ) {
        self.app.map(move |app| {
            if app.app_read.is_some() {
                let src = readbuf.as_ref().unwrap();
                let dest = app.app_read.as_mut().unwrap();
                let start = app.index - length;
                let end = start + length;

                let d = &mut dest.as_mut()[start..end];
                for (i, c) in src[0..length].iter().enumerate() {
                    d[i] = *c;
                }
            }

            self.kernel_read.put(readbuf);
            self.kernel_write.replace(writebuf);

            if app.index == app.len {
                self.busy.set(false);
                app.len = 0;
                app.index = 0;
                app.callback.take().map(|mut cb| {
                    cb.schedule(app.len, 0, 0);
                });
            } else {
                self.do_next_read_write(app);
            }
        });
    }
}

impl<S: SpiSlaveDevice> SpiSlave<'a, 'ker, S> {
    pub fn new(spi_slave: &'a S) -> SpiSlave<'a, 'ker, S> {
        SpiSlave {
            spi_slave: spi_slave,
            busy: Cell::new(false),
            app: MapCell::new(SlaveApp::default()),
            kernel_len: Cell::new(0),
            kernel_read: TakeCell::empty(),
            kernel_write: TakeCell::empty(),
        }
    }

    pub fn config_buffers(&mut self, read: &'static mut [u8], write: &'static mut [u8]) {
        let len = cmp::min(read.len(), write.len());
        self.kernel_len.set(len);
        self.kernel_read.replace(read);
        self.kernel_write.replace(write);
    }

    // Assumes checks for busy/etc. already done
    // Updates app.index to be index + length of op
    fn do_next_read_write(&self, app: &mut SlaveApp) {
        let start = app.index;
        let len = cmp::min(app.len - start, self.kernel_len.get());
        let end = start + len;
        app.index = end;

        self.kernel_write.map(|kwbuf| {
            app.app_write.as_mut().map(|src| {
                for (i, c) in src.as_ref()[start..end].iter().enumerate() {
                    kwbuf[i] = *c;
                }
            });
        });
        self.spi_slave
            .read_write_bytes(self.kernel_write.take(), self.kernel_read.take(), len);
    }
}

impl<S: SpiSlaveDevice> Driver<'ker> for SpiSlave<'a, 'ker, S> {
    /// Provide read/write buffers to SpiSlave
    ///
    /// - allow_num 0: Provides an app_read buffer to receive transfers into.
    ///
    /// - allow_num 1: Provides an app_write buffer to send transfers from.
    ///
    fn allow(
        &self,
        _appid: AppId<'ker>,
        allow_num: usize,
        slice: Option<AppSlice<'ker, Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            0 => {
                self.app.map(|app| app.app_read = slice);
                ReturnCode::SUCCESS
            }
            1 => {
                self.app.map(|app| app.app_write = slice);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks for SpiSlave
    ///
    /// - subscribe_num 0: Sets up a callback for when read_write completes. This
    ///                  is called after completing a transfer/reception with
    ///                  the Spi master. Note that this occurs after the pending
    ///                  DMA transfer initiated by read_write_bytes completes.
    ///
    /// - subscribe_num 1: Sets up a callback for when the chip select line is
    ///                  driven low, meaning that the slave was selected by
    ///                  the Spi master. This occurs immediately before
    ///                  a data transfer.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback<'ker>>,
        _app_id: AppId<'ker>,
    ) -> ReturnCode {
        match subscribe_num {
            0 /* read_write */ => {
                self.app.map(|app| app.callback = callback);
                ReturnCode::SUCCESS
            },
            1 /* chip selected */ => {
                self.app.map(|app| app.selected_callback = callback);
                ReturnCode::SUCCESS
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// - 0: check if present
    /// - 1: read/write buffers
    ///   - read and write buffers optional
    ///   - fails if arg1 (bytes to write) >
    ///     write_buffer.len()
    /// - 2: get chip select
    ///   - returns current selected peripheral
    ///   - in slave mode, always returns 0
    /// - 3: set clock phase on current peripheral
    ///   - 0 is sample leading
    ///   - non-zero is sample trailing
    /// - 4: get clock phase on current peripheral
    ///   - 0 is sample leading
    ///   - non-zero is sample trailing
    /// - 5: set clock polarity on current peripheral
    ///   - 0 is idle low
    ///   - non-zero is idle high
    /// - 6: get clock polarity on current peripheral
    ///   - 0 is idle low
    ///   - non-zero is idle high
    /// - x: lock spi
    ///   - if you perform an operation without the lock,
    ///     it implicitly acquires the lock before the
    ///     operation and releases it after
    ///   - while an app holds the lock no other app can issue
    ///     operations on SPI (they are buffered)
    ///   - not implemented or currently supported
    /// - x+1: unlock spi
    ///   - does nothing if lock not held
    ///   - not implemented or currently supported
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, _: AppId<'ker>) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* read_write_bytes */ => {
                if self.busy.get() {
                    return ReturnCode::EBUSY;
                }
                self.app.map_or(ReturnCode::FAIL /* XXX app is null? */, |app| {
                    let mut mlen = 0;
                    app.app_write.as_mut().map(|w| {
                        mlen = w.len();
                    });
                    app.app_read.as_mut().map(|r| {
                        mlen = cmp::min(mlen, r.len());
                    });
                    if mlen >= arg1 {
                        app.len = arg1;
                        app.index = 0;
                        self.busy.set(true);
                        self.do_next_read_write(app);
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL /* write buffer too small */
                    }
                })
            }
            2 /* get chip select */ => {
                // When in slave mode, the only possible chip select is 0
                ReturnCode::SuccessWithValue { value: 0 }
            }
            3 /* set phase */ => {
                match arg1 {
                    0 => self.spi_slave.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_slave.set_phase(ClockPhase::SampleTrailing),
                };
                ReturnCode::SUCCESS
            }
            4 /* get phase */ => {
                ReturnCode::SuccessWithValue { value: self.spi_slave.get_phase() as usize }
            }
            5 /* set polarity */ => {
                match arg1 {
                    0 => self.spi_slave.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_slave.set_polarity(ClockPolarity::IdleHigh),
                };
                ReturnCode::SUCCESS
            }
            6 /* get polarity */ => {
                ReturnCode::SuccessWithValue { value: self.spi_slave.get_polarity() as usize }
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl<S: SpiSlaveDevice> SpiSlaveClient for SpiSlave<'a, 'ker, S> {
    fn read_write_done(
        &self,
        writebuf: Option<&'static mut [u8]>,
        readbuf: Option<&'static mut [u8]>,
        length: usize,
    ) {
        self.app.map(move |app| {
            if app.app_read.is_some() {
                let src = readbuf.as_ref().unwrap();
                let dest = app.app_read.as_mut().unwrap();
                let start = app.index - length;
                let end = start + length;

                let d = &mut dest.as_mut()[start..end];
                for (i, c) in src[0..length].iter().enumerate() {
                    d[i] = *c;
                }
            }

            self.kernel_read.put(readbuf);
            self.kernel_write.put(writebuf);

            if app.index == app.len {
                self.busy.set(false);
                app.len = 0;
                app.index = 0;
                app.callback.take().map(|mut cb| {
                    cb.schedule(app.len, 0, 0);
                });
            } else {
                self.do_next_read_write(app);
            }
        });
    }

    // Simple callback for when chip has been selected
    fn chip_selected(&self) {
        self.app.map(move |app| {
            app.selected_callback.take().map(|mut cb| {
                cb.schedule(app.len, 0, 0);
            });
        });
    }
}
