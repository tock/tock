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

/// The `ProcessManagementCapability` allows the holder to control
/// process execution, such as related to creating, restarting, and
/// otherwise managing processes.
pub unsafe trait ProcessManagementCapability {}

/// The `MainLoopCapability` capability allows the holder to start executing as
/// well as manage the main scheduler loop in Tock. This is needed in a board's
/// main.rs file to start the kernel. It also allows an external implementation
/// of `ProcessType` to update state in the kernel struct used by the main loop.
pub unsafe trait MainLoopCapability {}

/// The `MemoryAllocationCapability` capability allows the holder to allocate
/// memory, for example by creating grants.
pub unsafe trait MemoryAllocationCapability {}

/// The `ExternalProcessCapability` capability allows the holder to use the core
/// kernel resources needed to successfully implement the `ProcessType` trait
/// from outside of the core kernel crate. Many of these operations are very
/// sensitive, that is they cannot just be made public. In particular, certain
/// objects can be used outside of the core kernel, but the constructors must be
/// restricted.
pub unsafe trait ExternalProcessCapability {}

/// The `UdpDriverCapability` capability allows the holder to use two functions
/// only allowed by the UDP driver. The first is the `driver_send_to()` function
/// in udp_send.rs, which does not require being bound to a single port, since
/// the driver manages port bindings for apps on its own. The second is the
/// `set_user_ports()` function in `udp_port_table.rs`, which gives the UDP port
/// table a reference to the UDP driver so that it can check which ports have
/// been bound by apps.
pub unsafe trait UdpDriverCapability {}

/// The `CreatePortTableCapability` capability allows the holder to instantiate
/// a new copy of the UdpPortTable struct. There should only ever be one
/// instance of this struct, so this capability should not be distributed to
/// capsules at all, as the port table should only be instantiated once by the
/// kernel
pub unsafe trait CreatePortTableCapability {}

/// The `NetworkCapabilityCreationCapability` allows the holder to instantiate
/// `NetworkCapability`S and visibility capabilities for the IP and UDP layers
/// of the networking stack. A capsule would never hold this capability although
/// it may hold capabilities created via this capability.
pub unsafe trait NetworkCapabilityCreationCapability {}
