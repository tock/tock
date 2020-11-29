//! Tock syscall number definitions and arch-agnostic interface trait.

use core::convert::TryFrom;
use core::fmt::Write;

use crate::driver::{AllowReadOnlyResult, AllowReadWriteResult, CommandResult, SubscribeResult};
use crate::errorcode::ErrorCode;
use crate::process;
use crate::returncode::ReturnCode;

/// Helper function to split a u64 into a higher and lower u32
///
/// Used in encoding 64-bit wide system call return values on 32-bit
/// platforms.
#[inline]
fn u64_to_be_u32s(src: u64) -> (u32, u32) {
    let src_bytes = src.to_be_bytes();
    let src_msb = u32::from_be_bytes([src_bytes[0], src_bytes[1], src_bytes[2], src_bytes[3]]);
    let src_lsb = u32::from_be_bytes([src_bytes[4], src_bytes[5], src_bytes[6], src_bytes[7]]);

    (src_msb, src_lsb)
}

// ---------- SYSTEMCALL ARGUMENT DECODING ----------

/// Enumeration over the possible system call classes
///
/// Each system call class is associated with the respective ID for
/// encoding a system call in registers.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum SyscallClass {
    Yield = 0,
    Subscribe = 1,
    Command = 2,
    ReadWriteAllow = 3,
    Memop = 5,
}

// Required as long as no solution to
// https://github.com/rust-lang/rfcs/issues/2783 is integrated into
// the standard library
impl TryFrom<u8> for SyscallClass {
    type Error = u8;

    fn try_from(syscall_class_id: u8) -> Result<SyscallClass, u8> {
        match syscall_class_id {
            0 => Ok(SyscallClass::Yield),
            1 => Ok(SyscallClass::Subscribe),
            2 => Ok(SyscallClass::Command),
            3 => Ok(SyscallClass::ReadWriteAllow),
            5 => Ok(SyscallClass::Memop),
            i => Err(i),
        }
    }
}

/// Decoded system calls
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Syscall {
    /// Return to the kernel to allow other processes to execute or to wait for
    /// interrupts and callbacks.
    ///
    /// System call class ID 0
    Yield,

    /// Pass a callback function to the kernel.
    ///
    /// System call class ID 1
    Subscribe {
        driver_number: usize,
        subdriver_number: usize,
        callback_ptr: *mut (),
        appdata: usize,
    },

    /// Instruct the kernel or a capsule to perform an operation.
    ///
    /// System call class ID 2
    Command {
        driver_number: usize,
        subdriver_number: usize,
        arg0: usize,
        arg1: usize,
    },

    /// Share a memory buffer with the kernel, which the kernel may
    /// read from and write to.
    ///
    /// System call class ID 3
    ReadWriteAllow {
        driver_number: usize,
        subdriver_number: usize,
        allow_address: *mut u8,
        allow_size: usize,
    },

    /// Various memory operations.
    ///
    /// System call class ID 5
    Memop { operand: usize, arg0: usize },
}

impl Syscall {
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
    pub fn from_register_arguments(
        syscall_number: u8,
        r0: usize,
        r1: usize,
        r2: usize,
        r3: usize,
    ) -> Option<Self> {
        match SyscallClass::try_from(syscall_number) {
            Ok(SyscallClass::Yield) => Some(Syscall::Yield),
            Ok(SyscallClass::Subscribe) => Some(Syscall::Subscribe {
                driver_number: r0,
                subdriver_number: r1,
                callback_ptr: r2 as *mut (),
                appdata: r3,
            }),
            Ok(SyscallClass::Command) => Some(Syscall::Command {
                driver_number: r0,
                subdriver_number: r1,
                arg0: r2,
                arg1: r3,
            }),
            Ok(SyscallClass::ReadWriteAllow) => Some(Syscall::ReadWriteAllow {
                driver_number: r0,
                subdriver_number: r1,
                allow_address: r2 as *mut u8,
                allow_size: r3,
            }),
            Ok(SyscallClass::Memop) => Some(Syscall::Memop {
                operand: r0,
                arg0: r1,
            }),
            Err(_) => None,
        }
    }
}

// ---------- SYSCALL RETURN VALUE ENCODING ----------

/// Enumeration over the possible system call return type variants.
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

/// Possible system call return variants, generic over the system call
/// type
///
/// This struct operates over primitive types such as integers of
/// fixed length and pointers. It is constructed by the scheduler and
/// passed down to the architecture to be encoded into registers,
/// possibly using the provided
/// [`encode_syscall_return`](GenericSyscallReturnValue::encode_syscall_return)
/// method.
///
/// Capsules use higher level Rust types
/// (e.g. [`AppSlice`](crate::AppSlice) and
/// [`Callback`](crate::Callback)) or wrappers around this struct
/// ([`CommandResult`](crate::CommandResult)) which limit the
/// available constructors to safely constructable variants.
#[derive(Copy, Clone, Debug)]
pub enum GenericSyscallReturnValue {
    /// Generic error case
    Failure(ErrorCode),
    /// Generic error case, with an additional 32-bit data field
    FailureU32(ErrorCode, u32),
    /// Generic error case, with two additional 32-bit data fields
    FailureU32U32(ErrorCode, u32, u32),
    /// Generic error case, with an additional 64-bit data field
    FailureU64(ErrorCode, u64),
    /// Generic success case
    Success,
    /// Generic success case, with an additional 32-bit data field
    SuccessU32(u32),
    /// Generic success case, with two additional 32-bit data fields
    SuccessU32U32(u32, u32),
    /// Generic success case, with three additional 32-bit data fields
    SuccessU32U32U32(u32, u32, u32),
    /// Generic success case, with an additional 64-bit data field
    SuccessU64(u64),
    /// Generic success case, with an additional 32-bit and 64-bit
    /// data field
    SuccessU64U32(u64, u32),

    // These following types are used by the scheduler so that it can
    // return values to userspace in an architecture (pointer-width)
    // independent way. The kernel passes these types (rather than
    // AppSlice or Callback) for two reasons. First, since the
    // kernel/scheduler makes promises about the lifetime and safety
    // of these types (e.g., an accepted allow does not overlap with
    // an existing accepted AppSlice), it does not want to leak them
    // to other code. Second, if subscribe or allow calls pass invalid
    // values (pointers out of valid memory), the kernel cannot
    // construct an AppSlice or Callback type but needs to be able to
    // return a failure. -pal 11/24/20
    /// Read/Write allow success case
    AllowReadWriteSuccess(*mut u8, usize),
    /// Read/Write allow failure case
    AllowReadWriteFailure(ErrorCode, *mut u8, usize),

    /// Read only allow success case
    AllowReadOnlySuccess(*const u8, usize),
    /// Read only allow failure case
    AllowReadOnlyFailure(ErrorCode, *const u8, usize),

    /// Subscribe success case
    SubscribeSuccess(*const u8, usize),
    /// Subscribe failure case
    SubscribeFailure(ErrorCode, *const u8, usize),

    Legacy(ReturnCode),
}

impl GenericSyscallReturnValue {
    // TODO: Make this crate-public, it only ever needs to be
    // constructed in the kernel
    pub fn from_command_result(res: CommandResult) -> Self {
        res.into_inner()
    }

    pub fn from_allow_readwrite_result(res: AllowReadWriteResult) -> Self {
        match res {
            AllowReadWriteResult::Success(mut slice) => {
                GenericSyscallReturnValue::AllowReadWriteSuccess(
                    slice.as_mut().as_mut_ptr(),
                    slice.len(),
                )
            }
            AllowReadWriteResult::Failure(mut slice, err) => {
                GenericSyscallReturnValue::AllowReadWriteFailure(
                    err,
                    slice.as_mut().as_mut_ptr(),
                    slice.len(),
                )
            }
        }
    }

    pub fn from_allow_readonly_result(res: AllowReadOnlyResult) -> Self {
        match res {
            AllowReadOnlyResult::Success(slice) => GenericSyscallReturnValue::AllowReadOnlySuccess(
                slice.as_ref().as_ptr(),
                slice.len(),
            ),
            AllowReadOnlyResult::Failure(slice, err) => {
                GenericSyscallReturnValue::AllowReadOnlyFailure(
                    err,
                    slice.as_ref().as_ptr(),
                    slice.len(),
                )
            }
        }
    }

    pub fn from_subscribe_result(res: SubscribeResult) -> Self {
        match res {
            SubscribeResult::Success(callback) => GenericSyscallReturnValue::SubscribeSuccess(
                callback.function_pointer(),
                callback.appdata() as usize,
            ),
            SubscribeResult::Failure(callback, err) => GenericSyscallReturnValue::SubscribeFailure(
                err,
                callback.function_pointer(),
                callback.appdata() as usize,
            ),
        }
    }

    /// Encode the system call return value into 4 registers
    ///
    /// Architectures are free to define their own encoding.
    pub fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self {
            &GenericSyscallReturnValue::Failure(e) => {
                *a0 = SyscallReturnVariant::Failure as u32;
                *a1 = usize::from(e) as u32;
            }
            &GenericSyscallReturnValue::FailureU32(e, data0) => {
                *a0 = SyscallReturnVariant::FailureU32 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0;
            }
            &GenericSyscallReturnValue::FailureU32U32(e, data0, data1) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0;
                *a3 = data1;
            }
            &GenericSyscallReturnValue::FailureU64(e, data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);
                *a0 = SyscallReturnVariant::FailureU64 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0_lsb;
                *a3 = data0_msb;
            }
            &GenericSyscallReturnValue::Success => {
                *a0 = SyscallReturnVariant::Success as u32;
            }
            &GenericSyscallReturnValue::SuccessU32(data0) => {
                *a0 = SyscallReturnVariant::SuccessU32 as u32;
                *a1 = data0;
            }
            &GenericSyscallReturnValue::SuccessU32U32(data0, data1) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = data0;
                *a2 = data1;
            }
            &GenericSyscallReturnValue::SuccessU32U32U32(data0, data1, data2) => {
                *a0 = SyscallReturnVariant::SuccessU32U32U32 as u32;
                *a1 = data0;
                *a2 = data1;
                *a3 = data2;
            }
            &GenericSyscallReturnValue::SuccessU64(data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
            }
            &GenericSyscallReturnValue::SuccessU64U32(data0, data1) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64U32 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
                *a3 = data1;
            }
            &GenericSyscallReturnValue::AllowReadWriteSuccess(ptr, len) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = len as u32;
            }
            &GenericSyscallReturnValue::AllowReadWriteFailure(err, ptr, len) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = len as u32;
            }
            &GenericSyscallReturnValue::AllowReadOnlySuccess(ptr, len) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = len as u32;
            }
            &GenericSyscallReturnValue::AllowReadOnlyFailure(err, ptr, len) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = len as u32;
            }
            &GenericSyscallReturnValue::SubscribeSuccess(ptr, data) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = data as u32;
            }
            &GenericSyscallReturnValue::SubscribeFailure(err, ptr, data) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = data as u32;
            }
            &GenericSyscallReturnValue::Legacy(rcode) => {
                *a0 = usize::from(rcode) as u32;
            }
        }
    }
}

// ---------- USERSPACE KERNEL BOUNDARY ----------

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
        return_value: GenericSyscallReturnValue,
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

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by its stack pointer.
    unsafe fn print_context(
        &self,
        stack_pointer: *const usize,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    );
}
