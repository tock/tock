// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mechanisms for handling and defining system calls.
//!
//! # System Call Overview
//!
//! Tock supports six system calls. The `allow_readonly`, `allow_readwrite`,
//! `subscribe`, `yield`, and `memop` system calls are handled by the core
//! kernel, while `command` is implemented by drivers. The main system calls:
//!
//! - `subscribe` passes a upcall to the driver which it can invoke on the
//!   process later, when an event has occurred or data of interest is
//!   available.
//! - `command` tells the driver to do something immediately.
//! - `allow_readwrite` provides the driver read-write access to an application
//!   buffer.
//! - `allow_userspace_readable` provides the driver read-write access to an
//!   application buffer that is still shared with the app.
//! - `allow_readonly` provides the driver read-only access to an application
//!   buffer.
//!
//! ## Mapping system-calls to drivers
//!
//! Each of these three system calls takes at least two parameters. The first is
//! a _driver identifier_ and tells the scheduler which driver to forward the
//! system call to. The second parameters is a __syscall number_ and is used by
//! the driver to differentiate instances of the call with different
//! driver-specific meanings (e.g. `subscribe` for "data received" vs
//! `subscribe` for "send completed"). The mapping between _driver identifiers_
//! and drivers is determined by a particular platform, while the _syscall
//! number_ is driver-specific.
//!
//! One convention in Tock is that _driver minor number_ 0 for the `command`
//! syscall can always be used to determine if the driver is supported by the
//! running kernel by checking the return code. If the return value is greater
//! than or equal to zero then the driver is present. Typically this is
//! implemented by a null command that only returns 0, but in some cases the
//! command can also return more information, like the number of supported
//! devices (useful for things like the number of LEDs).
//!
//! # The `yield` system call class
//!
//! While drivers do not handle `yield` system calls, it is important to
//! understand them and how they interact with `subscribe`, which registers
//! upcall functions with the kernel. When a process calls a `yield` system
//! call, the kernel checks if there are any pending upcalls for the process. If
//! there are pending upcalls, it pushes one upcall onto the process stack. If
//! there are no pending upcalls, `yield-wait` will cause the process to sleep
//! until a upcall is triggered, while `yield-no-wait` returns immediately.
//!
//! # Method result types
//!
//! Each driver method has a limited set of valid return types. Every method has
//! a single return type corresponding to success and a single return type
//! corresponding to failure. For the `subscribe` and `allow` system calls,
//! these return types are the same for every instance of those calls. Each
//! instance of the `command` system call, however, has its own specified return
//! types. A command that requests a timestamp, for example, might return a
//! 32-bit number on success and an error code on failure, while a command that
//! requests time of day in microsecond granularity might return a 64-bit number
//! and a 32-bit timezone encoding on success, and an error code on failure.
//!
//! These result types are represented as safe Rust types. The core kernel (the
//! scheduler and syscall dispatcher) is responsible for encoding these types
//! into the Tock system call ABI specification.

use core::fmt::Write;

use crate::errorcode::ErrorCode;
use crate::process;
use crate::utilities::capability_ptr::CapabilityPtr;
use crate::utilities::machine_register::MachineRegister;

pub use crate::syscall_driver::{CommandReturn, SyscallDriver};

// ---------- SYSTEMCALL ARGUMENT DECODING ----------

/// Enumeration of the system call classes based on the identifiers specified in
/// the Tock ABI.
///
/// These are encoded as 8 bit values as on some architectures the value can be
/// encoded in the instruction itself.
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
    UserspaceReadableAllow = 7,
}

/// Enumeration of the yield system calls based on the Yield identifier
/// values specified in the Tock ABI.
#[derive(Copy, Clone, Debug)]
pub enum YieldCall {
    NoWait = 0,
    Wait = 1,
    WaitFor = 2,
}

impl TryFrom<usize> for YieldCall {
    type Error = usize;

    fn try_from(yield_variant: usize) -> Result<YieldCall, usize> {
        match yield_variant {
            0 => Ok(YieldCall::NoWait),
            1 => Ok(YieldCall::Wait),
            2 => Ok(YieldCall::WaitFor),
            i => Err(i),
        }
    }
}

// Required as long as no solution to
// https://github.com/rust-lang/rfcs/issues/2783 is integrated into
// the standard library.
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
            7 => Ok(SyscallClass::UserspaceReadableAllow),
            i => Err(i),
        }
    }
}

/// Decoded system calls as defined in TRD104.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Syscall {
    /// Structure representing an invocation of the [`SyscallClass::Yield`]
    /// system call class. `which` is the Yield identifier value and `address`
    /// is the no wait field.
    Yield {
        which: usize,
        param_a: usize,
        param_b: usize,
    },

    /// Structure representing an invocation of the Subscribe system call class.
    Subscribe {
        /// The driver identifier.
        driver_number: usize,
        /// The subscribe identifier.
        subdriver_number: usize,
        /// Upcall pointer to the upcall function.
        upcall_ptr: CapabilityPtr,
        /// Userspace application data.
        appdata: MachineRegister,
    },

    /// Structure representing an invocation of the Command system call class.
    Command {
        /// The driver identifier.
        driver_number: usize,
        /// The command identifier.
        subdriver_number: usize,
        /// Value passed to the `Command` implementation.
        arg0: usize,
        /// Value passed to the `Command` implementation.
        arg1: usize,
    },

    /// Structure representing an invocation of the ReadWriteAllow system call
    /// class.
    ReadWriteAllow {
        /// The driver identifier.
        driver_number: usize,
        /// The buffer identifier.
        subdriver_number: usize,
        /// The address where the buffer starts.
        allow_address: *mut u8,
        /// The size of the buffer in bytes.
        allow_size: usize,
    },

    /// Structure representing an invocation of the UserspaceReadableAllow
    /// system call class that allows shared kernel and app access.
    UserspaceReadableAllow {
        /// The driver identifier.
        driver_number: usize,
        /// The buffer identifier.
        subdriver_number: usize,
        /// The address where the buffer starts.
        allow_address: *mut u8,
        /// The size of the buffer in bytes.
        allow_size: usize,
    },

    /// Structure representing an invocation of the ReadOnlyAllow system call
    /// class.
    ReadOnlyAllow {
        /// The driver identifier.
        driver_number: usize,
        /// The buffer identifier.
        subdriver_number: usize,
        /// The address where the buffer starts.
        allow_address: *const u8,
        /// The size of the buffer in bytes.
        allow_size: usize,
    },

    /// Structure representing an invocation of the Memop system call class.
    Memop {
        /// The operation.
        operand: usize,
        /// The operation argument.
        arg0: usize,
    },

    /// Structure representing an invocation of the Exit system call class.
    Exit {
        /// The exit identifier.
        which: usize,
        /// The completion code passed into the kernel.
        completion_code: usize,
    },
}

impl Syscall {
    /// Helper function for converting raw values passed back from an
    /// application into a `Syscall` type in Tock, representing an typed version
    /// of a system call invocation. The method returns None if the values do
    /// not specify a valid system call.
    ///
    /// Different architectures have different ABIs for a process and the kernel
    /// to exchange data. The 32-bit ABI for CortexM and RISCV microcontrollers
    /// is specified in TRD104.
    pub fn from_register_arguments(
        syscall_number: u8,
        r0: usize,
        r1: MachineRegister,
        r2: MachineRegister,
        r3: MachineRegister,
    ) -> Option<Syscall> {
        match SyscallClass::try_from(syscall_number) {
            Ok(SyscallClass::Yield) => Some(Syscall::Yield {
                which: r0,
                param_a: r1.as_usize(),
                param_b: r2.as_usize(),
            }),
            Ok(SyscallClass::Subscribe) => Some(Syscall::Subscribe {
                driver_number: r0,
                subdriver_number: r1.as_usize(),
                upcall_ptr: r2.as_capability_ptr(),
                appdata: r3,
            }),
            Ok(SyscallClass::Command) => Some(Syscall::Command {
                driver_number: r0,
                subdriver_number: r1.as_usize(),
                arg0: r2.as_usize(),
                arg1: r3.as_usize(),
            }),
            Ok(SyscallClass::ReadWriteAllow) => Some(Syscall::ReadWriteAllow {
                driver_number: r0,
                subdriver_number: r1.as_usize(),
                allow_address: r2.as_capability_ptr().as_ptr::<u8>().cast_mut(),
                allow_size: r3.as_usize(),
            }),
            Ok(SyscallClass::UserspaceReadableAllow) => Some(Syscall::UserspaceReadableAllow {
                driver_number: r0,
                subdriver_number: r1.as_usize(),
                allow_address: r2.as_capability_ptr().as_ptr::<u8>().cast_mut(),
                allow_size: r3.as_usize(),
            }),
            Ok(SyscallClass::ReadOnlyAllow) => Some(Syscall::ReadOnlyAllow {
                driver_number: r0,
                subdriver_number: r1.as_usize(),
                allow_address: r2.as_capability_ptr().as_ptr(),
                allow_size: r3.as_usize(),
            }),
            Ok(SyscallClass::Memop) => Some(Syscall::Memop {
                operand: r0,
                arg0: r1.as_usize(),
            }),
            Ok(SyscallClass::Exit) => Some(Syscall::Exit {
                which: r0,
                completion_code: r1.as_usize(),
            }),
            Err(_) => None,
        }
    }

    /// Get the `driver_number` for the syscall classes that use driver numbers.
    pub fn driver_number(&self) -> Option<usize> {
        match *self {
            Syscall::Subscribe {
                driver_number,
                subdriver_number: _,
                upcall_ptr: _,
                appdata: _,
            } => Some(driver_number),
            Syscall::Command {
                driver_number,
                subdriver_number: _,
                arg0: _,
                arg1: _,
            } => Some(driver_number),
            Syscall::ReadWriteAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => Some(driver_number),
            Syscall::UserspaceReadableAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => Some(driver_number),
            Syscall::ReadOnlyAllow {
                driver_number,
                subdriver_number: _,
                allow_address: _,
                allow_size: _,
            } => Some(driver_number),
            _ => None,
        }
    }

    /// Get the `subdriver_number` for the syscall classes that use sub driver
    /// numbers.
    pub fn subdriver_number(&self) -> Option<usize> {
        match *self {
            Syscall::Subscribe {
                driver_number: _,
                subdriver_number,
                upcall_ptr: _,
                appdata: _,
            } => Some(subdriver_number),
            Syscall::Command {
                driver_number: _,
                subdriver_number,
                arg0: _,
                arg1: _,
            } => Some(subdriver_number),
            Syscall::ReadWriteAllow {
                driver_number: _,
                subdriver_number,
                allow_address: _,
                allow_size: _,
            } => Some(subdriver_number),
            Syscall::UserspaceReadableAllow {
                driver_number: _,
                subdriver_number,
                allow_address: _,
                allow_size: _,
            } => Some(subdriver_number),
            Syscall::ReadOnlyAllow {
                driver_number: _,
                subdriver_number,
                allow_address: _,
                allow_size: _,
            } => Some(subdriver_number),
            _ => None,
        }
    }
}

// ---------- SYSCALL RETURN VALUES ----------

/// Enumeration of the possible system call return variants.
///
/// This struct operates over primitive types such as integers of fixed length
/// and pointers. It is constructed by the scheduler and passed down to the
/// architecture to be encoded into registers. Architectures may use the various
/// helper functions defined in
/// [`utilities::arch_helpers`](crate::utilities::arch_helpers).
///
/// Capsules do not use this struct. Capsules use higher level Rust types (e.g.
/// [`ReadWriteProcessBuffer`](crate::processbuffer::ReadWriteProcessBuffer) and
/// [`GrantKernelData`](crate::grant::GrantKernelData)) or wrappers around this
/// struct ([`CommandReturn`]) which limit the available constructors to safely
/// constructable variants.
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
    /// Generic success case, with an additional 32-bit and 64-bit data field
    SuccessU32U64(u32, u64),

    /// Generic success case with an additional address-sized value
    /// that does not impute access permissions to the process.
    SuccessAddr(usize),

    /// Generic success case, with an additional pointer.
    /// This pointer is provenance bearing and implies access
    /// permission to the process.
    SuccessPtr(CapabilityPtr),

    // These following types are used by the scheduler so that it can return
    // values to userspace in an architecture (pointer-width) independent way.
    // The kernel passes these types (rather than ProcessBuffer or Upcall) for
    // two reasons. First, since the kernel/scheduler makes promises about the
    // lifetime and safety of these types, it does not want to leak them to
    // other code. Second, if subscribe or allow calls pass invalid values
    // (pointers out of valid memory), the kernel cannot construct an
    // ProcessBuffer or Upcall type but needs to be able to return a failure.
    // -pal 11/24/20

    // FIXME: We need to think about what these look like on CHERI
    // Really, things that were capabilities should come back as capabilities.
    // However, we discarded all capability information at the syscall boundary.
    // We could always use our own DDC, with just the permissions and length implied by the
    // specific syscall. This would certainly got give userspace _extra_ authority,
    // but might rob them of some bounds / permissions. This is what is implemented currently.
    // Preferable behavior is not to discard the capability so early (it should make it as far
    // as grant is stored in grant allow slots)
    /// Read/Write allow success case, returns the previous allowed buffer and
    /// size to the process.
    AllowReadWriteSuccess(*mut u8, usize),
    /// Read/Write allow failure case, returns the passed allowed buffer and
    /// size to the process.
    AllowReadWriteFailure(ErrorCode, *mut u8, usize),

    /// Shared Read/Write allow success case, returns the previous allowed
    /// buffer and size to the process.
    UserspaceReadableAllowSuccess(*mut u8, usize),
    /// Shared Read/Write allow failure case, returns the passed allowed buffer
    /// and size to the process.
    UserspaceReadableAllowFailure(ErrorCode, *mut u8, usize),

    /// Read only allow success case, returns the previous allowed buffer and
    /// size to the process.
    AllowReadOnlySuccess(*const u8, usize),
    /// Read only allow failure case, returns the passed allowed buffer and size
    /// to the process.
    AllowReadOnlyFailure(ErrorCode, *const u8, usize),

    /// Subscribe success case, returns the previous upcall function pointer and
    /// application data.
    SubscribeSuccess(*const (), usize),
    /// Subscribe failure case, returns the passed upcall function pointer and
    /// application data.
    SubscribeFailure(ErrorCode, *const (), usize),

    /// Yield-WaitFor return value. These arguments match the arguments to an
    /// upcall, where the kernel does not define an error field. Therefore this
    /// does not have success/failure versions because the kernel cannot know if
    /// the upcall (i.e. Yield-WaitFor return value) represents success or
    /// failure.
    YieldWaitFor(usize, usize, usize),
}

impl SyscallReturn {
    /// Transforms a [`CommandReturn`], which is wrapper around a subset of
    /// [`SyscallReturn`], into a [`SyscallReturn`].
    ///
    /// This allows [`CommandReturn`] to include only the variants of
    /// [`SyscallReturn`] that can be returned from a Command, while having an
    /// inexpensive way to handle it as a [`SyscallReturn`] for more generic
    /// code paths.
    pub(crate) fn from_command_return(res: CommandReturn) -> Self {
        res.into_inner()
    }

    /// Returns true if the [`SyscallReturn`] is any success type.
    pub(crate) fn is_success(&self) -> bool {
        match self {
            SyscallReturn::Success => true,
            SyscallReturn::SuccessU32(_) => true,
            SyscallReturn::SuccessU32U32(_, _) => true,
            SyscallReturn::SuccessU32U32U32(_, _, _) => true,
            SyscallReturn::SuccessU64(_) => true,
            SyscallReturn::SuccessU32U64(_, _) => true,
            SyscallReturn::SuccessAddr(_) => true,
            SyscallReturn::SuccessPtr(_) => true,
            SyscallReturn::AllowReadWriteSuccess(_, _) => true,
            SyscallReturn::UserspaceReadableAllowSuccess(_, _) => true,
            SyscallReturn::AllowReadOnlySuccess(_, _) => true,
            SyscallReturn::SubscribeSuccess(_, _) => true,
            SyscallReturn::Failure(_) => false,
            SyscallReturn::FailureU32(_, _) => false,
            SyscallReturn::FailureU32U32(_, _, _) => false,
            SyscallReturn::FailureU64(_, _) => false,
            SyscallReturn::AllowReadWriteFailure(_, _, _) => false,
            SyscallReturn::UserspaceReadableAllowFailure(_, _, _) => false,
            SyscallReturn::AllowReadOnlyFailure(_, _, _) => false,
            SyscallReturn::SubscribeFailure(_, _, _) => false,
            SyscallReturn::YieldWaitFor(_, _, _) => true,
        }
    }
}

// ---------- USERSPACE KERNEL BOUNDARY ----------

/// [`ContextSwitchReason`] specifies why the process stopped executing and
/// execution returned to the kernel.
#[derive(PartialEq, Copy, Clone)]
pub enum ContextSwitchReason {
    /// Process called a syscall. Also returns the syscall and relevant values.
    SyscallFired { syscall: Syscall },
    /// Process triggered the hardfault handler. The implementation should still
    /// save registers in the event that the platform can handle the fault and
    /// allow the app to continue running. For more details on this see
    /// [`ProcessFault`](crate::platform::ProcessFault).
    Fault,
    /// Process was interrupted (e.g. by a hardware event).
    Interrupted,
}

/// The [`UserspaceKernelBoundary`] trait is implemented by the
/// architectural component of the chip implementation of Tock.
///
/// This trait allows the kernel to switch to and from processes in an
/// architecture-independent manner.
///
/// Exactly how upcalls and return values are passed between kernelspace and
/// userspace is architecture specific. The architecture may use process memory
/// to store state when switching. Therefore, functions in this trait are passed
/// the bounds of process-accessible memory so that the architecture
/// implementation can verify it is reading and writing memory that the process
/// has valid access to. These bounds are passed through
/// `accessible_memory_start` and `app_brk` pointers.
pub trait UserspaceKernelBoundary {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    ///
    /// Implementations should **not** rely on the [`Default`] constructor
    /// (custom or derived) for any initialization of a process's stored state.
    /// The initialization must happen in the
    /// [`initialize_process()`](UserspaceKernelBoundary::initialize_process())
    /// function.
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
    /// 1. A [`ContextSwitchReason`] indicating why the process stopped
    ///    executing and switched back to the kernel.
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

    /// Store architecture specific (e.g. CPU registers or status flags) data
    /// for a process. On success returns the number of elements written to out.
    fn store_context(&self, state: &Self::StoredState, out: &mut [u8]) -> Result<usize, ErrorCode>;
}
