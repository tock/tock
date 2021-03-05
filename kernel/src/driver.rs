//! System call interface for userspace processes.
//!
//! Drivers implement these interfaces to expose operations to processes.
//!
//! # System-call Overview
//!
//! Tock supports six system calls. The `yield` and `memop` system calls are
//! handled by the core kernel, while four others are implemented by drivers:
//!
//!   * `subscribe` passes a callback to the driver which it can
//!   invoke on the process later, when an event has occurred or data
//!   of interest is available.
//!
//!   * `command` tells the driver to do something immediately.
//!
//!   * `allow_readwrite` provides the driver read-write access to an
//!   application buffer.
//!
//!   * `allow_readonly` provides the driver read-only access to an
//!   application buffer.
//!
//! ## Mapping system-calls to drivers
//!
//! Each of these three system calls takes at least two
//! parameters. The first is a _driver identifier_ and tells the
//! scheduler which driver to forward the system call to. The second
//! parameters is a __syscall identifer_ and is used by the driver to
//! differentiate instances of the call with different driver-specific
//! meanings (e.g. `subscribe` for "data received" vs `subscribe` for
//! "send completed"). The mapping between _driver identifiers_ and
//! drivers is determined by a particular platform, while the _syscall
//! identifier_ is driver-specific.
//!
//! One convention in Tock is that _driver minor number_ 0 for the `command`
//! syscall can always be used to determine if the driver is supported by
//! the running kernel by checking the return code. If the return value is
//! greater than or equal to zero then the driver is present. Typically this is
//! implemented by a null command that only returns 0, but in some cases the
//! command can also return more information, like the number of supported
//! devices (useful for things like the number of LEDs).
//!
//! # The `yield` system call class
//!
//! While drivers do not handle `yield` system calls, it is important
//! to understand them and how they interact with `subscribe`, which
//! registers callback functions with the kernel. When a process calls
//! a `yield` system call, the kernel checks if there are any pending
//! callbacks for the process. If there are pending callbacks, it
//! pushes one callback onto the process stack. If there are no
//! pending callbacks, `yield-wait` will cause the process to sleep
//! until a callback is trigered, while `yield-no-wait` returns
//! immediately.
//!
//! # Method result types
//!
//! Each driver method has a limited set of valid return types. Every
//! method has a single return type corresponding to success and a
//! single return type corresponding to failure. For the `subscribe`
//! and `allow` system calls, these return types are the same for
//! every instance of those calls. Each instance of the `command`
//! system call, however, has its own specified return types. A
//! command that requests a timestamp, for example, might return a
//! 32-bit number on success and an error code on failure, while a
//! command that requests time of day in microsecond granularity might
//! return a 64-bit number and a 32-bit timezone encoding on success,
//! and an error code on failure.
//!
//! These result types are represented as safe Rust types. The core
//! kernel (the scheduler and syscall dispatcher) is responsible for
//! encoding these types into the Tock system call ABI specification.

use crate::callback::{AppId, Callback};
use crate::errorcode::ErrorCode;
use crate::mem::{ReadOnlyAppSlice, ReadWriteAppSlice};
use crate::process;
use crate::returncode::ReturnCode;
use crate::syscall::SyscallReturn;

/// Possible return values of a `command` driver method, as specified
/// in TRD104.
///
/// This is just a wrapper around
/// [`SyscallReturn`](SyscallReturn) since a
/// `command` driver method may only return primitve integer types as
/// payload.
///
/// It is important for this wrapper to only be constructable over
/// variants of
/// [`SyscallReturn`](SyscallReturn) that are
/// deemed safe for a capsule to construct and return to an
/// application (e.g. not
/// [`SubscribeSuccess`](crate::syscall::SyscallReturn::SubscribeSuccess)).
/// This means that the inner value **must** remain private.
pub struct CommandReturn(SyscallReturn);
impl CommandReturn {
    pub(crate) fn into_inner(self) -> SyscallReturn {
        self.0
    }

    /// Command error
    pub fn failure(rc: ErrorCode) -> Self {
        CommandReturn(SyscallReturn::Failure(rc))
    }

    /// Command error with an additional 32-bit data field
    pub fn failure_u32(rc: ErrorCode, data0: u32) -> Self {
        CommandReturn(SyscallReturn::FailureU32(rc, data0))
    }

    /// Command error with two additional 32-bit data fields
    pub fn failure_u32_u32(rc: ErrorCode, data0: u32, data1: u32) -> Self {
        CommandReturn(SyscallReturn::FailureU32U32(rc, data0, data1))
    }

    /// Command error with an additional 64-bit data field
    pub fn failure_u64(rc: ErrorCode, data0: u64) -> Self {
        CommandReturn(SyscallReturn::FailureU64(rc, data0))
    }

    /// Successful command
    pub fn success() -> Self {
        CommandReturn(SyscallReturn::Success)
    }

    /// Successful command with an additional 32-bit data field
    pub fn success_u32(data0: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU32(data0))
    }

    /// Successful command with two additional 32-bit data fields
    pub fn success_u32_u32(data0: u32, data1: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU32U32(data0, data1))
    }

    /// Successful command with three additional 32-bit data fields
    pub fn success_u32_u32_u32(data0: u32, data1: u32, data2: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU32U32U32(data0, data1, data2))
    }

    /// Successful command with an additional 64-bit data field
    pub fn success_u64(data0: u64) -> Self {
        CommandReturn(SyscallReturn::SuccessU64(data0))
    }

    /// Successful command with an additional 64-bit and 32-bit data field
    pub fn success_u64_u32(data0: u64, data1: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU64U32(data0, data1))
    }
}

use core::convert::TryFrom;
impl From<ReturnCode> for CommandReturn {
    fn from(rc: ReturnCode) -> Self {
        match rc {
            ReturnCode::SUCCESS => CommandReturn::success(),
            _ => CommandReturn::failure(ErrorCode::try_from(rc).unwrap()),
        }
    }
}

impl From<process::Error> for CommandReturn {
    fn from(perr: process::Error) -> Self {
        CommandReturn::failure(perr.into())
    }
}

/// Trait for capsules implemeting peripheral driver system calls
/// specified in TRD104. The kernel translates the values passed from
/// userspace into Rust types and includes which process is making the
/// call. All of these system calls perform very little synchronous work;
/// long running computations or I/O should be split-phase, with an upcall
/// indicating their completion.
///
/// The exact instances of each of these methods (which identifiers are valid
/// and what they represents) are specific to the peripheral system call
/// driver.
#[allow(unused_variables)]
pub trait Driver {
    /// System call for a process to provide an upcall function pointer to
    /// the kernel. Peripheral system call driver capsules invoke
    /// upcalls to indicate events have occurred. These events are typically triggered
    /// in response to `command` calls. For example, a command that sets a timer to
    /// fire in the future will cause an upcall to invoke after the command returns, when
    /// the timer expires, while a command to sample a sensor will cause an upcall to
    /// invoke when the sensor value is ready.
    fn subscribe(
        &self,
        subscribe_identifier: usize,
        callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        Err((callback, ErrorCode::NOSUPPORT))
    }

    /// System call for a process to perform a short synchronous operation
    /// or start a long-running split-phase operation (whose completion
    /// is signaled with an upcall). Command 0 is a reserved command to
    /// detect if a peripheral system call driver is installed and must
    /// always return a CommandReturn::Success.
    fn command(&self, which: usize, r2: usize, r3: usize, caller_id: AppId) -> CommandReturn {
        CommandReturn::failure(ErrorCode::NOSUPPORT)
    }

    /// System call for a process to pass a buffer (a ReadWriteAppSlice) to
    /// the kernel that the kernel can either read or write. The kernel calls
    /// this method only after it checks that the entire buffer is
    /// within memory the process can both read and write.
    fn allow_readwrite(
        &self,
        app: AppId,
        which: usize,
        slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }

    /// System call for a process to pass a read-only buffer (a
    /// ReadOnlyAppSlice) to the kernel that the kernel can read.
    /// The kernel calls this method only after it
    /// checks that that the entire buffer is within memory the
    /// process can read. This system call allows a process to pass
    /// read-only data (e.g., in flash) to the kernel.
    fn allow_readonly(
        &self,
        app: AppId,
        which: usize,
        slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }
}
