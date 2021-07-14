//! Provides userspace applications with the ability to communicate over the SPI
//! bus as a peripheral. Only supports chip select 0.

use core::cell::Cell;
use core::cmp;
use core::mem;

use kernel::grant::Grant;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::{SpiSlaveClient, SpiSlaveDevice};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::SpiPeripheral as usize;

/// Suggested length for the SPI read and write buffer
pub const DEFAULT_READ_BUF_LENGTH: usize = 1024;
pub const DEFAULT_WRITE_BUF_LENGTH: usize = 1024;

// Since we provide an additional callback in slave mode for
// when the chip is selected, we have added a "PeripheralApp" struct
// that includes this new callback field.
#[derive(Default)]
pub struct PeripheralApp {
    app_read: ReadWriteProcessBuffer,
    app_write: ReadOnlyProcessBuffer,
    len: usize,
    index: usize,
}

pub struct SpiPeripheral<'a, S: SpiSlaveDevice> {
    spi_slave: &'a S,
    busy: Cell<bool>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
    grants: Grant<PeripheralApp, 2>,
    current_process: OptionalCell<ProcessId>,
}

impl<'a, S: SpiSlaveDevice> SpiPeripheral<'a, S> {
    pub fn new(spi_slave: &'a S, grants: Grant<PeripheralApp, 2>) -> SpiPeripheral<'a, S> {
        SpiPeripheral {
            spi_slave: spi_slave,
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
    fn do_next_read_write(&self, app: &mut PeripheralApp) {
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
        let _ = self.spi_slave.read_write_bytes(
            self.kernel_write.take(),
            self.kernel_read.take(),
            write_len,
        );
    }
}

impl<S: SpiSlaveDevice> SyscallDriver for SpiPeripheral<'_, S> {
    /// Provide read/write buffers to SpiPeripheral
    ///
    /// - allow_num 0: Provides a buffer to receive transfers into.
    ///
    fn allow_readwrite(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = self
            .grants
            .enter(process_id, |grant, _| match allow_num {
                0 => {
                    mem::swap(&mut grant.app_read, &mut slice);
                    Ok(())
                }
                _ => Err(ErrorCode::NOSUPPORT),
            })
            .unwrap_or_else(|e| e.into());

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    /// Provide read-only buffers to SpiPeripheral
    ///
    /// - allow_num 0: Provides a buffer to transmit
    ///
    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = self
            .grants
            .enter(process_id, |grant, _| match allow_num {
                0 => {
                    mem::swap(&mut grant.app_write, &mut slice);
                    Ok(())
                }
                _ => Err(ErrorCode::NOSUPPORT),
            })
            .unwrap_or_else(|e| e.into());

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
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
            1 /* read_write_bytes */ => {
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants.enter(process_id, |app, _| {
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
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                }).unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }
            2 /* get chip select */ => {
                // Only 0 is supported
                CommandReturn::success_u32(0)
            }
            3 /* set phase */ => {
                match arg1 {
                    0 => self.spi_slave.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_slave.set_phase(ClockPhase::SampleTrailing),
                };
                CommandReturn::success()
            }
            4 /* get phase */ => {
                CommandReturn::success_u32(self.spi_slave.get_phase() as u32)
            }
            5 /* set polarity */ => {
                match arg1 {
                    0 => self.spi_slave.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_slave.set_polarity(ClockPolarity::IdleHigh),
                };
                CommandReturn::success()
            }
            6 /* get polarity */ => {
                CommandReturn::success_u32(self.spi_slave.get_polarity() as u32)
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

impl<S: SpiSlaveDevice> SpiSlaveClient for SpiPeripheral<'_, S> {
    fn read_write_done(
        &self,
        writebuf: Option<&'static mut [u8]>,
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
                        let end = cmp::min(end, cmp::min(src.len(), dest.len()));

                        // If the new endpoint is earlier than our expected
                        // startpoint, we set the startpoint to be the same;
                        // This results in a zero-length operation. -pal 12/9/20
                        let start = cmp::min(start, end);

                        let dest_area = &dest[start..end];
                        let real_len = end - start;

                        for (i, c) in src[0..real_len].iter().enumerate() {
                            dest_area[i].set(*c);
                        }
                    });
                    src
                });

                self.kernel_read.put(rbuf);
                self.kernel_write.put(writebuf);

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

    // Simple callback for when chip has been selected
    fn chip_selected(&self) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(*process_id, move |app, upcalls| {
                let len = app.len;
                upcalls.schedule_upcall(1, len, 0, 0).ok();
            });
        });
    }
}
