// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! This allows multiple apps to write their own flash region.
//!
//! All write requests from userland are checked to ensure that they are only
//! trying to write their own flash space, and not the TBF header either.
//!
//! This driver can handle non page aligned writes.
//!
//! Userland apps should allocate buffers in flash when they are compiled to
//! ensure that there is room to write to. This should be accomplished by
//! declaring `const` buffers.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let app_flash_buffer = static_init!([u8; 512], [0; 512]);
//! let app_flash = static_init!(
//!     capsules::app_flash_driver::AppFlash<'static>,
//!     capsules::app_flash_driver::AppFlash::new(nv_to_page,
//!         board_kernel.create_grant(&grant_cap), app_flash_buffer));
//! ```

use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::AppFlash as usize;

/// IDs for subscribed upcalls.
mod upcall {
    /// `write_done` callback.
    pub const WRITE_DONE: usize = 0;
    /// Number of upcalls.
    pub const COUNT: u8 = 1;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Set write buffer. This entire buffer will be written to flash.
    pub const BUFFER: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App {
    pending_command: bool,
    flash_address: usize,
}

pub struct AppFlash<'a> {
    driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'a>,
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,
    current_app: OptionalCell<ProcessId>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> AppFlash<'a> {
    pub fn new(
        driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'a>,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        buffer: &'static mut [u8],
    ) -> AppFlash<'a> {
        AppFlash {
            driver,
            apps: grant,
            current_app: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
        }
    }

    // Check to see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending command
    // completes.
    fn enqueue_write(&self, flash_address: usize, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, kernel_data| {
                // Check that this is a valid range in the app's flash.
                let flash_length = kernel_data
                    .get_readonly_processbuffer(ro_allow::BUFFER)
                    .map_or(0, |buffer| buffer.len());
                let (app_flash_start, app_flash_end) = processid.get_editable_flash_range();
                if flash_address < app_flash_start
                    || flash_address >= app_flash_end
                    || flash_address + flash_length >= app_flash_end
                {
                    return Err(ErrorCode::INVAL);
                }

                if self.current_app.is_none() {
                    self.current_app.set(processid);

                    kernel_data
                        .get_readonly_processbuffer(ro_allow::BUFFER)
                        .and_then(|buffer| {
                            buffer.enter(|app_buffer| {
                                // Copy contents to internal buffer and write it.
                                self.buffer
                                    .take()
                                    .map_or(Err(ErrorCode::RESERVE), |buffer| {
                                        let length = cmp::min(buffer.len(), app_buffer.len());
                                        let d = &app_buffer[0..length];
                                        for (i, c) in buffer[0..length].iter_mut().enumerate() {
                                            *c = d[i].get();
                                        }

                                        self.driver.write(buffer, flash_address, length)
                                    })
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))
                } else {
                    // Queue this request for later.
                    if app.pending_command {
                        Err(ErrorCode::NOMEM)
                    } else {
                        app.pending_command = true;
                        app.flash_address = flash_address;
                        Ok(())
                    }
                }
            })
            .unwrap_or_else(|err| Err(err.into()))
    }
}

impl hil::nonvolatile_storage::NonvolatileStorageClient for AppFlash<'_> {
    fn read_done(&self, _buffer: &'static mut [u8], _length: usize) {}

    fn write_done(&self, buffer: &'static mut [u8], _length: usize) {
        // Put our write buffer back.
        self.buffer.replace(buffer);

        // Notify the current application that the command finished.
        self.current_app.take().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                let _ = upcalls.schedule_upcall(upcall::WRITE_DONE, (0, 0, 0));
            });
        });

        // Check if there are any pending events.
        for cntr in self.apps.iter() {
            let processid = cntr.processid();
            let started_command = cntr.enter(|app, kernel_data| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(processid);
                    let flash_address = app.flash_address;

                    kernel_data
                        .get_readonly_processbuffer(ro_allow::BUFFER)
                        .and_then(|buffer| {
                            buffer.enter(|app_buffer| {
                                self.buffer.take().is_some_and(|buffer| {
                                    if app_buffer.len() != 512 {
                                        false
                                    } else {
                                        // Copy contents to internal buffer and write it.
                                        let length = cmp::min(buffer.len(), app_buffer.len());
                                        let d = &app_buffer[0..length];
                                        for (i, c) in buffer[0..length].iter_mut().enumerate() {
                                            *c = d[i].get();
                                        }

                                        if let Ok(()) =
                                            self.driver.write(buffer, flash_address, length)
                                        {
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                })
                            })
                        })
                        .unwrap_or(false)
                } else {
                    false
                }
            });
            if started_command {
                break;
            }
        }
    }
}

impl SyscallDriver for AppFlash<'_> {
    /// App flash control.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Write the memory from the `allow` buffer to the address in flash.
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                // Write to flash from the allowed buffer
                let flash_address = arg1;

                let res = self.enqueue_write(flash_address, processid);

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
