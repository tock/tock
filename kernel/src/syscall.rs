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
#[derive(PartialEq)]
pub enum ContextSwitchReason {
    /// Process called a syscall. Also returns the syscall and relevant values.
    SyscallFired { syscall: Syscall },
    /// Process triggered the hardfault handler.
    Fault,
    /// Process exceeded its timeslice.
    TimesliceExpired,
    /// Process interrupted (e.g. by a hardware event)
    Interrupted,
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage switching to and from processes
/// in an architecture-agnostic manner.
pub trait UserspaceKernelBoundary {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    type StoredState: Default + Copy;

    /// Called by the kernel after a new process has been created by before it
    /// is allowed to begin executing. Allows for architecture-specific process
    /// setup, e.g. allocating a syscall stack frame.
    unsafe fn initialize_new_process(
        &self,
        stack_pointer: *const usize,
        stack_size: usize,
        state: &mut Self::StoredState,
    ) -> Result<*const usize, ()>;

    /// Set the return value the process should see when it begins executing
    /// again after the syscall. This will only be called after a process has
    /// called a syscall.
    ///
    /// To help implementations, both the current stack pointer of the process
    /// and the saved state for the process are provided. The `return_value` is
    /// the value that should be passed to the process so that when it resumes
    /// executing it knows the return value of the syscall it called.
    unsafe fn set_syscall_return_value(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: isize,
    );

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
    /// - `stack_pointer` is the address of the stack pointer for the current
    ///   app.
    /// - `remaining_stack_memory` is the number of bytes below the
    ///   `stack_pointer` that is allocated for the process. This value is
    ///   checked by the implementer to ensure that there is room for this stack
    ///   frame without overflowing the stack.
    /// - `state` is the stored state for this process.
    /// - `callback` is the function that should be executed when the process
    ///   resumes.
    ///
    /// ### Return
    ///
    /// Returns `Ok` or `Err` with the current address of the stack pointer for
    /// the process. One reason for returning `Err` is that adding the function
    /// call requires adding to the stack, and there is insufficient room on the
    /// stack to add the function call.
    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        remaining_stack_memory: usize,
        state: &mut Self::StoredState,
        callback: process::FunctionCall,
    ) -> Result<*mut usize, *mut usize>;

    /// Context switch to a specific process.
    ///
    /// This returns a tuple:
    /// - The new stack pointer address of the process.
    /// - Why the process stopped executing and switched back to the kernel.
    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> (*mut usize, ContextSwitchReason);

    /// Display any general information about the fault.
    unsafe fn fault_fmt(&self, writer: &mut Write);

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by its stack pointer.
    unsafe fn process_detail_fmt(
        &self,
        stack_pointer: *const usize,
        state: &Self::StoredState,
        writer: &mut Write,
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
