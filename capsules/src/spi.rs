//! The SPI capsule provides userspace applications with the ability
//! to communicate over the SPI bus.

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::spi::{SpiMasterDevice, SpiMasterClient};
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;

// SPI operations are handled by coping into a kernel buffer for
// writes and copying out of a kernel buffer for reads.
//
// If the application buffer is larger than the kernel buffer,
// the driver issues multiple HAL operations. The len field
// of an application keeps track of the length of the desired
// operation, while the index variable keeps track of the
// index an ongoing operation is at in the buffers.

struct App {
    callback: Option<Callback>,
    app_read: Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    len: usize,
    index: usize,
}

pub struct Spi<'a, S: SpiMasterDevice + 'a> {
    spi_master: &'a S,
    busy: Cell<bool>,
    app: MapCell<App>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
}

impl<'a, S: SpiMasterDevice> Spi<'a, S> {
    pub fn new(spi_master: &'a S) -> Spi<'a, S> {
        Spi {
            spi_master: spi_master,
            busy: Cell::new(false),
            app: MapCell::empty(),
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
            app.app_write
                .as_mut()
                .map(|src| for (i, c) in src.as_ref()[start..end].iter().enumerate() {
                    kwbuf[i] = *c;
                });
        });
        self.spi_master.read_write_bytes(self.kernel_write.take().unwrap(),
                                         self.kernel_read.take(),
                                         len);
    }
}

impl<'a, S: SpiMasterDevice> Driver for Spi<'a, S> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                let appc = match self.app.take() {
                    None => {
                        App {
                            callback: None,
                            app_read: Some(slice),
                            app_write: None,
                            len: 0,
                            index: 0,
                        }
                    }
                    Some(mut appc) => {
                        appc.app_read = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            }
            1 => {
                let appc = match self.app.take() {
                    None => {
                        App {
                            callback: None,
                            app_read: None,
                            app_write: Some(slice),
                            len: 0,
                            index: 0,
                        }
                    }
                    Some(mut appc) => {
                        appc.app_write = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    #[inline(never)]
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 /* read_write */ => {
                let appc = match self.app.take() {
                    None => App {
                        callback: Some(callback),
                        app_read: None,
                        app_write: None,
                        len: 0,
                        index: 0,
                    },
                    Some(mut appc) => {
                        appc.callback = Some(callback);
                        appc
                    }
                };
                self.app.replace(appc);
                ReturnCode::SUCCESS
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }
    // 0: read/write a single byte (blocking)
    // 1: read/write buffers
    //   - requires write buffer registered with allow
    //   - read buffer optional
    // 2: set chip select
    //   - selects which peripheral (CS line) the SPI should
    //     activate
    //   - valid values are 0-3 for SAM4L
    //   - invalid value will result in CS 0
    // 3: get chip select
    //   - returns current selected peripheral
    //   - If none selected, returns 255
    // 4: set rate on current peripheral
    //   - parameter in bps
    // 5: get rate on current peripheral
    //   - value in bps
    // 6: set clock phase on current peripheral
    //   - 0 is sample leading
    //   - non-zero is sample trailing
    // 7: get clock phase on current peripheral
    //   - 0 is sample leading
    //   - non-zero is sample trailing
    // 8: set clock polarity on current peripheral
    //   - 0 is idle low
    //   - non-zero is idle high
    // 9: get clock polarity on current peripheral
    //   - 0 is idle low
    //   - non-zero is idle high
    // 10: hold CS line low between transfers
    //   - set CSAAT bit of control register
    // 11: release CS line (high) between transfers
    //   - clear CSAAT bit of control register
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

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // No longer supported, wrap inside a read_write_bytes
            1 /* read_write_byte */ => ReturnCode::ENOSUPPORT,
            2 /* read_write_bytes */ => {
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
            3 /* set chip select */ => {
                // do nothing, for now, until we fix interface
                // so virtual instances can use multiple chip selects
                ReturnCode::ENOSUPPORT
            }
            4 /* get chip select */ => {
                //XXX Was a naked, uncommented zero. I'm assuming that's
                //    because the only valid chip select for now is 0,
                //    and wrapping it appropriately -Pat, ReturnCode fixes
                ReturnCode::SuccessWithValue { value: 0 }
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

impl<'a, S: SpiMasterDevice> SpiMasterClient for Spi<'a, S> {
    fn read_write_done(&self,
                       writebuf: &'static mut [u8],
                       readbuf: Option<&'static mut [u8]>,
                       length: usize) {
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
                app.callback.take().map(|mut cb| { cb.schedule(app.len, 0, 0); });
            } else {
                self.do_next_read_write(app);
            }
        });
    }
}
