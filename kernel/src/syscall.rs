//! Tock syscall number definitions and arch-agnostic interface trait.

use process;

/// The syscall number assignments.
#[derive(Copy, Clone, Debug)]
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
pub enum ContextSwitchReason {
    /// Process exceeded its timeslice, otherwise catch-all.
    Other,
    /// Process called a syscall.
    SyscallFired,
    /// Process triggered the hardfault handler.
    Fault,
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage processes in an
/// architecture-agnostic manner.
pub trait SyscallInterface {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    type StoredState: Default;

    /// Allows the kernel to query to see why the process stopped running. This
    /// function can only be called once to get the last state of the process
    /// and why the process context switched back to the kernel.
    ///
    /// An implementor of this function is free to reset any state that was
    /// needed to gather this information when this function is called.
    unsafe fn get_context_switch_reason(&self) -> ContextSwitchReason;

    /// Get the syscall that the process called with the appropriate arguments.
    unsafe fn get_syscall(&self, stack_pointer: *const usize) -> Option<Syscall>;

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    unsafe fn set_syscall_return_value(&self, stack_pointer: *const usize, return_value: isize);

    /// Remove the last stack frame from the process and return the new stack
    /// pointer location.
    unsafe fn pop_syscall_stack(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> *mut usize;

    /// Add a stack frame with the new function call. This function
    /// is what should be executed when the process is resumed. Returns the new
    /// stack pointer.
    unsafe fn replace_function_call(
        &self,
        stack_pointer: *const usize,
        callback: process::FunctionCall,
        state: &Self::StoredState,
    ) -> *mut usize;

    /// Context switch to a specific process.
    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> *mut usize;
}
