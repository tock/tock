// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Data structure for storing an upcall from the kernel to a process.

use crate::config;
use crate::debug;
use crate::process;
use crate::process::ProcessId;
use crate::syscall::SyscallReturn;
use crate::utilities::capability_ptr::CapabilityPtr;
use crate::utilities::machine_register::MachineRegister;
use crate::ErrorCode;

/// Type to uniquely identify an upcall subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct UpcallId {
    /// The [`SyscallDriver`](crate::syscall_driver::SyscallDriver)
    /// implementation this upcall corresponds to.
    pub driver_num: usize,
    /// The subscription index the upcall corresponds to. Subscribe numbers
    /// start at 0 and increment for each upcall defined for a particular
    /// [`SyscallDriver`](crate::syscall_driver::SyscallDriver).
    pub subscribe_num: usize,
}

/// Errors which can occur when scheduling a process Upcall.
///
/// Scheduling a null-Upcall (which will not be delivered to a process) is
/// deliberately not an error, given that a null-Upcall is a well-defined Upcall
/// to be set by a process. It behaves essentially the same as if the process
/// would set a proper Upcall, and would ignore all invocations, with the
/// benefit that no task is inserted in the process' task queue.
#[derive(Copy, Clone, Debug)]
pub enum UpcallError {
    /// The passed `subscribe_num` exceeds the number of Upcalls available for
    /// this process.
    ///
    /// For a [`Grant`](crate::grant::Grant) with `n` upcalls, this error is
    /// returned when
    /// [`GrantKernelData::schedule_upcall`](crate::grant::GrantKernelData::schedule_upcall)
    /// is invoked with `subscribe_num >= n`.
    ///
    /// No Upcall has been scheduled, the call to
    /// [`GrantKernelData::schedule_upcall`](crate::grant::GrantKernelData::schedule_upcall)
    /// had no observable effects.
    ///
    InvalidSubscribeNum,
    /// The process' task queue is full.
    ///
    /// This error can occur when too many tasks (for example, Upcalls) have
    /// been scheduled for a process, without that process yielding or having a
    /// chance to resume execution.
    ///
    /// No Upcall has been scheduled, the call to
    /// [`GrantKernelData::schedule_upcall`](crate::grant::GrantKernelData::schedule_upcall)
    /// had no observable effects.
    QueueFull,
    /// A kernel-internal invariant has been violated.
    ///
    /// This error should never happen. It can be returned if the process is
    /// inactive (which should be caught by
    /// [`Grant::enter`](crate::grant::Grant::enter)) or `process.tasks` was
    /// taken.
    ///
    /// These cases cannot be reasonably handled.
    KernelError,
}

/// Type for calling an upcall in a process.
///
/// This is essentially a wrapper around a function pointer with associated
/// process data.
pub(crate) struct Upcall {
    /// The [`ProcessId`] of the process this upcall is for.
    pub(crate) process_id: ProcessId,

    /// A unique identifier of this particular upcall, representing the
    /// driver_num and subdriver_num used to submit it.
    pub(crate) upcall_id: UpcallId,

    /// The application data passed by the app when `subscribe()` was called.
    pub(crate) appdata: MachineRegister,

    /// A pointer to the first instruction of the function in the app that
    /// corresponds to this upcall.
    ///
    /// If this value is `null`, it should not actually be
    /// scheduled. An `Upcall` can be null when it is first created,
    /// or after an app unsubscribes from an upcall.
    pub(crate) fn_ptr: CapabilityPtr,
}

impl Upcall {
    pub(crate) fn new(
        process_id: ProcessId,
        upcall_id: UpcallId,
        appdata: MachineRegister,
        fn_ptr: CapabilityPtr,
    ) -> Upcall {
        Upcall {
            process_id,
            upcall_id,
            appdata,
            fn_ptr,
        }
    }

    /// Schedule the upcall.
    ///
    /// This will queue the [`Upcall`] for the given process. It returns `false`
    /// if the queue for the process is full and the upcall could not be
    /// scheduled or this is a null upcall.
    ///
    /// The arguments (`r0-r2`) are the values passed back to the process and
    /// are specific to the individual `Driver` interfaces.
    ///
    /// This function also takes `process` as a parameter (even though we have
    /// `process_id` in our struct) to avoid a search through the processes
    /// array to schedule the upcall. Currently, it is convenient to pass this
    /// parameter so we take advantage of it. If in the future that is not the
    /// case we could have `process` be an Option and just do the search with
    /// the stored [`ProcessId`].
    pub(crate) fn schedule(
        &self,
        process: &dyn process::Process,
        r0: usize,
        r1: usize,
        r2: usize,
    ) -> Result<(), UpcallError> {
        let enqueue_res = self.fn_ptr.map_or_else(
            || {
                process.enqueue_task(process::Task::ReturnValue(process::ReturnArguments {
                    upcall_id: self.upcall_id,
                    argument0: r0,
                    argument1: r1,
                    argument2: r2,
                }))
            },
            |fp| {
                process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                    source: process::FunctionCallSource::Driver(self.upcall_id),
                    argument0: r0,
                    argument1: r1,
                    argument2: r2,
                    argument3: self.appdata,
                    pc: *fp,
                }))
            },
        );

        let res = match enqueue_res {
            Ok(()) => Ok(()),
            Err(ErrorCode::NODEVICE) => {
                // There should be no code path to schedule an Upcall on a
                // process that is no longer alive. Indicate a kernel-internal
                // error.
                Err(UpcallError::KernelError)
            }
            Err(ErrorCode::NOMEM) => {
                // No space left in the process' task queue.
                Err(UpcallError::QueueFull)
            }
            Err(_) => {
                // All other errors returned by `Process::enqueue_task` must be
                // treated as kernel-internal errors
                Err(UpcallError::KernelError)
            }
        };

        if config::CONFIG.trace_syscalls {
            debug!(
                "[{:?}] schedule[{:#x}:{}] @{:#x}({:#x}, {:#x}, {:#x}, {:#x}) = {:?}",
                self.process_id,
                self.upcall_id.driver_num,
                self.upcall_id.subscribe_num,
                self.fn_ptr.map_or(core::ptr::null_mut::<()>(), |fp| fp
                    .as_ptr::<()>()
                    .cast_mut()) as usize,
                r0,
                r1,
                r2,
                self.appdata,
                res
            );
        }
        res
    }

    /// Create a successful syscall return type suitable for returning to
    /// userspace.
    ///
    /// This function is intended to be called on the "old upcall" that is being
    /// returned to userspace after a successful subscribe call and upcall swap.
    ///
    /// We provide this `.into` function because the return type needs to
    /// include the function pointer of the upcall.
    pub(crate) fn into_subscribe_success(self) -> SyscallReturn {
        self.fn_ptr.map_or(
            SyscallReturn::SubscribeSuccess(core::ptr::null::<()>(), self.appdata.as_usize()),
            |fp| SyscallReturn::SubscribeSuccess(fp.as_ptr(), self.appdata.as_usize()),
        )
    }

    /// Create a failure case syscall return type suitable for returning to
    /// userspace.
    ///
    /// This is intended to be used when a subscribe call cannot be handled and
    /// the function pointer passed from userspace must be returned back to
    /// userspace.
    ///
    /// We provide this `.into` function because the return type needs to
    /// include the function pointer of the upcall.
    pub(crate) fn into_subscribe_failure(self, err: ErrorCode) -> SyscallReturn {
        self.fn_ptr.map_or(
            SyscallReturn::SubscribeFailure(err, core::ptr::null::<()>(), self.appdata.as_usize()),
            |fp| SyscallReturn::SubscribeFailure(err, fp.as_ptr(), self.appdata.as_usize()),
        )
    }
}
