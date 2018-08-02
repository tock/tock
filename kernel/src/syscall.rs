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
    /// Process called a syscall.
    SyscallFired,
    /// Process triggered the hardfault handler.
    Fault,
    /// Process exceeded its timeslice.
    TimesliceExpired,
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage switching to and from processes
/// in an architecture-agnostic manner.
pub trait UserspaceKernelBoundary {
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
    unsafe fn get_and_reset_context_switch_reason(&self) -> ContextSwitchReason;

    /// Get the syscall that the process called with the appropriate arguments.
    unsafe fn get_syscall(&self, stack_pointer: *const usize) -> Option<Syscall>;

    /// Set the return value the process should see when it begins executing
    /// again after the syscall.
    unsafe fn set_syscall_return_value(&self, stack_pointer: *const usize, return_value: isize);

    /// Remove the last stack frame from the process and return the new stack
    /// pointer location.
    ///
    /// This function assumes that `stack_pointer` is valid and at the end of
    /// the process stack, that there is at least one stack frame on the
    /// stack, and that that frame is the syscall.
    unsafe fn pop_syscall_stack_frame(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> *mut usize;

    /// Add a stack frame with the new function call. This function
    /// is what should be executed when the process is resumed.
    ///
    /// `remaining_stack_memory` is the number of bytes below the
    /// `stack_pointer` that is allocated for the process. This value is checked
    /// by the implementer to ensure that there is room for this stack frame
    /// without overflowing the stack.
    ///
    /// Returns `Ok` with the new stack pointer after adding the stack frame if
    /// there was room for the stack frame, and an error with where the stack
    /// would have ended up if the function call had been added otherwise.
    unsafe fn push_function_call(
        &self,
        stack_pointer: *const usize,
        remaining_stack_memory: usize,
        callback: process::FunctionCall,
        state: &Self::StoredState,
    ) -> Result<*mut usize, *mut usize>;

    /// Context switch to a specific process.
    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> *mut usize;
}
