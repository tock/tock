//! Provides userspace applications with the ability to communicate over the SPI
//! bus as a peripheral. Only supports chip select 0.

use core::cell::Cell;
use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::{SpiSlaveClient, SpiSlaveDevice};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::SpiPeripheral as usize;

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

/// Suggested length for the SPI read and write buffer
pub const DEFAULT_READ_BUF_LENGTH: usize = 1024;
pub const DEFAULT_WRITE_BUF_LENGTH: usize = 1024;

// Since we provide an additional callback in slave mode for
// when the chip is selected, we have added a "PeripheralApp" struct
// that includes this new callback field.
#[derive(Default)]
pub struct PeripheralApp {
    len: usize,
    index: usize,
}

pub struct SpiPeripheral<'a, S: SpiSlaveDevice> {
    spi_slave: &'a S,
    busy: Cell<bool>,
    kernel_read: TakeCell<'static, [u8]>,
    kernel_write: TakeCell<'static, [u8]>,
    kernel_len: Cell<usize>,
    grants: Grant<
        PeripheralApp,
        UpcallCount<2>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,
}

impl<'a, S: SpiSlaveDevice> SpiPeripheral<'a, S> {
    pub fn new(
        spi_slave: &'a S,
        grants: Grant<
            PeripheralApp,
            UpcallCount<2>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> SpiPeripheral<'a, S> {
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
    fn do_next_read_write(&self, app: &mut PeripheralApp, kernel_data: &GrantKernelData) {
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
        // TODO verify SPI return value
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

    /// Provide read-only buffers to SpiPeripheral
    ///
    /// - allow_num 0: Provides a buffer to transmit

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
                self.grants.enter(process_id, |app, kernel_data| {
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
                }).unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }
            2 /* get chip select */ => {
                // Only 0 is supported
                CommandReturn::success_u32(0)
            }
            3 /* set phase */ => {
                match match arg1 {
                    0 => self.spi_slave.set_phase(ClockPhase::SampleLeading),
                    _ => self.spi_slave.set_phase(ClockPhase::SampleTrailing),
                } {
                    Ok(()) => CommandReturn::success(),
                    Err(error) => CommandReturn::failure(error.into())
                }
            }
            4 /* get phase */ => {
                CommandReturn::success_u32(self.spi_slave.get_phase() as u32)
            }
            5 /* set polarity */ => {
                match match arg1 {
                    0 => self.spi_slave.set_polarity(ClockPolarity::IdleLow),
                    _ => self.spi_slave.set_polarity(ClockPolarity::IdleHigh),
                } {
                    Ok(()) => CommandReturn::success(),
                    Err(error) => CommandReturn::failure(error.into())
                }

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
        _status: Result<(), ErrorCode>,
    ) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(*process_id, move |app, kernel_data| {
                let rbuf = readbuf.map(|src| {
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
                            })
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
                    kernel_data.schedule_upcall(0, (len, 0, 0)).ok();
                } else {
                    self.do_next_read_write(app, kernel_data);
                }
            });
        });
    }

    // Simple callback for when chip has been selected
    fn chip_selected(&self) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(*process_id, move |app, kernel_data| {
                let len = app.len;
                kernel_data.schedule_upcall(1, (len, 0, 0)).ok();
            });
        });
    }
}
