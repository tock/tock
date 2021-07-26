//! Tock syscall number definitions and arch-agnostic interface trait.

use core::convert::TryFrom;
use core::fmt::Write;

use crate::errorcode::ErrorCode;
use crate::process;

pub use crate::syscall_driver::{CommandReturn, SyscallDriver};

/// Helper function to split a u64 into a higher and lower u32.
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

/// Enumeration of the system call classes based on the identifiers
/// specified in the Tock ABI.
///
/// These are encoded as 8 bit values as on some architectures the value can
/// be encoded in the instruction itself.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum SyscallClass {
    Yield = 0,
    Subscribe = 1,
    Command = 2,
    ReadWriteAllow = 3,
    ReadOnlyAllow = 4,
    Memop = 5,
    Exit = 6,
}

/// Enumeration of the yield system calls based on the Yield identifier
/// values specified in the Tock ABI.
#[derive(Copy, Clone, Debug)]
pub enum YieldCall {
    NoWait = 0,
    Wait = 1,
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
            4 => Ok(SyscallClass::ReadOnlyAllow),
            5 => Ok(SyscallClass::Memop),
            6 => Ok(SyscallClass::Exit),
            i => Err(i),
        }
    }
}

/// Decoded system calls as defined in TRD 104.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Syscall {
    /// Structure representing an invocation of the Yield system call class.
    /// `which` is the Yield identifier value and `address` is the no wait field.
    Yield { which: usize, address: *mut u8 },

    /// Structure representing an invocation of the Subscribe system call
    /// class. `driver_number` is the driver identifier, `subdriver_number`
    /// is the subscribe identifier, `upcall_ptr` is upcall pointer,
    /// and `appdata` is the application data.
    Subscribe {
        driver_number: usize,
        subdriver_number: usize,
        upcall_ptr: *mut (),
        appdata: usize,
    },

    /// Structure representing an invocation of the Command system call class.
    /// `driver_number` is the driver identifier and `subdriver_number` is
    /// the command identifier.
    Command {
        driver_number: usize,
        subdriver_number: usize,
        arg0: usize,
        arg1: usize,
    },

    /// Structure representing an invocation of the ReadWriteAllow system call
    /// class. `driver_number` is the driver identifier, `subdriver_number` is
    /// the buffer identifier, `allow_address` is the address, and `allow_size`
    /// is the size.
    ReadWriteAllow {
        driver_number: usize,
        subdriver_number: usize,
        allow_address: *mut u8,
        allow_size: usize,
    },

    /// Structure representing an invocation of the ReadOnlyAllow system call
    /// class. `driver_number` is the driver identifier, `subdriver_number` is
    /// the buffer identifier, `allow_address` is the address, and `allow_size`
    /// is the size.
    ReadOnlyAllow {
        driver_number: usize,
        subdriver_number: usize,
        allow_address: *const u8,
        allow_size: usize,
    },

    /// Structure representing an invocation of the Memop system call
    /// class. `operand` is the operation and `arg0` is the operation
    /// argument.
    Memop { operand: usize, arg0: usize },

    /// Structure representing an invocation of the Exit system call
    /// class. `which` is the exit identifier and `completion_code` is
    /// the completion code passed into the kernel.
    Exit {
        which: usize,
        completion_code: usize,
    },
}

impl Syscall {
    /// Helper function for converting raw values passed back from an application
    /// into a `Syscall` type in Tock, representing an typed version of a system
    /// call invocation. The method returns None if the values do not specify
    /// a valid system call.
    ///
    /// Different architectures have different ABIs for a process and the kernel
    /// to exchange data. The 32-bit ABI for CortexM and RISCV microcontrollers is
    /// specified in TRD104.
    pub fn from_register_arguments(
        syscall_number: u8,
        r0: usize,
        r1: usize,
        r2: usize,
        r3: usize,
    ) -> Option<Syscall> {
        match SyscallClass::try_from(syscall_number) {
            Ok(SyscallClass::Yield) => Some(Syscall::Yield {
                which: r0,
                address: r1 as *mut u8,
            }),
            Ok(SyscallClass::Subscribe) => Some(Syscall::Subscribe {
                driver_number: r0,
                subdriver_number: r1,
                upcall_ptr: r2 as *mut (),
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
            Ok(SyscallClass::ReadOnlyAllow) => Some(Syscall::ReadOnlyAllow {
                driver_number: r0,
                subdriver_number: r1,
                allow_address: r2 as *const u8,
                allow_size: r3,
            }),
            Ok(SyscallClass::Memop) => Some(Syscall::Memop {
                operand: r0,
                arg0: r1,
            }),
            Ok(SyscallClass::Exit) => Some(Syscall::Exit {
                which: r0,
                completion_code: r1,
            }),
            Err(_) => None,
        }
    }
}

// ---------- SYSCALL RETURN VALUE ENCODING ----------

/// Enumeration of the system call return type variant identifiers described
/// in TRD104.
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
    SuccessU64 = 131,
    SuccessU32U32U32 = 132,
    SuccessU64U32 = 133,
}

/// Enumeration of the possible system call return variants specified
/// in TRD104.
///
/// This struct operates over primitive types such as integers of
/// fixed length and pointers. It is constructed by the scheduler and
/// passed down to the architecture to be encoded into registers,
/// using the provided
/// [`encode_syscall_return`](SyscallReturn::encode_syscall_return)
/// method.
///
/// Capsules do not use this struct. Capsules use higher level Rust
/// types
/// (e.g. [`ReadWriteProcessBuffer`](crate::processbuffer::ReadWriteProcessBuffer)
/// and `GrantUpcallTable`) or wrappers around this struct
/// ([`CommandReturn`](crate::syscall_driver::CommandReturn)) which limit the
/// available constructors to safely constructable variants.
#[derive(Copy, Clone, Debug)]
pub enum SyscallReturn {
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
    // ProcessBuffer or Upcall) for two reasons. First, since the
    // kernel/scheduler makes promises about the lifetime and safety
    // of these types, it does not want to leak them to other
    // code. Second, if subscribe or allow calls pass invalid values
    // (pointers out of valid memory), the kernel cannot construct an
    // ProcessBuffer or Upcall type but needs to be able to return a
    // failure. -pal 11/24/20
    /// Read/Write allow success case, returns the previous allowed
    /// buffer and size to the process.
    AllowReadWriteSuccess(*mut u8, usize),
    /// Read/Write allow failure case, returns the passed allowed
    /// buffer and size to the process.
    AllowReadWriteFailure(ErrorCode, *mut u8, usize),

    /// Read only allow success case, returns the previous allowed
    /// buffer and size to the process.
    AllowReadOnlySuccess(*const u8, usize),
    /// Read only allow failure case, returns the passed allowed
    /// buffer and size to the process.
    AllowReadOnlyFailure(ErrorCode, *const u8, usize),

    /// Subscribe success case, returns the previous upcall function
    /// pointer and application data.
    SubscribeSuccess(*const (), usize),
    /// Subscribe failure case, returns the passed upcall function
    /// pointer and application data.
    SubscribeFailure(ErrorCode, *const (), usize),
}

impl SyscallReturn {
    /// Transforms a CommandReturn, which is wrapper around a subset of
    /// SyscallReturn, into a SyscallReturn.
    ///
    /// This allows CommandReturn to include only the variants of SyscallReturn
    /// that can be returned from a Command, while having an inexpensive way to
    /// handle it as a SyscallReturn for more generic code paths.
    pub(crate) fn from_command_return(res: CommandReturn) -> Self {
        res.into_inner()
    }

    /// Returns true if the `SyscallReturn` is any success type.
    pub(crate) fn is_success(&self) -> bool {
        match self {
            SyscallReturn::Success => true,
            SyscallReturn::SuccessU32(_) => true,
            SyscallReturn::SuccessU32U32(_, _) => true,
            SyscallReturn::SuccessU32U32U32(_, _, _) => true,
            SyscallReturn::SuccessU64(_) => true,
            SyscallReturn::SuccessU64U32(_, _) => true,
            SyscallReturn::AllowReadWriteSuccess(_, _) => true,
            SyscallReturn::AllowReadOnlySuccess(_, _) => true,
            SyscallReturn::SubscribeSuccess(_, _) => true,
            SyscallReturn::Failure(_) => false,
            SyscallReturn::FailureU32(_, _) => false,
            SyscallReturn::FailureU32U32(_, _, _) => false,
            SyscallReturn::FailureU64(_, _) => false,
            SyscallReturn::AllowReadWriteFailure(_, _, _) => false,
            SyscallReturn::AllowReadOnlyFailure(_, _, _) => false,
            SyscallReturn::SubscribeFailure(_, _, _) => false,
        }
    }

    /// Encode the system call return value into 4 registers, following
    /// the encoding specified in TRD104. Architectures which do not follow
    /// TRD104 are free to define their own encoding.
    pub fn encode_syscall_return(&self, a0: &mut u32, a1: &mut u32, a2: &mut u32, a3: &mut u32) {
        match self {
            &SyscallReturn::Failure(e) => {
                *a0 = SyscallReturnVariant::Failure as u32;
                *a1 = usize::from(e) as u32;
            }
            &SyscallReturn::FailureU32(e, data0) => {
                *a0 = SyscallReturnVariant::FailureU32 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0;
            }
            &SyscallReturn::FailureU32U32(e, data0, data1) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0;
                *a3 = data1;
            }
            &SyscallReturn::FailureU64(e, data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);
                *a0 = SyscallReturnVariant::FailureU64 as u32;
                *a1 = usize::from(e) as u32;
                *a2 = data0_lsb;
                *a3 = data0_msb;
            }
            &SyscallReturn::Success => {
                *a0 = SyscallReturnVariant::Success as u32;
            }
            &SyscallReturn::SuccessU32(data0) => {
                *a0 = SyscallReturnVariant::SuccessU32 as u32;
                *a1 = data0;
            }
            &SyscallReturn::SuccessU32U32(data0, data1) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = data0;
                *a2 = data1;
            }
            &SyscallReturn::SuccessU32U32U32(data0, data1, data2) => {
                *a0 = SyscallReturnVariant::SuccessU32U32U32 as u32;
                *a1 = data0;
                *a2 = data1;
                *a3 = data2;
            }
            &SyscallReturn::SuccessU64(data0) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
            }
            &SyscallReturn::SuccessU64U32(data0, data1) => {
                let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

                *a0 = SyscallReturnVariant::SuccessU64U32 as u32;
                *a1 = data0_lsb;
                *a2 = data0_msb;
                *a3 = data1;
            }
            &SyscallReturn::AllowReadWriteSuccess(ptr, len) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = len as u32;
            }
            &SyscallReturn::AllowReadWriteFailure(err, ptr, len) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = len as u32;
            }
            &SyscallReturn::AllowReadOnlySuccess(ptr, len) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = len as u32;
            }
            &SyscallReturn::AllowReadOnlyFailure(err, ptr, len) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = len as u32;
            }
            &SyscallReturn::SubscribeSuccess(ptr, data) => {
                *a0 = SyscallReturnVariant::SuccessU32U32 as u32;
                *a1 = ptr as u32;
                *a2 = data as u32;
            }
            &SyscallReturn::SubscribeFailure(err, ptr, data) => {
                *a0 = SyscallReturnVariant::FailureU32U32 as u32;
                *a1 = usize::from(err) as u32;
                *a2 = ptr as u32;
                *a3 = data as u32;
            }
        }
    }
}

// ---------- USERSPACE KERNEL BOUNDARY ----------

/// `ContentSwitchReason` specifies why the process stopped executing and
/// execution returned to the kernel.
#[derive(PartialEq, Copy, Clone)]
pub enum ContextSwitchReason {
    /// Process called a syscall. Also returns the syscall and relevant values.
    SyscallFired { syscall: Syscall },
    /// Process triggered the hardfault handler.
    /// The implementation should still save registers in the event that the
    /// `Platform` can handle the fault and allow the app to continue running.
    /// For more details on this see `Platform::process_fault_hook()`.
    Fault,
    /// Process interrupted (e.g. by a hardware event)
    Interrupted,
}

/// The `UserspaceKernelBoundary` trait is implemented by the
/// architectural component of the chip implementation of Tock. This
/// trait allows the kernel to switch to and from processes
/// in an architecture-independent manner.
///
/// Exactly how upcalls and return values are passed between
/// kernelspace and userspace is architecture specific. The
/// architecture may use process memory to store state when
/// switching. Therefore, functions in this trait are passed the
/// bounds of process-accessible memory so that the architecture
/// implementation can verify it is reading and writing memory that
/// the process has valid access to. These bounds are passed through
/// `accessible_memory_start` and `app_brk` pointers.
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
    /// process by providing `accessible_memory_start`. It also provides the
    /// `app_brk` pointer which marks the end of process-accessible memory. The
    /// kernel guarantees that `accessible_memory_start` will be word-aligned.
    ///
    /// If successful, this function returns `Ok()`. If the process syscall
    /// state cannot be initialized with the available amount of memory, or for
    /// any other reason, it should return `Err()`.
    ///
    /// This function may be called multiple times on the same process. For
    /// example, if a process crashes and is to be restarted, this must be
    /// called. Or if the process is moved this may need to be called.
    ///
    /// ### Safety
    ///
    /// This function guarantees that it if needs to change process memory, it
    /// will only change memory starting at `accessible_memory_start` and before
    /// `app_brk`. The caller is responsible for guaranteeing that those
    /// pointers are valid for the process.
    unsafe fn initialize_process(
        &self,
        accessible_memory_start: *const u8,
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
    ///
    /// ### Safety
    ///
    /// This function guarantees that it if needs to change process memory, it
    /// will only change memory starting at `accessible_memory_start` and before
    /// `app_brk`. The caller is responsible for guaranteeing that those
    /// pointers are valid for the process.
    unsafe fn set_syscall_return_value(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: SyscallReturn,
    ) -> Result<(), ()>;

    /// Set the function that the process should execute when it is resumed.
    /// This has two major uses: 1) sets up the initial function call to
    /// `_start` when the process is started for the very first time; 2) tells
    /// the process to execute a upcall function after calling `yield()`.
    ///
    /// **Note:** This method cannot be called in conjunction with
    /// `set_syscall_return_value`, as the injected function will clobber the
    /// return value.
    ///
    /// ### Arguments
    ///
    /// - `accessible_memory_start` is the address of the start of the
    ///   process-accessible memory region for this process.
    /// - `app_brk` is the address of the current process break. This marks the
    ///   end of the memory region the process has access to. Note, this is not
    ///   the end of the entire memory region allocated to the process. Some
    ///   memory above this address is still allocated for the process, but if
    ///   the process tries to access it an MPU fault will occur.
    /// - `state` is the stored state for this process.
    /// - `upcall` is the function that should be executed when the process
    ///   resumes.
    ///
    /// ### Return
    ///
    /// Returns `Ok(())` if the function was successfully enqueued for the
    /// process. Returns `Err(())` if the function was not, likely because there
    /// is insufficient memory available to do so.
    ///
    /// ### Safety
    ///
    /// This function guarantees that it if needs to change process memory, it
    /// will only change memory starting at `accessible_memory_start` and before
    /// `app_brk`. The caller is responsible for guaranteeing that those
    /// pointers are valid for the process.
    unsafe fn set_process_function(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        upcall: process::FunctionCall,
    ) -> Result<(), ()>;

    /// Context switch to a specific process.
    ///
    /// This returns two values in a tuple.
    ///
    /// 1. A `ContextSwitchReason` indicating why the process stopped executing
    ///    and switched back to the kernel.
    /// 2. Optionally, the current stack pointer used by the process. This is
    ///    optional because it is only for debugging in process.rs. By sharing
    ///    the process's stack pointer with process.rs users can inspect the
    ///    state and see the stack depth, which might be useful for debugging.
    ///
    /// ### Safety
    ///
    /// This function guarantees that it if needs to change process memory, it
    /// will only change memory starting at `accessible_memory_start` and before
    /// `app_brk`. The caller is responsible for guaranteeing that those
    /// pointers are valid for the process.
    unsafe fn switch_to_process(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> (ContextSwitchReason, Option<*const u8>);

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by the stored state for that process.
    ///
    /// ### Safety
    ///
    /// This function guarantees that it if needs to change process memory, it
    /// will only change memory starting at `accessible_memory_start` and before
    /// `app_brk`. The caller is responsible for guaranteeing that those
    /// pointers are valid for the process.
    unsafe fn print_context(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    );
}
