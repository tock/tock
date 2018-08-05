//! Special restricted capabilities.
//!
//! Rust provides a mechanism for restricting certain operations to only be used
//! by trusted code through the `unsafe` keyword. This is very useful, but
//! doesn't provide very granular access: code can either access _all_ `unsafe`
//! things, or none.
//!
//! Capabilities are the mechanism in Tock that provides more granular access.
//! For sensitive operations (e.g. operations that could violate isolation)
//! callers must have a particular capability. The type system ensures that the
//! caller does in fact have the capability, and `unsafe` is used to ensure that
//! callers cannot create the capability type themselves.
//!
//! Capabilities are passed to modules from trusted code (i.e. code that can
//! call `unsafe`).
//!
//! Capabilities are expressed as `unsafe` traits. Only code that can use
//! `unsafe` mechanisms can instantiate an object that provides an `unsafe`
//! trait. Functions that require certain capabilities require that they are
//! passed an object that provides the correct capability trait. The object
//! itself does not have to be marked `unsafe`.
//!
//! Creating an object that expresses a capability is straightforward:
//!
//! ```
//! use kernel::capabilities::ProcessManagementCapability;
//!
//! struct ProcessMgmtCap;
//! unsafe impl ProcessManagementCapability for ProcessMgmtCap {}
//! ```
//!
//! Now anything that has a ProcessMgmtCap can call any function that requires
//! the `ProcessManagementCapability` capability.
//!
//! Requiring a certain capability is also straightforward:
//!
//! ```ignore
//! pub fn manage_process<C: ProcessManagementCapability>(_c: &C) {
//!    unsafe {
//!        ...
//!    }
//! }
//! ```
//!
//! Anything that calls `manage_process` must have a reference to some object
//! that provides the `ProcessManagementCapability` trait, which proves that it
//! has the correct capability.

/// The `ProcessManagementCapability` capability allows the holder to call
/// various functions related to creating, restarting, and otherwise managing
/// processes.
pub unsafe trait ProcessManagementCapability {}

/// The `MainLoopCapability` capability allows the holder to start executing
/// the main scheduler loop in Tock.
pub unsafe trait MainLoopCapability {}

/// The `MemoryAllocationCapability` capability allows the holder to allocate
/// memory, for example by creating grants.
pub unsafe trait MemoryAllocationCapability {}
