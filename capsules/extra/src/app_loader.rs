// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This capsule provides an interface between a dynamic loading userspace
//! app and the kernel.
//!
//! This is an initial implementation that gets the app size from the
//! userspace app and sets up the flash region in which the app will be
//! written. Then the app is actually written to flash. Finally, the
//! the userspace app sends a request for the app to be loaded.
//!
//!
//! Here is a diagram of the expected stack with this capsule:
//! Boxes are components and between the boxes are the traits that are the
//! interfaces between components.
//!
//! ```text
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |                         userspace                               |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!                         kernel::Driver
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |               capsules::app_loader::AppLoader (this)            |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!        kernel::dynamic_process_loading::DynamicProcessLoading
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |               Kernel  | Physical Nonvolatile Storage            |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!             hil::nonvolatile_storage::NonvolatileStorage
//! ```
//!
//! Example instantiation:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::app_loader::DRIVER_NUM,
//!     dynamic_process_loader,
//!     ).finalize(components::app_loader_component_static!());
//!
//! NOTE: This implementation currently only loads new apps. It does not update apps. That remains to be tested.
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::dynamic_process_loading;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::AppLoader as usize;

/// IDs for subscribed upcalls.
mod upcall {
    /// Write done callback.
    pub const WRITE_DONE: usize = 0;
    /// Number of upcalls.
    pub const COUNT: u8 = 1;
}

// Ids for read-only allow buffers
mod ro_allow {
    /// Setup a buffer to write bytes to the nonvolatile storage.
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Setup a buffer to read from the nonvolatile storage into.
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub const BUF_LEN: usize = 512;

#[derive(Clone, Copy, PartialEq)]
pub enum NonvolatileCommand {
    UserspaceRead,
    UserspaceWrite,
}

#[derive(Clone, Copy)]
pub enum NonvolatileUser {
    App { processid: ProcessId },
}

// struct to store pending commands for future execution
pub struct App {
    pending_command: bool,
    command: NonvolatileCommand,
    offset: usize,
    length: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            pending_command: false,
            command: NonvolatileCommand::UserspaceRead,
            offset: 0,
            length: 0,
        }
    }
}

pub struct AppLoader<'a> {
    // The underlying physical storage device.
    driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
    // Per-app state.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    // Internal buffer for copying appslices into.
    buffer: TakeCell<'static, [u8]>,
    // What issued the currently executing call.
    current_user: OptionalCell<NonvolatileUser>,
    new_app_length: Cell<usize>,
}

impl<'a> AppLoader<'a> {
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
        buffer: &'static mut [u8],
    ) -> AppLoader<'a> {
        AppLoader {
            driver: driver,
            apps: grant,
            buffer: TakeCell::new(buffer),
            current_user: OptionalCell::empty(),
            new_app_length: Cell::new(0),
        }
    }

    /// Check so see if we are doing something. If not, go ahead and do this
    /// command. If so, this is queued and will be run when the pending
    /// command completes.
    fn enqueue_command(
        &self,
        command: NonvolatileCommand,
        offset: usize,
        length: usize,
        processid: Option<ProcessId>,
    ) -> Result<(usize, usize, ProcessId), ErrorCode> {
        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                // Userspace sees memory that starts at address 0 even if it
                // is offset in the physical memory.
                match offset.checked_add(length) {
                    Some(result) => {
                        if length > self.new_app_length.get() || result > self.new_app_length.get()
                        {
                            // this means the app is out of bounds
                            return Err(ErrorCode::INVAL);
                        }
                    }
                    None => {
                        return Err(ErrorCode::INVAL); // untested
                    }
                }
            }
        }

        match command {
            NonvolatileCommand::UserspaceRead | NonvolatileCommand::UserspaceWrite => {
                processid.map_or(Err(ErrorCode::FAIL), |processid| {
                    self.apps
                        .enter(processid, |app, kernel_data| {
                            // Get the length of the correct allowed buffer.
                            let allow_buf_len = match command {
                                NonvolatileCommand::UserspaceRead => kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .map_or(0, |read| read.len()),
                                NonvolatileCommand::UserspaceWrite => kernel_data
                                    .get_readonly_processbuffer(ro_allow::WRITE)
                                    .map_or(0, |read| read.len()),
                            };

                            // Check that it exists.
                            if allow_buf_len == 0 || self.buffer.is_none() {
                                return Err(ErrorCode::RESERVE);
                            }

                            // Shorten the length if the application gave us nowhere to
                            // put it.
                            let active_len = cmp::min(length, allow_buf_len);

                            // First need to determine if we can execute this or must
                            // queue it.
                            if self.current_user.is_none() {
                                // No app is currently using the underlying storage.
                                // Mark this app as active, and then execute the command.
                                self.current_user.set(NonvolatileUser::App {
                                    processid: processid,
                                });

                                // Need to copy bytes if this is a write!
                                if command == NonvolatileCommand::UserspaceWrite {
                                    let _ = kernel_data
                                        .get_readonly_processbuffer(ro_allow::WRITE)
                                        .and_then(|write| {
                                            write.enter(|app_buffer| {
                                                self.buffer.map(|kernel_buffer| {
                                                    // Check that the internal buffer and the buffer that was
                                                    // allowed are long enough.
                                                    let write_len =
                                                        cmp::min(active_len, kernel_buffer.len());

                                                    let buf_data = &app_buffer[0..write_len];
                                                    for (i, c) in kernel_buffer[0..write_len]
                                                        .iter_mut()
                                                        .enumerate()
                                                    {
                                                        *c = buf_data[i].get();
                                                    }
                                                });
                                            })
                                        });
                                }
                            } else {
                                // Some app is using the storage, we must wait.
                                if app.pending_command {
                                    // No more room in the queue, nowhere to store this
                                    // request.
                                    return Err(ErrorCode::NOMEM);
                                } else {
                                    // We can store this, so lets do it.
                                    app.pending_command = true;
                                    app.command = command;
                                    app.offset = offset;
                                    app.length = active_len;
                                }
                            }
                            Ok((offset, length, processid))
                        })
                        .unwrap_or_else(|err| Err(err.into()))
                })
            }
        }
    }

    fn check_queue(&self) {
        // Check all of the apps.
        for cntr in self.apps.iter() {
            let processid = cntr.processid();
            let started_command = cntr.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_user.set(NonvolatileUser::App {
                        processid: processid,
                    });
                    if let Ok(()) = self
                        .buffer
                        .take()
                        .map_or(Err(ErrorCode::RESERVE), |buffer| {
                            let mut write_buffer = SubSliceMut::new(buffer);
                            write_buffer.slice(..write_buffer.len());
                            self.driver
                                .write_app_data(write_buffer, app.offset, processid)
                        })
                    {
                        true
                    } else {
                        false
                    }
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

impl kernel::dynamic_process_loading::DynamicProcessLoadingClient for AppLoader<'_> {
    fn app_data_write_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_user.take().map(|user| {
            match user {
                NonvolatileUser::App { processid } => {
                    let _ = self.apps.enter(processid, move |_app, kernel_data| {
                        // Replace the buffer we used to do this write.
                        self.buffer.replace(buffer);

                        // And then signal the app.
                        kernel_data
                            .schedule_upcall(upcall::WRITE_DONE, (length, 0, 0))
                            .ok();
                    });
                }
            }
        });
        self.check_queue();
    }
}

/// Provide an interface for userland.
impl SyscallDriver for AppLoader<'_> {
    /// Command interface.
    ///
    /// Commands are selected by the lowest 8 bits of the first argument.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Request kernel to setup for loading app.
    ///        - Returns appsize if the kernel has available space
    ///        - Returns ErrorCode::FAIL if the kernel is unable to allocate space for the new app
    /// - `2`: Request kernel to write app data to the nonvolatile_storage.
    ///        - Returns Ok(()) when write is successful
    ///        - Returns ErrorCode::INVAL when the app is violating bounds
    ///        - Returns ErrorCode::FAIL when the write fails
    /// - `3`: Request kernel to load app.
    ///        - Returns Ok(()) when the process is successfully loaded
    ///        - Returns ErrorCode::FAIL if:
    ///            - The kernel is unable to create a process object for the application
    ///            - The kernel fails to write a padding app (thereby potentially breaking the linkedlist)

    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                //setup phase

                let res = self.driver.setup(arg1); // pass the size of the app to the setup function
                match res {
                    Ok(app_len) => {
                        self.new_app_length.set(app_len);
                        CommandReturn::success()
                    }

                    Err(e) => {
                        self.new_app_length.set(0);
                        CommandReturn::failure(e)
                    }
                }
            }

            2 => {
                // Request kernel to write app to flash

                let res = self.enqueue_command(
                    NonvolatileCommand::UserspaceWrite,
                    arg1,
                    arg2,
                    Some(processid),
                );
                match res {
                    Ok((offset, _len, pid)) => {
                        let result = self
                            .buffer
                            .take()
                            .map_or(Err(ErrorCode::RESERVE), |buffer| {
                                let mut write_buffer = SubSliceMut::new(buffer);
                                write_buffer.slice(..write_buffer.len());
                                let res = self.driver.write_app_data(write_buffer, offset, pid);
                                match res {
                                    Ok(()) => Ok(()),
                                    Err(e) => Err(e),
                                }
                            });
                        match result {
                            Ok(()) => CommandReturn::success(),
                            Err(e) => {
                                self.new_app_length.set(0);
                                CommandReturn::failure(e)
                            }
                        }
                    }
                    Err(e) => {
                        self.new_app_length.set(0);
                        CommandReturn::failure(e)
                    }
                }
            }

            3 => {
                // Request kernel to load the new app

                let res = self.driver.load();
                match res {
                    Ok(()) => {
                        self.new_app_length.set(0); // reset the app length
                        CommandReturn::success()
                    }
                    Err(e) => {
                        self.new_app_length.set(0); // reset the app length
                        CommandReturn::failure(e)
                    }
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
