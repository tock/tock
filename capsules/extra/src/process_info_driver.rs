// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Allow userspace to inspect the list of processes on the board.

use kernel::capabilities::ProcessManagementCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::process;
use kernel::processbuffer::WriteableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::Kernel;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::ProcessInfo as usize;

mod rw_allow {
    pub const INFO: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub struct ProcessInfo<C: ProcessManagementCapability> {
    apps: Grant<(), UpcallCount<0>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,

    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,

    capability: C,
}

impl<C: ProcessManagementCapability> ProcessInfo<C> {
    pub fn new(
        kernel: &'static Kernel,
        grant: Grant<(), UpcallCount<0>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
        capability: C,
    ) -> Self {
        Self {
            kernel,
            apps: grant,
            capability,
        }
    }

    fn iterate_u32<F>(&self, process_id: ProcessId, func: F) -> u32
    where
        F: Fn(&dyn kernel::process::Process) -> u32,
    {
        let mut count = 0;
        let _ = self.apps.enter(process_id, |_app, kernel_data| {
            let _ = kernel_data
                .get_readwrite_processbuffer(rw_allow::INFO)
                .and_then(|shared| {
                    shared.mut_enter(|s| {
                        let mut chunks = s.chunks(size_of::<u32>());

                        self.kernel
                            .process_each_capability(&self.capability, |process| {
                                // Get the next chunk to write the next
                                // PID into.
                                if let Some(chunk) = chunks.nth(0) {
                                    let _ =
                                        chunk.copy_from_slice_or_err(&func(process).to_le_bytes());
                                }
                                count += 1;
                            });
                    })
                });
        });
        count as u32
    }
}

impl<C: ProcessManagementCapability> SyscallDriver for ProcessInfo<C> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Driver existence check
            0 => CommandReturn::success(),

            1 => {
                let mut count = 0;

                self.kernel
                    .process_each_capability(&self.capability, |_process| {
                        count += 1;
                    });

                CommandReturn::success_u32(count)
            }

            2 => {
                let count = self.iterate_u32(process_id, |process| process.processid().id() as u32);
                CommandReturn::success_u32(count)
            }

            3 => {
                let count = self.iterate_u32(process_id, |process| match process.short_app_id() {
                    kernel::process::ShortId::LocallyUnique => 0,
                    kernel::process::ShortId::Fixed(id) => id.into(),
                });
                CommandReturn::success_u32(count)
            }

            4 => {
                let _ = self.apps.enter(process_id, |_app, kernel_data| {
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::INFO)
                        .and_then(|shared| {
                            shared.mut_enter(|s| {
                                self.kernel
                                    .process_each_capability(&self.capability, |process| {
                                        if process.processid().id() == data1 {
                                            let n = process.get_process_name().as_bytes();
                                            let _ = s[0..n.len()].copy_from_slice_or_err(n);
                                            s[n.len()].set(0);
                                        }
                                    });
                            })
                        });
                });
                CommandReturn::success()
            }

            5 => {
                let _ = self.apps.enter(process_id, |_app, kernel_data| {
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::INFO)
                        .and_then(|shared| {
                            shared.mut_enter(|s| {
                                let mut chunks = s.chunks(size_of::<u32>());
                                self.kernel
                                    .process_each_capability(&self.capability, |process| {
                                        if process.processid().id() == data1 {
                                            if let Some(chunk) = chunks.nth(0) {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process
                                                        .debug_timeslice_expiration_count()
                                                        .to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.nth(0) {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process.debug_syscall_count().to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.nth(0) {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process.get_restart_count().to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.nth(0) {
                                                let process_state_id: u32 =
                                                    match process.get_state() {
                                                        process::State::Running => 0,
                                                        process::State::Yielded => 1,
                                                        process::State::YieldedFor(_) => 2,
                                                        process::State::Stopped(_) => 3,
                                                        process::State::Faulted => 4,
                                                        process::State::Terminated => 5,
                                                    };

                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process_state_id.to_le_bytes(),
                                                );
                                            }
                                        }
                                    });
                            })
                        });
                });
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
