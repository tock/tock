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
//! ```
//! # use kernel::static_init;
//!
//! pub static mut APP_FLASH_BUFFER: [u8; 512] = [0; 512];
//! let app_flash = static_init!(
//!     capsules::app_flash_driver::AppFlash<'static>,
//!     capsules::app_flash_driver::AppFlash::new(nv_to_page,
//!         board_kernel.create_grant(&grant_cap), &mut APP_FLASH_BUFFER));
//! ```

use core::cmp;
use core::mem;

use kernel::grant::Grant;
use kernel::hil;
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::AppFlash as usize;

#[derive(Default)]
pub struct App {
    buffer: ReadOnlyProcessBuffer,
    pending_command: bool,
    flash_address: usize,
}

pub struct AppFlash<'a> {
    driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'static>,
    apps: Grant<App, 1>,
    current_app: OptionalCell<ProcessId>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> AppFlash<'a> {
    pub fn new(
        driver: &'a dyn hil::nonvolatile_storage::NonvolatileStorage<'static>,
        grant: Grant<App, 1>,
        buffer: &'static mut [u8],
    ) -> AppFlash<'a> {
        AppFlash {
            driver: driver,
            apps: grant,
            current_app: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
        }
    }

    // Check to see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending command
    // completes.
    fn enqueue_write(&self, flash_address: usize, appid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(appid, |app, _| {
                // Check that this is a valid range in the app's flash.
                let flash_length = app.buffer.len();
                let (app_flash_start, app_flash_end) = appid.get_editable_flash_range();
                if flash_address < app_flash_start
                    || flash_address >= app_flash_end
                    || flash_address + flash_length >= app_flash_end
                {
                    return Err(ErrorCode::INVAL);
                }

                if self.current_app.is_none() {
                    self.current_app.set(appid);

                    app.buffer
                        .enter(|app_buffer| {
                            // Copy contents to internal buffer and write it.
                            self.buffer
                                .take()
                                .map_or(Err(ErrorCode::RESERVE), |buffer| {
                                    let length = cmp::min(buffer.len(), app_buffer.len());
                                    let d = &app_buffer[0..length];
                                    for (i, c) in buffer.as_mut()[0..length].iter_mut().enumerate()
                                    {
                                        *c = d[i].get();
                                    }

                                    self.driver.write(buffer, flash_address, length)
                                })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))
                } else {
                    // Queue this request for later.
                    if app.pending_command == true {
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

impl hil::nonvolatile_storage::NonvolatileStorageClient<'static> for AppFlash<'_> {
    fn read_done(&self, _buffer: &'static mut [u8], _length: usize) {}

    fn write_done(&self, buffer: &'static mut [u8], _length: usize) {
        // Put our write buffer back.
        self.buffer.replace(buffer);

        // Notify the current application that the command finished.
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |_app, upcalls| {
                upcalls.schedule_upcall(0, 0, 0, 0).ok();
            });
        });

        // Check if there are any pending events.
        for cntr in self.apps.iter() {
            let appid = cntr.processid();
            let started_command = cntr.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(appid);
                    let flash_address = app.flash_address;

                    app.buffer
                        .enter(|app_buffer| {
                            self.buffer.take().map_or(false, |buffer| {
                                if app_buffer.len() != 512 {
                                    false
                                } else {
                                    // Copy contents to internal buffer and write it.
                                    let length = cmp::min(buffer.len(), app_buffer.len());
                                    let d = &app_buffer[0..length];
                                    for (i, c) in buffer.as_mut()[0..length].iter_mut().enumerate()
                                    {
                                        *c = d[i].get();
                                    }

                                    if let Ok(()) = self.driver.write(buffer, flash_address, length)
                                    {
                                        true
                                    } else {
                                        false
                                    }
                                }
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
    /// Setup buffer to write from.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Set write buffer. This entire buffer will be written to flash.
    fn allow_readonly(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.buffer, &mut slice);
                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into())),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `0`: Set a write_done callback.

    /// App flash control.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Write the memory from the `allow` buffer to the address in flash.
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 /* This driver exists. */ => {
                CommandReturn::success()
            }

            1 /* Write to flash from the allowed buffer */ => {
                let flash_address = arg1;

                let res = self.enqueue_write(flash_address, appid);

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            _ /* Unknown command num */ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
