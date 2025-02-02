// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Dynamic Process Loader for application loading and updating at runtime.
//!
//! These functions facilitate dynamic application flashing and process creation
//! during runtime without requiring the user to restart the device.

use crate::config;
use crate::debug;
use crate::process;
use crate::process::{ProcessLoadingAsync, ProcessLoadingAsyncClient};
use crate::process_binary::ProcessBinaryError;
use crate::process_loading::ProcessLoadError;
use crate::utilities::cells::{MapCell, OptionalCell};
use crate::ErrorCode;

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoading {
    /// Call to request kernel to load a new process.
    fn load(&self) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the `load_done()`
    /// function.
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadingClient);

    /// Check if the app we have finished writing is valid.
    fn check_new_binary_validity(&self) -> Result<(), ErrorCode>;
}

/// The callback for dynamic process loading.
pub trait DynamicProcessLoadingClient {
    /// The new app has been loaded.
    fn load_done(&self);
}

/// Dynamic process loading machine.
pub struct DynamicProcessLoader<'a> {
    processes: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    loader_driver: &'a dyn ProcessLoadingAsync<'a>,
    load_client: OptionalCell<&'static dyn DynamicProcessLoadingClient>,
}

impl<'a> DynamicProcessLoader<'a> {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        loader_driver: &'a dyn ProcessLoadingAsync<'a>,
    ) -> Self {
        Self {
            processes: MapCell::new(processes),
            loader_driver,
            load_client: OptionalCell::empty(),
        }
    }
}

/// Callback client for the async process loader
impl ProcessLoadingAsyncClient for DynamicProcessLoader<'_> {
    fn process_loaded(&self, result: Result<(), ProcessLoadError>) {
        match result {
            Ok(()) => {
                self.load_client.map(|client| {
                    client.load_done();
                });
            }
            Err(_e) => {
                if config::CONFIG.debug_load_processes {
                    debug!("Load Failed.");
                }
            }
        }
    }

    fn process_loading_finished(&self) {
        if config::CONFIG.debug_load_processes {
            debug!("Processes Loaded:");
            self.processes.map(|procs| {
                for (i, proc) in procs.iter().enumerate() {
                    proc.map(|p| {
                        debug!("[{}] {}", i, p.get_process_name());
                        debug!("    ShortId: {}", p.short_app_id());
                    });
                }
            });
        }
    }
}

/// Loading interface exposed to the app_loader capsule
impl DynamicProcessLoading for DynamicProcessLoader<'_> {
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadingClient) {
        self.load_client.set(client);
    }

    fn check_new_binary_validity(&self) -> Result<(), ErrorCode> {
        // we've written a prepad header if required, so now we check
        // if the app we've written is valid

        let _ = match self.loader_driver.check_new_binary_validity() {
            Ok(()) => Ok::<(), ProcessBinaryError>(()),
            Err(_e) => {
                return Err(ErrorCode::FAIL);
            }
        };
        Ok(())
    }

    fn load(&self) -> Result<(), ErrorCode> {
        // We have finished writing the last user data segment, next step is to
        // load the process.
        let _ = match self.loader_driver.load_new_applications() {
            Ok(()) => Ok::<(), ProcessBinaryError>(()),
            Err(_e) => return Err(ErrorCode::FAIL),
        };
        Ok(())
    }
}
