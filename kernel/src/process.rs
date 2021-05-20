//! Types for Tock-compatible processes.

use core::cell::Cell;
use core::fmt;
use core::fmt::Write;
use core::ptr::NonNull;
use core::str;

use crate::capabilities;
use crate::errorcode::ErrorCode;
use crate::ipc;
use crate::mem::{ReadOnlyAppSlice, ReadWriteAppSlice};
use crate::platform::mpu::{self};
use crate::sched::Kernel;
use crate::syscall::{self, Syscall, SyscallReturn};
use crate::upcall::UpcallId;

/// Userspace process identifier.
///
/// This should be treated as an opaque type that can be used to represent a
/// process on the board without requiring an actual reference to a `Process`
/// object. Having this `ProcessId` reference type is useful for managing
/// ownership and type issues in Rust, but more importantly `ProcessId` serves
/// as a tool for capsules to hold pointers to applications.
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

impl ProcessId {
    /// Create a new `ProcessId` object based on the app identifier and its index
    /// in the processes array.
    pub(crate) fn new(kernel: &'static Kernel, identifier: usize, index: usize) -> ProcessId {
        ProcessId {
            kernel: kernel,
            identifier: identifier,
            index: index,
        }
    }

    /// Create a new `ProcessId` object based on the app identifier and its index
    /// in the processes array.
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
            kernel: kernel,
            identifier: identifier,
            index: index,
        }
    }

    /// Get the location of this app in the processes array.
    ///
    /// This will return `Some(index)` if the identifier stored in this `ProcessId`
    /// matches the app saved at the known index. If the identifier does not
    /// match then `None` will be returned.
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
    /// kernel. Therefore, calling `id()` is _not_ a valid way to check
    /// that an application still exists.
    pub fn id(&self) -> usize {
        self.identifier
    }

    /// Returns the full address of the start and end of the flash region that
    /// the app owns and can write to. This includes the app's code and data and
    /// any padding at the end of the app. It does not include the TBF header,
    /// or any space that the kernel is using for any potential bookkeeping.
    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        self.kernel.process_map_or((0, 0), *self, |process| {
            let start = process.flash_non_protected_start() as usize;
            let end = process.flash_end() as usize;
            (start, end)
        })
    }
}

/// This trait represents a generic process that the Tock scheduler can
/// schedule.
pub trait Process {
    /// Returns the process's identifier.
    fn processid(&self) -> ProcessId;

    /// Queue a `Task` for the process. This will be added to a per-process
    /// buffer and executed by the scheduler. `Task`s are some function the app
    /// should run, for example a upcall or an IPC call.
    ///
    /// This function returns `true` if the `Task` was successfully enqueued,
    /// and `false` otherwise. This is represented as a simple `bool` because
    /// this is passed to the capsule that tried to schedule the `Task`.
    ///
    /// This will fail if the process is no longer active, and therefore cannot
    /// execute any new tasks.
    fn enqueue_task(&self, task: Task) -> bool;

    /// Returns whether this process is ready to execute.
    fn ready(&self) -> bool;

    /// Return if there are any Tasks (upcalls/IPC requests) enqueued
    /// for the process.
    fn has_tasks(&self) -> bool;

    /// Remove the scheduled operation from the front of the queue and return it
    /// to be handled by the scheduler.
    ///
    /// If there are no `Task`s in the queue for this process this will return
    /// `None`.
    fn dequeue_task(&self) -> Option<Task>;

    /// Remove all scheduled upcalls for a given upcall id from the task
    /// queue.
    fn remove_pending_upcalls(&self, upcall_id: UpcallId);

    /// Returns the current state the process is in. Common states are "running"
    /// or "yielded".
    fn get_state(&self) -> State;

    /// Move this process from the running state to the yielded state.
    ///
    /// This will fail (i.e. not do anything) if the process was not previously
    /// running.
    fn set_yielded_state(&self);

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

    /// Put this process in the fault state. This will trigger the
    /// `FaultResponse` for this process to occur.
    fn set_fault_state(&self);

    /// Returns how many times this process has been restarted.
    fn get_restart_count(&self) -> usize;

    /// Get the name of the process. Used for IPC.
    fn get_process_name(&self) -> &'static str;

    /// Stop and clear a process's state, putting it into the `Terminated`
    /// state.
    ///
    /// This will end the process, but does not reset it such that it could be
    /// restarted and run again. This function instead frees grants and any
    /// queued tasks for this process, but leaves the debug information about
    /// the process and other state intact.
    fn terminate(&self, completion_code: u32);

    /// Terminates and attempts to restart the process. The process and current
    /// application always terminate. The kernel may, based on its own policy,
    /// restart the application using the same process, reuse the process for
    /// another application, or simply terminate the process and application.
    ///
    /// This function can be called when the process is in any state. It
    /// attempts to reset all process state and re-initialize it so that it can
    /// be reused.
    ///
    /// Restarting an application can fail for two general reasons:
    ///
    /// 1. The kernel chooses not to restart the application, based on its
    ///    policy.
    ///
    /// 2. The kernel decides to restart the application but fails to do so
    ///    because Some state can no long be configured for the process. For
    ///    example, the syscall state for the process fails to initialize.
    ///
    /// After `restart()` runs the process will either be queued to run its the
    /// application's `_start` function, terminated, or queued to run a
    /// different application's `_start` function.
    fn try_restart(&self, completion_code: u32);

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

    fn flash_protected(&self) -> u32;
    fn app_memory_break(&self) -> *const u8;
    fn get_app_heap_start(&self) -> Option<usize>;
    fn get_app_stack_start(&self) -> Option<usize>;
    fn get_app_stack_end(&self) -> Option<usize>;

    /// The start address of allocated RAM for this process.
    fn mem_start(&self) -> *const u8;

    /// The first address after the end of the allocated RAM for this process.
    fn mem_end(&self) -> *const u8;

    /// The start address of the flash region allocated for this process.
    fn flash_start(&self) -> *const u8;

    /// The first address after the end of the flash region allocated for this
    /// process.
    fn flash_end(&self) -> *const u8;

    /// The lowest address of the grant region for the process.
    fn kernel_memory_break(&self) -> *const u8;

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

    // additional memop like functions

    /// Creates a `ReadWriteAppSlice` from the given offset and size
    /// in process memory.
    ///
    /// ## Returns
    ///
    /// In case of success, this method returns the created
    /// [`ReadWriteAppSlice`].
    ///
    /// In case of an error, an appropriate ErrorCode is returned:
    ///
    /// - if the memory is not contained in the process-accessible
    ///   memory space / `buf_start_addr` and `size` are not a valid
    ///   read-write buffer (any byte in the range is not read/write
    ///   accessible to the process), [`ErrorCode::INVAL`]
    /// - if the process is not active: [`ErrorCode::FAIL`]
    /// - for all other errors: [`ErrorCode::FAIL`]
    fn build_readwrite_appslice(
        &self,
        buf_start_addr: *mut u8,
        size: usize,
    ) -> Result<ReadWriteAppSlice, ErrorCode>;

    /// Creates a [`ReadOnlyAppSlice`] from the given offset and size
    /// in process memory.
    ///
    /// ## Returns
    ///
    /// In case of success, this method returns the created
    /// [`ReadOnlyAppSlice`].
    ///
    /// In case of an error, an appropriate ErrorCode is returned:
    ///
    /// - if the memory is not contained in the process-accessible
    ///   memory space / `buf_start_addr` and `size` are not a valid
    ///   read-only buffer (any byte in the range is not
    ///   read-accessible to the process), [`ErrorCode::INVAL`]
    /// - if the process is not active: [`ErrorCode::FAIL`]
    /// - for all other errors: [`ErrorCode::FAIL`]
    fn build_readonly_appslice(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> Result<ReadOnlyAppSlice, ErrorCode>;

    /// Set a single byte within the process address space at
    /// `addr` to `value`. Return true if `addr` is within the RAM
    /// bounds currently exposed to the process (thereby writable
    /// by the process itself) and the value was set, false otherwise.
    ///
    /// ### Safety
    ///
    /// This function verifies that the byte to be written is in the process's
    /// accessible memory. However, to avoid undefined behavior the caller needs
    /// to ensure that no other references exist to the process's memory before
    /// calling this function.
    unsafe fn set_byte(&self, addr: *mut u8, value: u8) -> bool;

    /// Get the first address of process's flash that isn't protected by the
    /// kernel. The protected range of flash contains the TBF header and
    /// potentially other state the kernel is storing on behalf of the process,
    /// and cannot be edited by the process.
    fn flash_non_protected_start(&self) -> *const u8;

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

    // grants

    /// Allocate memory from the grant region and store the reference in the
    /// proper grant pointer index.
    ///
    /// This function must check that doing the allocation does not cause
    /// the kernel memory break to go below the top of the process accessible
    /// memory region allowed by the MPU. Note, this can be different from the
    /// actual app_brk, as MPU alignment and size constraints may result in the
    /// MPU enforced region differing from the app_brk.
    ///
    /// This will return `None` and fail if:
    /// - The process is inactive, or
    /// - There is not enough available memory to do the allocation, or
    /// - The grant_num is invalid, or
    /// - The grant_num already has an allocated grant.
    fn allocate_grant(&self, grant_num: usize, size: usize, align: usize) -> Option<NonNull<u8>>;

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
    /// If successful, return a Some() with an identifier that can be used with
    /// `enter_custom_grant()` to get access to the memory and the pointer to
    /// the memory which must be used to initialize the memory.
    fn allocate_custom_grant(
        &self,
        size: usize,
        align: usize,
    ) -> Option<(ProcessCustomGrantIdentifer, NonNull<u8>)>;

    /// Enter the grant based on `grant_num` for this process.
    ///
    /// Entering a grant means getting access to the actual memory for the
    /// object stored as the grant.
    ///
    /// This will return an `Err` if the process is inactive of the `grant_num`
    /// is invalid, if the grant has not been allocated, or if the grant is
    /// already entered. If this returns `Ok()` then the pointer points to the
    /// previously allocated memory for this grant.
    fn enter_grant(&self, grant_num: usize) -> Result<*mut u8, Error>;

    /// Enter a custom grant based on the `identifier`.
    ///
    /// This retrieves a pointer to the previously allocated custom grant based
    /// on the identifier returned when the custom grant was allocated.
    ///
    /// This returns an error if the custom grant is no longer accessible, or
    /// if the process is inactive.
    fn enter_custom_grant(&self, identifier: ProcessCustomGrantIdentifer)
        -> Result<*mut u8, Error>;

    /// Opposite of `enter_grant()`. Used to signal that the grant is no longer
    /// entered.
    ///
    /// If `grant_num` is valid, this function cannot fail. If `grant_num` is
    /// invalid, this function will do nothing. If the process is inactive then
    /// grants are invalid and are not entered or not entered, and this function
    /// will do nothing.
    fn leave_grant(&self, grant_num: usize);

    /// Return the count of the number of allocated grant pointers if the
    /// process is active. This does not count custom grants.
    ///
    /// Useful for debugging/inspecting the system.
    fn grant_allocated_count(&self) -> Option<usize>;

    // functions for processes that are architecture specific

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    ///
    /// This can fail, if the UKB implementation cannot correctly set the return value. An
    /// example of how this might occur:
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

    /// Print out the memory map (Grant region, heap, stack, program
    /// memory, BSS, and data sections) of this process.
    fn print_memory_map(&self, writer: &mut dyn Write);

    /// Print out the full state of the process: its memory map, its
    /// context, and the state of the memory protection unit (MPU).
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
pub struct ProcessCustomGrantIdentifer {
    pub(crate) offset: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NoSuchApp,
    OutOfMemory,
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

/// Various states a process can be in.
///
/// This is made public in case external implementations of `Process` want
/// to re-use these process states in the external implementation.
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

    /// The process is stopped, and its previous state was Running. This is used
    /// if the kernel forcibly stops a process when it is in the `Running`
    /// state. This state indicates to the kernel not to schedule the process,
    /// but if the process is to be resumed later it should be put back in the
    /// running state so it will execute correctly.
    StoppedRunning,

    /// The process is stopped, and it was stopped while it was yielded. If this
    /// process needs to be resumed it should be put back in the `Yield` state.
    StoppedYielded,

    /// The process faulted and cannot be run.
    Faulted,

    /// The process exited with the `exit-terminate` system call and
    /// cannot be run.
    Terminated,

    /// The process has never actually been executed. This of course happens
    /// when the board first boots and the kernel has not switched to any
    /// processes yet. It can also happen if an process is terminated and all
    /// of its state is reset as if it has not been executed yet.
    Unstarted,
}

/// A wrapper around `Cell<State>` is used by `Process` to prevent bugs arising from
/// the state duplication in the kernel work tracking and process state tracking.
pub(crate) struct ProcessStateCell<'a> {
    state: Cell<State>,
    kernel: &'a Kernel,
}

impl<'a> ProcessStateCell<'a> {
    pub(crate) fn new(kernel: &'a Kernel) -> Self {
        Self {
            state: Cell::new(State::Unstarted),
            kernel,
        }
    }

    pub(crate) fn get(&self) -> State {
        self.state.get()
    }

    pub(crate) fn update(&self, new_state: State) {
        let old_state = self.state.get();

        if old_state == State::Running && new_state != State::Running {
            self.kernel.decrement_work();
        } else if new_state == State::Running && old_state != State::Running {
            self.kernel.increment_work()
        }
        self.state.set(new_state);
    }
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
    pub source: FunctionCallSource,
    pub argument0: usize,
    pub argument1: usize,
    pub argument2: usize,
    pub argument3: usize,
    pub pc: usize,
}


/// State for helping with debugging apps.
///
/// These pointers and counters are not strictly required for kernel operation,
/// but provide helpful information when an app crashes.
struct ProcessDebug {
    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in flash. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_flash: Option<u32>,

    /// If this process was compiled for fixed addresses, save the address
    /// it must be at in RAM. This is useful for debugging and saves having
    /// to re-parse the entire TBF header.
    fixed_address_ram: Option<u32>,

    /// Where the process has started its heap in RAM.
    app_heap_start_pointer: Option<*const u8>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<*const u8>,

    /// How low have we ever seen the stack pointer.
    app_stack_min_pointer: Option<*const u8>,

    /// How many syscalls have occurred since the process started.
    syscall_count: usize,

    /// What was the most recent syscall.
    last_syscall: Option<Syscall>,

    /// How many callbacks were dropped because the queue was insufficiently
    /// long.
    dropped_callback_count: usize,

    /// How many times this process has been paused because it exceeded its
    /// timeslice.
    timeslice_expiration_count: usize,
}

/// A type for userspace processes in Tock.
pub struct Process<'a, C: 'static + Chip> {
    /// Identifier of this process and the index of the process in the process
    /// table.
    app_id: Cell<AppId>,

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
    ///     ╒════════ ← memory[memory.len()]
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
    ///  ╚═ ╘════════ ← memory[0]               ═╝
    /// ```
    ///
    /// The process's memory.
    memory: &'static mut [u8],

    /// Pointer to the end of the allocated (and MPU protected) grant region.
    kernel_memory_break: Cell<*const u8>,

    /// Pointer to the end of process RAM that has been sbrk'd to the process.
    app_break: Cell<*const u8>,

    /// Pointer to high water mark for process buffers shared through `allow`
    allow_high_water_mark: Cell<*const u8>,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    header: tock_tbf::types::TbfHeader,

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
    state: ProcessStateCell<'static>,

    /// How to deal with Faults occurring in the process
    fault_response: FaultResponse,

    /// Configuration data for the MPU
    mpu_config: MapCell<<<C as Chip>::MPU as MPU>::MpuConfig>,

    /// MPU regions are saved as a pointer-size pair.
    mpu_regions: [Cell<Option<mpu::Region>>; 6],

    /// Essentially a list of callbacks that want to call functions in the
    /// process.
    tasks: MapCell<RingBuffer<'a, Task>>,

    /// Count of how many times this process has entered the fault condition and
    /// been restarted. This is used by some `ProcessRestartPolicy`s to
    /// determine if the process should be restarted or not.
    restart_count: Cell<usize>,

    /// Name of the app.
    process_name: &'static str,

    /// Values kept so that we can print useful debug messages when apps fault.
    debug: MapCell<ProcessDebug>,
}

impl<C: Chip> ProcessType for Process<'_, C> {
    fn appid(&self) -> AppId {
        self.app_id.get()
    }

    fn enqueue_task(&self, task: Task) -> bool {
        // If this app is in a `Fault` state then we shouldn't schedule
        // any work for it.
        if !self.is_active() {
            return false;
        }

        let ret = self.tasks.map_or(false, |tasks| tasks.enqueue(task));

        // Make a note that we lost this callback if the enqueue function
        // fails.
        if ret == false {
            self.debug.map(|debug| {
                debug.dropped_callback_count += 1;
            });
        } else {
            self.kernel.increment_work();
        }

        ret
    }

    fn ready(&self) -> bool {
        self.tasks.map_or(false, |ring_buf| ring_buf.has_elements())
            || self.state.get() == State::Running
    }

    fn remove_pending_callbacks(&self, callback_id: CallbackId) {
        self.tasks.map(|tasks| {
            let count_before = tasks.len();
            tasks.retain(|task| match task {
                // Remove only tasks that are function calls with an id equal
                // to `callback_id`.
                Task::FunctionCall(function_call) => match function_call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => {
                        if id != callback_id {
                            true
                        } else {
                            self.kernel.decrement_work();
                            false
                        }
                    }
                },
                _ => true,
            });
            if config::CONFIG.trace_syscalls {
                let count_after = tasks.len();
                debug!(
                    "[{:?}] remove_pending_callbacks[{:#x}:{}] = {} callback(s) removed",
                    self.appid(),
                    callback_id.driver_num,
                    callback_id.subscribe_num,
                    count_before - count_after,
                );
            }
        });
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_yielded_state(&self) {
        if self.state.get() == State::Running {
            self.state.update(State::Yielded);
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.update(State::StoppedRunning),
            State::Yielded => self.state.update(State::StoppedYielded),
            _ => {} // Do nothing
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::StoppedRunning => self.state.update(State::Running),
            State::StoppedYielded => self.state.update(State::Yielded),
            _ => {} // Do nothing
        }
    }

    fn set_fault_state(&self) {
        self.state.update(State::Fault);

        match self.fault_response {
            FaultResponse::Panic => {
                // process faulted. Panic and print status
                panic!("Process {} had a fault", self.process_name);
            }
            FaultResponse::Restart(_) => {
                self.restart(State::StoppedFaulted);
            }
            FaultResponse::Stop => {
                // This looks a lot like restart, except we just leave the app
                // how it faulted and mark it as `StoppedFaulted`. By clearing
                // all of the app's todo work it will not be scheduled, and
                // clearing all of the grant regions will cause capsules to drop
                // this app as well.
                self.terminate();
            }
        }
    }

    fn get_restart_count(&self) -> usize {
        self.restart_count.get()
    }

    fn dequeue_task(&self) -> Option<Task> {
        self.tasks.map_or(None, |tasks| {
            tasks.dequeue().map(|cb| {
                self.kernel.decrement_work();
                cb
            })
        })
    }

    fn flash_protected(&self) -> u32 {
        self.header.get_protected_size()
    }
    fn app_memory_break(&self) -> *const u8 {
        self.app_break.get()
    }
    fn get_app_heap_start(&self) -> Option<usize> {
        self.debug.map_or(None, |debug| {
            debug.app_heap_start_pointer.map(|p| p as usize)
        })
    }
    fn get_app_stack_start(&self) -> Option<usize> {
        self.debug.map_or(None, |debug| {
            debug.app_stack_start_pointer.map(|p| p as usize)
        })
    }
    fn get_app_stack_end(&self) -> Option<usize> {
        self.debug.map_or(None, |debug| {
            debug.app_stack_min_pointer.map(|p| p as usize)
        })
    }
    fn mem_start(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    fn mem_end(&self) -> *const u8 {
        unsafe { self.memory.as_ptr().add(self.memory.len()) }
    }

    fn flash_start(&self) -> *const u8 {
        self.flash.as_ptr()
    }

    fn flash_non_protected_start(&self) -> *const u8 {
        ((self.flash.as_ptr() as usize) + self.header.get_protected_size() as usize) as *const u8
    }

    fn flash_end(&self) -> *const u8 {
        unsafe { self.flash.as_ptr().add(self.flash.len()) }
    }

    fn kernel_memory_break(&self) -> *const u8 {
        self.kernel_memory_break.get()
    }

    fn number_writeable_flash_regions(&self) -> usize {
        self.header.number_writeable_flash_regions()
    }

    fn get_writeable_flash_region(&self, region_index: usize) -> (u32, u32) {
        self.header.get_writeable_flash_region(region_index)
    }

    fn update_stack_start_pointer(&self, stack_pointer: *const u8) {
        if stack_pointer >= self.mem_start() && stack_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_stack_start_pointer = Some(stack_pointer);

                // We also reset the minimum stack pointer because whatever value
                // we had could be entirely wrong by now.
                debug.app_stack_min_pointer = Some(stack_pointer);
            });
        }
    }

    fn update_heap_start_pointer(&self, heap_pointer: *const u8) {
        if heap_pointer >= self.mem_start() && heap_pointer < self.mem_end() {
            self.debug.map(|debug| {
                debug.app_heap_start_pointer = Some(heap_pointer);
            });
        }
    }

    fn setup_mpu(&self) {
        self.mpu_config.map(|config| {
            self.chip.mpu().configure_mpu(&config, &self.appid());
        });
    }

    fn add_mpu_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
    ) -> Option<mpu::Region> {
        self.mpu_config.and_then(|mut config| {
            let new_region = self.chip.mpu().allocate_region(
                unallocated_memory_start,
                unallocated_memory_size,
                min_region_size,
                mpu::Permissions::ReadWriteOnly,
                &mut config,
            );

            if new_region.is_none() {
                return None;
            }

            for region in self.mpu_regions.iter() {
                if region.get().is_none() {
                    region.set(new_region);
                    return new_region;
                }
            }

            // Not enough room in Process struct to store the MPU region.
            None
        })
    }

    fn sbrk(&self, increment: isize) -> Result<*const u8, Error> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return Err(Error::InactiveApp);
        }

        let new_break = unsafe { self.app_break.get().offset(increment) };
        self.brk(new_break)
    }

    fn brk(&self, new_break: *const u8) -> Result<*const u8, Error> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return Err(Error::InactiveApp);
        }

        self.mpu_config
            .map_or(Err(Error::KernelError), |mut config| {
                if new_break < self.allow_high_water_mark.get() || new_break >= self.mem_end() {
                    Err(Error::AddressOutOfBounds)
                } else if new_break > self.kernel_memory_break.get() {
                    Err(Error::OutOfMemory)
                } else if let Err(_) = self.chip.mpu().update_app_memory_region(
                    new_break,
                    self.kernel_memory_break.get(),
                    mpu::Permissions::ReadWriteOnly,
                    &mut config,
                ) {
                    Err(Error::OutOfMemory)
                } else {
                    let old_break = self.app_break.get();
                    self.app_break.set(new_break);
                    self.chip.mpu().configure_mpu(&config, &self.appid());
                    Ok(old_break)
                }
            })
    }

    fn allow(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> Result<Option<AppSlice<Shared, u8>>, ReturnCode> {
        if !self.is_active() {
            // Do not modify an inactive process.
            return Err(ReturnCode::FAIL);
        }

        match NonNull::new(buf_start_addr as *mut u8) {
            None => {
                // A null buffer means pass in `None` to the capsule
                Ok(None)
            }
            Some(buf_start) => {
                if self.in_app_owned_memory(buf_start_addr, size) {
                    // Valid slice, we need to adjust the app's watermark
                    // note: in_app_owned_memory ensures this offset does not wrap
                    let buf_end_addr = buf_start_addr.wrapping_add(size);
                    let new_water_mark = cmp::max(self.allow_high_water_mark.get(), buf_end_addr);
                    self.allow_high_water_mark.set(new_water_mark);

                    // The `unsafe` promise we should be making here is that this
                    // buffer is inside of app memory and that it does not create any
                    // aliases (i.e. the same buffer has not been `allow`ed twice).
                    //
                    // TODO: We do not currently satisfy the second promise.
                    let slice = unsafe { AppSlice::new(buf_start, size, self.appid()) };
                    Ok(Some(slice))
                } else {
                    Err(ReturnCode::EINVAL)
                }
            }
        }
    }

    fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Do not modify an inactive process.
        if !self.is_active() {
            return None;
        }

        self.mpu_config.and_then(|mut config| {
            // First, compute the candidate new pointer. Note that at this
            // point we have not yet checked whether there is space for
            // this allocation or that it meets alignment requirements.
            let new_break_unaligned = self
                .kernel_memory_break
                .get()
                .wrapping_offset(-(size as isize));

            // The alignment must be a power of two, 2^a. The expression
            // `!(align - 1)` then returns a mask with leading ones,
            // followed by `a` trailing zeros.
            let alignment_mask = !(align - 1);
            let new_break = (new_break_unaligned as usize & alignment_mask) as *const u8;

            // Verify there is space for this allocation
            if new_break < self.app_break.get() {
                None
            // Verify it didn't wrap around
            } else if new_break > self.kernel_memory_break.get() {
                None
            } else if let Err(_) = self.chip.mpu().update_app_memory_region(
                self.app_break.get(),
                new_break,
                mpu::Permissions::ReadWriteOnly,
                &mut config,
            ) {
                None
            } else {
                self.kernel_memory_break.set(new_break);
                unsafe {
                    // Two unsafe steps here, both okay as we just made this pointer
                    Some(NonNull::new_unchecked(new_break as *mut u8))
                }
            }
        })
    }

    unsafe fn free(&self, _: *mut u8) {}

    // This is safe today, as MPU constraints ensure that `mem_end` will always
    // be aligned on at least a word boundary. While this is unlikely to
    // change, it should be more proactively enforced.
    //
    // TODO: https://github.com/tock/tock/issues/1739
    #[allow(clippy::cast_ptr_alignment)]
    fn get_grant_ptr(&self, grant_num: usize) -> Option<*mut u8> {
        // Do not try to access the grant region of inactive process.
        if !self.is_active() {
            return None;
        }

        // Sanity check the argument
        if grant_num >= self.kernel.get_grant_count_and_finalize() {
            return None;
        }

        let grant_num = grant_num as isize;
        let grant_pointer = unsafe {
            let grant_pointer_array = self.mem_end() as *const *mut u8;
            *grant_pointer_array.offset(-(grant_num + 1))
        };
        Some(grant_pointer)
    }

    // This is safe today, as MPU constraints ensure that `mem_end` will always
    // be aligned on at least a word boundary. While this is unlikely to
    // change, it should be more proactively enforced.
    //
    // TODO: https://github.com/tock/tock/issues/1739
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn set_grant_ptr(&self, grant_num: usize, grant_ptr: *mut u8) {
        let grant_num = grant_num as isize;
        let grant_pointer_array = self.mem_end() as *mut *mut u8;
        let grant_pointer_pointer = grant_pointer_array.offset(-(grant_num + 1));
        *grant_pointer_pointer = grant_ptr;
    }

    fn get_process_name(&self) -> &'static str {
        self.process_name
    }

    unsafe fn set_syscall_return_value(&self, return_value: isize) {
        match self.stored_state.map(|stored_state| {
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(
                    self.memory.as_ptr(),
                    self.app_break.get(),
                    stored_state,
                    return_value,
                )
        }) {
            Some(Ok(())) => {
                // If we get an `Ok` we are all set.
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

    unsafe fn set_process_function(&self, callback: FunctionCall) {
        // See if we can actually enqueue this function for this process.
        // Architecture-specific code handles actually doing this since the
        // exact method is both architecture- and implementation-specific.
        //
        // This can fail, for example if the process does not have enough memory
        // remaining.
        match self.stored_state.map(|stored_state| {
            self.chip.userspace_kernel_boundary().set_process_function(
                self.memory.as_ptr(),
                self.app_break.get(),
                stored_state,
                callback,
            )
        }) {
            Some(Ok(())) => {
                // If we got an `Ok` we are all set and should mark that this
                // process is ready to be scheduled.

                // Move this process to the "running" state so the scheduler
                // will schedule it.
                self.state.update(State::Running);
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

    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        // Cannot switch to an invalid process
        if !self.is_active() {
            return None;
        }

        let (switch_reason, stack_pointer) =
            self.stored_state.map_or((None, None), |stored_state| {
                let (switch_reason, optional_stack_pointer) = self
                    .chip
                    .userspace_kernel_boundary()
                    .switch_to_process(self.memory.as_ptr(), self.app_break.get(), stored_state);
                (Some(switch_reason), optional_stack_pointer)
            });

        // If the UKB implementation passed us a stack pointer, update our
        // debugging state. This is completely optional.
        stack_pointer.map(|sp| {
            self.debug.map(|debug| {
                match debug.app_stack_min_pointer {
                    None => debug.app_stack_min_pointer = Some(sp),
                    Some(asmp) => {
                        // Update max stack depth if needed.
                        if sp < asmp {
                            debug.app_stack_min_pointer = Some(sp);
                        }
                    }
                }
            });
        });

        switch_reason
    }

    fn debug_syscall_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.syscall_count)
    }

    fn debug_dropped_callback_count(&self) -> usize {
        self.debug.map_or(0, |debug| debug.dropped_callback_count)
    }

    fn debug_timeslice_expiration_count(&self) -> usize {
        self.debug
            .map_or(0, |debug| debug.timeslice_expiration_count)
    }

    fn debug_timeslice_expired(&self) {
        self.debug
            .map(|debug| debug.timeslice_expiration_count += 1);
    }

    fn debug_syscall_called(&self, last_syscall: Syscall) {
        self.debug.map(|debug| {
            debug.syscall_count += 1;
            debug.last_syscall = Some(last_syscall);
        });
    }

    unsafe fn print_memory_map(&self, writer: &mut dyn Write) {
        // Flash
        let flash_end = self.flash.as_ptr().add(self.flash.len()) as usize;
        let flash_start = self.flash.as_ptr() as usize;
        let flash_protected_size = self.header.get_protected_size() as usize;
        let flash_app_start = flash_start + flash_protected_size;
        let flash_app_size = flash_end - flash_app_start;

        // SRAM addresses
        let sram_end = self.memory.as_ptr().add(self.memory.len()) as usize;
        let sram_grant_start = self.kernel_memory_break.get() as usize;
        let sram_heap_end = self.app_break.get() as usize;
        let sram_heap_start: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_heap_start_pointer.map(|p| p as usize)
        });
        let sram_stack_start: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_stack_start_pointer.map(|p| p as usize)
        });
        let sram_stack_bottom: Option<usize> = self.debug.map_or(None, |debug| {
            debug.app_stack_min_pointer.map(|p| p as usize)
        });
        let sram_start = self.memory.as_ptr() as usize;

        // SRAM sizes
        let sram_grant_size = sram_end - sram_grant_start;
        let sram_grant_allocated = sram_end - sram_grant_start;

        // application statistics
        let events_queued = self.tasks.map_or(0, |tasks| tasks.len());
        let syscall_count = self.debug.map_or(0, |debug| debug.syscall_count);
        let last_syscall = self.debug.map(|debug| debug.last_syscall);
        let dropped_callback_count = self.debug.map_or(0, |debug| debug.dropped_callback_count);
        let restart_count = self.restart_count.get();

        let _ = writer.write_fmt(format_args!(
            "\
             𝐀𝐩𝐩: {}   -   [{:?}]\
             \r\n Events Queued: {}   Syscall Count: {}   Dropped Callback Count: {}\
             \r\n Restart Count: {}\r\n",
            self.process_name,
            self.state.get(),
            events_queued,
            syscall_count,
            dropped_callback_count,
            restart_count,
        ));

        let _ = match last_syscall {
            Some(syscall) => writer.write_fmt(format_args!(" Last Syscall: {:?}\r\n", syscall)),
            None => writer.write_str(" Last Syscall: None\r\n"),
        };

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n\
             \r\n ╔═══════════╤══════════════════════════════════════════╗\
             \r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
             \r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
             \r\n             │ ▼ Grant      {:6} | {:6}{}\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┼───────────────────────────────────────────",
            sram_end,
            sram_grant_size,
            sram_grant_allocated,
            exceeded_check(sram_grant_size, sram_grant_allocated),
            sram_grant_start,
            sram_heap_end,
        ));

        match sram_heap_start {
            Some(sram_heap_start) => {
                let sram_heap_size = sram_heap_end - sram_heap_start;
                let sram_heap_allocated = sram_grant_start - sram_heap_start;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n             │ ▲ Heap       {:6} | {:6}{}     S\
                     \r\n  {:#010X} ┼─────────────────────────────────────────── R",
                    sram_heap_size,
                    sram_heap_allocated,
                    exceeded_check(sram_heap_size, sram_heap_allocated),
                    sram_heap_start,
                ));
            }
            None => {
                let _ = writer.write_str(
                    "\
                     \r\n             │ ▲ Heap            ? |      ?               S\
                     \r\n  ?????????? ┼─────────────────────────────────────────── R",
                );
            }
        }

        match (sram_heap_start, sram_stack_start) {
            (Some(sram_heap_start), Some(sram_stack_start)) => {
                let sram_data_size = sram_heap_start - sram_stack_start;
                let sram_data_allocated = sram_data_size as usize;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n             │ Data         {:6} | {:6}               A",
                    sram_data_size, sram_data_allocated,
                ));
            }
            _ => {
                let _ = writer.write_str(
                    "\
                     \r\n             │ Data              ? |      ?               A",
                );
            }
        }

        match (sram_stack_start, sram_stack_bottom) {
            (Some(sram_stack_start), Some(sram_stack_bottom)) => {
                let sram_stack_size = sram_stack_start - sram_stack_bottom;
                let sram_stack_allocated = sram_stack_start - sram_start;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\n  {:#010X} ┼─────────────────────────────────────────── M\
                     \r\n             │ ▼ Stack      {:6} | {:6}{}",
                    sram_stack_start,
                    sram_stack_size,
                    sram_stack_allocated,
                    exceeded_check(sram_stack_size, sram_stack_allocated),
                ));
            }
            _ => {
                let _ = writer.write_str(
                    "\
                     \r\n  ?????????? ┼─────────────────────────────────────────── M\
                     \r\n             │ ▼ Stack           ? |      ?",
                );
            }
        }

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n  {:#010X} ┼───────────────────────────────────────────\
             \r\n             │ Unused\
             \r\n  {:#010X} ┴───────────────────────────────────────────\
             \r\n             .....\
             \r\n  {:#010X} ┬─────────────────────────────────────────── F\
             \r\n             │ App Flash    {:6}                        L\
             \r\n  {:#010X} ┼─────────────────────────────────────────── A\
             \r\n             │ Protected    {:6}                        S\
             \r\n  {:#010X} ┴─────────────────────────────────────────── H\
             \r\n",
            sram_stack_bottom.unwrap_or(0),
            sram_start,
            flash_end,
            flash_app_size,
            flash_app_start,
            flash_protected_size,
            flash_start
        ));
    }

    unsafe fn print_full_process(&self, writer: &mut dyn Write) {
        self.print_memory_map(writer);

        self.stored_state.map(|stored_state| {
            self.chip.userspace_kernel_boundary().print_context(
                self.memory.as_ptr(),
                self.app_break.get(),
                stored_state,
                writer,
            );
        });

        // Display grant information.
        let number_grants = self.kernel.get_grant_count_and_finalize();
        let _ = writer.write_fmt(format_args!(
            "\
             \r\n Total number of grant regions defined: {}\r\n",
            self.kernel.get_grant_count_and_finalize()
        ));
        let rows = (number_grants + 2) / 3;
        // Iterate each grant and show its address.
        for i in 0..rows {
            for j in 0..3 {
                let index = i + (rows * j);
                if index >= number_grants {
                    break;
                }

                match self.get_grant_ptr(index) {
                    Some(ptr) => {
                        if ptr.is_null() {
                            let _ =
                                writer.write_fmt(format_args!("  Grant {:>2}: --        ", index));
                        } else {
                            let _ =
                                writer.write_fmt(format_args!("  Grant {:>2}: {:p}", index, ptr));
                        }
                    }
                    None => {
                        // Don't display if the grant ptr is completely invalid.
                    }
                }
            }
            let _ = writer.write_fmt(format_args!("\r\n"));
        }

        // Display the current state of the MPU for this process.
        self.mpu_config.map(|config| {
            let _ = writer.write_fmt(format_args!("{}", config));
        });

        // Print a helpful message on how to re-compile a process to view the
        // listing file. If a process is PIC, then we also need to print the
        // actual addresses the process executed at so that the .lst file can be
        // generated for those addresses. If the process was already compiled
        // for a fixed address, then just generating a .lst file is fine.

        self.debug.map(|debug| {
            if debug.fixed_address_flash.is_some() {
                // Fixed addresses, can just run `make lst`.
                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\nTo debug, run `make lst` in the app's folder\
                     \r\nand open the arch.{:#x}.{:#x}.lst file.\r\n\r\n",
                    debug.fixed_address_flash.unwrap_or(0),
                    debug.fixed_address_ram.unwrap_or(0)
                ));
            } else {
                // PIC, need to specify the addresses.
                let sram_start = self.memory.as_ptr() as usize;
                let flash_start = self.flash.as_ptr() as usize;
                let flash_init_fn = flash_start + self.header.get_init_function_offset() as usize;

                let _ = writer.write_fmt(format_args!(
                    "\
                     \r\nTo debug, run `make debug RAM_START={:#x} FLASH_INIT={:#x}`\
                     \r\nin the app's folder and open the .lst file.\r\n\r\n",
                    sram_start, flash_init_fn
                ));
            }
        });
    }
}

fn exceeded_check(size: usize, allocated: usize) -> &'static str {
    if size > allocated {
        " EXCEEDED!"
    } else {
        "          "
    }
}

impl<C: 'static + Chip> Process<'_, C> {
    // Memory offset for callback ring buffer (10 element length).
    const CALLBACK_LEN: usize = 10;
    const CALLBACKS_OFFSET: usize = mem::size_of::<Task>() * Self::CALLBACK_LEN;

    // Memory offset to make room for this process's metadata.
    const PROCESS_STRUCT_OFFSET: usize = mem::size_of::<Process<C>>();

    pub(crate) unsafe fn create(
        kernel: &'static Kernel,
        chip: &'static C,
        app_flash: &'static [u8],
        header_length: usize,
        app_version: u16,
        remaining_memory: &'static mut [u8],
        fault_response: FaultResponse,
        index: usize,
    ) -> Result<(Option<&'static dyn ProcessType>, &'static mut [u8]), ProcessLoadError> {
        // Get a slice for just the app header.
        let header_flash = app_flash
            .get(0..header_length as usize)
            .ok_or(ProcessLoadError::NotEnoughFlash)?;

        // Parse the full TBF header to see if this is a valid app. If the
        // header can't parse, we will error right here.
        let tbf_header = tock_tbf::parse::parse_tbf_header(header_flash, app_version)?;

        // First thing: check that the process is at the correct location in
        // flash if the TBF header specified a fixed address. If there is a
        // mismatch we catch that early.
        if let Some(fixed_flash_start) = tbf_header.get_fixed_address_flash() {
            // The flash address in the header is based on the app binary,
            // so we need to take into account the header length.
            let actual_address = app_flash.as_ptr() as u32 + tbf_header.get_protected_size();
            let expected_address = fixed_flash_start;
            if actual_address != expected_address {
                return Err(ProcessLoadError::IncorrectFlashAddress {
                    actual_address,
                    expected_address,
                });
            }
        }

        let process_name = tbf_header.get_package_name();

        // If this isn't an app (i.e. it is padding) or it is an app but it
        // isn't enabled, then we can skip it and do not create a `Process`
        // object.
        if !tbf_header.is_app() || !tbf_header.enabled() {
            if config::CONFIG.debug_load_processes {
                if !tbf_header.is_app() {
                    debug!(
                        "Padding in flash={:#010X}-{:#010X}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1
                    );
                }
                if !tbf_header.enabled() {
                    debug!(
                        "Process not enabled flash={:#010X}-{:#010X} process={:?}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name
                    );
                }
            }
            // Return no process and the full memory slice we were given.
            return Ok((None, remaining_memory));
        }

        // Otherwise, actually load the app.
        let process_ram_requested_size = tbf_header.get_minimum_app_ram_size() as usize;
        let init_fn = app_flash
            .as_ptr()
            .offset(tbf_header.get_init_function_offset() as isize) as usize;

        // Initialize MPU region configuration.
        let mut mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig = Default::default();

        // Allocate MPU region for flash.
        if chip
            .mpu()
            .allocate_region(
                app_flash.as_ptr(),
                app_flash.len(),
                app_flash.len(),
                mpu::Permissions::ReadExecuteOnly,
                &mut mpu_config,
            )
            .is_none()
        {
            if config::CONFIG.debug_load_processes {
                debug!(
                    "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate MPU region for flash",
                    app_flash.as_ptr() as usize,
                    app_flash.as_ptr() as usize + app_flash.len() - 1,
                    process_name
                );
            }
            return Err(ProcessLoadError::MpuInvalidFlashLength);
        }

        // Determine how much space we need in the application's
        // memory space just for kernel and grant state. We need to make
        // sure we allocate enough memory just for that.

        // Make room for grant pointers.
        let grant_ptr_size = mem::size_of::<*const usize>();
        let grant_ptrs_num = kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        // Initial size of the kernel-owned part of process memory can be
        // calculated directly based on the initial size of all kernel-owned
        // data structures.
        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;

        // By default we start with the initial size of process-accessible
        // memory set to 0. This maximizes the flexibility that processes have
        // to allocate their memory as they see fit. If a process needs more
        // accessible memory it must use the `brk` memop syscalls to request more
        // memory.
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
        let min_total_memory_size = min_process_ram_size + initial_kernel_memory_size;

        // Check if this process requires a fixed memory start address. If so,
        // try to adjust the memory region to work for this process.
        //
        // Right now, we only support skipping some RAM and leaving a chunk
        // unused so that the memory region starts where the process needs it
        // to.
        let remaining_memory = if let Some(fixed_memory_start) = tbf_header.get_fixed_address_ram()
        {
            // The process does have a fixed address.
            if fixed_memory_start == remaining_memory.as_ptr() as u32 {
                // Address already matches.
                remaining_memory
            } else if fixed_memory_start > remaining_memory.as_ptr() as u32 {
                // Process wants a memory address farther in memory. Try to
                // advance the memory region to make the address match.
                let diff = (fixed_memory_start - remaining_memory.as_ptr() as u32) as usize;
                if diff > remaining_memory.len() {
                    // We ran out of memory.
                    let actual_address =
                        remaining_memory.as_ptr() as u32 + remaining_memory.len() as u32 - 1;
                    let expected_address = fixed_memory_start;
                    return Err(ProcessLoadError::MemoryAddressMismatch {
                        actual_address,
                        expected_address,
                    });
                } else {
                    // Change the memory range to start where the process
                    // requested it.
                    remaining_memory
                        .get_mut(diff..)
                        .ok_or(ProcessLoadError::InternalError)?
                }
            } else {
                // Address is earlier in memory, nothing we can do.
                let actual_address = remaining_memory.as_ptr() as u32;
                let expected_address = fixed_memory_start;
                return Err(ProcessLoadError::MemoryAddressMismatch {
                    actual_address,
                    expected_address,
                });
            }
        } else {
            remaining_memory
        };

        // Determine where process memory will go and allocate MPU region for
        // app-owned memory.
        let (app_memory_start, app_memory_size) = match chip.mpu().allocate_app_memory_region(
            remaining_memory.as_ptr() as *const u8,
            remaining_memory.len(),
            min_total_memory_size,
            min_process_memory_size,
            initial_kernel_memory_size,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        ) {
            Some((memory_start, memory_size)) => (memory_start, memory_size),
            None => {
                // Failed to load process. Insufficient memory.
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't allocate memory region of size >= {:#X}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name,
                        min_total_memory_size
                    );
                }
                return Err(ProcessLoadError::NotEnoughMemory);
            }
        };

        // Get a slice for the memory dedicated to the process. This can fail if
        // the MPU returns a region of memory that is not inside of the
        // `remaining_memory` slice passed to `create()` to allocate the
        // process's memory out of.
        let memory_start_offset = app_memory_start as usize - remaining_memory.as_ptr() as usize;
        // First split the remaining memory into a slice that contains the
        // process memory and a slice that will not be used by this process.
        let (app_memory_oversize, unused_memory) =
            remaining_memory.split_at_mut(memory_start_offset + app_memory_size);
        // Then since the process's memory need not start at the beginning of
        // the remaining slice given to create(), get a smaller slice as needed.
        let app_memory = app_memory_oversize
            .get_mut(memory_start_offset..)
            .ok_or(ProcessLoadError::InternalError)?;

        // Check if the memory region is valid for the process. If a process
        // included a fixed address for the start of RAM in its TBF header (this
        // field is optional, processes that are position independent do not
        // need a fixed address) then we check that we used the same address
        // when we allocated it in RAM.
        if let Some(fixed_memory_start) = tbf_header.get_fixed_address_ram() {
            let actual_address = app_memory.as_ptr() as u32;
            let expected_address = fixed_memory_start;
            if actual_address != expected_address {
                return Err(ProcessLoadError::MemoryAddressMismatch {
                    actual_address,
                    expected_address,
                });
            }
        }

        // Set the initial process-accessible memory to the amount specified by
        // the context switch implementation.
        let initial_app_brk = app_memory.as_ptr().add(min_process_memory_size);

        // Set the initial allow high water mark to the start of process memory
        // since no `allow` calls have been made yet.
        let initial_allow_high_water_mark = app_memory.as_ptr();

        // Set up initial grant region.
        let mut kernel_memory_break = app_memory.as_mut_ptr().add(app_memory.len());

        // Now that we know we have the space we can setup the grant
        // pointers.
        kernel_memory_break = kernel_memory_break.offset(-(grant_ptrs_offset as isize));

        // This is safe today, as MPU constraints ensure that `memory_start`
        // will always be aligned on at least a word boundary, and that
        // memory_size will be aligned on at least a word boundary, and
        // `grant_ptrs_offset` is a multiple of the word size. Thus,
        // `kernel_memory_break` must be word aligned. While this is unlikely to
        // change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set all grant pointers to null.
        let opts =
            slice::from_raw_parts_mut(kernel_memory_break as *mut *const usize, grant_ptrs_num);
        for opt in opts.iter_mut() {
            *opt = ptr::null()
        }

        // Now that we know we have the space we can setup the memory for the
        // callbacks.
        kernel_memory_break = kernel_memory_break.offset(-(Self::CALLBACKS_OFFSET as isize));

        // This is safe today, as MPU constraints ensure that `memory_start`
        // will always be aligned on at least a word boundary, and that
        // memory_size will be aligned on at least a word boundary, and
        // `grant_ptrs_offset` is a multiple of the word size. Thus,
        // `kernel_memory_break` must be word aligned. While this is unlikely to
        // change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set up ring buffer for callbacks to the process.
        let callback_buf =
            slice::from_raw_parts_mut(kernel_memory_break as *mut Task, Self::CALLBACK_LEN);
        let tasks = RingBuffer::new(callback_buf);

        // Last thing in the kernel region of process RAM is the process struct.
        kernel_memory_break = kernel_memory_break.offset(-(Self::PROCESS_STRUCT_OFFSET as isize));
        let process_struct_memory_location = kernel_memory_break;

        // Create the Process struct in the app grant region.
        let mut process: &mut Process<C> =
            &mut *(process_struct_memory_location as *mut Process<'static, C>);

        // Ask the kernel for a unique identifier for this process that is being
        // created.
        let unique_identifier = kernel.create_process_identifier();

        // Save copies of these in case the app was compiled for fixed addresses
        // for later debugging.
        let fixed_address_flash = tbf_header.get_fixed_address_flash();
        let fixed_address_ram = tbf_header.get_fixed_address_ram();

        process
            .app_id
            .set(AppId::new(kernel, unique_identifier, index));
        process.kernel = kernel;
        process.chip = chip;
        process.allow_high_water_mark = Cell::new(initial_allow_high_water_mark);
        process.memory = app_memory;
        process.header = tbf_header;
        process.kernel_memory_break = Cell::new(kernel_memory_break);
        process.app_break = Cell::new(initial_app_brk);

        process.flash = app_flash;

        process.stored_state = MapCell::new(Default::default());
        // Mark this process as unstarted
        process.state = ProcessStateCell::new(process.kernel);
        process.fault_response = fault_response;
        process.restart_count = Cell::new(0);

        process.mpu_config = MapCell::new(mpu_config);
        process.mpu_regions = [
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
            Cell::new(None),
        ];
        process.tasks = MapCell::new(tasks);
        process.process_name = process_name.unwrap_or("");

        process.debug = MapCell::new(ProcessDebug {
            fixed_address_flash: fixed_address_flash,
            fixed_address_ram: fixed_address_ram,
            app_heap_start_pointer: None,
            app_stack_start_pointer: None,
            app_stack_min_pointer: None,
            syscall_count: 0,
            last_syscall: None,
            dropped_callback_count: 0,
            timeslice_expiration_count: 0,
        });

        let flash_protected_size = process.header.get_protected_size() as usize;
        let flash_app_start_addr = app_flash.as_ptr() as usize + flash_protected_size;

        process.tasks.map(|tasks| {
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: flash_app_start_addr,
                argument1: process.memory.as_ptr() as usize,
                argument2: process.memory.len() as usize,
                argument3: process.app_break.get() as usize,
            }));
        });

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
                app_memory_start,
                initial_app_brk,
                stored_state,
            )
        }) {
            Some(Ok(())) => {}
            _ => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash={:#010X}-{:#010X} process={:?} - couldn't initialize process",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len() - 1,
                        process_name
                    );
                }
                return Err(ProcessLoadError::InternalError);
            }
        };

        kernel.increment_work();

        // Return the process object and a remaining memory for processes slice.
        Ok((Some(process), unused_memory))
    }

    /// Attempt to restart the process.
    ///
    /// This function can be called when the process is in any state and
    /// attempts to reset all of its state and re-initialize it so that it can
    /// start running again.
    ///
    /// Restarting can fail for two general reasons:
    ///
    /// 1. The kernel chooses not to restart the process based on the policy the
    ///    kernel is using for restarting a specific process. For example, if a
    ///    process has restarted a number of times in a row the kernel may
    ///    decide to stop executing it.
    ///
    /// 2. Some state can no long be configured for the process. For example,
    ///    the syscall state for the process fails to initialize.
    ///
    /// After `restart()` runs the process will either be queued to run its
    /// `_start` function, or it will be left in `failure_state`.
    fn restart(&self, failure_state: State) {
        // Start with the generic terminate operations. This frees state for
        // this process and removes any pending tasks from the scheduler's
        // queue.
        self.terminate();

        // Set the state the process will be in if it cannot be restarted.
        self.state.update(failure_state);

        // Check if the restart policy for this app allows us to continue with
        // the restart.
        match self.fault_response {
            FaultResponse::Restart(restart_policy) => {
                // Decide what to do with this process. Should it be restarted?
                // Or should we leave it in a stopped & faulted state? If the
                // process is faulting too often we might not want to restart.
                // If we are not going to restart the process then we can just
                // leave it in the stopped faulted state by returning
                // immediately. This has the same effect as using the
                // `FaultResponse::Stop` policy.
                if !restart_policy.should_restart(self) {
                    return;
                }
            }

            _ => {
                // In all other cases the kernel has chosen not to restart the
                // process if it fails or exits for any reason. We can just
                // leave the process in the `failure_state` and return.
                return;
            }
        }

        // We need a new process identifier for this process since the restarted
        // version is in effect a new process. This is also necessary to
        // invalidate any stored `AppId`s that point to the old version of the
        // process. However, the process has not moved locations in the
        // processes array, so we copy the existing index.
        let old_index = self.app_id.get().index;
        let new_identifier = self.kernel.create_process_identifier();
        self.app_id
            .set(AppId::new(self.kernel, new_identifier, old_index));

        // Reset debug information that is per-execution and not per-process.
        self.debug.map(|debug| {
            debug.syscall_count = 0;
            debug.last_syscall = None;
            debug.dropped_callback_count = 0;
            debug.timeslice_expiration_count = 0;
        });

        // FLASH

        // We are going to start this process over again, so need the init_fn
        // location.
        let app_flash_address = self.flash_start();
        let init_fn = unsafe {
            app_flash_address.offset(self.header.get_init_function_offset() as isize) as usize
        };

        // Reset MPU region configuration.
        // TODO: ideally, this would be moved into a helper function used by both
        // create() and reset(), but process load debugging complicates this.
        // We just want to create new config with only flash and memory regions.
        let mut mpu_config: <<C as Chip>::MPU as MPU>::MpuConfig = Default::default();
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
            return;
        }

        // RAM

        // Re-determine the minimum amount of RAM the kernel must allocate to the process
        // based on the specific requirements of the syscall implementation.
        let min_process_memory_size = self
            .chip
            .userspace_kernel_boundary()
            .initial_process_app_brk_size();

        // Recalculate initial_kernel_memory_size as was done in create()
        let grant_ptr_size = mem::size_of::<*const usize>();
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        let grant_ptrs_offset = grant_ptrs_num * grant_ptr_size;

        let initial_kernel_memory_size =
            grant_ptrs_offset + Self::CALLBACKS_OFFSET + Self::PROCESS_STRUCT_OFFSET;

        let app_mpu_mem = self.chip.mpu().allocate_app_memory_region(
            self.memory.as_ptr() as *const u8,
            self.memory.len(),
            self.memory.len(), //we want exactly as much as we had before restart
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
                return;
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

        // Drop the old config and use the clean one
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
            Err(_) => {
                // We couldn't initialize the architecture-specific
                // state for this process. This shouldn't happen since
                // the app was able to be started before, but at this
                // point the app is no longer valid. The best thing we
                // can do now is leave the app as still faulted and not
                // schedule it.
                return;
            }
        };

        // And queue up this app to be restarted.
        let flash_protected_size = self.header.get_protected_size() as usize;
        let flash_app_start = app_flash_address as usize + flash_protected_size;

        // Mark the state as `Unstarted` for the scheduler.
        self.state.update(State::Unstarted);

        // Mark that we restarted this process.
        self.restart_count.increment();

        // Enqueue the initial function.
        self.tasks.map(|tasks| {
            tasks.enqueue(Task::FunctionCall(FunctionCall {
                source: FunctionCallSource::Kernel,
                pc: init_fn,
                argument0: flash_app_start,
                argument1: self.memory.as_ptr() as usize,
                argument2: self.memory.len() as usize,
                argument3: self.app_break.get() as usize,
            }));
        });

        // Mark that the process is ready to run.
        self.kernel.increment_work();
    }

    /// Stop and clear a process's state.
    ///
    /// This will end the process, but does not reset it such that it could be
    /// restarted and run again. This function instead frees grants and any
    /// queued tasks for this process, but leaves the debug information about
    /// the process and other state intact.
    fn terminate(&self) {
        // Remove the tasks that were scheduled for the app from the
        // amount of work queue.
        let tasks_len = self.tasks.map_or(0, |tasks| tasks.len());
        for _ in 0..tasks_len {
            self.kernel.decrement_work();
        }

        // And remove those tasks
        self.tasks.map(|tasks| {
            tasks.empty();
        });

        // Clear any grant regions this app has setup with any capsules.
        unsafe {
            self.grant_ptrs_reset();
        }

        // Mark the app as stopped so the scheduler won't try to run it.
        self.state.update(State::StoppedFaulted);
    }

    /// Checks if the buffer represented by the passed in base pointer and size
    /// are within the memory bounds currently exposed to the processes (i.e.
    /// ending at `app_break`. If this method returns true, the buffer
    /// is guaranteed to be accessible to the process and to not overlap with
    /// the grant region.
    fn in_app_owned_memory(&self, buf_start_addr: *const u8, size: usize) -> bool {
        let buf_end_addr = buf_start_addr.wrapping_add(size);

        buf_end_addr >= buf_start_addr
            && buf_start_addr >= self.mem_start()
            && buf_end_addr <= self.app_break.get()
    }

    /// Reset all `grant_ptr`s to NULL.
    // This is safe today, as MPU constraints ensure that `mem_end` will always
    // be aligned on at least a word boundary. While this is unlikely to
    // change, it should be more proactively enforced.
    //
    // TODO: https://github.com/tock/tock/issues/1739
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn grant_ptrs_reset(&self) {
        let grant_ptrs_num = self.kernel.get_grant_count_and_finalize();
        for grant_num in 0..grant_ptrs_num {
            let grant_num = grant_num as isize;
            let ctr_ptr = (self.mem_end() as *mut *mut usize).offset(-(grant_num + 1));
            write_volatile(ctr_ptr, ptr::null_mut());
        }
    }

    /// Check if the process is active.
    ///
    /// "Active" is defined as the process can resume executing in the future.
    /// This means its state in the `Process` struct is still valid, and that
    /// the kernel could resume its execution without completely restarting and
    /// resetting its state.
    ///
    /// A process is inactive if the kernel cannot resume its execution, such as
    /// if the process faults and is in an invalid state, or if the process
    /// explicitly exits.
    fn is_active(&self) -> bool {
        let current_state = self.state.get();
        current_state != State::StoppedFaulted && current_state != State::Fault
    }
}

