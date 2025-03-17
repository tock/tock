// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace applications with the ability to communicate over the SPI
//! bus.

use core::cell::Cell;
use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Spi as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

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
    len: usize,
    index: usize,
}

pub struct Spi<'a, S: SpiMasterDevice<'a>> {
    spi_master: &'a S,
    busy: Cell<bool>,
    kernel_read: MapCell<SubSliceMut<'static, u8>>,
    kernel_write: MapCell<SubSliceMut<'static, u8>>,
    kernel_len: Cell<usize>,
    grants: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,
    command: Cell<UserCommand>,
}

#[derive(Debug, Clone, Copy)]
enum UserCommand {
    ReadBytes,
    InplaceReadWriteBytes,
}

impl<'a, S: SpiMasterDevice<'a>> Spi<'a, S> {
    pub fn new(
        spi_master: &'a S,
        grants: Grant<
            App,
            UpcallCount<1>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Spi<'a, S> {
        Spi {
            spi_master,
            busy: Cell::new(false),
            kernel_len: Cell::new(0),
            kernel_read: MapCell::empty(),
            kernel_write: MapCell::empty(),
            grants,
            current_process: OptionalCell::empty(),
            command: Cell::new(UserCommand::ReadBytes),
        }
    }

    pub fn config_buffers(&self, read: &'static mut [u8], write: &'static mut [u8]) {
        let len = cmp::min(read.len(), write.len());
        self.kernel_len.set(len);
        self.kernel_read.replace(read.into());
        self.kernel_write.replace(write.into());
    }

    // Assumes checks for busy/etc. already done
    // Updates app.index to be index + length of op
    fn do_next_read_write(&self, app: &mut App, kernel_data: &GrantKernelData) {
        let write_len = self.kernel_write.map_or(0, |kwbuf| {
            let mut start = app.index;
            let tmp_len = kernel_data
                .get_readonly_processbuffer(ro_allow::WRITE)
                .and_then(|write| {
                    write.enter(|src| {
                        let len = cmp::min(app.len - start, self.kernel_len.get());
                        let end = cmp::min(start + len, src.len());
                        start = cmp::min(start, end);

                        for (i, c) in src[start..end].iter().enumerate() {
                            kwbuf[i] = c.get();
                        }
                        end - start
                    })
                })
                .unwrap_or(0);
            app.index = start + tmp_len;
            tmp_len
        });

        let rlen = kernel_data
            .get_readwrite_processbuffer(rw_allow::READ)
            .map_or(0, |read| read.len());

        // TODO verify SPI return value
        let _ = if rlen == 0 {
            let mut kwbuf = self
                .kernel_write
                .take()
                .unwrap_or((&mut [] as &'static mut [u8]).into());
            kwbuf.slice(0..write_len);
            self.spi_master.read_write_bytes(kwbuf, None)
        } else if write_len == 0 {
            let read_len = self
                .kernel_write
                .map_or(0, |kwbuf| match self.command.get() {
                    UserCommand::ReadBytes => {
                        kwbuf[..].fill(0xFF);

                        cmp::min(kwbuf.len(), rlen)
                    }
                    UserCommand::InplaceReadWriteBytes => kernel_data
                        .get_readwrite_processbuffer(rw_allow::READ)
                        .and_then(|read| {
                            read.mut_enter(|src| {
                                let length = cmp::min(kwbuf.len(), rlen);

                                let start = app.index;
                                let end = cmp::min(app.index + length, src.len());

                                for (i, c) in src[start..end].iter().enumerate() {
                                    kwbuf[i] = c.get();
                                }

                                length
                            })
                        })
                        .unwrap_or(0),
                });
            app.index += read_len;
            let kwbuf = self
                .kernel_write
                .take()
                .unwrap_or((&mut [] as &'static mut [u8]).into());
            if let Some(mut krbuf) = self.kernel_read.take() {
                krbuf.slice(0..read_len);
                self.spi_master.read_write_bytes(kwbuf, Some(krbuf))
            } else {
                self.spi_master.read_write_bytes(kwbuf, None)
            }
        } else {
            let mut kwbuf = self
                .kernel_write
                .take()
                .unwrap_or((&mut [] as &'static mut [u8]).into());
            kwbuf.slice(0..write_len);
            if let Some(mut krbuf) = self.kernel_read.take() {
                krbuf.slice(0..rlen);
                self.spi_master.read_write_bytes(kwbuf, Some(krbuf))
            } else {
                self.spi_master.read_write_bytes(kwbuf, None)
            }
        };
    }
}

impl<'a, S: SpiMasterDevice<'a>> SyscallDriver for Spi<'a, S> {
    // 0: driver existence check
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
    // 11: read buffers
    //   - read buffer required
    // 12: inplace read/write buffers
    //   - requires read buffer registered with allow
    //   - write buffer not supported
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
            // Handle unconditional driver existence check.
            return CommandReturn::success();
        }

        // Check if this driver is free, or already dedicated to this process.
        let match_or_empty_or_nonexistant = self.current_process.map_or(true, |current_process| {
            self.grants
                .enter(current_process, |_, _| current_process == process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.current_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            // No longer supported, wrap inside a read_write_bytes
            1 => {
                // read_write_byte
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
            2 => {
                // read_write_bytes
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants
                    .enter(process_id, |app, kernel_data| {
                        // When we do a read/write, the read part is optional.
                        // So there are three cases:
                        // 1) Write and read buffers present: len is min of lengths
                        // 2) Only write buffer present: len is len of write
                        // 3) No write buffer present: no operation
                        let wlen = kernel_data
                            .get_readonly_processbuffer(ro_allow::WRITE)
                            .map_or(0, |write| write.len());
                        let rlen = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .map_or(0, |read| read.len());
                        // Note that non-shared and 0-sized read buffers both report 0 as size
                        let len = if rlen == 0 { wlen } else { wlen.min(rlen) };

                        if len >= arg1 && arg1 > 0 {
                            app.len = arg1;
                            app.index = 0;
                            self.busy.set(true);
                            self.do_next_read_write(app, kernel_data);
                            CommandReturn::success()
                        } else {
                            /* write buffer too small, or zero length write */
                            CommandReturn::failure(ErrorCode::INVAL)
                        }
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            3 => {
                // set chip select
                // XXX: TODO: do nothing, for now, until we fix interface
                // so virtual instances can use multiple chip selects
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
            4 => {
                // get chip select *
                // XXX: We don't really know what chip select is being used
                // since we can't set it. Return error until set chip select
                // works.
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
            5 => {
                // set baud rate
                match self.spi_master.set_rate(arg1 as u32) {
                    Ok(()) => CommandReturn::success(),
                    Err(error) => CommandReturn::failure(error),
                }
            }
            6 => {
                // get baud rate
                CommandReturn::success_u32(self.spi_master.get_rate())
            }
            7 => {
                // set phase
                match match arg1 {
                    0 => self.spi_master.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_master.set_phase(ClockPhase::SampleTrailing),
                } {
                    Ok(()) => CommandReturn::success(),
                    Err(error) => CommandReturn::failure(error),
                }
            }
            8 => {
                // get phase
                CommandReturn::success_u32(self.spi_master.get_phase() as u32)
            }
            9 => {
                // set polarity
                match match arg1 {
                    0 => self.spi_master.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_master.set_polarity(ClockPolarity::IdleHigh),
                } {
                    Ok(()) => CommandReturn::success(),
                    Err(error) => CommandReturn::failure(error),
                }
            }
            10 => {
                // get polarity
                CommandReturn::success_u32(self.spi_master.get_polarity() as u32)
            }
            11 => {
                // read_bytes
                // write 0xFF to the SPI bus and return the read values to
                // userspace
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants
                    .enter(process_id, |app, kernel_data| {
                        // When we do a read, we just write 0xFF on the bus.
                        let rlen = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .map_or(0, |read| read.len());

                        if rlen >= arg1 && rlen > 0 {
                            app.len = arg1;
                            app.index = 0;
                            self.busy.set(true);
                            self.command.set(UserCommand::ReadBytes);
                            self.do_next_read_write(app, kernel_data);
                            CommandReturn::success()
                        } else {
                            /* write buffer too small, or zero length write */
                            CommandReturn::failure(ErrorCode::INVAL)
                        }
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            12 => {
                // inplace read_write_bytes
                if self.busy.get() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.grants
                    .enter(process_id, |app, kernel_data| {
                        let rlen = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .map_or(0, |read| read.len());

                        if rlen >= arg1 && arg1 > 0 {
                            app.len = arg1;
                            app.index = 0;
                            self.busy.set(true);
                            self.command.set(UserCommand::InplaceReadWriteBytes);
                            self.do_next_read_write(app, kernel_data);
                            CommandReturn::success()
                        } else {
                            /* write buffer too small, or zero length write */
                            CommandReturn::failure(ErrorCode::INVAL)
                        }
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

impl<'a, S: SpiMasterDevice<'a>> SpiMasterClient for Spi<'a, S> {
    fn read_write_done(
        &self,
        mut writebuf: SubSliceMut<'static, u8>,
        readbuf: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, move |app, kernel_data| {
                let rbuf = readbuf.inspect(|src| {
                    let index = app.index;
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::READ)
                        .and_then(|read| {
                            read.mut_enter(|dest| {
                                // Need to be careful that app_read hasn't changed
                                // under us, so check all values against actual
                                // slice lengths.
                                //
                                // If app_read is shorter than before, and shorter
                                // than what we have read would require, then truncate.
                                // -pal 12/9/20
                                let end = index;
                                let start = index - status.unwrap_or(0);
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
                            })
                        });
                });

                if let Some(mut rb) = rbuf {
                    rb.reset();
                    self.kernel_read.put(rb);
                }

                writebuf.reset();
                self.kernel_write.replace(writebuf);

                if app.index == app.len {
                    self.busy.set(false);
                    let len = app.len;
                    app.len = 0;
                    app.index = 0;
                    kernel_data.schedule_upcall(0, (len, 0, 0)).ok();
                } else {
                    self.do_next_read_write(app, kernel_data);
                }
            });
        });
    }
}
