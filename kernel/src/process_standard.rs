// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock default Process implementation.
//!
//! `ProcessStandard` is an implementation for a userspace process running on
//! the Tock kernel.

use core::cell::Cell;
use core::cmp;
use core::fmt::Write;
use core::num::{NonZero, NonZeroU32};
use core::ptr::NonNull;
use core::{mem, ptr, slice, str};

use crate::collections::queue::Queue;
use crate::collections::ring_buffer::RingBuffer;
use crate::config;
use crate::debug;
use crate::errorcode::ErrorCode;
use crate::kernel::Kernel;
use crate::memory_management::configuration;
use crate::memory_management::granules::Granule as GranuleTrait;
use crate::memory_management::pages::Page4KiB;
use crate::memory_management::permissions::Permissions;
use crate::memory_management::pointers::{
    ImmutableKernelVirtualPointer, ImmutableUserVirtualPointer, ImmutableVirtualPointer,
    MutableKernelVirtualPointer, MutablePhysicalPointer, MutableUserVirtualPointer,
};
use crate::memory_management::regions::{
    AllocatedRegion, ProtectedAllocatedRegion, UserMappedProtectedAllocatedRegion,
};
use crate::memory_management::slices::{
    ImmutableKernelVirtualSlice, MutableKernelVirtualSlice, MutablePhysicalSlice,
};
use crate::platform::chip::Chip;
use crate::platform::mmu::MMU;
use crate::process::BinaryVersion;
use crate::process::ProcessBinary;
use crate::process::{Error, FunctionCall, FunctionCallSource, Process, Task};
use crate::process::{FaultAction, ProcessCustomGrantIdentifier, ProcessId};
use crate::process::{ProcessAddresses, ProcessSizes, ShortId};
use crate::process::{State, StoppedState};
use crate::process_checker::AcceptedCredential;
use crate::process_loading::ProcessLoadError;
use crate::process_policies::ProcessFaultPolicy;
use crate::process_policies::ProcessStandardStoragePermissionsPolicy;
use crate::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use crate::storage_permissions::StoragePermissions;
use crate::syscall::{self, Syscall, SyscallReturn, UserspaceKernelBoundary};
use crate::upcall::UpcallId;
use crate::utilities::capability_ptr::{CapabilityPtr, CapabilityPtrPermissions};
use crate::utilities::cells::{MapCell, OptionalCell};
use crate::utilities::misc::{
    align_down_usize, align_up_usize, ceil_non_zero_usize, ceil_usize, create_non_zero_usize,
    divide_exact_non_zero_usize,
};

use tock_tbf::types::CommandPermissions;

/// Interface supported by [`ProcessStandard`] for recording debug information.
///
/// This trait provides flexibility to users of [`ProcessStandard`] to determine
/// how debugging information should be recorded, or if debugging information
/// should be recorded at all.
///
/// Platforms that want to only maintain certain debugging information can
/// implement only part of this trait.
///
/// Tock provides a default implementation of this trait on the `()` type.
/// Kernels that wish to use [`ProcessStandard`] but do not need process-level
/// debugging information can use `()` as the `ProcessStandardDebug` type.
pub trait ProcessStandardDebug: Default {
    /// Record the address in flash the process expects to start at.
    fn set_fixed_address_flash(&self, address: u32);
    /// Get the address in flash the process expects to start at, if it was
    /// recorded.
    fn get_fixed_address_flash(&self) -> Option<u32>;
    /// Record the address in RAM the process expects to start at.
    fn set_fixed_address_ram(&self, address: u32);
    /// Get the address in RAM the process expects to start at, if it was
    /// recorded.
    fn get_fixed_address_ram(&self) -> Option<u32>;
    /// Record the address where the process placed its heap.
    fn set_app_heap_start_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>);
    /// Get the address where the process placed its heap, if it was recorded.
    fn get_app_heap_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>>;
    /// Record the address where the process placed its stack.
    fn set_app_stack_start_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>);
    /// Get the address where the process placed its stack, if it was recorded.
    fn get_app_stack_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>>;
    /// Update the lowest address that the process's stack has reached.
    fn set_app_stack_min_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>);
    /// Get the lowest address of the process's stack , if it was recorded.
    fn get_app_stack_min_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>>;
    /// Provide the current address of the bottom of the stack and record the
    /// address if it is the lowest address that the process's stack has
    /// reached.
    fn set_new_app_stack_min_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>);

    /// Record the most recent system call the process called.
    fn set_last_syscall(&self, syscall: Syscall);
    /// Get the most recent system call the process called, if it was recorded.
    fn get_last_syscall(&self) -> Option<Syscall>;
    /// Clear any record of the most recent system call the process called.
    fn reset_last_syscall(&self);

    /// Increase the recorded count of the number of system calls the process
    /// has called.
    fn increment_syscall_count(&self);
    /// Get the recorded count of the number of system calls the process has
    /// called.
    ///
    /// This should return 0 if
    /// [`ProcessStandardDebug::increment_syscall_count()`] is never called.
    fn get_syscall_count(&self) -> usize;
    /// Reset the recorded count of the number of system calls called by the app
    /// to 0.
    fn reset_syscall_count(&self);

    /// Increase the recorded count of the number of upcalls that have been
    /// dropped for the process.
    fn increment_dropped_upcall_count(&self);
    /// Get the recorded count of the number of upcalls that have been dropped
    /// for the process.
    ///
    /// This should return 0 if
    /// [`ProcessStandardDebug::increment_dropped_upcall_count()`] is never
    /// called.
    fn get_dropped_upcall_count(&self) -> usize;
    /// Reset the recorded count of the number of upcalls that have been dropped
    /// for the process to 0.
    fn reset_dropped_upcall_count(&self);

    /// Increase the recorded count of the number of times the process has
    /// exceeded its timeslice.
    fn increment_timeslice_expiration_count(&self);
    /// Get the recorded count of the number times the process has exceeded its
    /// timeslice.
    ///
    /// This should return 0 if
    /// [`ProcessStandardDebug::increment_timeslice_expiration_count()`] is
    /// never called.
    fn get_timeslice_expiration_count(&self) -> usize;
    /// Reset the recorded count of the number of the process has exceeded its
    /// timeslice to 0.
    fn reset_timeslice_expiration_count(&self);
}

/// A debugging implementation for [`ProcessStandard`] that records the full
/// debugging state.
pub struct ProcessStandardDebugFull {
    /// Inner field for the debug state that is in a [`MapCell`] to provide
    /// mutable access.
    debug: MapCell<ProcessStandardDebugFullInner>,
}

/// Struct for debugging [`ProcessStandard`] processes that records the full set
/// of debugging information.
///
/// These pointers and counters are not strictly required for kernel operation,
/// but provide helpful information when an app crashes.
#[derive(Default)]
struct ProcessStandardDebugFullInner {
    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in flash. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_flash: Option<u32>,

    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in RAM. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_ram: Option<u32>,

    /// Where the process has started its heap in RAM.
    app_heap_start_pointer: Option<ImmutableUserVirtualPointer<u8>>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<ImmutableUserVirtualPointer<u8>>,

    /// How low have we ever seen the stack pointer.
    app_stack_min_pointer: Option<ImmutableUserVirtualPointer<u8>>,

    /// How many syscalls have occurred since the process started.
    syscall_count: usize,

    /// What was the most recent syscall.
    last_syscall: Option<Syscall>,

    /// How many upcalls were dropped because the queue was insufficiently
    /// long.
    dropped_upcall_count: usize,

    /// How many times this process has been paused because it exceeded its
    /// timeslice.
    timeslice_expiration_count: usize,
}

impl ProcessStandardDebug for ProcessStandardDebugFull {
    fn set_fixed_address_flash(&self, address: u32) {
        self.debug.map(|d| d.fixed_address_flash = Some(address));
    }
    fn get_fixed_address_flash(&self) -> Option<u32> {
        self.debug.map_or(None, |d| d.fixed_address_flash)
    }
    fn set_fixed_address_ram(&self, address: u32) {
        self.debug.map(|d| d.fixed_address_ram = Some(address));
    }
    fn get_fixed_address_ram(&self) -> Option<u32> {
        self.debug.map_or(None, |d| d.fixed_address_ram)
    }
    fn set_app_heap_start_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>) {
        self.debug.map(|d| d.app_heap_start_pointer = Some(ptr));
    }
    fn get_app_heap_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        self.debug.map_or(None, |d| d.app_heap_start_pointer)
    }
    fn set_app_stack_start_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>) {
        self.debug.map(|d| d.app_stack_start_pointer = Some(ptr));
    }
    fn get_app_stack_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        self.debug.map_or(None, |d| d.app_stack_start_pointer)
    }
    fn set_app_stack_min_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>) {
        self.debug.map(|d| d.app_stack_min_pointer = Some(ptr));
    }
    fn get_app_stack_min_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        self.debug.map_or(None, |d| d.app_stack_min_pointer)
    }
    fn set_new_app_stack_min_pointer(&self, ptr: ImmutableUserVirtualPointer<u8>) {
        self.debug.map(|d| {
            match d.app_stack_min_pointer {
                None => d.app_stack_min_pointer = Some(ptr),
                Some(asmp) => {
                    // Update max stack depth if needed.
                    if ptr < asmp {
                        d.app_stack_min_pointer = Some(ptr);
                    }
                }
            }
        });
    }

    fn set_last_syscall(&self, syscall: Syscall) {
        self.debug.map(|d| d.last_syscall = Some(syscall));
    }
    fn get_last_syscall(&self) -> Option<Syscall> {
        self.debug.map_or(None, |d| d.last_syscall)
    }
    fn reset_last_syscall(&self) {
        self.debug.map(|d| d.last_syscall = None);
    }

    fn increment_syscall_count(&self) {
        self.debug.map(|d| d.syscall_count += 1);
    }
    fn get_syscall_count(&self) -> usize {
        self.debug.map_or(0, |d| d.syscall_count)
    }
    fn reset_syscall_count(&self) {
        self.debug.map(|d| d.syscall_count = 0);
    }

    fn increment_dropped_upcall_count(&self) {
        self.debug.map(|d| d.dropped_upcall_count += 1);
    }
    fn get_dropped_upcall_count(&self) -> usize {
        self.debug.map_or(0, |d| d.dropped_upcall_count)
    }
    fn reset_dropped_upcall_count(&self) {
        self.debug.map(|d| d.dropped_upcall_count = 0);
    }

    fn increment_timeslice_expiration_count(&self) {
        self.debug.map(|d| d.timeslice_expiration_count += 1);
    }
    fn get_timeslice_expiration_count(&self) -> usize {
        self.debug.map_or(0, |d| d.timeslice_expiration_count)
    }
    fn reset_timeslice_expiration_count(&self) {
        self.debug.map(|d| d.timeslice_expiration_count = 0);
    }
}

impl Default for ProcessStandardDebugFull {
    fn default() -> Self {
        Self {
            debug: MapCell::new(ProcessStandardDebugFullInner::default()),
        }
    }
}

impl ProcessStandardDebug for () {
    fn set_fixed_address_flash(&self, _address: u32) {}
    fn get_fixed_address_flash(&self) -> Option<u32> {
        None
    }
    fn set_fixed_address_ram(&self, _address: u32) {}
    fn get_fixed_address_ram(&self) -> Option<u32> {
        None
    }
    fn set_app_heap_start_pointer(&self, _ptr: ImmutableUserVirtualPointer<u8>) {}
    fn get_app_heap_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        None
    }
    fn set_app_stack_start_pointer(&self, _ptr: ImmutableUserVirtualPointer<u8>) {}
    fn get_app_stack_start_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        None
    }
    fn set_app_stack_min_pointer(&self, _ptr: ImmutableUserVirtualPointer<u8>) {}
    fn get_app_stack_min_pointer(&self) -> Option<ImmutableUserVirtualPointer<u8>> {
        None
    }
    fn set_new_app_stack_min_pointer(&self, _ptr: ImmutableUserVirtualPointer<u8>) {}

    fn set_last_syscall(&self, _syscall: Syscall) {}
    fn get_last_syscall(&self) -> Option<Syscall> {
        None
    }
    fn reset_last_syscall(&self) {}

    fn increment_syscall_count(&self) {}
    fn get_syscall_count(&self) -> usize {
        0
    }
    fn reset_syscall_count(&self) {}
    fn increment_dropped_upcall_count(&self) {}
    fn get_dropped_upcall_count(&self) -> usize {
        0
    }
    fn reset_dropped_upcall_count(&self) {}
    fn increment_timeslice_expiration_count(&self) {}
    fn get_timeslice_expiration_count(&self) -> usize {
        0
    }
    fn reset_timeslice_expiration_count(&self) {}
}

/// Entry that is stored in the grant pointer table at the top of process
/// memory.
///
/// One copy of this entry struct is stored per grant region defined in the
/// kernel. This type allows the core kernel to lookup a grant based on the
/// driver_num associated with the grant, and also holds the pointer to the
/// memory allocated for the particular grant.
#[repr(C)]
struct GrantPointerEntry {
    /// The syscall driver number associated with the allocated grant.
    ///
    /// This defaults to 0 if the grant has not been allocated. Note, however,
    /// that 0 is a valid driver_num, and therefore cannot be used to check if a
    /// grant is allocated or not.
    driver_num: usize,

    /// The start of the memory location where the grant has been allocated, or
    /// null if the grant has not been allocated.
    grant_ptr: *mut u8,
}

/// A type for userspace processes in Tock.
///
/// As its name implies, this is the standard implementation for Tock processes
/// that exposes the full support for processes running on embedded hardware.
///
/// [`ProcessStandard`] is templated on two parameters:
///
/// - `C`: [`Chip`]: The implementation must know the [`Chip`] the kernel is
///   running on to properly store architecture-specific and MPU state for the
///   process.
/// - `D`: [`ProcessStandardDebug`]: This configures the debugging mechanism the
///   process uses for storing optional debugging data. Kernels that do not wish
///   to store per-process debugging state can use the `()` type for this
///   parameter.
pub struct ProcessStandard<'a, C: 'static + Chip, D: 'static + ProcessStandardDebug + Default> {
    /// Identifier of this process and the index of the process in the process
    /// table.
    process_id: Cell<ProcessId>,

    /// An application ShortId, generated from process loading and
    /// checking, which denotes the security identity of this process.
    app_id: ShortId,

    /// Pointer to the main Kernel struct.
    kernel: &'static Kernel,

    /// Pointer to the struct that defines the actual chip the kernel is running
    /// on. This is used because processes have subtle hardware-based
    /// differences. Specifically, the actual syscall interface and how
    /// processes are switched to is architecture-specific, and how memory must
    /// be allocated for memory protection units is also hardware-specific.
    chip: &'static C,

    /// Application memory layout:
    ///
    /// ```text
    ///     ╒════════ ← memory_start + memory_len
    ///  ╔═ │ Grant Pointers
    ///  ║  │ ──────
    ///     │ Process Control Block
    ///  D  │ ──────
    ///  Y  │ Grant Regions
    ///  N  │
    ///  A  │   ↓
    ///  M  │ ──────  ← kernel_memory_break
    ///  I  │
    ///  C  │ ──────  ← app_break               ═╗
    ///     │                                    ║
    ///  ║  │   ↑                                  A
    ///  ║  │  Heap                              P C
    ///  ╠═ │ ──────  ← app_heap_start           R C
    ///     │  Data                              O E
    ///  F  │ ──────  ← data_start_pointer       C S
    ///  I  │ Stack                              E S
    ///  X  │   ↓                                S I
    ///  E  │                                    S B
    ///  D  │ ──────  ← current_stack_pointer      L
    ///     │                                    ║ E
    ///  ╚═ ╘════════ ← memory_start            ═╝
    /// ```

    /// Reference to the slice of `GrantPointerEntry`s stored in the process's
    /// memory reserved for the kernel. These driver numbers are zero and
    /// pointers are null if the grant region has not been allocated. When the
    /// grant region is allocated these pointers are updated to point to the
    /// allocated memory and the driver number is set to match the driver that
    /// owns the grant. No other reference to these pointers exists in the Tock
    /// kernel.
    grant_pointers: MapCell<&'static mut [GrantPointerEntry]>,

    /// Address to the end of the allocated (and MMU protected) grant region.
    kernel_memory_break: Cell<NonZero<usize>>,

    /// Address to the end of process RAM that has been sbrk'd to the process.
    app_break: Cell<NonZero<usize>>,

    /// Address to high water mark for process buffers shared through `allow`
    allow_high_water_mark: Cell<NonZero<usize>>,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: ImmutableKernelVirtualSlice<'static, u8>,

    /// Process RAM segment
    ram: MutableKernelVirtualSlice<'static, u8>,

    /// Collection of pointers to the TBF header in flash.
    header: tock_tbf::types::TbfHeader<'static>,

    /// Credential that was approved for this process, or `None` if the
    /// credential was permitted to run without an accepted credential.
    credential: Option<AcceptedCredential>,

    /// State saved on behalf of the process each time the app switches to the
    /// kernel.
    stored_state:
        MapCell<<<C as Chip>::UserspaceKernelBoundary as UserspaceKernelBoundary>::StoredState>,

    /// The current state of the app. The scheduler uses this to determine
    /// whether it can schedule this app to execute.
    ///
    /// The `state` is used both for bookkeeping for the scheduler as well as
    /// for enabling control by other parts of the system. The scheduler keeps
    /// track of if a process is ready to run or not by switching between the
    /// `Running` and `Yielded` states. The system can control the process by
    /// switching it to a "stopped" state to prevent the scheduler from
    /// scheduling it.
    state: Cell<State>,

    /// How to respond if this process faults.
    fault_policy: &'a dyn ProcessFaultPolicy,

    /// Storage permissions for this process.
    storage_permissions: OptionalCell<StoragePermissions>,

    memory_configuration: configuration::ValidProcessConfiguration<'a, Page4KiB>,

    /// Essentially a list of upcalls that want to call functions in the
    /// process.
    tasks: MapCell<RingBuffer<'a, Task>>,

    /// Count of how many times this process has entered the fault condition and
    /// been restarted. This is used by some `ProcessRestartPolicy`s to
    /// determine if the process should be restarted or not.
    restart_count: Cell<usize>,

    /// The completion code set by the process when it last exited, restarted,
    /// or was terminated. If the process is has never terminated, then the
    /// `OptionalCell` will be empty (i.e. `None`). If the process has exited,
    /// restarted, or terminated, the `OptionalCell` will contain an optional 32
    /// bit value. The option will be `None` if the process crashed or was
    /// stopped by the kernel and there is no provided completion code. If the
    /// process called the exit syscall then the provided completion code will
    /// be stored as `Some(completion code)`.
    completion_code: OptionalCell<Option<u32>>,

    /// Values kept so that we can print useful debug messages when apps fault.
    debug: D,
}

impl<C: Chip, D: 'static + ProcessStandardDebug> Process for ProcessStandard<'_, C, D> {
    fn processid(&self) -> ProcessId {
        self.process_id.get()
    }

    fn short_app_id(&self) -> ShortId {
        self.app_id
    }

    fn binary_version(&self) -> Option<BinaryVersion> {
        let version = self.header.get_binary_version();
        match NonZeroU32::new(version) {
            Some(version_nonzero) => Some(BinaryVersion::new(version_nonzero)),
            None => None,
        }
    }

    fn get_credential(&self) -> Option<AcceptedCredential> {
        self.credential
    }

    fn enqueue_task(&self, task: Task) -> Result<(), ErrorCode> {
        // If this app is in a `Fault` state then we shouldn't schedule
        // any work for it.
        if !self.is_running() {
            return Err(ErrorCode::NODEVICE);
        }

        let ret = self.tasks.map_or(Err(ErrorCode::FAIL), |tasks| {
            match tasks.enqueue(task) {
                true => {
                    // The task has been successfully enqueued.
                    Ok(())
                }
                false => {
                    // The task could not be enqueued as there is
                    // insufficient space in the ring buffer.
                    Err(ErrorCode::NOMEM)
                }
            }
        });

        if ret.is_err() {
            // On any error we were unable to enqueue the task. Record the
            // error, but importantly do _not_ increment kernel work.
            self.debug.increment_dropped_upcall_count();
        }

        ret
    }

    fn ready(&self) -> bool {
        self.tasks.map_or(false, |ring_buf| ring_buf.has_elements())
            || self.state.get() == State::Running
    }

    fn remove_pending_upcalls(&self, upcall_id: UpcallId) -> usize {
        self.tasks.map_or(0, |tasks| {
            let count_before = tasks.len();
            tasks.retain(|task| match task {
                // Remove only tasks that are function calls with an id equal
                // to `upcall_id`.
                Task::FunctionCall(function_call) => match function_call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => id != upcall_id,
                },
                _ => true,
            });
            let count_after = tasks.len();
            if config::CONFIG.trace_syscalls {
                debug!(
                    "[{:?}] remove_pending_upcalls[{:#x}:{}] = {} upcall(s) removed",
                    self.processid(),
                    upcall_id.driver_num,
                    upcall_id.subscribe_num,
                    count_before - count_after,
                );
            }
            count_after - count_before
        })
    }

    fn is_running(&self) -> bool {
        match self.state.get() {
            State::Running | State::Yielded | State::YieldedFor(_) | State::Stopped(_) => true,
            _ => false,
        }
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.set(State::Yielded);
        }
    }

    fn set_yielded_for_state(&self, upcall_id: UpcallId) {
        if self.state.get() == State::Running {
            self.state.set(State::YieldedFor(upcall_id));
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.set(State::Stopped(StoppedState::Running)),
            State::Yielded => self.state.set(State::Stopped(StoppedState::Yielded)),
            State::YieldedFor(upcall_id) => self
                .state
                .set(State::Stopped(StoppedState::YieldedFor(upcall_id))),
            State::Stopped(_stopped_state) => {
                // Already stopped, nothing to do.
            }
            State::Faulted | State::Terminated => {
                // Stop has no meaning on a inactive process.
            }
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::Stopped(stopped_state) => match stopped_state {
                StoppedState::Running => self.state.set(State::Running),
                StoppedState::Yielded => self.state.set(State::Yielded),
                StoppedState::YieldedFor(upcall_id) => self.state.set(State::YieldedFor(upcall_id)),
            },
            _ => {} // Do nothing
        }
    }

    fn set_fault_state(&self) {
        // Use the per-process fault policy to determine what action the kernel
        // should take since the process faulted.
        let action = self.fault_policy.action(self);
        match action {
            FaultAction::Panic => {
                // process faulted. Panic and print status
                self.state.set(State::Faulted);
                panic!("Process {} had a fault", self.get_process_name());
            }
            FaultAction::Restart => {
                self.try_restart(None);
            }
            FaultAction::Stop => {
                // This looks a lot like restart, except we just leave the app
                // how it faulted and mark it as `Faulted`. By clearing
                // all of the app's todo work it will not be scheduled, and
                // clearing all of the grant regions will cause capsules to drop
                // this app as well.
                self.terminate(None);
                self.state.set(State::Faulted);
            }
        }
    }

    fn start(&self, _cap: &dyn crate::capabilities::ProcessStartCapability) {
        // `start()` can only be called on a terminated process.
        if self.get_state() != State::Terminated {
            return;
        }

        // Reset to start the process.
        if let Ok(()) = self.reset() {
            self.state.set(State::Yielded);
        }
    }

    fn try_restart(&self, completion_code: Option<u32>) {
        // `try_restart()` cannot be called if the process is terminated. Only
        // `start()` can start a terminated process.
        if self.get_state() == State::Terminated {
            return;
        }

        // Terminate the process, freeing its state and removing any
        // pending tasks from the scheduler's queue.
        self.terminate(completion_code);

        // If there is a kernel policy that controls restarts, it should be
        // implemented here. For now, always restart.
        if let Ok(()) = self.reset() {
            self.state.set(State::Yielded);
        }

        // Decide what to do with res later. E.g., if we can't restart
        // want to reclaim the process resources.
    }

    fn terminate(&self, completion_code: Option<u32>) {
        // A process can be terminated if it is running or in the `Faulted`
        // state. Otherwise, you cannot terminate it and this method return
        // early.
        //
        // The kernel can terminate in the `Faulted` state to return the process
        // to a state in which it can run again (e.g., reset it).
        if !self.is_running() && self.get_state() != State::Faulted {
            return;
        }

        // And remove those tasks
        self.tasks.map(|tasks| {
            tasks.empty();
        });

        // Clear any grant regions this app has setup with any capsules.
        unsafe {
            self.grant_ptrs_reset();
        }

        // Save the completion code.
        self.completion_code.set(completion_code);

        // Mark the app as stopped so the scheduler won't try to run it.
        self.state.set(State::Terminated);
    }

    fn get_restart_count(&self) -> usize {
        self.restart_count.get()
    }

    fn has_tasks(&self) -> bool {
        self.tasks.map_or(false, |tasks| tasks.has_elements())
    }

    fn dequeue_task(&self) -> Option<Task> {
        self.tasks.map_or(None, |tasks| tasks.dequeue())
    }

    fn remove_upcall(&self, upcall_id: UpcallId) -> Option<Task> {
        self.tasks.map_or(None, |tasks| {
            tasks.remove_first_matching(|task| match task {
                Task::FunctionCall(fc) => match fc.source {
                    FunctionCallSource::Driver(upid) => upid == upcall_id,
                    _ => false,
                },
                Task::ReturnValue(rv) => rv.upcall_id == upcall_id,
                Task::IPC(_) => false,
            })
        })
    }

    fn pending_tasks(&self) -> usize {
        self.tasks.map_or(0, |tasks| tasks.len())
    }

    fn get_command_permissions(&self, driver_num: usize, offset: usize) -> CommandPermissions {
        self.header.get_command_permissions(driver_num, offset)
    }

    fn get_storage_permissions(&self) -> StoragePermissions {
        // This is set during ProcessStandard::create()
        self.storage_permissions.get().unwrap()
    }

    fn number_writeable_flash_regions(&self) -> usize {
        self.header.number_writeable_flash_regions()
    }

    fn get_writeable_flash_region(&self, region_index: usize) -> (usize, usize) {
        self.header.get_writeable_flash_region(region_index)
    }

    fn update_stack_start_pointer(&self, stack_pointer: ImmutableUserVirtualPointer<u8>) {
        let ram_region = self.get_ram_region();
        if ram_region.is_containing_protected_virtual_byte(&stack_pointer) {
            self.debug.set_app_stack_start_pointer(stack_pointer);
            // We also reset the minimum stack pointer because whatever
            // value we had could be entirely wrong by now.
            self.debug.set_app_stack_min_pointer(stack_pointer);
        }
    }

    fn update_heap_start_pointer(&self, heap_pointer: ImmutableUserVirtualPointer<u8>) {
        let ram_region = self.get_ram_region();
        if ram_region.is_containing_protected_virtual_byte(&heap_pointer) {
            self.debug.set_app_heap_start_pointer(heap_pointer);
        }
    }

    fn get_memory_configuration(&self) -> &configuration::ValidProcessConfiguration<Page4KiB> {
        &self.memory_configuration
    }

    fn sbrk(&self, increment: isize) -> Result<CapabilityPtr, Error> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        let app_break = self.app_memory_break().get() as isize;
        let new_break_address = app_break.checked_add(increment).unwrap() as usize;
        let new_break = unsafe {
            ImmutableKernelVirtualPointer::new_from_raw_byte(new_break_address as *const u8)
        }
        .unwrap();
        self.internal_brk(new_break)
    }

    fn brk(&self, new_break: ImmutableUserVirtualPointer<u8>) -> Result<CapabilityPtr, Error> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        let memory_configuration = self.get_memory_configuration();
        let kernel_new_break = match self
            .kernel
            .internal_translate_user_allocated_virtual_pointer_byte(memory_configuration, new_break)
        {
            Err(_new_break) => return Err(Error::AddressOutOfBounds),
            Ok(kernel_new_break) => kernel_new_break,
        };

        self.internal_brk(kernel_new_break)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn build_readwrite_process_buffer(
        &self,
        buf_start_addr: Option<MutableKernelVirtualPointer<u8>>,
        size: usize,
    ) -> Result<ReadWriteProcessBuffer, ErrorCode> {
        if !self.is_running() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        let non_zero_size = match NonZero::new(size) {
            // A process is allowed to pass any pointer if the buffer length is 0,
            // as to revoke kernel access to a memory region without granting access
            // to another one
            None => {
                // Clippy complains that we're dereferencing a pointer in a public
                // and safe function here. While we are not dereferencing the
                // pointer here, we pass it along to an unsafe function, which is as
                // dangerous (as it is likely to be dereferenced down the line).
                //
                // Relevant discussion:
                // https://github.com/rust-lang/rust-clippy/issues/3045
                //
                // It should be fine to ignore the lint here, as a buffer of length
                // 0 will never allow dereferencing any memory in a safe manner.
                //
                // ### Safety
                //
                // We specify a zero-length buffer, so the implementation of
                // `ReadWriteProcessBuffer` will handle any safety issues.
                // Therefore, we can encapsulate the unsafe.
                return Ok(unsafe {
                    ReadWriteProcessBuffer::new(buf_start_addr, 0, self.processid())
                });
            }
            Some(non_zero_size) => non_zero_size,
        };

        let buf_start_addr = match buf_start_addr {
            None => return Err(ErrorCode::INVAL),
            Some(buf_start_addr) => buf_start_addr,
        };

        if self.in_app_owned_memory(buf_start_addr.as_immutable(), non_zero_size) {
            // TODO: Check for buffer aliasing here

            // Valid buffer, we need to adjust the app's watermark
            // PANIC: `in_app_owned_memory` ensures this offset does not wrap
            let buf_end_addr = buf_start_addr.checked_add(non_zero_size).unwrap();
            let new_water_mark_address =
                cmp::max(self.allow_high_water_mark.get(), buf_end_addr.get_address());
            self.allow_high_water_mark.set(new_water_mark_address);

            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as we make
            // sure that we're pointing towards userspace memory (verified using
            // `in_app_owned_memory`) and respect alignment and other
            // constraints of the Rust references created by
            // `ReadWriteProcessBuffer`.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadWriteProcessBuffer` will
            // be the only reference to this memory.
            Ok(
                unsafe {
                    ReadWriteProcessBuffer::new(Some(buf_start_addr), size, self.processid())
                },
            )
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn build_readonly_process_buffer(
        &self,
        buf_start_addr: Option<ImmutableKernelVirtualPointer<u8>>,
        size: usize,
    ) -> Result<ReadOnlyProcessBuffer, ErrorCode> {
        if !self.is_running() {
            // Do not operate on an inactive process
            return Err(ErrorCode::FAIL);
        }

        let non_zero_size = match NonZero::new(size) {
            // A process is allowed to pass any pointer if the buffer length is 0,
            // as to revoke kernel access to a memory region without granting access
            // to another one
            None => {
                // Clippy complains that we're dereferencing a pointer in a public
                // and safe function here. While we are not dereferencing the
                // pointer here, we pass it along to an unsafe function, which is as
                // dangerous (as it is likely to be dereferenced down the line).
                //
                // Relevant discussion:
                // https://github.com/rust-lang/rust-clippy/issues/3045
                //
                // It should be fine to ignore the lint here, as a buffer of length
                // 0 will never allow dereferencing any memory in a safe manner.
                //
                // ### Safety
                //
                // We specify a zero-length buffer, so the implementation of
                // `ReadOnlyProcessBuffer` will handle any safety issues. Therefore,
                // we can encapsulate the unsafe.
                return Ok(unsafe {
                    ReadOnlyProcessBuffer::new(buf_start_addr, 0, self.processid())
                });
            }
            Some(non_zero_size) => non_zero_size,
        };

        let buf_start_addr = match buf_start_addr {
            None => return Err(ErrorCode::INVAL),
            Some(buf_start_addr) => buf_start_addr,
        };

        if self.in_app_owned_memory(&buf_start_addr, non_zero_size)
            || self.in_app_flash_memory(&buf_start_addr, non_zero_size)
        {
            // TODO: Check for buffer aliasing here

            if self.in_app_owned_memory(&buf_start_addr, non_zero_size) {
                // Valid buffer, and since this is in read-write memory (i.e.
                // not flash), we need to adjust the process's watermark. Note:
                // `in_app_owned_memory()` ensures this offset does not wrap.
                let buf_end_addr = buf_start_addr.checked_add(non_zero_size).unwrap();
                let new_water_mark_address =
                    cmp::max(self.allow_high_water_mark.get(), buf_end_addr.get_address());
                self.allow_high_water_mark.set(new_water_mark_address);
            }

            // Clippy complains that we're dereferencing a pointer in a public
            // and safe function here. While we are not dereferencing the
            // pointer here, we pass it along to an unsafe function, which is as
            // dangerous (as it is likely to be dereferenced down the line).
            //
            // Relevant discussion:
            // https://github.com/rust-lang/rust-clippy/issues/3045
            //
            // It should be fine to ignore the lint here, as long as we make
            // sure that we're pointing towards userspace memory (verified using
            // `in_app_owned_memory` or `in_app_flash_memory`) and respect
            // alignment and other constraints of the Rust references created by
            // `ReadWriteProcessBuffer`.
            //
            // ### Safety
            //
            // We encapsulate the unsafe here on the condition in the TODO
            // above, as we must ensure that this `ReadOnlyProcessBuffer` will
            // be the only reference to this memory.
            Ok(unsafe { ReadOnlyProcessBuffer::new(Some(buf_start_addr), size, self.processid()) })
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    unsafe fn set_byte(&self, mut addr: MutableKernelVirtualPointer<u8>, value: u8) -> bool {
        if self.in_app_owned_memory(addr.as_immutable(), create_non_zero_usize(1)) {
            // We verify that this will only write process-accessible memory,
            // but this can still be undefined behavior if something else holds
            // a reference to this memory.
            addr.write(value);
            true
        } else {
            false
        }
    }

    fn grant_is_allocated(&self, grant_num: usize) -> Option<bool> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return None;
        }

        // Update the grant pointer to the address of the new allocation.
        self.grant_pointers.map_or(None, |grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a chance of a
            // panic.
            grant_pointers
                .get(grant_num)
                .map(|grant_entry| !grant_entry.grant_ptr.is_null())
        })
    }

    fn allocate_grant(
        &self,
        grant_num: usize,
        driver_num: usize,
        size: usize,
        align: usize,
    ) -> Result<(), ()> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(());
        }

        // Verify the grant_num is valid.
        if grant_num >= self.kernel.get_grant_count_and_finalize() {
            return Err(());
        }

        // Verify that the grant is not already allocated. If the pointer is not
        // null then the grant is already allocated.
        if let Some(is_allocated) = self.grant_is_allocated(grant_num) {
            if is_allocated {
                return Err(());
            }
        }

        // Verify that there is not already a grant allocated with the same
        // `driver_num`.
        let exists = self.grant_pointers.map_or(false, |grant_pointers| {
            // Check our list of grant pointers if the driver number is used.
            grant_pointers.iter().any(|grant_entry| {
                // Check if the grant is both allocated (its grant pointer is
                // non null) and the driver number matches.
                (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
            })
        });
        // If we find a match, then the `driver_num` must already be used and
        // the grant allocation fails.
        if exists {
            return Err(());
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(grant_ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Update the grant pointer to the address of the new allocation.
            self.grant_pointers.map_or(Err(()), |grant_pointers| {
                // Implement `grant_pointers[grant_num] = grant_ptr` without a
                // chance of a panic.
                grant_pointers
                    .get_mut(grant_num)
                    .map_or(Err(()), |grant_entry| {
                        // Actually set the driver num and grant pointer.
                        grant_entry.driver_num = driver_num;
                        grant_entry.grant_ptr = grant_ptr.as_ptr();

                        // If all of this worked, return true.
                        Ok(())
                    })
            })
        } else {
            // Could not allocate the memory for the grant region.
            Err(())
        }
    }

    fn allocate_custom_grant(
        &self,
        size: usize,
        align: usize,
    ) -> Result<(ProcessCustomGrantIdentifier, NonNull<u8>), ()> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(());
        }

        // Use the shared grant allocator function to actually allocate memory.
        // Returns `None` if the allocation cannot be created.
        if let Some(ptr) = self.allocate_in_grant_region_internal(size, align) {
            // Create the identifier that the caller will use to get access to
            // this custom grant in the future.
            let identifier = self.create_custom_grant_identifier(ptr);

            Ok((identifier, ptr))
        } else {
            // Could not allocate memory for the custom grant.
            Err(())
        }
    }

    fn enter_grant(&self, grant_num: usize) -> Result<NonNull<u8>, Error> {
        // Do not try to access the grant region of an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        // Retrieve the grant pointer from the `grant_pointers` slice. We use
        // `[slice].get()` so that if the grant number is invalid this will
        // return `Err` and not panic.
        self.grant_pointers
            .map_or(Err(Error::KernelError), |grant_pointers| {
                // Implement `grant_pointers[grant_num]` without a chance of a
                // panic.
                match grant_pointers.get_mut(grant_num) {
                    Some(grant_entry) => {
                        // Get a copy of the actual grant pointer.
                        let grant_ptr = grant_entry.grant_ptr;

                        // Check if the grant pointer is marked that the grant
                        // has already been entered. If so, return an error.
                        if (grant_ptr as usize) & 0x1 == 0x1 {
                            // Lowest bit is one, meaning this grant has been
                            // entered.
                            Err(Error::AlreadyInUse)
                        } else {
                            // Now, to mark that the grant has been entered, we
                            // set the lowest bit to one and save this as the
                            // grant pointer.
                            grant_entry.grant_ptr = (grant_ptr as usize | 0x1) as *mut u8;

                            // And we return the grant pointer to the entered
                            // grant.
                            Ok(unsafe { NonNull::new_unchecked(grant_ptr) })
                        }
                    }
                    None => Err(Error::AddressOutOfBounds),
                }
            })
    }

    fn enter_custom_grant(
        &self,
        identifier: ProcessCustomGrantIdentifier,
    ) -> Result<*mut u8, Error> {
        // Do not try to access the grant region of an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        // Get the address of the custom grant based on the identifier.
        let custom_grant_address = self.get_custom_grant_address(identifier);

        // We never deallocate custom grants and only we can change the
        // `identifier` so we know this is a valid address for the custom grant.
        Ok(custom_grant_address as *mut u8)
    }

    unsafe fn leave_grant(&self, grant_num: usize) {
        // Do not modify an inactive process.
        if !self.is_running() {
            return;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Implement `grant_pointers[grant_num]` without a chance of a
            // panic.
            if let Some(grant_entry) = grant_pointers.get_mut(grant_num) {
                // Get a copy of the actual grant pointer.
                let grant_ptr = grant_entry.grant_ptr;

                // Now, to mark that the grant has been released, we set the
                // lowest bit back to zero and save this as the grant
                // pointer.
                grant_entry.grant_ptr = (grant_ptr as usize & !0x1) as *mut u8;
            }
        });
    }

    fn grant_allocated_count(&self) -> Option<usize> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return None;
        }

        self.grant_pointers.map(|grant_pointers| {
            // Filter our list of grant pointers into just the non-null ones,
            // and count those. A grant is allocated if its grant pointer is
            // non-null.
            grant_pointers
                .iter()
                .filter(|grant_entry| !grant_entry.grant_ptr.is_null())
                .count()
        })
    }

    fn lookup_grant_from_driver_num(&self, driver_num: usize) -> Result<usize, Error> {
        self.grant_pointers
            .map_or(Err(Error::KernelError), |grant_pointers| {
                // Filter our list of grant pointers into just the non null
                // ones, and count those. A grant is allocated if its grant
                // pointer is non-null.
                match grant_pointers.iter().position(|grant_entry| {
                    // Only consider allocated grants.
                    (!grant_entry.grant_ptr.is_null()) && grant_entry.driver_num == driver_num
                }) {
                    Some(idx) => Ok(idx),
                    None => Err(Error::OutOfMemory),
                }
            })
    }

    fn is_valid_upcall_function_pointer(
        &self,
        upcall_fn: ImmutableKernelVirtualPointer<u8>,
    ) -> bool {
        const SIZE: NonZero<usize> = create_non_zero_usize(mem::size_of::<*const u8>());

        // It is okay if this function is in memory or flash.
        self.in_app_flash_memory(&upcall_fn, SIZE) || self.in_app_owned_memory(&upcall_fn, SIZE)
    }

    fn get_process_name(&self) -> &'static str {
        self.header.get_package_name().unwrap_or("")
    }

    fn get_completion_code(&self) -> Option<Option<u32>> {
        self.completion_code.get()
    }

    fn set_syscall_return_value(&self, return_value: SyscallReturn) {
        match self.stored_state.map(|stored_state| unsafe {
            let kernel_accessible_memory_start =
                self.get_ram_start().as_immutable().infallible_cast();

            // SAFETY: `app_memory_break` represents a valid kernel virtual pointer.
            let kernel_app_brk = ImmutableKernelVirtualPointer::new_from_raw_byte(
                self.app_memory_break().get() as *const u8,
            )
            .unwrap();

            // Actually set the return value for a particular process.
            //
            // The UKB implementation uses the bounds of process-accessible
            // memory to verify that any memory changes are valid. Here, the
            // unsafe promise we are making is that the bounds passed to the UKB
            // are correct.
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(
                    &kernel_accessible_memory_start,
                    &kernel_app_brk,
                    stored_state,
                    return_value,
                )
        }) {
            Some(Ok(())) => {
                // If we get an `Ok` we are all set.

                // The process is either already in the running state (having
                // just called a nonblocking syscall like command) or needs to
                // be moved to the running state having called Yield-WaitFor and
                // now needing to be resumed. Either way we can set the state to
                // running.
                self.state.set(State::Running);
            }

            Some(Err(())) => {
                // If we get an `Err`, then the UKB implementation could not set
                // the return value, likely because the process's stack is no
                // longer accessible to it. All we can do is fault.
                self.set_fault_state();
            }

            None => {
                // We should never be here since `stored_state` should always be
                // occupied.
                self.set_fault_state();
            }
        }
    }

    fn set_process_function(&self, callback: FunctionCall) {
        // See if we can actually enqueue this function for this process.
        // Architecture-specific code handles actually doing this since the
        // exact method is both architecture- and implementation-specific.
        //
        // This can fail, for example if the process does not have enough memory
        // remaining.
        match self.stored_state.map(|stored_state| {
            let kernel_accessible_memory_start =
                self.get_ram_start().as_immutable().infallible_cast();

            // SAFETY: `app_memory_break` represents a valid kernel virtual pointer.
            let kernel_app_brk = unsafe {
                ImmutableKernelVirtualPointer::new_from_raw_byte(
                    self.app_memory_break().get() as *const u8
                )
                .unwrap()
            };

            // Let the UKB implementation handle setting the process's PC so
            // that the process executes the upcall function. We encapsulate
            // unsafe here because we are guaranteeing that the memory bounds
            // passed to `set_process_function` are correct.
            unsafe {
                self.chip.userspace_kernel_boundary().set_process_function(
                    &kernel_accessible_memory_start,
                    &kernel_app_brk,
                    stored_state,
                    callback,
                )
            }
        }) {
            Some(Ok(())) => {
                // If we got an `Ok` we are all set and should mark that this
                // process is ready to be scheduled.

                // Move this process to the "running" state so the scheduler
                // will schedule it.
                self.state.set(State::Running);
            }

            Some(Err(())) => {
                // If we got an Error, then there was likely not enough room on
                // the stack to allow the process to execute this function given
                // the details of the particular architecture this is running
                // on. This process has essentially faulted, so we mark it as
                // such.
                self.set_fault_state();
            }

            None => {
                // We should never be here since `stored_state` should always be
                // occupied.
                self.set_fault_state();
            }
        }
    }

    fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        // Cannot switch to an invalid process
        if !self.is_running() {
            return None;
        }

        let (switch_reason, stack_pointer) =
            self.stored_state.map_or((None, None), |stored_state| {
                let kernel_accessible_memory_start =
                    self.get_ram_start().as_immutable().infallible_cast();

                // SAFETY: `app_memory_break` represents a valid kernel virtual pointer.
                let kernel_app_brk = unsafe {
                    ImmutableKernelVirtualPointer::new_from_raw_byte(
                        self.app_memory_break().get() as *const u8
                    )
                    .unwrap()
                };

                // Switch to the process. We guarantee that the memory pointers
                // we pass are valid, ensuring this context switch is safe.
                // Therefore we encapsulate the `unsafe`.
                unsafe {
                    let (switch_reason, optional_stack_pointer) =
                        self.chip.userspace_kernel_boundary().switch_to_process(
                            &kernel_accessible_memory_start,
                            &kernel_app_brk,
                            stored_state,
                        );
                    (Some(switch_reason), optional_stack_pointer)
                }
            });

        // If the UKB implementation passed us a stack pointer, update our
        // debugging state. This is completely optional.
        if let Some(sp) = stack_pointer {
            self.debug.set_new_app_stack_min_pointer(sp);
        }

        switch_reason
    }

    fn debug_syscall_count(&self) -> usize {
        self.debug.get_syscall_count()
    }

    fn debug_dropped_upcall_count(&self) -> usize {
        self.debug.get_dropped_upcall_count()
    }

    fn debug_timeslice_expiration_count(&self) -> usize {
        self.debug.get_timeslice_expiration_count()
    }

    fn debug_timeslice_expired(&self) {
        self.debug.increment_timeslice_expiration_count();
    }

    fn debug_syscall_called(&self, last_syscall: Syscall) {
        self.debug.increment_syscall_count();
        self.debug.set_last_syscall(last_syscall);
    }

    fn debug_syscall_last(&self) -> Option<Syscall> {
        self.debug.get_last_syscall()
    }

    fn get_addresses(&self) -> ProcessAddresses {
        ProcessAddresses {
            flash_start: self.flash_start().get_address().get(),
            flash_non_protected_start: self.flash_non_protected_start().get_address().get(),
            flash_integrity_end: self
                .flash
                .get_starting_pointer()
                .checked_add(NonZero::new(self.header.get_binary_end() as usize).unwrap())
                .unwrap()
                .get_address()
                .get(),
            flash_end: self.flash_end().get_address().get(),
            sram_start: self.get_ram_start().get_address().get(),
            sram_app_brk: self.app_memory_break().get(),
            sram_grant_start: self.kernel_memory_break().get(),
            sram_end: self.get_ram_end().get_address().get(),
            sram_heap_start: self
                .debug
                .get_app_heap_start_pointer()
                .map(|p| p.get_address().get()),
            sram_stack_top: self
                .debug
                .get_app_stack_start_pointer()
                .map(|p| p.get_address().get()),
            sram_stack_bottom: self
                .debug
                .get_app_stack_min_pointer()
                .map(|p| p.get_address().get()),
        }
    }

    fn get_sizes(&self) -> ProcessSizes {
        ProcessSizes {
            grant_pointers: mem::size_of::<GrantPointerEntry>()
                * self.kernel.get_grant_count_and_finalize(),
            upcall_list: Self::CALLBACKS_OFFSET,
            process_control_block: Self::PROCESS_STRUCT_OFFSET,
        }
    }

    fn print_full_process(&self, writer: &mut dyn Write) {
        if !config::CONFIG.debug_panics {
            return;
        }

        self.stored_state.map(|stored_state| {
            // We guarantee the memory bounds pointers provided to the UKB are
            // correct.
            unsafe {
                self.chip.userspace_kernel_boundary().print_context(
                    self.get_ram_start().as_immutable().infallible_cast_ref(),
                    &ImmutableKernelVirtualPointer::new_from_raw_byte(
                        self.app_memory_break().get() as *const u8,
                    )
                    .unwrap(),
                    stored_state,
                    writer,
                );
            }
        });

        // Display grant information.
        let number_grants = self.kernel.get_grant_count_and_finalize();
        let _ = writer.write_fmt(format_args!(
            "\
            \r\n Total number of grant regions defined: {}\r\n",
            self.kernel.get_grant_count_and_finalize()
        ));
        let rows = number_grants.div_ceil(3);

        // Access our array of grant pointers.
        self.grant_pointers.map(|grant_pointers| {
            // Iterate each grant and show its address.
            for i in 0..rows {
                for j in 0..3 {
                    let index = i + (rows * j);
                    if index >= number_grants {
                        break;
                    }

                    // Implement `grant_pointers[grant_num]` without a chance of
                    // a panic.
                    grant_pointers.get(index).map(|grant_entry| {
                        if grant_entry.grant_ptr.is_null() {
                            let _ =
                                writer.write_fmt(format_args!("  Grant {:>2} : --        ", index));
                        } else {
                            let _ = writer.write_fmt(format_args!(
                                "  Grant {:>2} {:#x}: {:p}",
                                index, grant_entry.driver_num, grant_entry.grant_ptr
                            ));
                        }
                    });
                }
                let _ = writer.write_fmt(format_args!("\r\n"));
            }
        });

        // Display the current state of the MPU for this process.
        // NOTE: The caller's signature does not allow reporting any errors, so ignore them
        let _ = writer.write_fmt(format_args!("{}\n", self.get_memory_configuration()));

        // Print a helpful message on how to re-compile a process to view the
        // listing file. If a process is PIC, then we also need to print the
        // actual addresses the process executed at so that the .lst file can be
        // generated for those addresses. If the process was already compiled
        // for a fixed address, then just generating a .lst file is fine.

        if self.debug.get_fixed_address_flash().is_some() {
            // Fixed addresses, can just run `make lst`.
            let _ = writer.write_fmt(format_args!(
                "\
                    \r\nTo debug libtock-c apps, run `make lst` in the app's\
                    \r\nfolder and open the arch.{:#x}.{:#x}.lst file.\r\n\r\n",
                self.debug.get_fixed_address_flash().unwrap_or(0),
                self.debug.get_fixed_address_ram().unwrap_or(0)
            ));
        } else {
            // PIC, need to specify the addresses.
            let sram_start = self.get_ram_start().get_address();
            let flash_start = self.flash.get_starting_pointer().get_address().get();
            let flash_init_fn = flash_start + self.header.get_init_function_offset() as usize;

            let _ = writer.write_fmt(format_args!(
                "\
                    \r\nTo debug libtock-c apps, run\
                    \r\n`make debug RAM_START={:#x} FLASH_INIT={:#x}`\
                    \r\nin the app's folder and open the .lst file.\r\n\r\n",
                sram_start, flash_init_fn
            ));
        }
    }

    fn get_stored_state(&self, out: &mut [u8]) -> Result<usize, ErrorCode> {
        self.stored_state
            .map(|stored_state| {
                self.chip
                    .userspace_kernel_boundary()
                    .store_context(stored_state, out)
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }
}

impl<C: 'static + Chip, D: 'static + ProcessStandardDebug> ProcessStandard<'_, C, D> {
    // Memory offset for upcall ring buffer (10 element length).
    const CALLBACK_LEN: usize = 10;
    const CALLBACKS_OFFSET: usize = mem::size_of::<Task>() * Self::CALLBACK_LEN;

    // Memory offset to make room for this process's metadata.
    const PROCESS_STRUCT_OFFSET: usize = mem::size_of::<ProcessStandard<C, D>>();

    /// Create a `ProcessStandard` object based on the found `ProcessBinary`.
    pub(crate) unsafe fn create(
        kernel: &'static Kernel,
        chip: &'static C,
        pb: ProcessBinary,
        fault_policy: &'static dyn ProcessFaultPolicy,
        storage_permissions_policy: &'static dyn ProcessStandardStoragePermissionsPolicy<C, D>,
        app_id: ShortId,
        index: usize,
    ) -> Result<Option<&'static dyn Process>, ProcessLoadError> {
        let process_memory_manager = kernel.get_process_memory_manager();

        let process_name = pb.header.get_package_name();
        let process_ram_requested_size = pb.header.get_minimum_app_ram_size() as usize;

        /****************/
        /* FLASH MEMORY */
        /****************/

        // Allocate region
        let flash_start_address = pb.flash.as_ptr();
        // PANIC: TODO: Return an error instead of panicking.
        let app_flash_physical_starting_pointer =
            MutablePhysicalPointer::new_raw(flash_start_address as *mut Page4KiB).unwrap();
        let flat_app_flash_virtual_starting_pointer =
            app_flash_physical_starting_pointer.to_valid_virtual_pointer();
        let app_flash_length_bytes = match NonZero::new(pb.flash.len()) {
            None => todo!("Return error"),
            Some(app_flash_length) => app_flash_length,
        };
        let app_flash_length_granules =
            divide_exact_non_zero_usize(app_flash_length_bytes, Page4KiB::SIZE_U8);
        let app_flash_slice = MutablePhysicalSlice::from_raw_parts(
            app_flash_physical_starting_pointer,
            app_flash_length_granules,
        );

        let app_flash_allocated_region = AllocatedRegion::new(app_flash_slice);

        // Associate permissions
        let protected_app_flash_region = match ProtectedAllocatedRegion::new(
            app_flash_allocated_region,
            app_flash_length_granules,
            Permissions::ReadExecute,
        ) {
            Err(()) => todo!("Return error"),
            Ok(protected_app_flash_region) => protected_app_flash_region,
        };

        let app_flash_virtual_starting_pointer = match pb.header.get_fixed_address_flash() {
            None => flat_app_flash_virtual_starting_pointer,
            Some(fixed_address) => {
                // CAST: size_of::<usize>() >= size_of::<u32>() on 32 and 64-bit platforms
                let raw_aligned_pointer =
                    align_down_usize(fixed_address as usize, Page4KiB::SIZE_U8) as *mut _;
                // TODO: return error instead of panicking
                MutableUserVirtualPointer::new_from_raw(raw_aligned_pointer).unwrap()
            }
        };

        let mapped_app_flash_region = UserMappedProtectedAllocatedRegion::new_from_protected(
            protected_app_flash_region,
            app_flash_virtual_starting_pointer,
            // TODO: Don't panic
        )
        .unwrap();

        // Determine how much space we need in the application's memory space
        // just for kernel and grant state. We need to make sure we allocate
        // enough memory just for that.

        // Make room for grant pointers.
        let grant_ptr_size = mem::size_of::<GrantPointerEntry>();
        let grant_ptrs_num = kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        // Initial size of the kernel-owned part of process memory can be
        // calculated directly based on the initial size of all kernel-owned
        // data structures.
        //
        // We require our kernel memory break (located at the end of the
        // MPU-returned allocated memory region) to be word-aligned. However, we
        // don't have any explicit alignment constraints from the MPU. To ensure
        // that the below kernel-owned data structures still fit into the
        // kernel-owned memory even with padding for alignment, add an extra
        // `sizeof(usize)` bytes.
        let initial_kernel_memory_size = grant_ptrs_offset
            + Self::CALLBACKS_OFFSET
            + Self::PROCESS_STRUCT_OFFSET
            + core::mem::size_of::<usize>();

        // By default we start with the initial size of process-accessible
        // memory set to 0. This maximizes the flexibility that processes have
        // to allocate their memory as they see fit. If a process needs more
        // accessible memory it must use the `brk` memop syscalls to request
        // more memory.
        //
        // We must take into account any process-accessible memory required by
        // the context switching implementation and allocate at least that much
        // memory so that we can successfully switch to the process. This is
        // architecture and implementation specific, so we query that now.
        let min_process_memory_size = chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // We have to ensure that we at least ask the MPU for
        // `min_process_memory_size` so that we can be sure that `app_brk` is
        // not set inside the kernel-owned memory region. Now, in practice,
        // processes should not request 0 (or very few) bytes of memory in their
        // TBF header (i.e. `process_ram_requested_size` will almost always be
        // much larger than `min_process_memory_size`), as they are unlikely to
        // work with essentially no available memory. But, we still must protect
        // for that case.
        let min_process_ram_size = cmp::max(process_ram_requested_size, min_process_memory_size);

        // Minimum memory size for the process.
        let optional_min_total_memory_size =
            NonZero::new(min_process_ram_size + initial_kernel_memory_size);

        let min_total_memory_size = match optional_min_total_memory_size {
            None => todo!("Return error"),
            Some(min_total_memory_size) => min_total_memory_size,
        };

        /*******/
        /* RAM */
        /*******/

        let allocation_granule_count =
            ceil_non_zero_usize(min_total_memory_size, Page4KiB::SIZE_U8);
        // Allocate region
        let ram_allocated_region = match process_memory_manager.allocate(allocation_granule_count) {
            Err(()) => todo!("Return error"),
            Ok(ram_allocated_region) => ram_allocated_region,
        };
        let allocation_size_bytes = ram_allocated_region.get_length_bytes();
        let protected_granule_count = ceil_usize(min_process_memory_size, Page4KiB::SIZE_U8);
        let non_zero_protected_granule_count = match NonZero::new(protected_granule_count) {
            None => todo!("Should granule count be really non-zero?"),
            Some(non_zero_protected_granule_count) => non_zero_protected_granule_count,
        };

        // Associate permissions
        let ram_protected_region = ProtectedAllocatedRegion::new(
            ram_allocated_region,
            non_zero_protected_granule_count,
            Permissions::ReadWrite,
            // PANIC: the granule count is computed in such a way that it is smaller than the length of
            // the allocated region
        )
        .unwrap();

        let ram_starting_physical_pointer = ram_protected_region.get_starting_pointer();
        let ram_kernel_starting_virtual_pointer = kernel
            .translate_kernel_allocated_physical_pointer_byte_to_kernel_virtual_pointer_byte(
                ram_starting_physical_pointer.infallible_cast(),
            )
            // TODO: Don't panic
            .unwrap();
        let ram_starting_virtual_pointer = match pb.header.get_fixed_address_ram() {
            None => ram_starting_physical_pointer.to_valid_virtual_pointer(),
            Some(ram_fixed_address) => {
                // TODO: don't panic
                MutableUserVirtualPointer::new_from_raw(ram_fixed_address as *mut _).unwrap()
            }
        };

        let mapped_ram_protected_region = UserMappedProtectedAllocatedRegion::new_from_protected(
            ram_protected_region,
            ram_starting_virtual_pointer,
            // TODO: don't panic
        )
        .unwrap();

        let asid = chip.mmu().create_asid();

        let app_memory_configuration = process_memory_manager.new_configuration(
            asid,
            mapped_app_flash_region,
            mapped_ram_protected_region,
        );

        let valid_app_memory_configuration = match kernel
            .is_process_memory_configuration_valid(app_memory_configuration)
        {
            Err(mapping_error) => return Err(ProcessLoadError::MemoryMappingError(mapping_error)),
            Ok(valid_app_memory_configuration) => valid_app_memory_configuration,
        };

        // With our MPU allocation, we can begin to divide up the
        // `remaining_memory` slice into individual regions for the process and
        // kernel, as follows:
        //
        //
        //  +-----------------------------------------------------------------
        //  | remaining_memory
        //  +----------------------------------------------------+------------
        //  v                                                    v
        //  +----------------------------------------------------+
        //  | allocated_padded_memory                            |
        //  +--+-------------------------------------------------+
        //     v                                                 v
        //     +-------------------------------------------------+
        //     | allocated_memory                                |
        //     +-------------------------------------------------+
        //     v                                                 v
        //     +-----------------------+-------------------------+
        //     | app_accessible_memory | allocated_kernel_memory |
        //     +-----------------------+-------------------+-----+
        //                                                 v
        //                               kernel memory break
        //                                                  \---+/
        //                                                      v
        //                                        optional padding
        //
        //
        // First split the `remaining_memory` into two slices:
        //
        // - `allocated_padded_memory`: the allocated memory region, containing
        //
        //   1. optional padding at the start of the memory region of
        //      `app_memory_start_offset` bytes,
        //
        //   2. the app accessible memory region of `min_process_memory_size`,
        //
        //   3. optional unallocated memory, and
        //
        //   4. kernel-reserved memory, growing downward starting at
        //      `app_memory_padding`.
        //
        // - `unused_memory`: the rest of the `remaining_memory`, not assigned
        //   to this app.
        //

        let allocated_memory: MutableKernelVirtualSlice<'static, u8> =
            MutableKernelVirtualSlice::from_raw_parts(
                ram_kernel_starting_virtual_pointer,
                allocation_size_bytes,
            );

        // Slice off the process-accessible memory:
        // TODO: min_process_memory_size may be zero.
        let (app_accessible_memory, optional_allocated_kernel_memory) = match allocated_memory
            .split_at_checked(create_non_zero_usize(min_process_memory_size))
        {
            Err(_) => todo!(),
            Ok(result) => result,
        };

        let allocated_kernel_memory = match optional_allocated_kernel_memory {
            None => todo!(),
            Some(allocated_kernel_memory) => allocated_kernel_memory,
        };

        // Set the initial process-accessible memory:
        let kernel_initial_app_brk = app_accessible_memory
            .get_starting_pointer()
            .to_immutable()
            .checked_add(app_accessible_memory.get_length())
            .unwrap();

        // Set the initial allow high water mark to the start of process memory
        // since no `allow` calls have been made yet.
        let initial_allow_high_water_mark = app_accessible_memory.get_starting_pointer();

        // Set up initial grant region.
        //
        // `kernel_memory_break` is set to the end of kernel-accessible memory
        // and grows downward.
        //
        // We require the `kernel_memory_break` to be aligned to a
        // word-boundary, as we rely on this during offset calculations to
        // kernel-accessed structs (e.g. the grant pointer table) below. As it
        // moves downward in the address space, we can't use the `align_offset`
        // convenience functions.
        //
        // Calling `wrapping_sub` is safe here, as we've factored in an optional
        // padding of at most `sizeof(usize)` bytes in the calculation of
        // `initial_kernel_memory_size` above.
        let mut virtual_kernel_memory_break = allocated_kernel_memory
            .get_starting_pointer()
            .checked_add(allocated_kernel_memory.get_length())
            .unwrap();

        // Now that we know we have the space we can setup the grant pointers.
        virtual_kernel_memory_break = virtual_kernel_memory_break
            .checked_offset(NonZero::new(-(grant_ptrs_offset as isize)).unwrap())
            .unwrap();

        // This is safe, `kernel_memory_break` is aligned to a word-boundary,
        // and `grant_ptrs_offset` is a multiple of the word size.
        #[allow(clippy::cast_ptr_alignment)]
        // Set all grant pointers to null.
        let grant_pointers = slice::from_raw_parts_mut(
            virtual_kernel_memory_break.to_raw() as *mut GrantPointerEntry,
            grant_ptrs_num,
        );
        for grant_entry in grant_pointers.iter_mut() {
            grant_entry.driver_num = 0;
            grant_entry.grant_ptr = ptr::null_mut();
        }

        // Now that we know we have the space we can setup the memory for the
        // upcalls.
        virtual_kernel_memory_break = virtual_kernel_memory_break
            .checked_offset(const { NonZero::new(-(Self::CALLBACKS_OFFSET as isize)).unwrap() })
            .unwrap();

        // This is safe today, as MPU constraints ensure that `memory_start`
        // will always be aligned on at least a word boundary, and that
        // memory_size will be aligned on at least a word boundary, and
        // `grant_ptrs_offset` is a multiple of the word size. Thus,
        // `kernel_memory_break` must be word aligned. While this is unlikely to
        // change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set up ring buffer for upcalls to the process.
        let upcall_buf = slice::from_raw_parts_mut(
            virtual_kernel_memory_break.to_raw() as *mut Task,
            Self::CALLBACK_LEN,
        );
        let tasks = RingBuffer::new(upcall_buf);

        // Last thing in the kernel region of process RAM is the process struct.
        virtual_kernel_memory_break = virtual_kernel_memory_break
            .checked_offset(
                const { NonZero::new(-(Self::PROCESS_STRUCT_OFFSET as isize)).unwrap() },
            )
            .unwrap();
        let process_struct_memory_location = virtual_kernel_memory_break;

        // Ask the kernel for a unique identifier for this process that is being
        // created.
        let unique_identifier = kernel.create_process_identifier();

        // Save copies of these in case the app was compiled for fixed addresses
        // for later debugging.
        let fixed_address_flash = pb.header.get_fixed_address_flash();
        let fixed_address_ram = pb.header.get_fixed_address_ram();

        let kernel_flash_virtual_starting_pointer = kernel
            .translate_kernel_allocated_physical_pointer_byte_to_kernel_virtual_pointer_byte(
                app_flash_physical_starting_pointer.infallible_cast(),
            )
            .unwrap();

        let virtual_flash_slice = ImmutableKernelVirtualSlice::from_raw_parts(
            kernel_flash_virtual_starting_pointer.to_immutable(),
            app_flash_length_bytes,
        );

        let virtual_ram_slice = MutableKernelVirtualSlice::from_raw_parts(
            ram_kernel_starting_virtual_pointer,
            min_total_memory_size,
        );

        let process = ProcessStandard {
            process_id: Cell::new(ProcessId::new(kernel, unique_identifier, index)),
            app_id,
            kernel,
            chip,
            allow_high_water_mark: Cell::new(initial_allow_high_water_mark.get_address()),
            header: pb.header,
            kernel_memory_break: Cell::new(virtual_kernel_memory_break.get_address()),
            app_break: Cell::new(kernel_initial_app_brk.get_address()),
            grant_pointers: MapCell::new(grant_pointers),
            credential: pb.credential.get(),
            flash: virtual_flash_slice,
            ram: virtual_ram_slice,
            stored_state: MapCell::new(Default::default()),
            // Mark this process as approved and leave it to the kernel to start it.
            state: Cell::new(State::Yielded),
            fault_policy,
            restart_count: Cell::new(0),
            completion_code: OptionalCell::empty(),
            memory_configuration: valid_app_memory_configuration,
            tasks: MapCell::new(tasks),
            debug: D::default(),
            storage_permissions: OptionalCell::empty(),
        };

        if let Some(fix_addr_flash) = fixed_address_flash {
            process.debug.set_fixed_address_flash(fix_addr_flash);
        }
        if let Some(fix_addr_ram) = fixed_address_ram {
            process.debug.set_fixed_address_ram(fix_addr_ram);
        }

        let kernel_app_accessible_memory_starting_pointer =
            app_accessible_memory.get_starting_pointer().to_immutable();
        let user_app_accessible_memory_starting_pointer = kernel
            .translate_kernel_allocated_to_user_protected_byte(
                &process,
                kernel_app_accessible_memory_starting_pointer,
            )
            .unwrap();
        let user_initial_app_brk = kernel
            .translate_kernel_allocated_to_user_protected_byte(&process, kernel_initial_app_brk)
            .unwrap();

        // Handle any architecture-specific requirements for a new process.
        //
        // NOTE! We have to ensure that the start of process-accessible memory
        // (`app_memory_start`) is word-aligned. Since we currently start
        // process-accessible memory at the beginning of the allocated memory
        // region, we trust the MPU to give us a word-aligned starting address.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        match process.stored_state.map(|stored_state| {
            chip.userspace_kernel_boundary().initialize_process(
                &user_app_accessible_memory_starting_pointer,
                &user_initial_app_brk,
                stored_state,
            )
        }) {
            Some(Ok(())) => {}
            _ => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't initialize process",
                        pb.flash.as_ptr() as usize,
                        pb.flash.as_ptr() as usize + pb.flash.len() - 1,
                        process_name
                    );
                }
                // Note that since remaining_memory was split by split_at_mut into
                // application memory and unused_memory, a failure here will leak
                // the application memory. Not leaking it requires being able to
                // reconstitute the original memory slice.
                return Err(ProcessLoadError::InternalError);
            }
        }

        let flash_start = process.flash.get_starting_pointer();
        // `flash_start` is used to enqueue the initial task of a process. A task runs in user
        // space, so it needs user virtual pointers, not kernel virtual pointers.
        let flash_start = kernel
            .translate_kernel_allocated_to_user_protected_byte(&process, *flash_start)
            .unwrap();
        let app_start = flash_start
            .checked_add(NonZero::new(process.header.get_app_start_offset() as usize).unwrap())
            .unwrap()
            .get_address();
        let init_addr = flash_start
            .checked_add(NonZero::new(process.header.get_init_function_offset() as usize).unwrap())
            .unwrap()
            .get_address();
        let fn_base = flash_start.get_address();
        let fn_len = process.flash.get_length().get();

        // We need to construct a capability with sufficient authority to cover all of a user's
        // code, with permissions to execute it. The entirety of flash is sufficient.

        let init_fn = CapabilityPtr::new_with_authority(
            init_addr.get() as *const (),
            fn_base.get(),
            fn_len,
            CapabilityPtrPermissions::Execute,
        );

        // `ram_start` is used to enqueue the initial task of a process. A task runs in user
        // space, so it needs user virtual pointers, not kernel virtual pointers.
        let ram_start = kernel
            .translate_kernel_allocated_to_user_protected_byte(&process, process.get_ram_start())
            .unwrap();

        let app_brk = ImmutableKernelVirtualPointer::new_from_raw_byte(
            process.app_break.get().get() as *const u8,
        )
        .unwrap();
        // `app_brk` is used to enqueue the initial task of a process. A task runs in user
        // space, so it needs user virtual pointers, not kernel virtual pointers.
        let app_brk = kernel
            .translate_kernel_allocated_to_user_protected_byte(&process, app_brk)
            .unwrap();

        process.tasks.map(|tasks| {
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: app_start.get(),
                argument1: ram_start.get_address().get(),
                argument2: process.get_ram_length_bytes().get(),
                argument3: app_brk.get_address().get().into(),
            }));
        });

        process
            .storage_permissions
            .set(storage_permissions_policy.get_permissions(&process));

        let mut process_location = process_struct_memory_location
            .cast::<ProcessStandard<C, D>>()
            // TODO: Don't panic
            .unwrap();
        process_location.write(process);
        let process = &*process_location.to_raw();

        // Return the process object and a remaining memory for processes slice.
        Ok(Some(process))
    }

    /// Reset the process, resetting all of its state and re-initializing it so
    /// it can start running. Assumes the process is not running but is still in
    /// flash and still has its memory region allocated to it.
    fn reset(&self) -> Result<(), ErrorCode> {
        // We need a new process identifier for this process since the restarted
        // version is in effect a new process. This is also necessary to
        // invalidate any stored `ProcessId`s that point to the old version of
        // the process. However, the process has not moved locations in the
        // processes array, so we copy the existing index.
        let old_index = self.process_id.get().index;
        let new_identifier = self.kernel.create_process_identifier();
        self.process_id
            .set(ProcessId::new(self.kernel, new_identifier, old_index));

        // Reset debug information that is per-execution and not per-process.
        self.debug.reset_last_syscall();
        self.debug.reset_syscall_count();
        self.debug.reset_dropped_upcall_count();
        self.debug.reset_timeslice_expiration_count();

        todo!()

        /*
        // Reset MPU region configuration.
        //
        // TODO: ideally, this would be moved into a helper function used by
        // both create() and reset(), but process load debugging complicates
        // this. We just want to create new config with only flash and memory
        // regions.
        //
        // We must have a previous MPU configuration stored, fault the
        // process if this invariant is violated. We avoid allocating
        // a new MPU configuration, as this may eventually exhaust the
        // number of available MPU configurations.
        let mut mpu_config = self.mpu_config.take().ok_or(ErrorCode::FAIL)?;
        self.chip.mpu().reset_config(&mut mpu_config);

        // Allocate MPU region for flash.
        let app_mpu_flash = self.chip.mpu().allocate_region(
            self.flash.as_ptr(),
            self.flash.len(),
            self.flash.len(),
            mpu::Permissions::ReadExecuteOnly,
            &mut mpu_config,
        );
        if app_mpu_flash.is_none() {
            // We were unable to allocate an MPU region for flash. This is very
            // unexpected since we previously ran this process. However, we
            // return now and leave the process faulted and it will not be
            // scheduled.
            return Err(ErrorCode::FAIL);
        }

        // RAM

        // Re-determine the minimum amount of RAM the kernel must allocate to
        // the process based on the specific requirements of the syscall
        // implementation.
        let min_process_memory_size = self
            .chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // Recalculate initial_kernel_memory_size as was done in create()
        let grant_ptr_size = mem::size_of::<(usize, *mut u8)>();
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;
        let app_mpu_mem = self.chip.mpu().allocate_app_memory_region(
            self.mem_start(),
            self.memory_len,
            self.memory_len, //we want exactly as much as we had before restart
            min_process_memory_size,
            initial_kernel_memory_size,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        );
        let (app_mpu_mem_start, app_mpu_mem_len) = match app_mpu_mem {
            Some((start, len)) => (start, len),
            None => {
                // We couldn't configure the MPU for the process. This shouldn't
                // happen since we were able to start the process before, but at
                // this point it is better to leave the app faulted and not
                // schedule it.
                return Err(ErrorCode::NOMEM);
            }
        };

        // Reset memory pointers now that we know the layout of the process
        // memory and know that we can configure the MPU.

        // app_brk is set based on minimum syscall size above the start of
        // memory.
        let app_brk = app_mpu_mem_start.wrapping_add(min_process_memory_size);
        self.app_break.set(app_brk);
        // kernel_brk is calculated backwards from the end of memory the size of
        // the initial kernel data structures.
        let kernel_brk = app_mpu_mem_start
            .wrapping_add(app_mpu_mem_len)
            .wrapping_sub(initial_kernel_memory_size);
        self.kernel_memory_break.set(kernel_brk);
        // High water mark for `allow`ed memory is reset to the start of the
        // process's memory region.
        self.allow_high_water_mark.set(app_mpu_mem_start);

        // Store the adjusted MPU configuration:
        self.mpu_config.replace(mpu_config);

        // Handle any architecture-specific requirements for a process when it
        // first starts (as it would when it is new).
        let ukb_init_process = self.stored_state.map_or(Err(()), |stored_state| unsafe {
            self.chip.userspace_kernel_boundary().initialize_process(
                app_mpu_mem_start,
                app_brk,
                stored_state,
            )
        });
        match ukb_init_process {
            Ok(()) => {}
            Err(()) => {
                // We couldn't initialize the architecture-specific state for
                // this process. This shouldn't happen since the app was able to
                // be started before, but at this point the app is no longer
                // valid. The best thing we can do now is leave the app as still
                // faulted and not schedule it.
                return Err(ErrorCode::RESERVE);
            }
        };

        self.restart_count.increment();

        // Mark the state as `Yielded` for the scheduler.
        self.state.set(State::Yielded);

        // And queue up this app to be restarted.
        let flash_start = self.flash_start();
        let app_start =
            flash_start.wrapping_add(self.header.get_app_start_offset() as usize) as usize;
        let init_addr =
            flash_start.wrapping_add(self.header.get_init_function_offset() as usize) as usize;

        // We need to construct a capability with sufficient authority to cover all of a user's
        // code, with permissions to execute it. The entirety of flash is sufficient.

        let init_fn = unsafe {
            CapabilityPtr::new_with_authority(
                init_addr as *const (),
                flash_start as usize,
                (self.flash_end() as usize) - (flash_start as usize),
                CapabilityPtrPermissions::Execute,
            )
        };

        self.enqueue_task(Task::FunctionCall(FunctionCall {
            source: FunctionCallSource::Kernel,
            pc: init_fn,
            argument0: app_start,
            argument1: self.memory_start as usize,
            argument2: self.memory_len,
            argument3: (self.app_break.get() as usize).into(),
        }))
        */
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// is within the RAM bounds currently exposed to the processes (i.e. ending
    /// at `app_break`). If this method returns `true`, the buffer is guaranteed
    /// to be accessible to the process and to not overlap with the grant
    /// region.
    fn in_app_owned_memory(
        &self,
        buf_start_addr: &ImmutableKernelVirtualPointer<u8>,
        size: NonZero<usize>,
    ) -> bool {
        // TODO: On some platforms, CapabilityPtr has sufficient authority that we
        // could skip this check.
        // CapabilityPtr needs to make it slightly further, and we need to add
        // interfaces that tell us how much assurance it gives on the current
        // platform.
        // TODO: This shouldn't panic
        let buf_end_addr = buf_start_addr.checked_add(size).unwrap();

        &buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.get_ram_start().as_immutable()
            && buf_end_addr.get_address() <= self.app_memory_break()
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// are within the readable region of an application's flash memory.  If
    /// this method returns true, the buffer is guaranteed to be readable to the
    /// process.
    fn in_app_flash_memory(
        &self,
        buf_start_addr: &ImmutableKernelVirtualPointer<u8>,
        size: NonZero<usize>,
    ) -> bool {
        // TODO: On some platforms, CapabilityPtr has sufficient authority that we
        // could skip this check.
        // CapabilityPtr needs to make it slightly further, and we need to add
        // interfaces that tell us how much assurance it gives on the current
        // platform.
        // TODO: This shouldn't panic
        let buf_end_addr = buf_start_addr.checked_add(size).unwrap();

        &buf_end_addr >= buf_start_addr
            && buf_start_addr >= &self.flash_non_protected_start()
            && buf_end_addr <= self.flash_end()
    }

    /// Reset all `grant_ptr`s to NULL.
    unsafe fn grant_ptrs_reset(&self) {
        self.grant_pointers.map(|grant_pointers| {
            for grant_entry in grant_pointers.iter_mut() {
                grant_entry.driver_num = 0;
                grant_entry.grant_ptr = ptr::null_mut();
            }
        });
    }

    /// Allocate memory in a process's grant region.
    ///
    /// Ensures that the allocation is of `size` bytes and aligned to `align`
    /// bytes.
    ///
    /// If there is not enough memory, or the MPU cannot isolate the process
    /// accessible region from the new kernel memory break after doing the
    /// allocation, then this will return `None`.
    fn allocate_in_grant_region_internal(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // First, compute the candidate new pointer. Note that at this point
        // we have not yet checked whether there is space for this
        // allocation or that it meets alignment requirements.
        let new_break_unaligned = self.kernel_memory_break().get().wrapping_sub(size);

        // Our minimum alignment requirement is two bytes, so that the
        // lowest bit of the address will always be zero and we can use it
        // as a flag. It doesn't hurt to increase the alignment (except for
        // potentially a wasted byte) so we make sure `align` is at least
        // two.
        let align = cmp::max(align, 2);

        // The alignment must be a power of two, 2^a. The expression
        // `!(align - 1)` then returns a mask with leading ones, followed by
        // `a` trailing zeros.
        let alignment_mask = !(align - 1);
        let raw_new_break = new_break_unaligned & alignment_mask;
        // PANIC: a pointer to u8 is always aligned
        // SAFETY: raw_new_break is a valid virtual pointer.
        let new_break =
            unsafe { ImmutableVirtualPointer::new_from_raw_byte(raw_new_break as *const u8) }
                .unwrap();
        let raw_aligned_new_break = align_down_usize(raw_new_break, Page4KiB::SIZE_U8) as *const u8;
        // PANIC: a pointer to u8 is always aligned
        // SAFETY: raw_new_break is a valid virtual pointer.
        let aligned_new_break =
            unsafe { ImmutableVirtualPointer::new_from_raw_byte(raw_aligned_new_break) }.unwrap();

        // Verify there is space for this allocation
        if aligned_new_break.get_address() < self.app_memory_break() {
            None
            // Verify it didn't wrap around
        } else if aligned_new_break.get_address() > self.kernel_memory_break() {
            None
        } else {
            // Allocation is valid.

            // We always allocate down, so we must lower the
            // kernel_memory_break.
            self.kernel_memory_break.set(new_break.get_address());

            // We need `grant_ptr` as a mutable pointer.
            let grant_ptr = raw_new_break as *mut u8;

            // ### Safety
            //
            // Here we are guaranteeing that `grant_ptr` is not null. We can
            // ensure this because we just created `grant_ptr` based on the
            // process's allocated memory, and we know it cannot be null.
            unsafe { Some(NonNull::new_unchecked(grant_ptr)) }
        }
    }

    /// Create the identifier for a custom grant that grant.rs uses to access
    /// the custom grant.
    ///
    /// We create this identifier by calculating the number of bytes between
    /// where the custom grant starts and the end of the process memory.
    fn create_custom_grant_identifier(&self, ptr: NonNull<u8>) -> ProcessCustomGrantIdentifier {
        let custom_grant_address = ptr.as_ptr() as usize;
        let process_memory_end = self.get_ram_end().get_address();

        ProcessCustomGrantIdentifier {
            offset: process_memory_end.get() - custom_grant_address,
        }
    }

    /// Use a `ProcessCustomGrantIdentifier` to find the address of the
    /// custom grant.
    ///
    /// This reverses `create_custom_grant_identifier()`.
    fn get_custom_grant_address(&self, identifier: ProcessCustomGrantIdentifier) -> usize {
        let process_memory_end = self.get_ram_end().get_address();

        // Subtract the offset in the identifier from the end of the process
        // memory to get the address of the custom grant.
        process_memory_end.get() - identifier.offset
    }

    /// Return the app's read and modify storage permissions from the TBF header
    /// if it exists.
    ///
    /// If the header does not exist then return `None`. If the header does
    /// exist, this returns a 5-tuple with:
    ///
    /// - `write_allowed`: bool. If this process should have write permissions.
    /// - `read_count`: usize. How many read IDs are valid.
    /// - `read_ids`: [u32]. The read IDs.
    /// - `modify_count`: usze. How many modify IDs are valid.
    /// - `modify_ids`: [u32]. The modify IDs.
    pub fn get_tbf_storage_permissions(&self) -> Option<(bool, usize, [u32; 8], usize, [u32; 8])> {
        let read_perms = self.header.get_storage_read_ids();
        let modify_perms = self.header.get_storage_modify_ids();

        match (read_perms, modify_perms) {
            (Some((read_count, read_ids)), Some((modify_count, modify_ids))) => Some((
                self.header.get_storage_write_id().is_some(),
                read_count,
                read_ids,
                modify_count,
                modify_ids,
            )),
            _ => None,
        }
    }

    fn get_ram_region(&self) -> &UserMappedProtectedAllocatedRegion<Page4KiB> {
        self.get_memory_configuration()
            .get_ram_region()
            .as_mapped_protected_allocated_region()
    }

    fn get_ram_start(&self) -> MutableKernelVirtualPointer<u8> {
        *self.ram.get_starting_pointer()
    }

    fn get_ram_end(&self) -> MutableKernelVirtualPointer<u8> {
        self.ram.get_ending_pointer()
    }

    fn get_ram_length_bytes(&self) -> NonZero<usize> {
        self.ram.get_length()
    }

    /// The start address of the flash region allocated for this process.
    fn flash_start(&self) -> &ImmutableKernelVirtualPointer<u8> {
        self.flash.get_starting_pointer()
    }

    /// Get the first address of process's flash that isn't protected by the
    /// kernel. The protected range of flash contains the TBF header and
    /// potentially other state the kernel is storing on behalf of the process,
    /// and cannot be edited by the process.
    fn flash_non_protected_start(&self) -> ImmutableKernelVirtualPointer<u8> {
        let flash_start = self.flash_start();
        // CAST: size_of::<usize>() >= size_of::<u32>() on 32 and 64-bit architectures
        flash_start
            .checked_add(NonZero::new(self.header.get_protected_size() as usize).unwrap())
            .unwrap()
    }

    /// The first address after the end of the flash region allocated for this
    /// process.
    fn flash_end(&self) -> ImmutableKernelVirtualPointer<u8> {
        self.flash.get_ending_pointer()
    }

    /// The lowest address of the grant region for the process.
    fn kernel_memory_break(&self) -> NonZero<usize> {
        self.kernel_memory_break.get()
    }

    /// Return the highest address the process has access to, or the current
    /// process memory brk.
    fn app_memory_break(&self) -> NonZero<usize> {
        self.app_break.get()
    }

    fn internal_brk(
        &self,
        new_break: ImmutableKernelVirtualPointer<u8>,
    ) -> Result<CapabilityPtr, Error> {
        // Do not modify an inactive process.
        if !self.is_running() {
            return Err(Error::InactiveApp);
        }

        // CAST: TODO
        let raw_aligned_new_break =
            align_up_usize(new_break.get_address().get(), Page4KiB::SIZE_U8) as *const u8;
        // PANIC: a pointer to u8 is always aligned
        // TODO: this may panic if `raw_aligned_new_break` is null.
        // SAFETY: new_break is a valid virtual pointer.
        let aligned_new_break =
            unsafe { ImmutableVirtualPointer::new_from_raw_byte(raw_aligned_new_break) }.unwrap();

        if new_break.get_address() < self.allow_high_water_mark.get() {
            Err(Error::AddressOutOfBounds)
        } else if aligned_new_break.get_address() > self.kernel_memory_break() {
            Err(Error::OutOfMemory)
        } else {
            let old_break_address = self.app_memory_break();
            let new_break_address = new_break.get_address();
            let ram_start_address = self.get_ram_start().get_address();

            let new_length_bytes =
                match new_break_address.get().checked_sub(ram_start_address.get()) {
                    None => return Err(Error::OutOfMemory),
                    Some(new_length_bytes) => new_length_bytes,
                };

            let non_zero_new_length_bytes = match NonZero::new(new_length_bytes) {
                None => todo!(),
                Some(non_zero_new_length_bytes) => non_zero_new_length_bytes,
            };

            let new_length_granules =
                ceil_non_zero_usize(non_zero_new_length_bytes, Page4KiB::SIZE_U8);

            let ram_region = self.memory_configuration.get_ram_region();
            if ram_region.resize(new_length_granules).is_err() {
                return Err(Error::OutOfMemory);
            }

            self.app_break.set(new_break.get_address());
            let base = self.get_ram_start().get_address();
            let break_result = unsafe {
                CapabilityPtr::new_with_authority(
                    old_break_address.get() as *const (),
                    base.get(),
                    new_break.get_address().get() - base.get(),
                    CapabilityPtrPermissions::ReadWrite,
                )
            };

            Ok(break_result)
        }
    }
}
