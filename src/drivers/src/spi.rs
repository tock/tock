use common::take_cell::TakeCell;
use core::cell::Cell;
use main::{AppId,Driver,Callback,AppSlice,Shared};
use hil::spi_master::{SpiMaster,SpiCallback};
use core::cmp;
use hil::spi_master::ClockPolarity;
use hil::spi_master::ClockPhase;


/* SPI operations are handled by coping into a kernel buffer for
 * writes and copying out of a kernel buffer for reads.
 *
 * If the application buffer is larger than the kernel buffer,
 * the driver issues multiple HAL operations. The len field
 * of an application keeps track of the length of the desired
 * operation, while the index variable keeps track of the 
 * index an ongoing operation is at in the buffers. */

struct App {
    callback:  Option<Callback>,
    app_read:  Option<AppSlice<Shared, u8>>,
    app_write: Option<AppSlice<Shared, u8>>,
    len:       usize,
    index:     usize,
}

pub struct Spi<'a, S: SpiMaster + 'a> {
    spi_master:   &'a mut S,
    busy:         Cell<bool>,
    app:         TakeCell<App>,
    kernel_read:  TakeCell<&'static mut [u8]>,
    kernel_write: TakeCell<&'static mut [u8]>,
    kernel_len:   Cell<usize>
}

impl<'a, S: SpiMaster> Spi<'a, S> {
    pub fn new(spi_master: &'a mut S) -> Spi<S> {
        Spi {
            spi_master: spi_master,
            busy: Cell::new(false),
            app: TakeCell::empty(),
            kernel_len: Cell::new(0),
            kernel_read : TakeCell::empty(),
            kernel_write : TakeCell::empty()
        }
    }

    pub fn config_buffers(&mut self,
                          read: &'static mut [u8],
                          write: &'static mut [u8]) {
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
                for (i, c) in src.as_ref()[start .. end].iter().enumerate() {
                    kwbuf[i] = *c;
                }
            });
        });

        self.spi_master.read_write_bytes(self.kernel_write.take(),
                                         self.kernel_read.take(), len);
    }
}

impl<'a, S: SpiMaster> Driver for Spi<'a, S> {
    fn allow(&self, _appid: AppId,
             allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            0 => {
                let appc = match self.app.take() {
                    None => App {
                        callback: None,
                        app_read: Some(slice),
                        app_write: None,
                        len: 0,
                        index: 0,
                    },
                    Some(mut appc) => {
                        appc.app_read = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                0
            },
            1 => {
                let appc = match self.app.take() {
                    None => App {
                        callback: None,
                        app_read: None,
                        app_write: Some(slice),
                        len: 0,
                        index: 0,
                    },
                    Some(mut appc) => {
                        appc.app_write = Some(slice);
                        appc
                    }
                };
                self.app.replace(appc);
                0
            }
            _ => -1
        }
    }

    #[inline(never)]
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
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
                0
            },
            _ => -1
        }
    }
    /*
     * 0: read/write a single byte (blocking)
     * 1: read/write buffers
     *   - requires write buffer registered with allow
     *   - read buffer optional
     * 2: set chip select
     *   - selects which peripheral (CS line) the SPI should
     *     activate
     *   - valid values are 0-3 for SAM4L
     *   - invalid value will result in CS 0
     * 3: get chip select
     *   - returns current selected peripheral
     *   - If none selected, returns 255
     * 4: set rate on current peripheral
     *   - parameter in bps
     * 5: get rate on current peripheral
     *   - value in bps
     * 6: set clock phase on current peripheral
     *   - 0 is sample leading
     *   - non-zero is sample trailing
     * 7: get clock phase on current peripheral
     *   - 0 is sample leading
     *   - non-zero is sample trailing
     * 8: set clock polarity on current peripheral
     *   - 0 is idle low
     *   - non-zero is idle high
     * 9: get clock polarity on current peripheral
     *   - 0 is idle low
     *   - non-zero is idle high
     * 10: hold CS line low between transfers
     *   - set CSAAT bit of control register
     * 11: release CS line (high) between transfers
     *   - clear CSAAT bit of control register
     *
     * x: lock spi
     *   - if you perform an operation without the lock,
     *     it implicitly acquires the lock before the
     *     operation and releases it after
     *   - while an app holds the lock no other app can issue
     *     operations on SPI (they are buffered)
     * x+1: unlock spi
     *   - does nothing if lock not held
     */

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> isize {
        match cmd_num {
            0 /* read_write_byte */ => { 
                self.spi_master.read_write_byte(arg1 as u8) as isize
            },
            1 /* read_write_bytes */ => { 
                if self.busy.get() {
                    return -1;
                }
                let mut result = -1;
                self.app.map(|app| {
                    let mut mlen = 0;
                    // If write buffer too small, return
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
                        result = 0;
                    }
                });
                return result;
            }
            2 /* set chip select */ => {
                let cs = arg1 as u8;
                if cs <= 3 {
                    self.spi_master.set_chip_select(cs);
                    0
                } else {
                    -1
                }
            }
            3 /* get chip select */ => {
                self.spi_master.get_chip_select() as isize
            }
            4 /* set baud rate */ => {
                self.spi_master.set_rate(arg1 as u32) as isize
            }
            5 /* get baud rate */ => {
                self.spi_master.get_rate() as isize
            }
            6 /* set phase */ => {
                match arg1 {
                    0 => self.spi_master.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_master.set_phase(ClockPhase::SampleTrailing),
                };
                0
            }
            7 /* get phase */ => {
                self.spi_master.get_phase() as isize
            }
            8 /* set polarity */ => {
                match arg1 {
                    0 => self.spi_master.set_clock(ClockPolarity::IdleLow),
                    _ => self.spi_master.set_clock(ClockPolarity::IdleHigh),
                };
                0
            }
            9 /* get polarity */ => {
                self.spi_master.get_clock() as isize
            }
            10 /* hold low */ => {
                self.spi_master.hold_low();
                0
            }
            11 /* release low */ => {
                self.spi_master.release_low();
                0
            }
            _ => -1
        }
    }
}

impl<'a, S: SpiMaster> SpiCallback for Spi<'a, S> {
    fn read_write_done(&self,
                       writebuf: Option<&'static mut [u8]>,
                       readbuf:  Option<&'static mut [u8]>,
                       length: usize) {
        self.app.map(|app| {
            if app.app_read.is_some() {
                let src = readbuf.as_ref().unwrap();
                let dest = app.app_read.as_mut().unwrap();
                let start = app.index - length;
                let end = start + length;

                let d = &mut dest.as_mut()[start .. end];
                for (i, c) in src[0 .. length].iter().enumerate() {
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
}

