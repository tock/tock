//! Support for creating and running userspace applications.

use core::cell::Cell;
use core::convert::TryInto;
use core::fmt;
use core::fmt::Write;
use core::ptr::{write_volatile, NonNull};
use core::{mem, ptr, slice, str};

use crate::callback::{AppId, CallbackId};
use crate::capabilities::ProcessManagementCapability;
use crate::common::cells::{MapCell, NumericCellExt};
use crate::common::{Queue, RingBuffer};
use crate::config;
use crate::debug;
use crate::ipc;
use crate::mem::{AppSlice, Shared};
use crate::platform::mpu::{self, MPU};
use crate::platform::Chip;
use crate::returncode::ReturnCode;
use crate::sched::Kernel;
use crate::syscall::{self, Syscall, UserspaceKernelBoundary};
use crate::tbfheader;
use core::cmp::max;

/// Errors that can occur when trying to load and create processes.
pub enum ProcessLoadError {
    /// The TBF header for the process could not be successfully parsed.
    TbfHeaderParseFailure(tbfheader::TbfParseError),

    /// Not enough flash remaining to parse a process and its header.
    NotEnoughFlash,

    /// Not enough memory to meet the amount requested by a process. Modify the
    /// process to request less memory, flash fewer processes, or increase the
    /// size of the region your board reserves for process memory.
    NotEnoughMemory,

    /// A process was loaded with a length in flash that the MPU does not
    /// support. The fix is probably to correct the process size, but this could
    /// also be caused by a bad MPU implementation.
    MpuInvalidFlashLength,

    /// A process specified a fixed memory address that it needs its memory
    /// range to start at, and the kernel did not or could not give the process
    /// a memory region starting at that address.
    MemoryAddressMismatch {
        actual_address: u32,
        expected_address: u32,
    },

    /// A process specified that its binary must start at a particular address,
    /// and that is not the address the binary is actually placed at.
    IncorrectFlashAddress {
        actual_address: u32,
        expected_address: u32,
    },

    /// Process loading error due (likely) to a bug in the kernel. If you get
    /// this error please open a bug report.
    InternalError,
}

impl From<tbfheader::TbfParseError> for ProcessLoadError {
    /// Convert between a TBF Header parse error and a process load error.
    ///
    /// We note that the process load error is because a TBF header failed to
    /// parse, and just pass through the parse error.
    fn from(error: tbfheader::TbfParseError) -> Self {
        ProcessLoadError::TbfHeaderParseFailure(error)
    }
}

impl fmt::Debug for ProcessLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessLoadError::TbfHeaderParseFailure(tbf_parse_error) => {
                write!(f, "Error parsing TBF header\n")?;
                write!(f, "{:?}", tbf_parse_error)
            }

            ProcessLoadError::NotEnoughFlash => {
                write!(f, "Not enough flash available for app linked list")
            }

            ProcessLoadError::NotEnoughMemory => {
                write!(f, "Not able to meet memory requirements requested by apps")
            }

            ProcessLoadError::MpuInvalidFlashLength => {
                write!(f, "App flash length not supported by MPU")
            }

            ProcessLoadError::MemoryAddressMismatch {
                actual_address,
                expected_address,
            } => write!(
                f,
                "App memory does not match requested address Actual:{:#x}, Expected:{:#x}",
                actual_address, expected_address
            ),

            ProcessLoadError::IncorrectFlashAddress {
                actual_address,
                expected_address,
            } => write!(
                f,
                "App flash does not match requested address. Actual:{:#x}, Expected:{:#x}",
                actual_address, expected_address
            ),

            ProcessLoadError::InternalError => write!(f, "Error in kernel. Likely a bug."),
        }
    }
}

/// Helper function to load processes from flash into an array of active
/// processes. This is the default template for loading processes, but a board
/// is able to create its own `load_processes()` function and use that instead.
///
/// Processes are found in flash starting from the given address and iterating
/// through Tock Binary Format headers. Processes are given memory out of the
/// `app_memory` buffer until either the memory is exhausted or the allocated
/// number of processes are created, with process structures placed in the
/// provided array. How process faults are handled by the kernel is also
/// selected.
pub fn load_processes<C: Chip>(
    kernel: &'static Kernel,
    chip: &'static C,
    app_flash: &'static [u8],
    app_memory: &mut [u8],
    procs: &'static mut [Option<&'static dyn ProcessType>],
    fault_response: FaultResponse,
    _capability: &dyn ProcessManagementCapability,
) -> Result<(), ProcessLoadError> {
    let mut remaining_flash = app_flash;
    let mut app_memory_ptr = app_memory.as_mut_ptr();
    let mut app_memory_size = app_memory.len();

    if config::CONFIG.debug_load_processes {
        debug!(
            "Loading processes from flash={:#010X} into sram=[{:#010X}:{:#010X}]",
            app_flash.as_ptr() as usize,
            app_memory_ptr as usize,
            app_memory_ptr as usize + app_memory_size
        );
    }

    for i in 0..procs.len() {
        unsafe {
            // Get the first eight bytes of flash to check if there is another
            // app.
            let test_header_slice = match remaining_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // Not enough flash to test for another app. This just means
                    // we are at the end of flash, and there are no more apps to
                    // load.
                    return Ok(());
                }
            };

            // Pass the first eight bytes to tbfheader to parse out the length
            // of the tbf header and app. We then use those values to see if we
            // have enough flash remaining to parse the remainder of the header.
            let (version, header_length, app_length) = match tbfheader::parse_tbf_header_lengths(
                test_header_slice
                    .try_into()
                    .or(Err(ProcessLoadError::InternalError))?,
            ) {
                Ok((v, hl, al)) => (v, hl, al),
                Err(_tbferr) => {
                    // Since Tock apps use a linked list, it is very possible
                    // the header we started to parse is intentionally invalid
                    // to signal the end of apps. This is ok and just means we
                    // have finished loading apps.
                    return Ok(());
                }
            };

            // Now we can get a slice which only encompasses the app. At this
            // point, since the version number in the beginning of the header is
            // valid, we consider further parsing errors to be actual errors and
            // report them to the caller.
            let app_flash = remaining_flash
                .get(0..app_length as usize)
                .ok_or(ProcessLoadError::NotEnoughFlash)?;

            // Try to create a process object from that app slice.
            let (process, memory_offset) = Process::create(
                kernel,
                chip,
                app_flash,
                header_length as usize,
                version,
                app_memory_ptr,
                app_memory_size,
                fault_response,
                i,
            )?;

            // Check to see if actually got a valid process to execute. If we
            // didn't and we didn't get a loading error (aka we got to this
            // point), then the app is a disabled process or just padding.
            if process.is_some() {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "Loaded process[{}] from flash=[{:#010X}:{:#010X}] into sram=[{:#010X}:{:#010X}] = {:?}",
                        i,
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len(),
                        app_memory_ptr as usize,
                        app_memory_ptr as usize + memory_offset,
                        process.map(|p| p.get_process_name())
                    );
                }
                procs[i] = process;
            }

            // Advance in our buffers before seeing if there is an additional
            // process to load.
            remaining_flash = remaining_flash
                .get(app_flash.len()..)
                .ok_or(ProcessLoadError::NotEnoughFlash)?;
            app_memory_ptr = app_memory_ptr.add(memory_offset);
            app_memory_size -= memory_offset;
        }
    }

    Ok(())
}

/// This trait is implemented by process structs.
pub trait ProcessType {
    /// Returns the process's identifier
    fn appid(&self) -> AppId;

    /// Queue a `Task` for the process. This will be added to a per-process
    /// buffer and executed by the scheduler. `Task`s are some function the app
    /// should run, for example a callback or an IPC call.
    ///
    /// This function returns `true` if the `Task` was successfully enqueued,
    /// and `false` otherwise. This is represented as a simple `bool` because
    /// this is passed to the capsule that tried to schedule the `Task`.
    ///
    /// This will fail if the process is no longer active, and therefore cannot
    /// execute any new tasks.
    fn enqueue_task(&self, task: Task) -> bool;

    /// Remove the scheduled operation from the front of the queue and return it
    /// to be handled by the scheduler.
    ///
    /// If there are no `Task`s in the queue for this process this will return
    /// `None`.
    fn dequeue_task(&self) -> Option<Task>;

    /// Remove all scheduled callbacks for a given callback id from the task
    /// queue.
    fn remove_pending_callbacks(&self, callback_id: CallbackId);

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

    /// Creates an `AppSlice` from the given offset and size in process memory.
    ///
    /// If `buf_start_addr` is NULL this will have no effect and the return
    /// value will be `None` to signal the capsule to drop the buffer.
    ///
    /// If the process is not active then this will return an error as it is not
    /// valid to "allow" a buffer for a process that will not resume executing.
    /// In practice this case should not happen as the process will not be
    /// executing to call the allow syscall.
    ///
    /// ## Returns
    ///
    /// If the buffer is null (a zero-valued offset) this returns `None`,
    /// signaling the capsule to delete the entry. If the buffer is within the
    /// process's accessible memory, returns an `AppSlice` wrapping that buffer.
    /// Otherwise, returns an error `ReturnCode`.
    fn allow(
        &self,
        buf_start_addr: *const u8,
        size: usize,
    ) -> Result<Option<AppSlice<Shared, u8>>, ReturnCode>;

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

    /// Create new memory in the grant region, and check that the MPU region
    /// covering program memory does not extend past the kernel memory break.
    ///
    /// This will return `None` and fail if the process is inactive.
    fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>>;

    unsafe fn free(&self, _: *mut u8);

    /// Get the grant pointer for this grant number.
    ///
    /// This will return `None` if the process is inactive and the grant region
    /// cannot be used.
    ///
    /// Caution: The grant may not have been allocated yet, so it is possible
    /// for this grant pointer to be null.
    fn get_grant_ptr(&self, grant_num: usize) -> Option<*mut u8>;

    /// Set the grant pointer for this grant number.
    ///
    /// Note: This method trusts arguments completely, that is, it assumes the
    /// index into the grant array is valid and the pointer is to an allocated
    /// grant region in the process memory.
    unsafe fn set_grant_ptr(&self, grant_num: usize, grant_ptr: *mut u8);

    // functions for processes that are architecture specific

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    unsafe fn set_syscall_return_value(&self, return_value: isize);

    /// Set the function that is to be executed when the process is resumed.
    ///
    /// It is not valid to call this function when the process is inactive (i.e.
    /// the process will not run again).
    unsafe fn set_process_function(&self, callback: FunctionCall);

    /// Context switch to a specific process.
    ///
    /// This will return `None` if the process is inactive and cannot be
    /// switched to.
    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason>;

    /// Print out the memory map (Grant region, heap, stack, program
    /// memory, BSS, and data sections) of this process.
    unsafe fn print_memory_map(&self, writer: &mut dyn Write);

    /// Print out the full state of the process: its memory map, its
    /// context, and the state of the memory protection unit (MPU).
    unsafe fn print_full_process(&self, writer: &mut dyn Write);

    // debug

    /// Returns how many syscalls this app has called.
    fn debug_syscall_count(&self) -> usize;

    /// Returns how many callbacks for this process have been dropped.
    fn debug_dropped_callback_count(&self) -> usize;

    /// Returns how many times this process has exceeded its timeslice.
    fn debug_timeslice_expiration_count(&self) -> usize;

    /// Increment the number of times the process has exceeded its timeslice.
    fn debug_timeslice_expired(&self);

    /// Increment the number of times the process called a syscall and record
    /// the last syscall that was called.
    fn debug_syscall_called(&self, last_syscall: Syscall);
}

/// Generic trait for implementing process restart policies.
///
/// This policy allows a board to specify how the kernel should decide whether
/// to restart an app after it crashes.
pub trait ProcessRestartPolicy {
    /// Decide whether to restart the `process` or not.
    ///
    /// Returns `true` if the process should be restarted, `false` otherwise.
    fn should_restart(&self, process: &dyn ProcessType) -> bool;
}

/// Implementation of `ProcessRestartPolicy` that uses a threshold to decide
/// whether to restart an app. If the app has been restarted more times than the
/// threshold then the app will no longer be restarted.
pub struct ThresholdRestart {
    threshold: usize,
}

impl ThresholdRestart {
    pub const fn new(threshold: usize) -> ThresholdRestart {
        ThresholdRestart { threshold }
    }
}

impl ProcessRestartPolicy for ThresholdRestart {
    fn should_restart(&self, process: &dyn ProcessType) -> bool {
        process.get_restart_count() <= self.threshold
    }
}

/// Implementation of `ProcessRestartPolicy` that uses a threshold to decide
/// whether to restart an app. If the app has been restarted more times than the
/// threshold then the system will panic.
pub struct ThresholdRestartThenPanic {
    threshold: usize,
}

impl ThresholdRestartThenPanic {
    pub const fn new(threshold: usize) -> ThresholdRestartThenPanic {
        ThresholdRestartThenPanic { threshold }
    }
}

impl ProcessRestartPolicy for ThresholdRestartThenPanic {
    fn should_restart(&self, process: &dyn ProcessType) -> bool {
        if process.get_restart_count() <= self.threshold {
            true
        } else {
            panic!("Restart threshold surpassed!");
        }
    }
}

/// Implementation of `ProcessRestartPolicy` that unconditionally restarts the
/// app.
pub struct AlwaysRestart {}

impl AlwaysRestart {
    pub const fn new() -> AlwaysRestart {
        AlwaysRestart {}
    }
}

impl ProcessRestartPolicy for AlwaysRestart {
    fn should_restart(&self, _process: &dyn ProcessType) -> bool {
        true
    }
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
}

impl From<Error> for ReturnCode {
    fn from(err: Error) -> ReturnCode {
        match err {
            Error::OutOfMemory => ReturnCode::ENOMEM,
            Error::AddressOutOfBounds => ReturnCode::EINVAL,
            Error::NoSuchApp => ReturnCode::EINVAL,
            Error::InactiveApp => ReturnCode::FAIL,
            Error::KernelError => ReturnCode::FAIL,
        }
    }
}

/// Various states a process can be in.
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

    /// The process is stopped, and it was stopped after it faulted. This
    /// basically means the app crashed, and the kernel decided to just stop it
    /// and continue executing other things. The process cannot be restarted
    /// without being reset first.
    StoppedFaulted,

    /// The process has caused a fault.
    Fault,

    /// The process has never actually been executed. This of course happens
    /// when the board first boots and the kernel has not switched to any
    /// processes yet. It can also happen if an process is terminated and all
    /// of its state is reset as if it has not been executed yet.
    Unstarted,
}

/// The reaction the kernel should take when an app encounters a fault.
///
/// When an exception occurs during an app's execution (a common example is an
/// app trying to access memory outside of its allowed regions) the system will
/// trap back to the kernel, and the kernel has to decide what to do with the
/// app at that point.
#[derive(Copy, Clone)]
pub enum FaultResponse {
    /// Generate a `panic!()` call and crash the entire system. This is useful
    /// for debugging applications as the error is displayed immediately after
    /// it occurs.
    Panic,

    /// Attempt to cleanup and restart the app which caused the fault. This
    /// resets the app's memory to how it was when the app was started and
    /// schedules the app to run again from its init function.
    ///
    /// The provided restart policy is used to determine whether to reset the
    /// app, and can be specified on a per-app basis.
    Restart(&'static dyn ProcessRestartPolicy),

    /// Stop the app by no longer scheduling it to run.
    Stop,
}

#[derive(Copy, Clone)]
pub enum Task {
    FunctionCall(FunctionCall),
    IPC((AppId, ipc::IPCCallbackType)),
}

/// Enumeration to identify whether a function call comes directly from the
/// kernel or from a callback subscribed through a driver.
///
/// An example of kernel function is the application entry point.
#[derive(Copy, Clone, Debug)]
pub enum FunctionCallSource {
    Kernel, // For functions coming directly from the kernel, such as `init_fn`.
    Driver(CallbackId),
}

/// Struct that defines a callback that can be passed to a process. The callback
/// takes four arguments that are `Driver` and callback specific, so they are
/// represented generically here.
///
/// Likely these four arguments will get passed as the first four register
/// values, but this is architecture-dependent.
///
/// A `FunctionCall` also identifies the callback that scheduled it, if any, so
/// that it can be unscheduled when the process unsubscribes from this callback.
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
    /// Where the process has started its heap in RAM.
    app_heap_start_pointer: Option<*const u8>,

    /// Where the start of the stack is for the process. If the kernel does the
    /// PIC setup for this app then we know this, otherwise we need the app to
    /// tell us where it put its stack.
    app_stack_start_pointer: Option<*const u8>,

    /// How low have we ever seen the stack pointer.
    min_stack_pointer: *const u8,

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

    /// Copy of where the kernel memory break is when the app is first started.
    /// This is handy if the app is restarted so we know where to reset
    /// the kernel_memory break to without having to recalculate it.
    original_kernel_memory_break: *const u8,

    /// Pointer to the end of process RAM that has been sbrk'd to the process.
    app_break: Cell<*const u8>,
    original_app_break: *const u8,

    /// Pointer to high water mark for process buffers shared through `allow`
    allow_high_water_mark: Cell<*const u8>,
    original_allow_high_water_mark: *const u8,

    /// Saved when the app switches to the kernel.
    current_stack_pointer: Cell<*const u8>,
    original_stack_pointer: *const u8,

    /// Process flash segment. This is the region of nonvolatile flash that
    /// the process occupies.
    flash: &'static [u8],

    /// Collection of pointers to the TBF header in flash.
    header: tbfheader::TbfHeader,

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

impl<C: Chip> ProcessType for Process<'a, C> {
    fn appid(&self) -> AppId {
        self.app_id.get()
    }

    fn enqueue_task(&self, task: Task) -> bool {
        // If this app is in a `Fault` state then we shouldn't schedule
        // any work for it.
        if !self.is_active() {
            return false;
        }

        self.kernel.increment_work();

        let ret = self.tasks.map_or(false, |tasks| tasks.enqueue(task));

        // Make a note that we lost this callback if the enqueue function
        // fails.
        if ret == false {
            self.debug.map(|debug| {
                debug.dropped_callback_count += 1;
            });
        }

        ret
    }

    fn remove_pending_callbacks(&self, callback_id: CallbackId) {
        self.tasks.map(|tasks| {
            let count_before = tasks.len();
            tasks.retain(|task| match task {
                // Remove only tasks that are function calls with an id equal
                // to `callback_id`.
                Task::FunctionCall(function_call) => match function_call.source {
                    FunctionCallSource::Kernel => true,
                    FunctionCallSource::Driver(id) => id != callback_id,
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
            self.state.set(State::Yielded);
            self.kernel.decrement_work();
        }
    }

    fn stop(&self) {
        match self.state.get() {
            State::Running => self.state.set(State::StoppedRunning),
            State::Yielded => self.state.set(State::StoppedYielded),
            _ => {} // Do nothing
        }
    }

    fn resume(&self) {
        match self.state.get() {
            State::StoppedRunning => self.state.set(State::Running),
            State::StoppedYielded => self.state.set(State::Yielded),
            _ => {} // Do nothing
        }
    }

    fn set_fault_state(&self) {
        self.state.set(State::Fault);

        match self.fault_response {
            FaultResponse::Panic => {
                // process faulted. Panic and print status
                panic!("Process {} had a fault", self.process_name);
            }
            FaultResponse::Restart(restart_policy) => {
                // Start with the generic terminate operations. This frees state
                // for this process and removes any pending tasks from the
                // scheduler's queue.
                self.terminate();

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

                // We need a new process identifier for this app since the
                // restarted version is in effect a new process. This is also
                // necessary to invalidate any stored `AppId`s that point to the
                // old version of the app. However, the app has not moved
                // locations in the processes array, so we copy the existing
                // index.
                let old_index = self.app_id.get().index;
                let new_identifier = self.kernel.create_process_identifier();
                self.app_id
                    .set(AppId::new(self.kernel, new_identifier, old_index));

                // Update debug information
                self.debug.map(|debug| {
                    // Reset some state for the process.
                    debug.syscall_count = 0;
                    debug.last_syscall = None;
                    debug.dropped_callback_count = 0;
                    debug.timeslice_expiration_count = 0;
                });

                // We are going to start this process over again, so need
                // the init_fn location.
                let app_flash_address = self.flash_start();
                let init_fn = unsafe {
                    app_flash_address.offset(self.header.get_init_function_offset() as isize)
                        as usize
                };

                // Reset memory pointers.
                self.kernel_memory_break
                    .set(self.original_kernel_memory_break);
                self.app_break.set(self.original_app_break);
                self.current_stack_pointer.set(self.original_stack_pointer);
                self.allow_high_water_mark
                    .set(self.original_allow_high_water_mark);

                // Handle any architecture-specific requirements for a process
                // when it first starts (as it would when it is new).
                let new_stack_pointer_res =
                    self.stored_state.map_or(Err(()), |stored_state| unsafe {
                        self.chip.userspace_kernel_boundary().initialize_process(
                            self.sp(),
                            self.sp() as usize - self.memory.as_ptr() as usize,
                            stored_state,
                        )
                    });
                match new_stack_pointer_res {
                    Ok(new_stack_pointer) => {
                        self.current_stack_pointer.set(new_stack_pointer as *mut u8);
                        self.debug_set_max_stack_depth();
                    }
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
                self.state.set(State::Unstarted);

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
                debug.min_stack_pointer = stack_pointer;
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
            self.chip.mpu().configure_mpu(&config);
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
                    self.chip.mpu().configure_mpu(&config);
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
                    let new_water_mark = max(self.allow_high_water_mark.get(), buf_end_addr);
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
        self.stored_state.map(|stored_state| {
            self.chip
                .userspace_kernel_boundary()
                .set_syscall_return_value(self.sp(), stored_state, return_value);
        });
    }

    unsafe fn set_process_function(&self, callback: FunctionCall) {
        // First we need to get how much memory is available for this app's
        // stack. Since the stack is at the bottom of the process's memory
        // region, this is straightforward.
        let remaining_stack_bytes = self.sp() as usize - self.memory.as_ptr() as usize;

        // Next we should see if we can actually add the frame to the process's
        // stack. Architecture-specific code handles actually doing the push
        // since we don't know the details of exactly what the stack frames look
        // like.
        match self.stored_state.map(|stored_state| {
            self.chip.userspace_kernel_boundary().set_process_function(
                self.sp(),
                remaining_stack_bytes,
                stored_state,
                callback,
            )
        }) {
            Some(Ok(stack_bottom)) => {
                // If we got an `Ok` with the new stack pointer we are all
                // set and should mark that this process is ready to be
                // scheduled.

                // We just setup up a new callback to do, which means this
                // process wants to execute, so we set that there is work to
                // be done.
                self.kernel.increment_work();

                // Move this process to the "running" state so the scheduler
                // will schedule it.
                self.state.set(State::Running);

                // Update helpful debugging metadata.
                self.current_stack_pointer.set(stack_bottom as *mut u8);
                self.debug_set_max_stack_depth();
            }

            Some(Err(bad_stack_bottom)) => {
                // If we got an Error, then there was not enough room on the
                // stack to allow the process to execute this function given the
                // details of the particular architecture this is running on.
                // This process has essentially faulted, so we mark it as such.
                // We also update the debugging metadata so that if the process
                // fault message prints then it should be easier to debug that
                // the process exceeded its stack.
                self.debug.map(|debug| {
                    let bad_stack_bottom = bad_stack_bottom as *const u8;
                    if bad_stack_bottom < debug.min_stack_pointer {
                        debug.min_stack_pointer = bad_stack_bottom;
                    }
                });
                self.set_fault_state();
            }

            None => {
                // We should never be here since `stored_state` should always be occupied.
                self.set_fault_state();
            }
        }
    }

    unsafe fn switch_to(&self) -> Option<syscall::ContextSwitchReason> {
        // Cannot switch to an invalid process
        if !self.is_active() {
            return None;
        }

        let switch_reason = self.stored_state.map(|stored_state| {
            let (stack_pointer, switch_reason) = self
                .chip
                .userspace_kernel_boundary()
                .switch_to_process(self.sp(), stored_state);
            self.current_stack_pointer.set(stack_pointer as *const u8);
            switch_reason
        });

        // Update debug state as needed after running this process.
        self.debug.map(|debug| {
            // Update max stack depth if needed.
            if self.current_stack_pointer.get() < debug.min_stack_pointer {
                debug.min_stack_pointer = self.current_stack_pointer.get();
            }

            // More debugging help. If this occurred because of a timeslice
            // expiration, mark that so we can check later if a process is
            // exceeding its timeslices too often.
            if switch_reason == Some(syscall::ContextSwitchReason::TimesliceExpired) {
                debug.timeslice_expiration_count += 1;
            }
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
        let sram_stack_bottom =
            self.debug
                .map_or(ptr::null(), |debug| debug.min_stack_pointer) as usize;
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
             App: {}   -   [{:?}]\
             \r\n Events Queued: {}   Syscall Count: {}   Dropped Callback Count: {}\
             \n Restart Count: {}\n",
            self.process_name,
            self.state.get(),
            events_queued,
            syscall_count,
            dropped_callback_count,
            restart_count,
        ));

        let _ = match last_syscall {
            Some(syscall) => writer.write_fmt(format_args!(" Last Syscall: {:?}", syscall)),
            None => writer.write_str(" Last Syscall: None"),
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

        match sram_stack_start {
            Some(sram_stack_start) => {
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
            None => {
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
            sram_stack_bottom,
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
            self.chip
                .userspace_kernel_boundary()
                .print_context(self.sp(), stored_state, writer);
        });

        // Display the current state of the MPU for this process.
        self.mpu_config.map(|config| {
            let _ = writer.write_fmt(format_args!("{}", config));
        });

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
}

fn exceeded_check(size: usize, allocated: usize) -> &'static str {
    if size > allocated {
        " EXCEEDED!"
    } else {
        "          "
    }
}

impl<C: 'static + Chip> Process<'a, C> {
    crate unsafe fn create(
        kernel: &'static Kernel,
        chip: &'static C,
        app_flash: &'static [u8],
        header_length: usize,
        app_version: u16,
        remaining_app_memory: *mut u8,
        remaining_app_memory_size: usize,
        fault_response: FaultResponse,
        index: usize,
    ) -> Result<(Option<&'static dyn ProcessType>, usize), ProcessLoadError> {
        // Get a slice for just the app header.
        let header_flash = app_flash
            .get(0..header_length as usize)
            .ok_or(ProcessLoadError::NotEnoughFlash)?;

        // Parse the full TBF header to see if this is a valid app. If the
        // header can't parse, we will error right here.
        let tbf_header = tbfheader::parse_tbf_header(header_flash, app_version)?;

        // First thing: check that the process is at the correct location in
        // flash if the TBF header specified a fixed address. If there is a
        // mismatch we catch that early.
        if let Some(fixed_flash_start) = tbf_header.get_fixed_address_flash() {
            // The flash address in the header is based on the app binary,
            // so we need to take into account the header length.
            if fixed_flash_start + tbf_header.get_protected_size() != app_flash.as_ptr() as u32 {
                let actual_address = app_flash.as_ptr() as u32 + tbf_header.get_protected_size();
                let expected_address = fixed_flash_start;
                return Err(ProcessLoadError::IncorrectFlashAddress {
                    actual_address,
                    expected_address,
                });
            }
        }

        let process_name = tbf_header.get_package_name();

        // If this isn't an app (i.e. it is padding) or it is an app but it
        // isn't enabled, then we can skip it but increment past its flash.
        if !tbf_header.is_app() || !tbf_header.enabled() {
            if config::CONFIG.debug_load_processes {
                if !tbf_header.is_app() {
                    debug!(
                        "[!] flash=[{:#010X}:{:#010X}] process={:?} - process isn't an app",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len(),
                        process_name
                    );
                }
                if !tbf_header.enabled() {
                    debug!(
                        "[!] flash=[{:#010X}:{:#010X}] process={:?} - process isn't enabled",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len(),
                        process_name
                    );
                }
            }
            return Ok((None, 0));
        }

        // Otherwise, actually load the app.
        let mut min_app_ram_size = tbf_header.get_minimum_app_ram_size() as usize;
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
                    "[!] flash=[{:#010X}:{:#010X}] process={:?} - couldn't allocate MPU region for flash",
                    app_flash.as_ptr() as usize,
                    app_flash.as_ptr() as usize + app_flash.len(),
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

        // Allocate memory for callback ring buffer.
        let callback_size = mem::size_of::<Task>();
        let callback_len = 10;
        let callbacks_offset = callback_len * callback_size;

        // Make room to store this process's metadata.
        let process_struct_offset = mem::size_of::<Process<C>>();

        // Initial sizes of the app-owned and kernel-owned parts of process memory.
        // Provide the app with plenty of initial process accessible memory.
        let initial_kernel_memory_size =
            grant_ptrs_offset + callbacks_offset + process_struct_offset;
        let initial_app_memory_size = 3 * 1024;

        if min_app_ram_size < initial_app_memory_size {
            min_app_ram_size = initial_app_memory_size;
        }

        // Minimum memory size for the process.
        let min_total_memory_size = min_app_ram_size + initial_kernel_memory_size;

        // Determine where process memory will go and allocate MPU region for app-owned memory.
        let (memory_start, memory_size) = match chip.mpu().allocate_app_memory_region(
            remaining_app_memory as *const u8,
            remaining_app_memory_size,
            min_total_memory_size,
            initial_app_memory_size,
            initial_kernel_memory_size,
            mpu::Permissions::ReadWriteOnly,
            &mut mpu_config,
        ) {
            Some((memory_start, memory_size)) => (memory_start, memory_size),
            None => {
                // Failed to load process. Insufficient memory.
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash=[{:#010X}:{:#010X}] process={:?} - couldn't allocate memory region of size >= {:#X}",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len(),
                        process_name,
                        min_total_memory_size
                    );
                }
                return Err(ProcessLoadError::NotEnoughMemory);
            }
        };

        // Compute how much padding before start of process memory.
        let memory_padding_size = (memory_start as usize) - (remaining_app_memory as usize);

        // Set up process memory.
        let app_memory = slice::from_raw_parts_mut(memory_start as *mut u8, memory_size);

        // Check if the memory region is valid for the process. If a process
        // included a fixed address for the start of RAM in its TBF header (this
        // field is optional, processes that are position independent do not
        // need a fixed address) then we check that we used the same address
        // when we allocated it RAM.
        if let Some(fixed_memory_start) = tbf_header.get_fixed_address_ram() {
            if fixed_memory_start != app_memory.as_ptr() as u32 {
                let actual_address = app_memory.as_ptr() as u32;
                let expected_address = fixed_memory_start;
                return Err(ProcessLoadError::MemoryAddressMismatch {
                    actual_address,
                    expected_address,
                });
            }
        }

        // Set the initial process stack and memory to 3072 bytes.
        let initial_stack_pointer = memory_start.add(initial_app_memory_size);
        let initial_sbrk_pointer = memory_start.add(initial_app_memory_size);

        // Set up initial grant region.
        let mut kernel_memory_break = app_memory.as_mut_ptr().add(app_memory.len());

        // Now that we know we have the space we can setup the grant
        // pointers.
        kernel_memory_break = kernel_memory_break.offset(-(grant_ptrs_offset as isize));

        // This is safe today, as MPU constraints ensure that `memory_start` will always
        // be aligned on at least a word boundary, and that memory_size will be aligned on at least
        // a word boundary, and `grant_ptrs_offset` is a multiple of the word size.
        // Thus, `kernel_memory_break` must be word aligned.
        // While this is unlikely to change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set all pointers to null.
        let opts =
            slice::from_raw_parts_mut(kernel_memory_break as *mut *const usize, grant_ptrs_num);
        for opt in opts.iter_mut() {
            *opt = ptr::null()
        }

        // Now that we know we have the space we can setup the memory
        // for the callbacks.
        kernel_memory_break = kernel_memory_break.offset(-(callbacks_offset as isize));

        // This is safe today, as MPU constraints ensure that `memory_start` will always
        // be aligned on at least a word boundary, and that memory_size will be aligned on at least
        // a word boundary, and `grant_ptrs_offset` is a multiple of the word size.
        // Thus, `kernel_memory_break` must be word aligned.
        // While this is unlikely to change, it should be more proactively enforced.
        //
        // TODO: https://github.com/tock/tock/issues/1739
        #[allow(clippy::cast_ptr_alignment)]
        // Set up ring buffer.
        let callback_buf =
            slice::from_raw_parts_mut(kernel_memory_break as *mut Task, callback_len);
        let tasks = RingBuffer::new(callback_buf);

        // Last thing is the process struct.
        kernel_memory_break = kernel_memory_break.offset(-(process_struct_offset as isize));
        let process_struct_memory_location = kernel_memory_break;

        // Determine the debug information to the best of our
        // understanding. If the app is doing all of the PIC fixup and
        // memory management we don't know much.
        let app_heap_start_pointer = None;
        let app_stack_start_pointer = None;

        // Create the Process struct in the app grant region.
        let mut process: &mut Process<C> =
            &mut *(process_struct_memory_location as *mut Process<'static, C>);

        // Ask the kernel for a unique identifier for this process that is
        // being created.
        let unique_identifier = kernel.create_process_identifier();

        process
            .app_id
            .set(AppId::new(kernel, unique_identifier, index));
        process.kernel = kernel;
        process.chip = chip;
        process.memory = app_memory;
        process.header = tbf_header;
        process.kernel_memory_break = Cell::new(kernel_memory_break);
        process.original_kernel_memory_break = kernel_memory_break;
        process.app_break = Cell::new(initial_sbrk_pointer);
        process.original_app_break = initial_sbrk_pointer;
        process.allow_high_water_mark = Cell::new(remaining_app_memory);
        process.original_allow_high_water_mark = remaining_app_memory;
        process.current_stack_pointer = Cell::new(initial_stack_pointer);
        process.original_stack_pointer = initial_stack_pointer;

        process.flash = app_flash;

        process.stored_state = MapCell::new(Default::default());
        process.state = Cell::new(State::Unstarted);
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
            app_heap_start_pointer: app_heap_start_pointer,
            app_stack_start_pointer: app_stack_start_pointer,
            min_stack_pointer: initial_stack_pointer,
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

        // Handle any architecture-specific requirements for a new process
        match process.stored_state.map(|stored_state| {
            chip.userspace_kernel_boundary().initialize_process(
                process.sp(),
                process.sp() as usize - process.memory.as_ptr() as usize,
                stored_state,
            )
        }) {
            Some(Ok(new_stack_pointer)) => {
                process
                    .current_stack_pointer
                    .set(new_stack_pointer as *mut u8);
                process.debug_set_max_stack_depth();
            }
            _ => {
                if config::CONFIG.debug_load_processes {
                    debug!(
                        "[!] flash=[{:#010X}:{:#010X}] process={:?} - couldn't initialize process",
                        app_flash.as_ptr() as usize,
                        app_flash.as_ptr() as usize + app_flash.len(),
                        process_name
                    );
                }
                return Err(ProcessLoadError::InternalError);
            }
        };

        // Mark this process as having something to do (it has to start!)
        kernel.increment_work();

        // return
        Ok((Some(process), memory_padding_size + memory_size))
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
        self.state.set(State::StoppedFaulted);
    }

    /// Get the current stack pointer as a pointer.
    // This is currently safe as the the userspace/kernel boundary
    // implementations of both Risc-V and ARM would fault on context switch if
    // the stack pointer were misaligned.
    //
    // This is a bit of an undocumented assumption, but not sure there is
    // likely to be an architecture in the near future where this is
    // realistically a risk.
    #[allow(clippy::cast_ptr_alignment)]
    fn sp(&self) -> *const usize {
        self.current_stack_pointer.get() as *const usize
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

    fn debug_set_max_stack_depth(&self) {
        self.debug.map(|debug| {
            if self.current_stack_pointer.get() < debug.min_stack_pointer {
                debug.min_stack_pointer = self.current_stack_pointer.get();
            }
        });
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
