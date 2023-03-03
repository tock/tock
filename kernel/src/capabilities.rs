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
//! Capabilities are expressed as types wrapped in the [`Capability`] type.
//! This wrapper cannot be constructed directly, only through the `unsafe`
//! `new` method.
//!
//! Creating an object that expresses a capability is straightforward:
//!
//! ```
//! use kernel::capabilities::{Capability, ProcessManagement};
//!
//! let process_mgmt_cap = unsafe { Capability::<ProcessManagement>::new() };
//! ```
//!
//! Now anything that has `process_mgmt_cap` can call any function that requires
//! the `Capability<ProcessManagement>` capability.
//!
//! Requiring a certain capability is also straightforward:
//!
//! ```ignore
//! pub fn manage_process(_c: &Capability<ProcessManagement>) {
//!    unsafe {
//!        // ...
//!    }
//! }
//! ```
//!
//! Anything that calls `manage_process` must have a reference to some object
//! of type `Capability<ProcessManagement>`, which proves that it has the
//! correct capability.

use core::marker::PhantomData;

/// Wrapper type which can only be constructed through the unsafe `new` method.
///
/// Prevents capabilities from being created in safe code.
pub struct Capability<T> {
    /// Private type to prevent `Capability` being constructed manually.
    capability: PhantomData<T>,
}

impl<T> Capability<T> {
    /// Only allow `Capability`s to be created from `unsafe` code.
    pub unsafe fn new() -> Self {
        Self {
            capability: PhantomData {},
        }
    }
}

/// The `ProcessManagement` capability allows the holder to control process
/// execution, such as related to creating, restarting, and otherwise managing
/// processes.
pub struct ProcessManagement;

/// The `ProcessApproval` capability allows the holder to approve the
/// cryptographic credentials of a process, indicating they have permission to
/// be run.
pub struct ProcessApproval;

/// The `ProcessInit` capability allows the holder to start a process to run by
/// pushing an init function stack frame. This is controlled and separate from
/// the `ProcessManagement` capability because the process must have a unique
/// application identifier and so only modules which check this may do so.
pub struct ProcessInit;

/// The `MainLoop` capability allows the holder to start executing as well as
/// manage the main scheduler loop in Tock. This is needed in a board's main.rs
/// file to start the kernel. It also allows an external implementation of
/// `Process` to update state in the kernel struct used by the main loop.
pub struct MainLoop;

/// The `MemoryAllocation` capability allows the holder to allocate memory, for
/// example by creating grants.
pub struct MemoryAllocation;

/// The `ExternalProcess` capability allows the holder to use the core kernel
/// resources needed to successfully implement the `Process` trait from outside
/// of the core kernel crate. Many of these operations are very sensitive, that
/// is they cannot just be made public. In particular, certain objects can be
/// used outside of the core kernel, but the constructors must be restricted.
pub struct ExternalProcess;

/// The `UdpDriver` capability allows the holder to use two functions only
/// allowed by the UDP driver. The first is the `driver_send_to()` function in
/// udp_send.rs, which does not require being bound to a single port, since the
/// driver manages port bindings for apps on its own. The second is the
/// `set_user_ports()` function in `udp_port_table.rs`, which gives the UDP port
/// table a reference to the UDP driver so that it can check which ports have
/// been bound by apps.
pub struct UdpDriver;

/// The `CreatePortTable` capability allows the holder to instantiate a new copy
/// of the UdpPortTable struct. There should only ever be one instance of this
/// struct, so this capability should not be distributed to capsules at all, as
/// the port table should only be instantiated once by the kernel
pub struct CreatePortTable;

/// The `NetworkCapabilityCreation` allows the holder to instantiate
/// `NetworkCapability`s and visibility capabilities for the IP and UDP layers
/// of the networking stack. A capsule would never hold this capability although
/// it may hold capabilities created via this capability.
pub struct NetworkCapabilityCreation;
