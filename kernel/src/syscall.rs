//! Tock syscall number definitions and arch-agnostic interface trait.

use core::fmt::Write;

use crate::process;

/// The syscall number assignments.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Syscall {
    /// Return to the kernel to allow other processes to execute or to wait for
    /// interrupts and callbacks.
    ///
    /// SVC_NUM = 0
    YIELD,

    /// Pass a callback function to the kernel.
    ///
    /// SVC_NUM = 1
    SUBSCRIBE {
        driver_number: usize,
        subdriver_number: usize,
        callback_ptr: *mut (),
        appdata: usize,
    },

    /// Instruct the kernel or a capsule to perform an operation.
    ///
    /// SVC_NUM = 2
    COMMAND {
        driver_number: usize,
        subdriver_number: usize,
        arg0: usize,
        arg1: usize,
    },

    /// Share a memory buffer with the kernel.
    ///
    /// SVC_NUM = 3
    ALLOW {
        driver_number: usize,
        subdriver_number: usize,
        allow_address: *mut u8,
        allow_size: usize,
    },

    /// Various memory operations.
    ///
    /// SVC_NUM = 4
    MEMOP { operand: usize, arg0: usize },
}

/// Why the process stopped executing and execution returned to the kernel.
#[derive(PartialEq, Copy, Clone)]
pub enum ContextSwitchReason {
    /// Process called a syscall. Also returns the syscall and relevant values.
    SyscallFired { syscall: Syscall },
    /// Process triggered the hardfault handler.
    Fault,
    /// Process interrupted (e.g. by a hardware event)
    Interrupted,
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage switching to and from processes
/// in an architecture-agnostic manner.
///
/// Since exactly how callbacks and return values are passed between kernelspace
/// and userspace is implementation specific, and may use process memory to
/// store state when switching, functions in this trait are passed the bounds of
/// process-accessible memory so that the implementation can verify it is
/// reading and writing memory that the process has valid access to. These
/// bounds are passed through `memory_start` and `app_brk` pointers.
pub trait UserspaceKernelBoundary {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    ///
    /// Implementations should **not** rely on the `Default` constructor (custom
    /// or derived) for any initialization of a process's stored state. The
    /// initialization must happen in the `initialize_process()` function.
    type StoredState: Default;

    /// Called by the kernel during process creation to inform the kernel of the
    /// minimum amount of process-accessible RAM needed by a new process. This
    /// allows for architecture-specific process layout decisions, such as stack
    /// pointer initialization.
    ///
    /// This returns the minimum number of bytes of process-accessible memory
    /// the kernel must allocate to a process so that a successful context
    /// switch is possible.
    ///
    /// Some architectures may not need any allocated memory, and this should
    /// return 0. In general, implementations should try to pre-allocate the
    /// minimal amount of process-accessible memory (i.e. return as close to 0
    /// as possible) to provide the most flexibility to the process. However,
    /// the return value will be nonzero for architectures where values are
    /// passed in memory between kernelspace and userspace during syscalls or a
    /// stack needs to be setup.
    fn initial_process_app_brk_size(&self) -> usize;

    /// Called by the kernel after it has memory allocated to it but before it
    /// is allowed to begin executing. Allows for architecture-specific process
    /// setup, e.g. allocating a syscall stack frame.
    ///
    /// This function must also initialize the stored state (if needed).
    ///
    /// The kernel calls this function with the start of memory allocated to the
    /// process by providing `memory_start`. It also provides the app_brk pointer which
    /// marks the end of process-accessible memory.
    ///
    /// If successful, this function returns `Ok()`. If the process syscall
    /// state cannot be initialized with the available amount of memory, or for
    /// any other reason, it should return `Err()`.
    ///
    /// This function may be called multiple times on the same process. For
    /// example, if a process crashes and is to be restarted, this must be
    /// called. Or if the process is moved this may need to be called.
    unsafe fn initialize_process(
        &self,
        memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> Result<(), ()>;

    /// Set the return value the process should see when it begins executing
    /// again after the syscall. This will only be called after a process has
    /// called a syscall.
    ///
    /// The process to set the return value for is specified by the `state`
    /// value. The `return_value` is the value that should be passed to the
    /// process so that when it resumes executing it knows the return value of
    /// the syscall it called.
    unsafe fn set_syscall_return_value(
        &self,
        memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: isize,
    ) -> Result<(), ()>;

    /// Set the function that the process should execute when it is resumed.
    /// This has two major uses: 1) sets up the initial function call to
    /// `_start` when the process is started for the very first time; 2) tells
    /// the process to execute a callback function after calling `yield()`.
    ///
    /// **Note:** This method cannot be called in conjunction with
    /// `set_syscall_return_value`, as the injected function will clobber the
    /// return value.
    ///
    /// ### Arguments
    ///
    /// - `memory_start` is the address of the start of the memory region
    ///   allocated to this process.
    /// - `app_brk` is the address of the current process break. This marks the
    ///   end of the memory region the process has access to. Note, this is not
    ///   the end of the entire memory region allocated to the process. Some
    ///   memory above this address is still allocated for the process, but if
    ///   the process tries to access it an MPU fault will occur.
    /// - `state` is the stored state for this process.
    /// - `callback` is the function that should be executed when the process
    ///   resumes.
    ///
    /// ### Return
    ///
    /// Returns `Ok(())` if the function was successfully enqueued for the
    /// process. Returns `Err(())` if the function was not, likely because there
    /// is insufficient memory available to do so.
    unsafe fn set_process_function(
        &self,
        memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        callback: process::FunctionCall,
    ) -> Result<(), ()>;

    /// Context switch to a specific process.
    ///
    /// This returns why the process stopped executing and switched back to the
    /// kernel.
    unsafe fn switch_to_process(
        &self,
        memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> ContextSwitchReason;

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by the stored state for that process.
    unsafe fn print_context(
        &self,
        memory_start: *const u8,
        app_brk: *const u8,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    );
}

/// Helper function for converting raw values passed back from an application
/// into a `Syscall` type in Tock.
///
/// Different architectures may have different mechanisms for passing
/// information about what syscall an app called, but they will have have to
/// convert the series of raw values into a more useful Rust type. While
/// implementations are free to do this themselves, this provides a generic
/// helper function which should help reduce duplicated code.
///
/// The mappings between raw `syscall_number` values and the associated syscall
/// type are specified and fixed by Tock. After that, this function only
/// converts raw values to more meaningful types based on the syscall.
pub fn arguments_to_syscall(
    syscall_number: u8,
    r0: usize,
    r1: usize,
    r2: usize,
    r3: usize,
) -> Option<Syscall> {
    match syscall_number {
        0 => Some(Syscall::YIELD),
        1 => Some(Syscall::SUBSCRIBE {
            driver_number: r0,
            subdriver_number: r1,
            callback_ptr: r2 as *mut (),
            appdata: r3,
        }),
        2 => Some(Syscall::COMMAND {
            driver_number: r0,
            subdriver_number: r1,
            arg0: r2,
            arg1: r3,
        }),
        3 => Some(Syscall::ALLOW {
            driver_number: r0,
            subdriver_number: r1,
            allow_address: r2 as *mut u8,
            allow_size: r3,
        }),
        4 => Some(Syscall::MEMOP {
            operand: r0,
            arg0: r1,
        }),
        _ => None,
    }
}
