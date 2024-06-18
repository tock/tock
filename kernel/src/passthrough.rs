// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! The userspace implementation of device passthrough
//!
//! This currently supports a single device.

use crate::capabilities::MemoryAllocationCapability;
use crate::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use crate::hil::passthrough;
use crate::kernel::Kernel;
use crate::platform::DevicePassthroughFilter;
use crate::process::ProcessId;
use crate::syscall_driver::{CommandReturn, SyscallDriver};
use crate::utilities::cells::OptionalCell;
use crate::ErrorCode;

/// Syscall number
pub const DRIVER_NUM: usize = 0x10001;

#[derive(Default)]
struct PassThroughData;

/// The Device Passthrough struct.
pub struct PassThrough<'a, F: DevicePassthroughFilter> {
    /// The grant regions for each process that holds the per-process PassThrough data.
    data: Grant<PassThroughData, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    resources: OptionalCell<&'a F>,
    app: OptionalCell<ProcessId>,
}

impl<'a, F: DevicePassthroughFilter> PassThrough<'a, F> {
    pub fn new(
        kernel: &'static Kernel,
        driver_num: usize,
        capability: &dyn MemoryAllocationCapability,
    ) -> Self {
        Self {
            data: kernel.create_grant(driver_num, capability),
            resources: OptionalCell::empty(),
            app: OptionalCell::empty(),
        }
    }

    pub fn set_resources(&self, resources: &'a F) {
        self.resources.set(resources);
    }
}

impl<'a, F: DevicePassthroughFilter> passthrough::Client for PassThrough<'a, F> {
    fn interrupt_occurred(&self, intstat: usize) {
        self.app.take().map(|processid| {
            self.data
                .enter(processid, |_app, kernel_data| {
                    kernel_data.schedule_upcall(0, (intstat, 0, 0)).unwrap();
                })
                .unwrap();
            self.app.set(processid);
        });
    }
}

impl<'a, F: DevicePassthroughFilter> SyscallDriver for PassThrough<'a, F> {
    fn command(
        &self,
        command_number: usize,
        address: usize,
        size: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            // Query memory information?
            1 => {
                todo!()
            }
            // Allocate
            2 => {
                if self.app.is_some() {
                    // We only support a single application
                    return CommandReturn::failure(ErrorCode::BUSY);
                }

                if let Some(resources) = self.resources.take() {
                    if self
                        .data
                        .kernel
                        .allocate_device_passthrough(resources, &processid, address, size)
                        .is_ok()
                    {
                        self.app.set(processid);
                        self.resources.set(resources);
                        CommandReturn::success()
                    } else {
                        self.resources.set(resources);
                        CommandReturn::failure(ErrorCode::NOSUPPORT)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }
            // Deallocate
            3 => {
                todo!()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), crate::process::Error> {
        self.data.enter(processid, |_, _| {})
    }
}
