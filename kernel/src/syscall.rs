//! Tock syscall number definitions.

/// The syscall number assignments.
#[derive(Copy, Clone, Debug)]
crate enum Syscall {
    /// Return to the kernel to allow other processes to execute or to wait for
    /// interrupts and callbacks.
    YIELD = 0,

    /// Pass a callback function to the kernel.
    SUBSCRIBE = 1,

    /// Instruct the kernel or a capsule to perform an operation.
    COMMAND = 2,

    /// Share a memory buffer with the kernel.
    ALLOW = 3,

    /// Various memory operations.
    MEMOP = 4,
}
