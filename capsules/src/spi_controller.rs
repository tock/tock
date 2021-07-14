//! Provides userspace applications with the ability to communicate over the SPI
//! bus.

use core::cell::Cell;
use core::{cmp, mem};

use kernel::grant::Grant;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Spi as usize;

/// Suggested length for the Spi read and write buffer
pub const DEFAULT_READ_BUF_LENGTH: usize = 1024;
pub const DEFAULT_WRITE_BUF_LENGTH: usize = 1024;

// SPI operations are handled by coping into a kernel buffer for
// writes and copying out of a kernel buffer for reads.
//
// If the application buffer is larger than the kernel buffer,
// the driver issues multiple HAL operations. The len field
// of an application keeps track of the length of the desired
// operation, while the index variable keeps track of the
// index an ongoing operation is at in the buffers.

#[derive(Default)]
pub struct App {
    app_read: ReadWriteProcessBuffer,
    app_write: ReadOnlyProcessBuffer,
    len: usize,
    index: usize,
}

pub struct Spi<'a, S: SpiMasterDevice> {
    spi_master: &'a S,
    busy: Cell<bool>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
    grants: Grant<App, 1>,
    current_process: OptionalCell<ProcessId>,
}

impl<'a, S: SpiMasterDevice> Spi<'a, S> {
    pub fn new(spi_master: &'a S, grants: Grant<App, 1>) -> Spi<'a, S> {
        Spi {
            spi_master: spi_master,
            busy: Cell::new(false),
            kernel_len: Cell::new(0),
            kernel_read: TakeCell::empty(),
            kernel_write: TakeCell::empty(),
            grants,
            current_process: OptionalCell::empty(),
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
        let write_len = self.kernel_write.map_or(0, |kwbuf| {
            let mut start = app.index;
            let tmp_len = app
                .app_write
                .enter(|src| {
                    let len = cmp::min(app.len - start, self.kernel_len.get());
                    let end = cmp::min(start + len, src.len());
                    start = cmp::min(start, end);

                    for (i, c) in src[start..end].iter().enumerate() {
                        kwbuf[i] = c.get();
                    }
                    end - start
                })
                .unwrap_or(0);
            app.index = start + tmp_len;
            tmp_len
        });
        let _ = self.spi_master.read_write_bytes(
            self.kernel_write.take().unwrap(),
            self.kernel_read.take(),
            write_len,
        );
    }
}

impl<'a, S: SpiMasterDevice> SyscallDriver for Spi<'a, S> {
    fn allow_readwrite(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass in a read buffer to receive bytes into.
            0 => self
                .grants
                .enter(process_id, |grant, _| {
                    mem::swap(&mut grant.app_read, &mut slice);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass in a write buffer to transmit bytes from.
            0 => self
                .grants
                .enter(process_id, |grant, _| {
                    mem::swap(&mut grant.app_write, &mut slice);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
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
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned unconditionally.
            return CommandReturn::success();
        }

        // Check if this driver is free, or already dedicated to this process.
        let match_or_empty_or_nonexistant = self.current_process.map_or(true, |current_process| {
            self.grants
                .enter(*current_process, |_, _| current_process == &process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.current_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            // No longer supported, wrap inside a read_write_bytes
            1 /* read_write_byte */ => CommandReturn::failure(ErrorCode::NOSUPPORT),
            2 /* read_write_bytes */ => {
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants.enter(process_id, |app, _| {
                    // When we do a read/write, the read part is optional.
                    // So there are three cases:
                    // 1) Write and read buffers present: len is min of lengths
                    // 2) Only write buffer present: len is len of write
                    // 3) No write buffer present: no operation
                    let mut mlen = app.app_write.enter(|w| w.len()).unwrap_or(0);
                    let rlen = app.app_read.enter(|r| r.len()).unwrap_or(mlen);
                    mlen = cmp::min(mlen, rlen);

                    if mlen >= arg1 && arg1 > 0 {
                        app.len = arg1;
                        app.index = 0;
                        self.busy.set(true);
                        self.do_next_read_write(app);
                        CommandReturn::success()
                    } else {
                        /* write buffer too small, or zero length write */
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            3 /* set chip select */ => {
                // XXX: TODO: do nothing, for now, until we fix interface
                // so virtual instances can use multiple chip selects
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
            4 /* get chip select */ => {
                // XXX: We don't really know what chip select is being used
                // since we can't set it. Return error until set chip select
                // works.
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
            5 /* set baud rate */ => {
                self.spi_master.set_rate(arg1 as u32);
                CommandReturn::success()
            }
            6 /* get baud rate */ => {
                CommandReturn::success_u32(self.spi_master.get_rate() as u32)
            }
            7 /* set phase */ => {
                match arg1 {
                    0 => self.spi_master.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_master.set_phase(ClockPhase::SampleTrailing),
                };
                CommandReturn::success()
            }
            8 /* get phase */ => {
                CommandReturn::success_u32(self.spi_master.get_phase() as u32)
            }
            9 /* set polarity */ => {
                match arg1 {
                    0 => self.spi_master.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_master.set_polarity(ClockPolarity::IdleHigh),
                };
                CommandReturn::success()
            }
            10 /* get polarity */ => {
                CommandReturn::success_u32(self.spi_master.get_polarity() as u32)
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

impl<S: SpiMasterDevice> SpiMasterClient for Spi<'_, S> {
    fn read_write_done(
        &self,
        writebuf: &'static mut [u8],
        readbuf: Option<&'static mut [u8]>,
        length: usize,
    ) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(*process_id, move |app, upcalls| {
                let rbuf = readbuf.map(|src| {
                    let index = app.index;
                    let _ = app.app_read.mut_enter(|dest| {
                        // Need to be careful that app_read hasn't changed
                        // under us, so check all values against actual
                        // slice lengths.
                        //
                        // If app_read is shorter than before, and shorter
                        // than what we have read would require, then truncate.
                        // -pal 12/9/20
                        let end = index;
                        let start = index - length;
                        let end = cmp::min(end, dest.len());

                        // If the new endpoint is earlier than our expected
                        // startpoint, we set the startpoint to be the same;
                        // This results in a zero-length operation. -pal 12/9/20
                        let start = cmp::min(start, end);

                        // The amount to copy can't be longer than the size of the
                        // read buffer. -pal 6/8/21
                        let real_len = cmp::min(end - start, src.len());
                        let dest_area = &dest[start..end];
                        for (i, c) in src[0..real_len].iter().enumerate() {
                            dest_area[i].set(*c);
                        }
                    });
                    src
                });

                self.kernel_read.put(rbuf);
                self.kernel_write.replace(writebuf);

                if app.index == app.len {
                    self.busy.set(false);
                    let len = app.len;
                    app.len = 0;
                    app.index = 0;
                    upcalls.schedule_upcall(0, len, 0, 0).ok();
                } else {
                    self.do_next_read_write(app);
                }
            });
        });
    }
}
