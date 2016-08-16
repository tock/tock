//! Driver implemented system call interface.
//!
//! Drivers implement these interfaces to expose operations to applications.
//!
//! # System-call Overview
//!
//! Tock supports four system calls. The `wait` system call is handled entirely
//! by the scheduler, while three others are passed along to drivers:
//!
//!   * `subscribe` lets an application pass a callback to the driver to be
//!   called later, when an event has occured or data of interest is available.
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
//! # The `wait` System-call
//!
//! While drivers to handle the `wait` system call, it's important to understand
//! it's function and it interacts with `subscribe`.

/// `Driver`s implement the three driver-specific system calls: `subscribe`,
/// `command` and `allow`.
///
/// See [the module level documentation](index.html) for an overview of how
/// system calls are assigned to drivers.
pub trait Driver {
    /// `subscribe` lets an application pass a callback to the driver to be
    /// called later.
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
    fn subscribe(&self, minor_num: usize, callback: ::Callback) -> isize {
        -1
    }

    /// `command` instructs a driver to perform some action synchronously.
    ///
    /// The return value should reflect the result of an action. For example,
    /// enabling/disabling a peripheral should return a success or error code.
    /// Reading the current system time should return the time as an integer.
    ///
    /// Commands should not execute long running tasks synchronously. However,
    /// commands might "kick-off" asynchronous tasks in coordination with a
    /// `subscribe` call.
    #[allow(unused_variables)]
    fn command(&self, minor_num: usize, r2: usize, caller_id: ::AppId) -> isize {
        -1
    }

    /// `allow` lets an application give the driver access to a buffer in the
    /// application's memory.
    ///
    /// The buffer is __shared__ between the application and driver, meaning the
    /// driver should not rely on the contents of the buffer to remain
    /// unchanged.
    #[allow(unused_variables)]
    fn allow(&self, app: ::AppId, minor_num: usize, slice: ::AppSlice<::Shared, u8>) -> isize {
        -1
    }
}

