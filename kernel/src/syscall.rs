//! Tock syscall number definitions and arch-agnostic interface trait.

use core::fmt::Write;

use crate::driver::CommandResult;
use crate::process;
use crate::ReturnCode;

// TODO: Maybe change the variant identifiers to have errors have the
// most significant bit set as discussed in the core team call?
// (e.g. negative numbers with two's complement)
/// Enumeration over the possible system call return type variants
///
/// Each variant is associated with the respective variant identifier
/// that would be passed along with the return value to userspace.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SyscallReturnVariant {
    Failure = 0,
    FailureU32 = 1,
    FailureU32U32 = 2,
    FailureU64 = 3,
    Success = 128,
    SuccessU32 = 129,
    SuccessU32U32 = 130,
    SuccessU32U32U32 = 131,
    SuccessU64 = 132,
    SuccessU64U32 = 133,
}

#[inline]
fn u64_to_be_u32s(src: u64) -> (u32, u32) {
    let src_bytes = src.to_be_bytes();
    let src_msb = u32::from_be_bytes([src_bytes[0], src_bytes[1], src_bytes[2], src_bytes[3]]);
    let src_lsb = u32::from_be_bytes([src_bytes[4], src_bytes[5], src_bytes[6], src_bytes[7]]);

    (src_msb, src_lsb)
}

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

/// Possible return values of an `allow`-type system call
///
/// Since this only contains the raw pointer to the allowed buffer, it
/// implements `Copy`.
#[derive(Copy, Clone, Debug)]
pub enum AllowReturnValue {
    /// In the success case, bass back the pointer and respective
    /// length of the buffer shared from userspace
    Success(*mut u8, usize),
    /// An `allow` operation is allowed to error, returning a
    /// `ReturnCode` to the userspace app, along with the pointer and
    /// length passed with the original allow syscall
    Error(ReturnCode, *mut u8, usize),
}

impl AllowReturnValue {
    // TODO: This would break on 64-bit systems
    fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self {
            &AllowReturnValue::Success(ptr, length) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = length as u32;
            }
            &AllowReturnValue::Error(rc, ptr, length) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = isize::from(rc) as u32;
                *a2 = ptr as u32;
                *a3 = length as u32;
            }
        }
    }
}

/// Possible return values of a `command`-type system call
///
/// Since this has the exact same variants and fields as the
/// `CommandResult` type for use the in the `Driver`'s `command`
/// method return value, it simply wraps this value
///
/// The `CommandReturnValue` features a default encoding function
#[derive(Copy, Clone, Debug)]
pub struct CommandReturnValue(CommandResult);

impl CommandReturnValue {
    // TODO: Make this crate-public, it only ever needs to be constructed in the kernel
    pub fn from_command_result(res: CommandResult) -> Self {
        CommandReturnValue(res)
    }

    /// Encode the `command` system call return value into 4 registers
    ///
    /// Architectures are free to define their own encoding.
    ///
    /// Most architectures will want to use the (generic over all
    /// system call types) [`SyscallReturnValue::encode_syscall_return`] instead.
    fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self.0 {
            CommandResult::Error(rc) => {
                *a0 = SyscallReturnVariant::Failure as u32;
                *a1 = isize::from(rc) as u32;
            }
            CommandResult::ErrorU32(rc, data0) => {
                *a0 = SyscallReturnVariant::FailureU32 as u32;
                *a1 = isize::from(rc) as u32;
                *a2 = data0;
            }
            CommandResult::ErrorU32U32(rc, data0, data1) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = isize::from(rc) as u32;
                *a2 = data0;
                *a3 = data1;
            }
            CommandResult::ErrorU64(rc, data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::FailureU64 as u32;
                *a1 = isize::from(rc) as u32;
                *a2 = data0_lsb;
                *a3 = data0_msb;
            }
            CommandResult::Success => {
                *a0 = SyscallReturnVariant::Success as u32;
            }
            CommandResult::SuccessU32(data0) => {
                *a0 = SyscallReturnVariant::SuccessU32 as u32;
                *a1 = data0;
            }
            CommandResult::SuccessU32U32(data0, data1) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = data0;
                *a2 = data1;
            }
            CommandResult::SuccessU32U32U32(data0, data1, data2) => {
                *a0 = SyscallReturnVariant::SuccessU32U32U32 as u32;
                *a1 = data0;
                *a2 = data1;
                *a3 = data2;
            }
            CommandResult::SuccessU64(data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
            }
            CommandResult::SuccessU64U32(data0, data1) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64U32 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
                *a3 = data1;
            }
        }
    }
}

/// Possible return values of a `subscribe`-type system call
///
/// Since this only contains the raw pointer to the callback function,
/// it implements `Copy`.
#[derive(Copy, Clone, Debug)]
pub enum SubscribeReturnValue {
    /// In the success case, pass back the callback function pointer
    /// and the supplied userdata to an userspace app
    Success(*mut (), usize),
    /// A `subscribe` operation is allowed to error, returning a
    /// `ReturnCode` to the userspace app, along with the pointer and
    /// userdata passed with the original syscall
    Error(ReturnCode, *mut (), usize),
}

impl SubscribeReturnValue {
    // TODO: This would break on 64-bit systems
    fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self {
            &SubscribeReturnValue::Success(ptr, userdata) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = userdata as u32;
            }
            &SubscribeReturnValue::Error(rc, ptr, userdata) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = isize::from(rc) as u32;
                *a2 = ptr as u32;
                *a3 = userdata as u32;
            }
        }
    }
}

/// A union over all system call type's return values
///
/// This is passed down to the architecture which then determines how
/// to encode the system call return arguments for the userspace app.
///
/// For encoding, the architecture *may* decide use the provided
/// `syscall_return_to_arguments`, which can be seen as a counterpart
/// to `arguments_to_syscall`. Architectures are however free to
/// define their own encoding.
#[derive(Copy, Clone, Debug)]
pub enum SyscallReturnValue {
    /// `yield`-type system call return value
    ///
    /// The return type vairant is dependent on whether a callback has
    /// been executed, indicated by the associated boolean field.
    Yield(bool),
    /// `allow`-type system call return values
    Allow(AllowReturnValue),
    /// `command`-type system call return values
    Command(CommandReturnValue),
    /// `subscribe`-type system call return values
    Subscribe(SubscribeReturnValue),
    /// `memop`-type system call return values
    ///
    /// The precise return value variant is dependent on the
    /// specific `memop` system call.
    Memop(SyscallReturnVariant, u32, u32, u32),
}

impl SyscallReturnValue {
    /// Encode the system call return values into a series of
    /// `u32`-values to be passed to the userspace app
    ///
    /// An architecture may decide to use this function, or define its
    /// own method for encoding the return values for the userspace
    /// app.
    ///
    /// The provided `u32` variables should be made available to the
    /// userspace app as it will be scheduled again. They can be
    /// provided as registers or on the stack, depending on the
    /// architecture.
    #[inline]
    pub fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self {
            SyscallReturnValue::Yield(callback_executed) => {
                *a0 = if *callback_executed {
                    SyscallReturnVariant::Success as u32
                } else {
                    SyscallReturnVariant::Failure as u32
                };
            }
            SyscallReturnValue::Allow(rv) => rv.encode_syscall_return(a0, a1, a2, a3),
            SyscallReturnValue::Command(rv) => rv.encode_syscall_return(a0, a1, a2, a3),
            SyscallReturnValue::Subscribe(rv) => rv.encode_syscall_return(a0, a1, a2, a3),
            SyscallReturnValue::Memop(_, _, _, _) => {
                // TODO: Would be duplicate of CommandReturnValue
                unimplemented!();
            }
        }
    }
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage switching to and from processes
/// in an architecture-agnostic manner.
pub trait UserspaceKernelBoundary {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    ///
    /// Implementations should **not** rely on the `Default` constructor (custom
    /// or derived) for any initialization of a process's stored state. The
    /// initialization must happen in the `initialize_process()` function.
    type StoredState: Default;

    /// Called by the kernel after it has memory allocated to it but before it
    /// is allowed to begin executing. Allows for architecture-specific process
    /// setup, e.g. allocating a syscall stack frame.
    ///
    /// This function must also initialize the stored state (if needed).
    ///
    /// This function may be called multiple times on the same process. For
    /// example, if a process crashes and is to be restarted, this must be
    /// called. Or if the process is moved this may need to be called.
    unsafe fn initialize_process(
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

    // Tock 2.0 method preview:
    //
    // unsafe fn set_syscall_return_value(
    //     &self,
    //     stack_pointer: *const usize,
    //     state: &mut Self::StoredState,
    //     return_value: SyscallReturnValue,
    // );

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

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by its stack pointer.
    unsafe fn print_context(
        &self,
        stack_pointer: *const usize,
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
