// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Types for Tock-compatible processes.

use core::fmt;
use core::fmt::Write;
use core::num::NonZeroU32;
use core::ptr::NonNull;
use core::str;

use crate::capabilities;
use crate::errorcode::ErrorCode;
use crate::ipc;
use crate::kernel::Kernel;
use crate::platform::mpu::{self};
use crate::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use crate::storage_permissions;
use crate::syscall::{self, Syscall, SyscallReturn};
use crate::upcall::UpcallId;
use tock_tbf::types::CommandPermissions;

// Export all process related types via `kernel::process::`.
pub use crate::process_binary::ProcessBinary;
pub use crate::process_checker::{ProcessCheckerMachine, ProcessCheckerMachineClient};
pub use crate::process_loading::load_processes;
pub use crate::process_loading::ProcessLoadError;
pub use crate::process_loading::SequentialProcessLoaderMachine;
pub use crate::process_loading::{ProcessLoadingAsync, ProcessLoadingAsyncClient};
pub use crate::process_policies::ProcessFaultPolicy;
pub use crate::process_printer::{ProcessPrinter, ProcessPrinterContext};
pub use crate::process_standard::ProcessStandard;

/// Userspace process identifier.
///
/// This is an opaque type that can be used to represent a running process on
/// the board without requiring an actual reference to a `Process` object.
/// Having this `ProcessId` reference type is useful for managing ownership and
/// type issues in Rust, but more importantly `ProcessId` serves as a tool for
/// capsules to hold pointers to applications.
///
/// Since `ProcessId` implements `Copy`, having an `ProcessId` does _not_ ensure
/// that the process the `ProcessId` refers to is still valid. The process may
/// have been removed, terminated, or restarted as a new process. Therefore, all
/// uses of `ProcessId` in the kernel must check that the `ProcessId` is still
/// valid. This check happens automatically when `.index()` is called, as noted
/// by the return type: `Option<usize>`. `.index()` will return the index of the
/// process in the processes array, but if the process no longer exists then
/// `None` is returned.
///
/// Outside of the kernel crate, holders of an `ProcessId` may want to use
/// `.id()` to retrieve a simple identifier for the process that can be
/// communicated over a UART bus or syscall interface. This call is guaranteed
/// to return a suitable identifier for the `ProcessId`, but does not check that
/// the corresponding application still exists.
///
/// This type also provides capsules an interface for interacting with processes
/// since they otherwise would have no reference to a `Process`. Very limited
/// operations are available through this interface since capsules should not
/// need to know the details of any given process. However, certain information
/// makes certain capsules possible to implement. For example, capsules can use
/// the `get_editable_flash_range()` function so they can safely allow an app to
/// modify its own flash.
#[derive(Clone, Copy)]
pub struct ProcessId {
    /// Reference to the main kernel struct. This is needed for checking on
    /// certain properties of the referred app (like its editable bounds), but
    /// also for checking that the index is valid.
    pub(crate) kernel: &'static Kernel,

    /// The index in the kernel.PROCESSES[] array where this app's state is
    /// stored. This makes for fast lookup of the process and helps with
    /// implementing IPC.
    ///
    /// This value is crate visible to enable optimizations in sched.rs. Other
    /// users should call `.index()` instead.
    pub(crate) index: usize,

    /// The unique identifier for this process. This can be used to refer to the
    /// process in situations where a single number is required, for instance
    /// when referring to specific applications across the syscall interface.
    ///
    /// The combination of (index, identifier) is used to check if the app this
    /// `ProcessId` refers to is still valid. If the stored identifier in the
    /// process at the given index does not match the value saved here, then the
    /// process moved or otherwise ended, and this `ProcessId` is no longer
    /// valid.
    identifier: usize,
}

impl PartialEq for ProcessId {
    fn eq(&self, other: &ProcessId) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for ProcessId {}

impl fmt::Debug for ProcessId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // We handle alignment and width.
        if let Some(width) = formatter.width() {
            match formatter.align() {
                Some(fmt::Alignment::Left) => {
                    write!(formatter, "{:<width$}", self.identifier, width = width)
                }
                Some(fmt::Alignment::Right) => {
                    write!(formatter, "{:width$}", self.identifier, width = width)
                }
                Some(fmt::Alignment::Center) => {
                    write!(formatter, "{:^width$}", self.identifier, width = width)
                }
                None => write!(formatter, "{:width$}", self.identifier, width = width),
            }
        } else {
            // Otherwise just do default.
            write!(formatter, "{}", self.identifier)
        }
    }
}

impl ProcessId {
    /// Create a new `ProcessId` object based on the app identifier and its
    /// index in the processes array.
    pub(crate) fn new(kernel: &'static Kernel, identifier: usize, index: usize) -> ProcessId {
        ProcessId {
            kernel,
            index,
            identifier,
        }
    }

    /// Create a new `ProcessId` object based on the app identifier and its
    /// index in the processes array.
    ///
    /// This constructor is public but protected with a capability so that
    /// external implementations of `Process` can use it.
    pub fn new_external(
        kernel: &'static Kernel,
        identifier: usize,
        index: usize,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) -> ProcessId {
        ProcessId {
            kernel,
            index,
            identifier,
        }
    }

    /// Get the location of this app in the processes array.
    ///
    /// This will return `Some(index)` if the identifier stored in this
    /// `ProcessId` matches the app saved at the known index. If the identifier
    /// does not match then `None` will be returned.
    pub(crate) fn index(&self) -> Option<usize> {
        // Do a lookup to make sure that the index we have is correct.
        if self.kernel.processid_is_valid(self) {
            Some(self.index)
        } else {
            None
        }
    }

    /// Get a `usize` unique identifier for the app this `ProcessId` refers to.
    ///
    /// This function should not generally be used, instead code should just use
    /// the `ProcessId` object itself to refer to various apps on the system.
    /// However, getting just a `usize` identifier is particularly useful when
    /// referring to a specific app with things outside of the kernel, say for
    /// userspace (e.g. IPC) or tockloader (e.g. for debugging) where a concrete
    /// number is required.
    ///
    /// Note, this will always return the saved unique identifier for the app
    /// originally referred to, even if that app no longer exists. For example,
    /// the app may have restarted, or may have been ended or removed by the
    /// kernel. Therefore, calling `id()` is _not_ a valid way to check that an
    /// application still exists.
    pub fn id(&self) -> usize {
        self.identifier
    }

    /// Get the `ShortId` for this application this process is an execution of.
    ///
    /// The `ShortId` is an identifier for the _application_, not the particular
    /// execution (i.e. the currently running process). This makes `ShortId`
    /// distinct from `ProcessId`.
    ///
    /// This function is a helper function as capsules typically use `ProcessId`
    /// as a handle to the running process and corresponding app.
    pub fn short_app_id(&self) -> ShortId {
        self.kernel
            .process_map_or(ShortId::LocallyUnique, *self, |process| {
                process.short_app_id()
            })
    }

    /// Returns the full address of the start and end of the flash region that
    /// the app owns and can write to. This includes the app's code and data and
    /// any padding at the end of the app. It does not include the TBF header,
    /// or any space that the kernel is using for any potential bookkeeping.
    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        self.kernel.process_map_or((0, 0), *self, |process| {
            let addresses = process.get_addresses();
            (addresses.flash_non_protected_start, addresses.flash_end)
        })
    }

    /// Get the storage permissions for the process. These permissions indicate
    /// what the process is allowed to read and write. Returns `None` if the
    /// process has no storage permissions.
    pub fn get_storage_permissions(&self) -> Option<storage_permissions::StoragePermissions> {
        self.kernel
            .process_map_or(None, *self, |process| process.get_storage_permissions())
    }
}

/// A compressed form of an Application Identifier.
///
/// ShortIds are useful for more efficient operations with app identifiers
/// within the kernel. They are guaranteed to be unique among all running
/// processes on the same board. However, as they are only 32 bits they are not
/// globally unique.
///
/// ShortIds are persistent across restarts of the same app (whereas ProcessIDs
/// are not).
///
/// As ShortIds must be unique for each app on a board, and since not every
/// platform may have a use for ShortIds, the definition of a ShortId provides a
/// convenient mechanism for meeting the uniqueness requirement without actually
/// requiring assigning unique discrete values to each app. This is done with
/// the `LocallyUnique` variant which is an abstract ID that is guaranteed to be
/// unique (i.e. an equality comparison with any other ShortId will always
/// return `false`). Platforms which have a use for an actual number for a
/// `ShortId` should use the `Fixed(NonZeroU32)` variant. Note, for type space
/// efficiency, we disallow using the number 0 as a fixed ShortId.
///
/// ShortIds are assigned to the app as part of the credential checking process.
/// Specifically, an implementation of the `process_checker::Compress` trait
/// assigns ShortIds.
#[derive(Clone, Copy)]
pub enum ShortId {
    /// An abstract `ShortId` that is always guaranteed to be unique. As this is
    /// not an actual discrete value, it cannot be used for anything other than
    /// meeting the uniqueness requirement.
    LocallyUnique,
    /// A 32 bit number `ShortId`. This fixed value is guaranteed to be unique
    /// among all running processes as the kernel will not start two processes
    /// with the same ShortId.
    Fixed(core::num::NonZeroU32),
}

impl PartialEq for ShortId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ShortId::Fixed(a), ShortId::Fixed(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for ShortId {}

impl core::convert::From<Option<core::num::NonZeroU32>> for ShortId {
    fn from(id: Option<core::num::NonZeroU32>) -> ShortId {
        match id {
            Some(fixed) => ShortId::Fixed(fixed),
            None => ShortId::LocallyUnique,
        }
    }
}

impl core::fmt::Display for ShortId {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> fmt::Result {
        match *self {
            ShortId::LocallyUnique => {
                write!(fmt, "Unique")
            }
            ShortId::Fixed(id) => {
                write!(fmt, "0x{:<8x} ", id)
            }
        }
    }
}

/// Enum used to inform scheduler why a process stopped executing (aka why
/// `do_process()` returned).
///
/// This is publicly exported to allow for schedulers implemented outside of the
/// kernel crate.
#[derive(PartialEq, Eq)]
pub enum StoppedExecutingReason {
    /// The process returned because it is no longer ready to run.
    NoWorkLeft,

    /// The process faulted, and the board restart policy was configured such
    /// that it was not restarted and there was not a kernel panic.
    StoppedFaulted,

    /// The kernel stopped the process.
    Stopped,

    /// The process was preempted because its timeslice expired.
    TimesliceExpired,

    /// The process returned because it was preempted by the kernel. This can
    /// mean that kernel work became ready (most likely because an interrupt
    /// fired and the kernel thread needs to execute the bottom half of the
    /// interrupt), or because the scheduler no longer wants to execute that
    /// process.
    KernelPreemption,
}

/// The version of a binary.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BinaryVersion(NonZeroU32);

impl BinaryVersion {
    /// Creates a new binary version.
    pub fn new(value: NonZeroU32) -> Self {
        Self(value)
    }
}

/// This trait represents a generic process that the Tock scheduler can
/// schedule.
pub trait Process {
    /// Returns the process's identifier.
    fn processid(&self) -> ProcessId;

    /// Returns the ShortId generated by the application binary checker at
    /// loading.
    fn short_app_id(&self) -> ShortId;

    /// Returns the version number of the binary in this process, as specified
    /// in a TBF Program Header; if the binary has no version assigned, return [None]
    fn binary_version(&self) -> Option<BinaryVersion>;

    /// Returns how many times this process has been restarted.
    fn get_restart_count(&self) -> usize;

    /// Get the name of the process. Used for IPC.
    fn get_process_name(&self) -> &'static str;

    /// Return if there are any Tasks (upcalls/IPC requests) enqueued for the
    /// process.
    fn has_tasks(&self) -> bool;

    /// Returns the number of pending tasks. If 0 then `dequeue_task()` will
    /// return `None` when called.
    fn pending_tasks(&self) -> usize;

    /// Queue a `Task` for the process. This will be added to a per-process
    /// buffer and executed by the scheduler. `Task`s are some function the app
    /// should run, for example a upcall or an IPC call.
    ///
    /// This function returns:
    /// - `Ok(())` if the `Task` was successfully enqueued.
    /// - `Err(ErrorCode::NODEVICE)` if the process is no longer alive.
    /// - `Err(ErrorCode::NOMEM)` if the task could not be enqueued because
    ///   there is insufficient space in the internal task queue. is returned.
    /// Other return values must be treated as kernel-internal errors.
    fn enqueue_task(&self, task: Task) -> Result<(), ErrorCode>;

    /// Remove the scheduled operation from the front of the queue and return it
    /// to be handled by the scheduler.
    ///
    /// If there are no `Task`s in the queue for this process this will return
    /// `None`.
    fn dequeue_task(&self) -> Option<Task>;

    /// Search the work queue for a specific upcall_id. If it is present,
    /// return the associated `Task`, otherwise return `None`.
    fn remove_upcall(&self, upcall_id: UpcallId) -> Option<Task>;

    /// Remove all scheduled upcalls for a given upcall id from the task queue.
    fn remove_pending_upcalls(&self, upcall_id: UpcallId);

    /// Returns the current state the process is in. Common states are "running"
    /// or "yielded".
    fn get_state(&self) -> State;

    /// Returns whether this process is ready to execute.
    fn ready(&self) -> bool;

    /// Returns whether the process is running (has active stack frames) or not
    /// (has never run, has faulted, or has completed).
    fn is_running(&self) -> bool;

    /// Move this process from the running state to the yielded state.
    ///
    /// This will fail (i.e. not do anything) if the process was not previously
    /// running.
    fn set_yielded_state(&self);

    /// Move this process from the running state to the yielded-for state.
    ///
    /// This will fail (i.e. not do anything) if the process was not previously
    /// running.
    fn set_yielded_for_state(&self, upcall_id: UpcallId);

    /// Move this process from running or yielded state into the stopped state.
    ///
    /// This will fail (i.e. not do anything) if the process was not either
    /// running or yielded.
    fn stop(&self);

    /// Move this stopped process back into its original state.
    ///
    /// This transitions a process from `StoppedRunning` -> `Running` or
    /// `StoppedYielded` -> `Yielded`.
    fn resume(&self);

    /// Put this process in the fault state. The kernel will use its process
    /// fault policy to decide what action to take in regards to the faulted
    /// process.
    fn set_fault_state(&self);

    /// Start a terminated process. This function can only be called on a
    /// terminated process.
    ///
    /// The caller MUST verify this process is unique before calling this
    /// function. This requires a capability to call to ensure that the caller
    /// have verified that this process is unique before trying to start it.
    fn start(&self, cap: &dyn crate::capabilities::ProcessStartCapability);

    /// Terminates and attempts to restart the process. The process and current
    /// application always terminate. The kernel may, based on its own policy,
    /// restart the application using the same process, reuse the process for
    /// another application, or simply terminate the process and application.
    ///
    /// This function can be called when the process is in any state except for
    /// `Terminated`. It attempts to reset all process state and re-initialize
    /// it so that it can be reused.
    ///
    /// Restarting an application can fail for three general reasons:
    ///
    /// 1. The process is already terminated. Use `start()` instead.
    ///
    /// 2. The kernel chooses not to restart the application, based on its
    ///    policy.
    ///
    /// 3. The kernel decides to restart the application but fails to do so
    ///    because some state can no long be configured for the process. For
    ///    example, the syscall state for the process fails to initialize.
    ///
    /// After `restart()` runs the process will either be queued to run its the
    /// application's `_start` function, terminated, or queued to run a
    /// different application's `_start` function.
    ///
    /// As the process will be terminated before being restarted, this function
    /// accepts an optional `completion_code`. If the process provided a
    /// completion code (e.g. via the exit syscall), then this should be called
    /// with `Some(u32)`. If the kernel is trying to restart the process and the
    /// process did not provide a completion code, then this should be called
    /// with `None`.
    fn try_restart(&self, completion_code: Option<u32>);

    /// Stop and clear a process's state and put it into the `Terminated` state.
    ///
    /// This will end the process, but does not reset it such that it could be
    /// restarted and run again. This function instead frees grants and any
    /// queued tasks for this process, but leaves the debug information about
    /// the process and other state intact.
    ///
    /// When a process is terminated, an optional `completion_code` should be
    /// stored for the process. If the process provided the completion code
    /// (e.g. via the exit syscall), then this function should be called with a
    /// completion code of `Some(u32)`. If the kernel is terminating the process
    /// and therefore has no completion code from the process, it should provide
    /// `None`.
    fn terminate(&self, completion_code: Option<u32>);

    /// Get the completion code if the process has previously terminated.
    ///
    /// If the process has never terminated then there has been no opportunity
    /// for a completion code to be set, and this will return `None`.
    ///
    /// If the process has previously terminated this will return `Some()`. If
    /// the last time the process terminated it did not provide a completion
    /// code (e.g. the process faulted), then this will return `Some(None)`. If
    /// the last time the process terminated it did provide a completion code,
    /// this will return `Some(Some(completion_code))`.
    fn get_completion_code(&self) -> Option<Option<u32>>;

    // memop operations

    /// Change the location of the program break and reallocate the MPU region
    /// covering program memory.
    ///
    /// This will fail with an error if the process is no longer active. An
    /// inactive process will not run again without being reset, and changing
    /// the memory pointers is not valid at this point.
    fn brk(&self, new_break: *const u8) -> Result<*const u8, Error>;

    /// Change the location of the program break, reallocate the MPU region
    /// covering program memory, and return the previous break address.
    ///
    /// This will fail with an error if the process is no longer active. An
    /// inactive process will not run again without being reset, and changing
    /// the memory pointers is not valid at this point.
    fn sbrk(&self, increment: isize) -> Result<*const u8, Error>;

    /// How many writeable flash regions defined in the TBF header for this
    /// process.
    fn number_writeable_flash_regions(&self) -> usize;

    /// Get the offset from the beginning of flash and the size of the defined
    /// writeable flash region.
    fn get_writeable_flash_region(&self, region_index: usize) -> (u32, u32);

    /// Debug function to update the kernel on where the stack starts for this
    /// process. Processes are not required to call this through the memop
    /// system call, but it aids in debugging the process.
    fn update_stack_start_pointer(&self, stack_pointer: *const u8);

    /// Debug function to update the kernel on where the process heap starts.
    /// Also optional.
    fn update_heap_start_pointer(&self, heap_pointer: *const u8);

    /// Creates a [`ReadWriteProcessBuffer`] from the given offset and size in
    /// process memory.
    ///
    /// ## Returns
    ///
    /// In case of success, this method returns the created
    /// [`ReadWriteProcessBuffer`].
    ///
    /// In case of an error, an appropriate `ErrorCode` is returned:
    ///
    /// - If the memory is not contained in the process-accessible memory space
    ///   / `buf_start_addr` and `size` are not a valid read-write buffer (any
    ///   byte in the range is not read/write accessible to the process):
    ///   [`ErrorCode::INVAL`].
    /// - If the process is not active: [`ErrorCode::FAIL`].
    /// - For all other errors: [`ErrorCode::FAIL`].
    fn build_readwrite_process_buffer(
        &self,
        buf_start_addr: *mut u8,
        size: usize,
    ) -> Result<ReadWriteProcessBuffer, ErrorCode>;

    /// Creates a [`ReadOnlyProcessBuffer`] from the given offset and size in
    /// process memory.
    ///
    /// ## Returns
    ///
    /// In case of success, this method returns the created
    /// [`ReadOnlyProcessBuffer`].
    ///
    /// In case of an error, an appropriate ErrorCode is returned:
    ///
    /// - If the memory is not contained in the process-accessible memory space
    ///   / `buf_start_addr` and `size` are not a valid read-only buffer (any
    ///   byte in the range is not read-accessible to the process):
    ///   [`ErrorCode::INVAL`].
    /// - If the process is not active: [`ErrorCode::FAIL`].
    /// - For all other errors: [`ErrorCode::FAIL`].
    fn build_readonly_process_buffer(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> Result<ReadOnlyProcessBuffer, ErrorCode>;

    /// Set a single byte within the process address space at `addr` to `value`.
    /// Return true if `addr` is within the RAM bounds currently exposed to the
    /// process (thereby writable by the process itself) and the value was set,
    /// false otherwise.
    ///
    /// ### Safety
    ///
    /// This function verifies that the byte to be written is in the process's
    /// accessible memory. However, to avoid undefined behavior the caller needs
    /// to ensure that no other references exist to the process's memory before
    /// calling this function.
    unsafe fn set_byte(&self, addr: *mut u8, value: u8) -> bool;

    /// Return the permissions for this process for a given `driver_num`.
    ///
    /// The returned `CommandPermissions` will indicate if any permissions for
    /// individual command numbers are specified. If there are permissions set
    /// they are returned as a 64 bit bitmask for sequential command numbers.
    /// The offset indicates the multiple of 64 command numbers to get
    /// permissions for.
    fn get_command_permissions(&self, driver_num: usize, offset: usize) -> CommandPermissions;

    /// Get the storage permissions for the process.
    ///
    /// Returns `None` if the process has no storage permissions.
    fn get_storage_permissions(&self) -> Option<storage_permissions::StoragePermissions>;

    // mpu

    /// Configure the MPU to use the process's allocated regions.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    fn setup_mpu(&self);

    /// Allocate a new MPU region for the process that is at least
    /// `min_region_size` bytes and lies within the specified stretch of
    /// unallocated memory.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    fn add_mpu_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
    ) -> Option<mpu::Region>;

    /// Removes an MPU region from the process that has been previously added
    /// with `add_mpu_region`.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    fn remove_mpu_region(&self, region: mpu::Region) -> Result<(), ErrorCode>;

    // grants

    /// Allocate memory from the grant region and store the reference in the
    /// proper grant pointer index.
    ///
    /// This function must check that doing the allocation does not cause the
    /// kernel memory break to go below the top of the process accessible memory
    /// region allowed by the MPU. Note, this can be different from the actual
    /// app_brk, as MPU alignment and size constraints may result in the MPU
    /// enforced region differing from the app_brk.
    ///
    /// This will return `Err(())` and fail if:
    /// - The process is inactive, or
    /// - There is not enough available memory to do the allocation, or
    /// - The grant_num is invalid, or
    /// - The grant_num already has an allocated grant.
    fn allocate_grant(
        &self,
        grant_num: usize,
        driver_num: usize,
        size: usize,
        align: usize,
    ) -> Result<(), ()>;

    /// Check if a given grant for this process has been allocated.
    ///
    /// Returns `None` if the process is not active. Otherwise, returns `true`
    /// if the grant has been allocated, `false` otherwise.
    fn grant_is_allocated(&self, grant_num: usize) -> Option<bool>;

    /// Allocate memory from the grant region that is `size` bytes long and
    /// aligned to `align` bytes. This is used for creating custom grants which
    /// are not recorded in the grant pointer array, but are useful for capsules
    /// which need additional process-specific dynamically allocated memory.
    ///
    /// If successful, return a Ok() with an identifier that can be used with
    /// `enter_custom_grant()` to get access to the memory and the pointer to
    /// the memory which must be used to initialize the memory.
    fn allocate_custom_grant(
        &self,
        size: usize,
        align: usize,
    ) -> Result<(ProcessCustomGrantIdentifier, NonNull<u8>), ()>;

    /// Enter the grant based on `grant_num` for this process.
    ///
    /// Entering a grant means getting access to the actual memory for the
    /// object stored as the grant.
    ///
    /// This will return an `Err` if the process is inactive of the `grant_num`
    /// is invalid, if the grant has not been allocated, or if the grant is
    /// already entered. If this returns `Ok()` then the pointer points to the
    /// previously allocated memory for this grant.
    fn enter_grant(&self, grant_num: usize) -> Result<NonNull<u8>, Error>;

    /// Enter a custom grant based on the `identifier`.
    ///
    /// This retrieves a pointer to the previously allocated custom grant based
    /// on the identifier returned when the custom grant was allocated.
    ///
    /// This returns an error if the custom grant is no longer accessible, or if
    /// the process is inactive.
    fn enter_custom_grant(
        &self,
        identifier: ProcessCustomGrantIdentifier,
    ) -> Result<*mut u8, Error>;

    /// Opposite of `enter_grant()`. Used to signal that the grant is no longer
    /// entered.
    ///
    /// If `grant_num` is valid, this function cannot fail. If `grant_num` is
    /// invalid, this function will do nothing. If the process is inactive then
    /// grants are invalid and are not entered or not entered, and this function
    /// will do nothing.
    ///
    /// ### Safety
    ///
    /// The caller must ensure that no references to the memory inside the grant
    /// exist after calling `leave_grant()`. Otherwise, it would be possible to
    /// effectively enter the grant twice (once using the existing reference,
    /// once with a new call to `enter_grant()`) which breaks the memory safety
    /// requirements of grants.
    unsafe fn leave_grant(&self, grant_num: usize);

    /// Return the count of the number of allocated grant pointers if the
    /// process is active. This does not count custom grants. This is used to
    /// determine if a new grant has been allocated after a call to
    /// `SyscallDriver::allocate_grant()`.
    ///
    /// Useful for debugging/inspecting the system.
    fn grant_allocated_count(&self) -> Option<usize>;

    /// Get the grant number (grant_num) associated with a given driver number
    /// if there is a grant associated with that driver_num.
    fn lookup_grant_from_driver_num(&self, driver_num: usize) -> Result<usize, Error>;

    // subscribe

    /// Verify that an upcall function pointer is within process-accessible
    /// memory.
    ///
    /// Returns `true` if the upcall function pointer is valid for this process,
    /// and `false` otherwise.
    fn is_valid_upcall_function_pointer(&self, upcall_fn: NonNull<()>) -> bool;

    // functions for processes that are architecture specific

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    ///
    /// This can fail, if the UKB implementation cannot correctly set the return
    /// value. An example of how this might occur:
    ///
    /// 1. The UKB implementation uses the process's stack to transfer values
    ///    between kernelspace and userspace.
    /// 2. The process calls memop.brk and reduces its accessible memory region
    ///    below its current stack.
    /// 3. The UKB implementation can no longer set the return value on the
    ///    stack since the process no longer has access to its stack.
    ///
    /// If it fails, the process will be put into the faulted state.
    fn set_syscall_return_value(&self, return_value: SyscallReturn);

    /// Set the function that is to be executed when the process is resumed.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    fn set_process_function(&self, callback: FunctionCall);

    /// Context switch to a specific process.
    ///
    /// This will return `None` if the process is inactive and cannot be
    /// switched to.
    fn switch_to(&self) -> Option<syscall::ContextSwitchReason>;

    /// Return process state information related to the location in memory of
    /// various process data structures.
    fn get_addresses(&self) -> ProcessAddresses;

    /// Return process state information related to the size in memory of
    /// various process data structures.
    fn get_sizes(&self) -> ProcessSizes;

    /// Write stored state as a binary blob into the `out` slice. Returns the
    /// number of bytes written to `out` on success.
    ///
    /// Returns `ErrorCode::SIZE` if `out` is too short to hold the stored state
    /// binary representation. Returns `ErrorCode::FAIL` on an internal error.
    fn get_stored_state(&self, out: &mut [u8]) -> Result<usize, ErrorCode>;

    /// Print out the full state of the process: its memory map, its context,
    /// and the state of the memory protection unit (MPU).
    fn print_full_process(&self, writer: &mut dyn Write);

    // debug

    /// Returns how many syscalls this app has called.
    fn debug_syscall_count(&self) -> usize;

    /// Returns how many upcalls for this process have been dropped.
    fn debug_dropped_upcall_count(&self) -> usize;

    /// Returns how many times this process has exceeded its timeslice.
    fn debug_timeslice_expiration_count(&self) -> usize;

    /// Increment the number of times the process has exceeded its timeslice.
    fn debug_timeslice_expired(&self);

    /// Increment the number of times the process called a syscall and record
    /// the last syscall that was called.
    fn debug_syscall_called(&self, last_syscall: Syscall);

    /// Return the last syscall the process called. Returns `None` if the
    /// process has not called any syscalls or the information is unknown.
    fn debug_syscall_last(&self) -> Option<Syscall>;
}

/// Opaque identifier for custom grants allocated dynamically from a process's
/// grant region.
///
/// This type allows Process to provide a handle to a custom grant within a
/// process's memory that `ProcessGrant` can use to access the custom grant
/// memory later.
///
/// We use this type rather than a direct pointer so that any attempt to access
/// can ensure the process still exists and is valid, and that the custom grant
/// has not been freed.
///
/// The fields of this struct are private so only Process can create this
/// identifier.
#[derive(Copy, Clone)]
pub struct ProcessCustomGrantIdentifier {
    pub(crate) offset: usize,
}

/// Error types related to processes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The process has been removed and no longer exists. For example, the
    /// kernel could stop a process and re-claim its resources.
    NoSuchApp,
    /// The process does not have enough memory to complete the requested
    /// operation.
    OutOfMemory,
    /// The provided memory address is not accessible or not valid for the
    /// process.
    AddressOutOfBounds,
    /// The process is inactive (likely in a fault or exit state) and the
    /// attempted operation is therefore invalid.
    InactiveApp,
    /// This likely indicates a bug in the kernel and that some state is
    /// inconsistent in the kernel.
    KernelError,
    /// Indicates some process data, such as a Grant, is already borrowed.
    AlreadyInUse,
}

impl From<Error> for Result<(), ErrorCode> {
    fn from(err: Error) -> Result<(), ErrorCode> {
        match err {
            Error::OutOfMemory => Err(ErrorCode::NOMEM),
            Error::AddressOutOfBounds => Err(ErrorCode::INVAL),
            Error::NoSuchApp => Err(ErrorCode::INVAL),
            Error::InactiveApp => Err(ErrorCode::FAIL),
            Error::KernelError => Err(ErrorCode::FAIL),
            Error::AlreadyInUse => Err(ErrorCode::FAIL),
        }
    }
}

impl From<Error> for ErrorCode {
    fn from(err: Error) -> ErrorCode {
        match err {
            Error::OutOfMemory => ErrorCode::NOMEM,
            Error::AddressOutOfBounds => ErrorCode::INVAL,
            Error::NoSuchApp => ErrorCode::INVAL,
            Error::InactiveApp => ErrorCode::FAIL,
            Error::KernelError => ErrorCode::FAIL,
            Error::AlreadyInUse => ErrorCode::FAIL,
        }
    }
}

/// States a process can be in.
///
/// This is public so external implementations of `Process` can re-use these
/// process states.
///
/// While a process is running, it transitions between the `Running`, `Yielded`,
/// `YieldedFor`, and `Stopped` states. If an error occurs (e.g., a memory
/// access error), the kernel faults it and either leaves it in the `Faulted`
/// state, restarts it, or takes some other action defined by the kernel fault
/// policy. If the process issues an `exit-terminate` system call, it enters the
/// `Terminated` state. If it issues an `exit-restart` system call, it
/// terminates then tries to back to a runnable state.
///
/// When a process faults, it enters the `Faulted` state. To be restarted, it
/// must first transition to the `Terminated` state, which means that all of its
/// state has been cleaned up.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    /// Process expects to be running code. The process may not be currently
    /// scheduled by the scheduler, but the process has work to do if it is
    /// scheduled.
    Running,

    /// Process stopped executing and returned to the kernel because it called
    /// the `yield` syscall. This likely means it is waiting for some event to
    /// occur, but it could also mean it has finished and doesn't need to be
    /// scheduled again.
    Yielded,

    /// Process stopped executing and returned to the kernel because it called
    /// the `WaitFor` variant of the `yield` syscall. The process should not be
    /// scheduled until the specified driver attempts to execute the specified
    /// upcall.
    YieldedFor(UpcallId),

    /// The process is stopped and the previous state the process was in when it
    /// was stopped. This is used if the kernel forcibly stops a process. This
    /// state indicates to the kernel not to schedule the process, but if the
    /// process is to be resumed later it should be put back in its previous
    /// state so it will execute correctly.
    Stopped(StoppedState),

    /// The process ran, faulted while running, and is no longer runnable. For a
    /// faulted process to be made runnable, it must first be terminated (to
    /// clean up its state).
    Faulted,

    /// The process is not running: it exited with the `exit-terminate` system
    /// call or was terminated for some other reason (e.g., by the process
    /// console). Processes in the `Terminated` state can be run again.
    Terminated,
}

/// States a process could previously have been in when stopped.
///
/// This is public so external implementations of `Process` can re-use these
/// process stopped states.
///
/// These are recorded so the process can be returned to its previous state when
/// it is resumed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StoppedState {
    /// The process was in the running state when it was stopped.
    Running,
    /// The process was in the yielded state when it was stopped.
    Yielded,
    /// The process was in the yielded for state when it was stopped with a
    /// particular upcall it was waiting for.
    YieldedFor(UpcallId),
}

/// The action the kernel should take when a process encounters a fault.
///
/// When an exception occurs during a process's execution (a common example is a
/// process trying to access memory outside of its allowed regions) the system
/// will trap back to the kernel, and the kernel has to decide what to do with
/// the process at that point.
///
/// The actions are separate from the policy on deciding which action to take. A
/// separate process-specific policy should determine which action to take.
#[derive(Copy, Clone)]
pub enum FaultAction {
    /// Generate a `panic!()` call and crash the entire system. This is useful
    /// for debugging applications as the error is displayed immediately after
    /// it occurs.
    Panic,

    /// Attempt to cleanup and restart the process which caused the fault. This
    /// resets the process's memory to how it was when the process was started
    /// and schedules the process to run again from its init function.
    Restart,

    /// Stop the process by no longer scheduling it to run.
    Stop,
}

/// Tasks that can be enqueued for a process.
///
/// This is public for external implementations of `Process`.
#[derive(Copy, Clone)]
pub enum Task {
    /// Function pointer in the process to execute. Generally this is a upcall
    /// from a capsule.
    FunctionCall(FunctionCall),
    /// Data to return to the process. This is used to resume a suspended
    /// process without invoking any callbacks in userspace (e.g., in response
    /// to a YieldFor).
    ReturnValue(ReturnArguments),
    /// An IPC operation that needs additional setup to configure memory access.
    IPC((ProcessId, ipc::IPCUpcallType)),
}

/// Enumeration to identify whether a function call for a process comes directly
/// from the kernel or from a upcall subscribed through a `Driver`
/// implementation.
///
/// An example of a kernel function is the application entry point.
#[derive(Copy, Clone, Debug)]
pub enum FunctionCallSource {
    /// For functions coming directly from the kernel, such as `init_fn`.
    Kernel,
    /// For functions coming from capsules or any implementation of `Driver`.
    Driver(UpcallId),
}

/// Struct that defines a upcall that can be passed to a process. The upcall
/// takes four arguments that are `Driver` and upcall specific, so they are
/// represented generically here.
///
/// Likely these four arguments will get passed as the first four register
/// values, but this is architecture-dependent.
///
/// A `FunctionCall` also identifies the upcall that scheduled it, if any, so
/// that it can be unscheduled when the process unsubscribes from this upcall.
#[derive(Copy, Clone, Debug)]
pub struct FunctionCall {
    /// Whether the kernel called this directly or this is an upcall.
    pub source: FunctionCallSource,
    /// The first argument to the function.
    pub argument0: usize,
    /// The second argument to the function.
    pub argument1: usize,
    /// The third argument to the function.
    pub argument2: usize,
    /// The fourth argument to the function.
    pub argument3: usize,
    /// The PC of the function to execute.
    pub pc: usize,
}

/// This is similar to `FunctionCall` but for the special case of the Null
/// Upcall for a subscribe. Because there is no function pointer in a Null
/// Upcall we can only return these values to userspace. This is used to pass
/// around upcall parameters when there is no associated upcall to actually call
/// or userdata.
#[derive(Copy, Clone, Debug)]
pub struct ReturnArguments {
    /// Which upcall generates this event.
    pub upcall_id: UpcallId,
    /// The first argument to return.
    pub argument0: usize,
    /// The second argument to return.
    pub argument1: usize,
    /// The third argument to return.
    pub argument2: usize,
}

/// Collection of process state information related to the memory addresses of
/// different elements of the process.
pub struct ProcessAddresses {
    /// The address of the beginning of the process's region in nonvolatile
    /// memory.
    pub flash_start: usize,
    /// The address of the beginning of the region the process has access to in
    /// nonvolatile memory. This is after the TBF header and any other memory
    /// the kernel has reserved for its own use.
    pub flash_non_protected_start: usize,
    /// The address immediately after the end of part of the process binary that
    /// is covered by integrity; the integrity region is [flash_start -
    /// flash_integrity_end). Footers are stored in the flash after
    /// flash_integrity_end.
    pub flash_integrity_end: *const u8,
    /// The address immediately after the end of the region allocated for this
    /// process in nonvolatile memory.
    pub flash_end: usize,
    /// The address of the beginning of the process's allocated region in
    /// memory.
    pub sram_start: usize,
    /// The address of the application break. This is the address immediately
    /// after the end of the memory the process has access to.
    pub sram_app_brk: usize,
    /// The lowest address of any allocated grant. This is the start of the
    /// region the kernel is using for its own internal state on behalf of this
    /// process.
    pub sram_grant_start: usize,
    /// The address immediately after the end of the region allocated for this
    /// process in memory.
    pub sram_end: usize,

    /// The address of the start of the process's heap, if known. Note, managing
    /// this is completely up to the process, and the kernel relies on the
    /// process explicitly notifying it of this address. Therefore, its possible
    /// the kernel does not know the start address, or its start address could
    /// be incorrect.
    pub sram_heap_start: Option<usize>,
    /// The address of the top (or start) of the process's stack, if known.
    /// Note, managing the stack is completely up to the process, and the kernel
    /// relies on the process explicitly notifying it of where it started its
    /// stack. Therefore, its possible the kernel does not know the start
    /// address, or its start address could be incorrect.
    pub sram_stack_top: Option<usize>,
    /// The lowest address the kernel has seen the stack pointer. Note, the
    /// stack is entirely managed by the process, and the process could
    /// intentionally obscure this address from the kernel. Also, the stack may
    /// have reached a lower address, this is only the lowest address seen when
    /// the process calls a syscall.
    pub sram_stack_bottom: Option<usize>,
}

/// Collection of process state related to the size in memory of various process
/// structures.
pub struct ProcessSizes {
    /// The number of bytes used for the grant pointer table.
    pub grant_pointers: usize,
    /// The number of bytes used for the pending upcall queue.
    pub upcall_list: usize,
    /// The number of bytes used for the process control block (i.e. the
    /// `ProcessX` struct).
    pub process_control_block: usize,
}
