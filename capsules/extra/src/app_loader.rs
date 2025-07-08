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
//!         kernel::dynamic_binary_storage::DynamicBinaryStore
//!         kernel::dynamic_binary_storage::DynamicProcessLoad
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
//! type NonVolatilePages = components::dynamic_binary_storage::NVPages<nrf52840::nvmc::Nvmc>;
//! type DynamicBinaryStorage<'a> = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
//! 'static,
//! nrf52840::chip::NRF52<'a, Nrf52840DefaultPeripherals<'a>>,
//! kernel::process::ProcessStandardDebugFull,
//! NonVolatilePages,
//! >;
//!
//! let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::app_loader::DRIVER_NUM,
//!     dynamic_binary_storage,
//!     dynamic_binary_storage,
//!     ).finalize(components::app_loader_component_static!(
//!     DynamicBinaryStorage<'static>,
//!     DynamicBinaryStorage<'static>,
//!     ));
//!
//! NOTE:
//! 1. This capsule is not virtualized, and can only serve one app at a time.
//! 2. This implementation currently only loads new apps. It does not update apps.
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::dynamic_binary_storage;
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::process::ProcessLoadError;
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
    /// Finalize done callback.
    pub const FINALIZE_DONE: usize = 2;
    /// Load done callback.
    pub const LOAD_DONE: usize = 3;
    /// Abort done callback.
    pub const ABORT_DONE: usize = 4;
    /// Number of upcalls.
    pub const COUNT: u8 = 5;
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
pub struct App {
    pending_command: bool,
}

pub struct AppLoader<
    S: dynamic_binary_storage::DynamicBinaryStore + 'static,
    L: dynamic_binary_storage::DynamicProcessLoad + 'static,
> {
    // The underlying driver for the process flashing and loading.
    storage_driver: &'static S,
    load_driver: &'static L,
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

impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > AppLoader<S, L>
{
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<0>,
        >,
        storage_driver: &'static S,
        load_driver: &'static L,
        buffer: &'static mut [u8],
    ) -> AppLoader<S, L> {
        AppLoader {
            apps: grant,
            storage_driver,
            load_driver,
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
                if result > self.new_app_length.get() {
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
                let mut active_len = 0;

                let result = kernel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .and_then(|write| {
                        write.enter(|app_buffer| {
                            self.buffer
                                .map(|kernel_buffer| {
                                    // Get the length of the allowed buffer
                                    let allow_buf_len = app_buffer.len();

                                    // Check that the buffer length is not zero
                                    if allow_buf_len == 0 {
                                        return Err(ErrorCode::RESERVE);
                                    }

                                    // Shorten the length if the application did not give us
                                    // enough bytes in the allowed buffer.
                                    active_len = cmp::min(length, allow_buf_len);

                                    // copy data into the kernel buffer!
                                    let write_len = cmp::min(active_len, kernel_buffer.len());
                                    let () = app_buffer[..write_len]
                                        .copy_to_slice(&mut kernel_buffer[..write_len]);

                                    Ok(())
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))
                        })
                    });

                if result.is_err() {
                    return Err(ErrorCode::RESERVE);
                }

                self.buffer
                    .take()
                    .map_or(Err(ErrorCode::RESERVE), |buffer| {
                        let mut write_buffer = SubSliceMut::new(buffer);
                        // should be the length supported by the app
                        // (currently only powers of 2 work)
                        write_buffer.slice(..length);
                        let res = self.storage_driver.write(write_buffer, offset);
                        match res {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e),
                        }
                    })
            })
            .unwrap_or_else(|err| Err(err.into()))
    }
}

impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > dynamic_binary_storage::DynamicBinaryStoreClient for AppLoader<S, L>
{
    /// Let the requesting app know we are done setting up for the new app
    fn setup_done(&self, result: Result<(), ErrorCode>) {
        // Switch on which user of this capsule generated this callback.
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |app, kernel_data| {
                app.pending_command = false;
                // Signal the app.
                kernel_data
                    .schedule_upcall(upcall::SETUP_DONE, (into_statuscode(result), 0, 0))
                    .ok();
            });
        });
    }

    /// Let the app know we are done writing the block of data
    fn write_done(&self, result: Result<(), ErrorCode>, buffer: &'static mut [u8], length: usize) {
        // Switch on which user of this capsule generated this callback.
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |app, kernel_data| {
                // Replace the buffer we used to do this write.
                self.buffer.replace(buffer);
                app.pending_command = false;

                // And then signal the app.
                kernel_data
                    .schedule_upcall(upcall::WRITE_DONE, (into_statuscode(result), length, 0))
                    .ok();
            });
        });
    }

    /// Let the app know we are done finalizing, and are ready to load
    fn finalize_done(&self, result: Result<(), ErrorCode>) {
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |app, kernel_data| {
                // And then signal the app.
                app.pending_command = false;

                self.current_process.take();
                kernel_data
                    .schedule_upcall(upcall::FINALIZE_DONE, (into_statuscode(result), 0, 0))
                    .ok();
            });
        });
    }

    /// Let the app know we have aborted the new app writing process
    fn abort_done(&self, result: Result<(), ErrorCode>) {
        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |app, kernel_data| {
                // And then signal the app.
                app.pending_command = false;

                self.current_process.take();
                kernel_data
                    .schedule_upcall(upcall::ABORT_DONE, (into_statuscode(result), 0, 0))
                    .ok();
            });
        });
    }
}

impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > dynamic_binary_storage::DynamicProcessLoadClient for AppLoader<S, L>
{
    /// Let the requesting app know we are done loading the new process
    ///
    /// Error Type Mapping.
    ///
    /// This method converts `ProcessLoadError` to `ErrorCode` so it can be
    /// passed to userspace.
    ///
    /// Currently,
    /// 1. ProcessLoadError::NotEnoughMemory       <==> ErrorCode::NOMEM
    /// 2. ProcessLoadError::MpuInvalidFlashLength <==> ErrorCode::INVAL
    /// 3. ProcessLoadError::InternalError         <==> ErrorCode::OFF
    /// 4. All other ProcessLoadError types        <==> ErrorCode::FAIL

    fn load_done(&self, result: Result<(), ProcessLoadError>) {
        let status_code = match result {
            Ok(()) => Ok(()),
            Err(e) => match e {
                ProcessLoadError::NotEnoughMemory => Err(ErrorCode::NOMEM),
                ProcessLoadError::MpuInvalidFlashLength => Err(ErrorCode::INVAL),
                ProcessLoadError::MpuConfigurationError => Err(ErrorCode::FAIL),
                ProcessLoadError::MemoryAddressMismatch { .. } => Err(ErrorCode::FAIL),
                ProcessLoadError::NoProcessSlot => Err(ErrorCode::FAIL),
                ProcessLoadError::BinaryError(_) => Err(ErrorCode::FAIL),
                ProcessLoadError::CheckError(_) => Err(ErrorCode::FAIL),
                // This error is usually a result of bug in the kernel
                // so we return Powered OFF error, because that is unlikely.
                ProcessLoadError::InternalError => Err(ErrorCode::OFF),
            },
        };

        self.current_process.map(|processid| {
            let _ = self.apps.enter(processid, move |app, kernel_data| {
                app.pending_command = false;
                // Signal the app.
                self.current_process.take();
                kernel_data
                    .schedule_upcall(upcall::LOAD_DONE, (into_statuscode(status_code), 0, 0))
                    .ok();
            });
        });
    }
}

/// Provide an interface for userland.
impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > SyscallDriver for AppLoader<S, L>
{
    /// Command interface.
    ///
    /// The driver returns ErrorCode::BUSY if:
    ///    - The kernel has already dedicated this driver to another process.
    ///    - The kernel is busy executing another command for this process.
    ///
    /// Currently, this capsule is not virtualized and can only be used by one
    /// application at a time.
    ///
    /// Commands are selected by the lowest 8 bits of the first argument.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Request kernel to setup for loading app.
    ///  - Returns appsize if the kernel has available space
    ///  - Returns ErrorCode::FAIL if the kernel is unable to allocate space for the new app
    /// - `2`: Request kernel to write app data to the nonvolatile_storage
    ///  - Returns Ok(()) when write is successful
    ///  - Returns ErrorCode::INVAL when the app is violating bounds
    ///  - Returns ErrorCode::FAIL when the write fails
    /// - `3`: Signal to the kernel that the writing is done.
    ///  - Returns Ok(()) if the kernel successfully verified it and
    ///  set the stage for `load()`.
    ///  - Returns ErrorCode::FAIL if:
    ///  a. The kernel needs to write a leading padding app but is unable to.
    ///  b. The command is called during setup or load phases.
    /// - `4`: Request kernel to load app.
    ///  - Returns Ok(()) when the process is successfully loaded
    ///  - Returns ErrorCode::FAIL if:
    ///  a. The kernel is unable to create a process object for the application
    /// - `5`: Request kernel to abort setup/write operation.
    ///  - Returns Ok(()) when the operation is cancelled successfully
    ///  - Returns ErrorCode::BUSY when the abort fails
    ///  (due to padding app being unable to be written, so try again)
    ///  - Returns ErrorCode::FAIL if the driver is not dedicated to this process
    ///
    /// The driver returns ErrorCode::INVAL if any operation is called before the
    /// preceeding operation was invoked. For example, `write()` cannot be called before
    /// `setup()`, and `load()` cannot be called before `write()` (for this implementation).
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
                .unwrap_or_else(|_e| {
                    let _ = self.storage_driver.abort();
                    self.new_app_length.set(0);
                    self.current_process.take();
                    false
                })
        });
        if match_or_nonexistent {
            self.current_process.set(processid);
            let _ = self.apps.enter(processid, |app, _| {
                if app.pending_command {
                    CommandReturn::failure(ErrorCode::BUSY);
                } else {
                    app.pending_command = true;
                }
            });
        } else {
            return CommandReturn::failure(ErrorCode::BUSY);
        }

        match command_num {
            0 => {
                // Remove ownership from the current process so
                // other processes can discover this driver
                self.current_process.take();
                CommandReturn::success()
            }

            1 => {
                // Request kernel to allocate resources for
                // an app with size passed via `arg1`.
                let res = self.storage_driver.setup(arg1);
                match res {
                    Ok(app_len) => {
                        self.new_app_length.set(app_len);
                        CommandReturn::success()
                    }
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }

            2 => {
                // Request kernel to write app to flash.
                let res = self.write(arg1, arg2, processid);
                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => {
                        let command_result = if let Some(buffer) = self.buffer.take() {
                            self.buffer.replace(buffer);
                            self.new_app_length.set(0);
                            self.current_process.take();
                            CommandReturn::failure(e)
                        } else {
                            CommandReturn::failure(ErrorCode::RESERVE)
                        };
                        command_result
                    }
                }
            }

            3 => {
                // Signal to kernel writing is done.
                let result = self.storage_driver.finalize();
                match result {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }

            4 => {
                // Request kernel to load the new app.
                let res = self.load_driver.load();
                match res {
                    Ok(()) => {
                        self.new_app_length.set(0);
                        CommandReturn::success()
                    }
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }

            5 => {
                // Request kernel to abort setup/write operation.
                let result = self.storage_driver.abort();
                match result {
                    Ok(()) => {
                        self.new_app_length.set(0);
                        CommandReturn::success()
                    }
                    Err(e) => {
                        self.new_app_length.set(0);
                        self.current_process.take();
                        CommandReturn::failure(e)
                    }
                }
            }
            // Unsupported command numbers.
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
