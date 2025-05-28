// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Allow userspace to inspect and control processes on the board.
//!
//! ## Warning!
//!
//! This capsule is designed for testing and experimental use cases only. It
//! should not be used in production! It was originally written for use in an
//! educational tutorial to make it easy to interact with processes stored on
//! the board from userspace using a screen.
//!
//! This capsule does require a capability to also indicate that this interacts
//! with processes in a way that common capsules should not.
//!
//! ## Commands
//!
//! - 0: Check driver exists.
//! - 1: Get the count of processes running on the board.
//! - 2: Fill the allow RW buffer with the process IDs for the running processes
//!   on the board. Returns the number of running processes.
//! - 3: Fill the allow RW buffer with the short IDs for the running processes
//!   on the board. Returns the number of running processes.
//! - 4: Put the name of the process specified by the process ID in `data1` in
//!   the allow RW buffer (as much as will fit). Returns the full length of the
//!   process name.
//! - 5: Fill the allow RW buffer with the following information for the process
//!   specified by the process ID in `data1`.
//!   - The number of timeslice expirations.
//!   - The number of syscalls called.
//!   - The number of restarts.
//!   - The current process state (running=0, yielded=1, yieldedfor=2,
//!     stopped=3, faulted=4, terminated=5).
//! - 6: Change the process state. `data1` is the process ID, and `data2` is the
//!   new state.(1=start, 2=stop, 3=fault, 4=terminate, 5=boot).

use kernel::capabilities::{ProcessManagementCapability, ProcessStartCapability};
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

pub struct ProcessInfo<C: ProcessManagementCapability + ProcessStartCapability> {
    apps: Grant<(), UpcallCount<0>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,
    /// Capability needed to interact with and control processes.
    capability: C,
}

impl<C: ProcessManagementCapability + ProcessStartCapability> ProcessInfo<C> {
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
                                if let Some(chunk) = chunks.next() {
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

impl<C: ProcessManagementCapability + ProcessStartCapability> SyscallDriver for ProcessInfo<C> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
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

            4 => self
                .apps
                .enter(process_id, |_app, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::INFO)
                        .and_then(|shared| {
                            shared.mut_enter(|s| {
                                // We need to differentiate between no matching
                                // apps (based on ProcessId) and an app with a
                                // 0 length name.
                                let mut matched_name_len: Option<usize> = None;

                                self.kernel
                                    .process_each_capability(&self.capability, |process| {
                                        if process.processid().id() == data1 {
                                            let n = process.get_process_name().as_bytes();

                                            let name_len = n.len();
                                            let buffer_len = s.len();
                                            let copy_len = core::cmp::min(name_len, buffer_len);

                                            // Copy as much as we can into the
                                            // allowed buffer.
                                            s.get(0..copy_len).map(|dest| {
                                                n.get(0..copy_len).map(|src| {
                                                    let _ = dest.copy_from_slice_or_err(src);
                                                });
                                            });

                                            // Return that we did find a
                                            // matching app with a name of a
                                            // specific length.
                                            matched_name_len = Some(name_len);
                                        }
                                    });
                                if let Some(nlen) = matched_name_len {
                                    CommandReturn::success_u32(nlen as u32)
                                } else {
                                    CommandReturn::failure(ErrorCode::INVAL)
                                }
                            })
                        })
                        .unwrap_or_else(|err| CommandReturn::failure(err.into()))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            5 => self
                .apps
                .enter(process_id, |_app, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::INFO)
                        .and_then(|shared| {
                            shared.mut_enter(|s| {
                                let mut chunks = s.chunks(size_of::<u32>());
                                self.kernel
                                    .process_each_capability(&self.capability, |process| {
                                        if process.processid().id() == data1 {
                                            if let Some(chunk) = chunks.next() {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process
                                                        .debug_timeslice_expiration_count()
                                                        .to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.next() {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process.debug_syscall_count().to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.next() {
                                                let _ = chunk.copy_from_slice_or_err(
                                                    &process.get_restart_count().to_le_bytes(),
                                                );
                                            }
                                            if let Some(chunk) = chunks.next() {
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
                                CommandReturn::success()
                            })
                        })
                        .unwrap_or_else(|err| CommandReturn::failure(err.into()))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            6 => {
                let mut matched = false;
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process.processid().id() == data1 {
                            matched = true;

                            match data2 {
                                1 => {
                                    // START
                                    process.resume();
                                }
                                2 => {
                                    // STOP
                                    process.stop();
                                }

                                3 => {
                                    // FAULT
                                    process.set_fault_state();
                                }

                                4 => {
                                    // TERMINATE
                                    process.terminate(None);
                                }

                                5 => {
                                    // BOOT
                                    if process.get_state() == process::State::Terminated {
                                        process.start(&self.capability);
                                    }
                                }

                                _ => {}
                            }
                        }
                    });
                if matched {
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
