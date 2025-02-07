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
use crate::process::{SequentialProcessLoaderMachine, ProcessLoadingAsyncClient};
use crate::process_loading::ProcessLoadError;
use crate::utilities::cells::{MapCell, OptionalCell};
use crate::ErrorCode;
use crate::platform::chip::Chip;
use crate::process_standard::ProcessStandardDebug;

/// This interface supports loading processes at runtime.
pub trait DynamicProcessLoading {
    /// Call to request kernel to load a new process.
    fn load(&self, app_address: usize, app_size: usize) -> Result<(), ErrorCode>;

    /// Sets a client for the DynamicProcessLoading Object
    ///
    /// When the client operation is done, it calls the `load_done()`
    /// function.
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadingClient);
}

/// The callback for dynamic process loading.
pub trait DynamicProcessLoadingClient {
    /// The new app has been loaded.
    fn load_done(&self);
}

/// Dynamic process loading machine.
pub struct DynamicProcessLoader<'a, C: Chip + 'static, D: ProcessStandardDebug + 'static> {
    processes: MapCell<&'static mut [Option<&'static dyn process::Process>]>,
    loader_driver: &'a SequentialProcessLoaderMachine<'a, C, D>,
    load_client: OptionalCell<&'static dyn DynamicProcessLoadingClient>,
}

impl<'a, C: Chip + 'static, D: ProcessStandardDebug + 'static> DynamicProcessLoader<'a, C, D> {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        loader_driver: &'a SequentialProcessLoaderMachine<'a, C, D>,
    ) -> Self {
        Self {
            processes: MapCell::new(processes),
            loader_driver,
            load_client: OptionalCell::empty(),
        }
    }
}

/// Callback client for the async process loader
impl <C: Chip + 'static, D: ProcessStandardDebug + 'static> ProcessLoadingAsyncClient for DynamicProcessLoader<'_, C, D> {
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
impl <C: Chip + 'static, D: ProcessStandardDebug + 'static> DynamicProcessLoading for DynamicProcessLoader<'_, C, D> {
    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadingClient) {
        self.load_client.set(client);
    }

    fn load(&self, app_address: usize, app_size: usize) -> Result<(), ErrorCode> {
        // We have finished writing the last user data segment, next step is to
        // load the process.
        let _ = match self
            .loader_driver
            .load_new_applications(app_address, app_size)
        {
            Ok(()) => Ok::<(), ProcessLoadError>(()),
            Err(_e) => return Err(ErrorCode::FAIL),
        };
        Ok(())
    }
}
