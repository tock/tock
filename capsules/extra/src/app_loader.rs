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
//!                         kernel::SyscallDriver
//! +-----------------------------------------------------------------+
//! |                                                                 |
//! |               capsules::app_loader::AppLoader (this)            |
//! |                                                                 |
//! +-----------------------------------------------------------------+
//!         kernel::dynamic_process_loading::DynamicBinaryFlashing
//!         kernel::dynamic_process_loading::DynamicProcessLoading
//! +-----------------------------------------------------------------+
//! |                                     |                           |
//! |  Physical Nonvolatile Storage       |           Kernel          |
//! |                                     |                           |
//! +-----------------------------------------------------------------+
//!             hil::nonvolatile_storage::NonvolatileStorage
//! ```
//!
//! Example instantiation:
//!
//! ```rust, ignore
//! # use kernel::static_init;
//!
//! let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::app_loader::DRIVER_NUM,
//!     dynamic_process_loader,
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
    /// Setup Done callback
    pub const SETUP_DONE: usize = 0;
    /// Write done callback.
    pub const WRITE_DONE: usize = 1;
    /// Load done callback.
    pub const LOAD_DONE: usize = 2;
    /// Number of upcalls.
    pub const COUNT: u8 = 3;
}

// Ids for read-only allow buffers
mod ro_allow {
    /// Setup a buffer to write bytes to the nonvolatile storage.
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub const BUF_LEN: usize = 512;

#[derive(Default)]
pub struct App {}

pub struct AppLoader<'a> {
    // The underlying driver for the process flashing and loading.
    storage_driver: &'a dyn dynamic_process_loading::DynamicBinaryFlashing,
    loading_driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
    // Per-app state.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<0>,
    >,

    // Internal buffer for copying appslices into.
    buffer: TakeCell<'static, [u8]>,
    // What issued the currently executing call.
    current_process: OptionalCell<ProcessId>,
    new_app_length: Cell<usize>,
}

impl<'a> AppLoader<'a> {
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        storage_driver: &'a dyn dynamic_process_loading::DynamicBinaryFlashing,
        loading_driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
        buffer: &'static mut [u8],
    ) -> AppLoader<'a> {
        AppLoader {
            apps: grant,
            storage_driver,
            loading_driver,
            buffer: TakeCell::new(buffer),
            current_process: OptionalCell::empty(),
            new_app_length: Cell::new(0),
        }
    }

    /// Copy data from the shared buffer with app and request kernel to
    /// write the app data to flash.
    fn write(&self, offset: usize, length: usize, processid: ProcessId) -> Result<(), ErrorCode> {
        // Userspace sees memory that starts at address 0 even if it
        // is offset in the physical memory.
        match offset.checked_add(length) {
            Some(result) => {
                if length > self.new_app_length.get() || result > self.new_app_length.get() {
                    // this means the app is out of bounds
                    return Err(ErrorCode::INVAL);
                }
            }
            None => {
                return Err(ErrorCode::INVAL);
            }
        }
        self.apps
            .enter(processid, |_app, kernel_data| {
                // Get the length of the correct allowed buffer.
                let allow_buf_len = kernel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .map_or(0, |read| read.len());

                // Check that it exists.
                if allow_buf_len == 0 || self.buffer.is_none() {
                    return Err(ErrorCode::RESERVE);
                }

                // Shorten the length if the application did not give us
                // enough bytes in the allowed buffer.
                let active_len = cmp::min(length, allow_buf_len);

                // copy data into the kernel buffer!
                let _ = kernel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .and_then(|write| {
                        write.enter(|app_buffer| {
                            self.buffer.map(|kernel_buffer| {
                                // Check that the internal buffer and the buffer that was
                                // allowed are long enough.
                                let write_len = cmp::min(active_len, kernel_buffer.len());

                                let buf_data = &app_buffer[0..write_len];
                                for (i, c) in kernel_buffer[0..write_len].iter_mut().enumerate() {
                                    *c = buf_data[i].get();
                                }
                            });
                        })
                    });
                self.buffer
                    .take()
                    .map_or(Err(ErrorCode::RESERVE), |buffer| {
                        let mut write_buffer = SubSliceMut::new(buffer);
                        write_buffer.slice(..length); // should be the length supported by the app (currently only powers of 2 work)
                        let res = self.storage_driver.write_app_data(write_buffer, offset);
                        match res {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e),
                        }
                    })
            })
            .unwrap_or_else(|err| Err(err.into()))
    }
}

impl kernel::dynamic_process_loading::DynamicBinaryFlashingClient for AppLoader<'_> {
    /// Let the requesting app know we are done setting up for the new app
    fn setup_done(&self) {
        // Switch on which user of this capsule generated this callback.
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |_app, kernel_data| {
                // Signal the app.
                kernel_data
                    .schedule_upcall(upcall::SETUP_DONE, (0, 0, 0))
                    .ok();
            });
        });
    }

    /// Let the app know we are done writing the block of data
    fn write_app_data_done(&self, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |_app, kernel_data| {
                // Replace the buffer we used to do this write.
                self.buffer.replace(buffer);

                // And then signal the app.
                kernel_data
                    .schedule_upcall(upcall::WRITE_DONE, (length, 0, 0))
                    .ok();
            });
        });
    }
}

impl kernel::dynamic_process_loading::DynamicProcessLoadingClient for AppLoader<'_> {
    /// Let the requesting app know we are done loading the new process
    fn load_done(&self) {
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |_app, kernel_data| {
                // Signal the app.
                kernel_data
                    .schedule_upcall(upcall::LOAD_DONE, (0, 0, 0))
                    .ok();
            });
        });
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
        // Check if this driver is free, or already dedicated to this process.
        let match_or_nonexistent = self.current_process.map_or(true, |current_process| {
            self.apps
                .enter(current_process, |_, _| current_process == processid)
                .unwrap_or(true)
        });
        if match_or_nonexistent {
            self.current_process.set(processid);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            0 => CommandReturn::success(),

            1 => {
                //setup phase
                let res = self.storage_driver.setup(arg1); // pass the size of the app to the setup function
                match res {
                    Ok((app_len, setup_done)) => {
                        // schedule the upcall here so the userspace always has to wait for the
                        // setup done yield wait

                        self.new_app_length.set(app_len);
                        if setup_done {
                            self.current_process.map(|processid| {
                                let _ = self.apps.enter(processid, move |_app, kernel_data| {
                                    // Signal the app.
                                    kernel_data
                                        .schedule_upcall(upcall::SETUP_DONE, (0, 0, 0))
                                        .ok();
                                });
                            });
                            CommandReturn::success()
                        } else {
                            // the setup done upcall is scheduled when the setup_done() function is
                            // called from the DynamicProcessLoader
                            CommandReturn::success()
                        }
                    }
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }

            2 => {
                // Request kernel to write app to flash

                let res = self.write(arg1, arg2, processid);
                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }

            3 => {
                // Request kernel to load the new app

                let res = self.loading_driver.load();
                match res {
                    Ok(()) => {
                        self.new_app_length.set(0); // reset the app length
                        self.current_process.take();
                        CommandReturn::success()
                    }
                    Err(e) => {
                        self.new_app_length.set(0); // reset the app length
                        self.current_process.take();
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
