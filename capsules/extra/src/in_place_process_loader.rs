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
//! +----------------------------------------------------------------------+
//! |                                                                      |
//! |                              userspace                               |
//! |                                                                      |
//! +----------------------------------------------------------------------+
//!                          kernel::SyscallDriver
//! +----------------------------------------------------------------------+
//! |                                                                      |
//! |capsules_extra::in_place_process_loading::InPlaceProcessLoading (this)|
//! |                                                                      |
//! +----------------------------------------------------------------------+
//!         kernel::dynamic_process_loading::DynamicProcessLoading
//! +----------------------------------------------------------------------+
//! |                                                                      |
//! |                               Kernel                                 |
//! |                                                                      |
//! +----------------------------------------------------------------------+
//! ```
//!
//! Example instantiation:
//!
//! ```rust, ignore
//! # use kernel::static_init;
//!
//! let in_place_process_loader = components::in_place_process_loader::InPlaceProcessLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::in_place_process_loader::DRIVER_NUM,
//!     dynamic_process_loader,
//!     ).finalize(components::in_place_process_loader_component_static!());
//! ``

use kernel::dynamic_process_loading;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::InPlaceProcessLoader as usize;

/// IDs for subscribed upcalls.
mod upcall {
    /// Load done callback.
    pub const LOAD_DONE: usize = 0;
    /// Number of upcalls.
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App {}

pub struct InPlaceProcessLoader<'a> {
    // The underlying driver for the process flashing and loading.
    loading_driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
    // Per-app state.
    apps: Grant<App, UpcallCount<{ upcall::COUNT }>, AllowRoCount<0>, AllowRwCount<0>>,
    // What issued the currently executing call.
    current_process: OptionalCell<ProcessId>,
}

impl<'a> InPlaceProcessLoader<'a> {
    pub fn new(
        grant: Grant<App, UpcallCount<{ upcall::COUNT }>, AllowRoCount<0>, AllowRwCount<0>>,
        loading_driver: &'a dyn dynamic_process_loading::DynamicProcessLoading,
    ) -> InPlaceProcessLoader<'a> {
        InPlaceProcessLoader {
            apps: grant,
            loading_driver,
            current_process: OptionalCell::empty(),
        }
    }
}

impl kernel::dynamic_process_loading::DynamicProcessLoadingClient for InPlaceProcessLoader<'_> {
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
impl SyscallDriver for InPlaceProcessLoader<'_> {
    /// Command interface.
    ///
    /// Commands are selected by the lowest 8 bits of the first argument.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Request kernel to load app.
    ///        - Returns Ok(()) when the process is successfully loaded
    ///        - Returns ErrorCode::FAIL if:
    ///            - The kernel is unable to create a process object for the application
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
                // Request kernel to load the new app
                let res = self.loading_driver.load(Some((arg1, arg2)));
                match res {
                    Ok(()) => {
                        self.current_process.take();
                        CommandReturn::success()
                    }
                    Err(e) => {
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
