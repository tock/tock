//! System call interface for userspace applications.
//!
//! Drivers implement these interfaces to expose operations to applications.
//!
//! # System-call Overview
//!
//! Tock supports four system calls. The `yield` system call is handled entirely
//! by the scheduler, while three others are passed along to drivers:
//!
//!   * `subscribe` lets an application pass a callback to the driver to be
//!   called later, when an event has occurred or data of interest is available.
//!
//!   * `command` tells the driver to do something immediately.
//!
//!   * `allow` provides the driver access to an application buffer.
//!
//! ## Mapping system-calls to drivers
//!
//! Each of these three system calls takes at least two parameters. The first is
//! a _driver major number_ and tells the scheduler which driver to forward the
//! system call to. The second parameters is a _driver minor number_ and is used
//! by the driver to differentiate system calls with different driver-specific
//! meanings (e.g. `subscribe` to "data ready" vs `subscribe` to "send
//! complete"). The mapping between _driver major numbers_ and drivers is
//! determined by a particular platform, while the _driver minor number_ is
//! driver-specific.
//!
//! One convention in Tock is that _driver minor number_ 0 for the `command`
//! syscall can always be used to determine if the driver is supported by
//! the running kernel by checking the return code. If the return value is
//! greater than or equal to zero then the driver is present. Typically this is
//! implemented by a null command that only returns 0, but in some cases the
//! command can also return more information, like the number of supported
//! devices (useful for things like the number of LEDs).
//!
//! # The `yield` System-call
//!
//! While drivers do not handle the `yield` system call, it is important to
//! understand its function and how it interacts with `subscribe`.

use crate::callback::{AppId, Callback};
use crate::errorcode::ErrorCode;
use crate::mem::{AppSlice, Shared};
use crate::returncode::ReturnCode;

/// Possible return values of an `allow` driver method
pub enum AllowResult {
    /// The allow operation succeeded and the AppSlice has been stored
    /// with the capsule
    ///
    /// The capsule **must** return any previously shared (potentially
    /// empty, default constructed) AppSlice.
    Success(AppSlice<Shared, u8>),
    /// The allow operation was refused. The capsule has not stored
    /// the AppSlice instance
    ///
    /// The capsule **must** return the passed AppSlice back.
    Refuse(AppSlice<Shared, u8>, ErrorCode),
}

impl AllowResult {
    pub fn success(old_appslice: AppSlice<Shared, u8>) -> Self {
        AllowResult::Success(old_appslice)
    }

    pub fn refuse_allow(new_appslice: AppSlice<Shared, u8>, reason: ErrorCode) -> Self {
        AllowResult::Refuse(new_appslice, reason)
    }
}

// TODO: (to confirm) This can be the same type in the driver return
// value and for the system call return value encoding (in contrast to
// the current proposed AllowReturnValue and SubscribeReturnValue)
//
// Therefore, it is named CommandResult (analog to what the
// Driver-trait return values for allow and subscribe would be called)
// and it moved to driver.rs, as it's more closely associated with the
// driver trait
/// Possible return values of a `command` driver method
#[derive(Copy, Clone, Debug)]
pub enum CommandResult {
    /// Generic error case
    Error(ErrorCode),
    /// Generic error case, with an additional 32-bit data field
    ErrorU32(ErrorCode, u32),
    /// Generic error case, with two additional 32-bit data fields
    ErrorU32U32(ErrorCode, u32, u32),
    /// Generic error case, with an additional 64-bit data field
    ErrorU64(ErrorCode, u64),
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
}

impl CommandResult {
    // TODO: Verify that these constructors get inlined
    pub fn error(rc: ErrorCode) -> Self {
        CommandResult::Error(rc)
    }

    pub fn error_u32(rc: ErrorCode, data0: u32) -> Self {
        CommandResult::ErrorU32(rc, data0)
    }

    pub fn error_u32_u32(rc: ErrorCode, data0: u32, data1: u32) -> Self {
        CommandResult::ErrorU32U32(rc, data0, data1)
    }

    pub fn error_u64(rc: ErrorCode, data0: u64) -> Self {
        CommandResult::ErrorU64(rc, data0)
    }

    pub fn success() -> Self {
        CommandResult::Success
    }

    pub fn success_u32(data0: u32) -> Self {
        CommandResult::SuccessU32(data0)
    }

    pub fn success_u32_u32(data0: u32, data1: u32) -> Self {
        CommandResult::SuccessU32U32(data0, data1)
    }

    pub fn success_u32_u32_u32(data0: u32, data1: u32, data2: u32) -> Self {
        CommandResult::SuccessU32U32U32(data0, data1, data2)
    }

    pub fn success_u64(data0: u64) -> Self {
        CommandResult::SuccessU64(data0)
    }

    pub fn success_u64_u32(data0: u64, data1: u32) -> Self {
        CommandResult::SuccessU64U32(data0, data1)
    }
}

/// `Driver`s implement the three driver-specific system calls: `subscribe`,
/// `command` and `allow`.
///
/// See [the module level documentation](index.html) for an overview of how
/// system calls are assigned to drivers.
pub trait Driver {
    /// `subscribe` lets an application pass a callback to the driver to be
    /// called later. This returns `ENOSUPPORT` if not used.
    ///
    /// Calls to subscribe should do minimal synchronous work.  Instead, they
    /// should defer most work and returns results to the application via the
    /// callback. For example, a subscribe call might setup a DMA transfer to
    /// read from a sensor, and asynchronously respond to the application by
    /// passing the result to the application via the callback.
    ///
    /// Drivers should allow each application to register a single callback for
    /// each minor number subscription. Thus, a second call to subscribe from
    /// the same application would replace a previous callback.
    ///
    /// This pushes most per-application virtualization to the application
    /// itself. For example, a timer driver exposes only one timer to each
    /// application, and the application is responsible for virtualizing that
    /// timer if it needs to.
    ///
    /// The driver should signal success or failure through the sign of the
    /// return value from `subscribe`. A negative return value signifies an
    /// error, while positive a return values signifies success. In addition,
    /// the magnitude of the return value of can signify extra information such
    /// as error type.
    #[allow(unused_variables)]
    fn subscribe(&self, minor_num: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    /// `command` instructs a driver to perform some action synchronously. This
    /// returns `ENOSUPPORT` if not used.
    ///
    /// The return value should reflect the result of an action. For example,
    /// enabling/disabling a peripheral should return a success or error code.
    /// Reading the current system time should return the time as an integer.
    ///
    /// Commands should not execute long running tasks synchronously. However,
    /// commands might "kick-off" asynchronous tasks in coordination with a
    /// `subscribe` call.
    ///
    /// All drivers must support the command with `minor_num` 0, and return 0
    /// or greater if the driver is supported. This command should not have any
    /// side effects. This convention ensures that applications can query the
    /// kernel for supported drivers on a given platform.
    #[allow(unused_variables)]
    fn command(&self, minor_num: usize, r2: usize, r3: usize, caller_id: AppId) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    // Tock 2.0 method preview:
    //
    // #[allow(unused_variables)]
    // fn command(&self, minor_num: usize, r2: usize, r3: usize, caller_id: AppId) -> CommandResult {
    //     CommandResult::error(ErrorCode::ENOSUPPORT)
    // }

    /// `allow` lets an application give the driver access to a buffer in the
    /// application's memory. This returns `ENOSUPPORT` if not used.
    ///
    /// The buffer is __shared__ between the application and driver, meaning the
    /// driver should not rely on the contents of the buffer to remain
    /// unchanged.
    #[allow(unused_variables)]
    fn allow(
        &self,
        app: AppId,
        minor_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    // Tock 2.0 method preview:
    //
    // #[allow(unused_variables)]
    // fn allow(
    //     &self,
    //     app: AppId,
    //     minor_num: usize,
    //     slice: AppSlice<Shared, u8>,
    // ) -> AllowResult {
    //     AllowResult::refuse_allow(slice, ErrorCode::ENOSUPPORT)
    // }
}
