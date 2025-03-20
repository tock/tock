// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Core Tock Kernel
//!
//! The kernel crate implements the core features of Tock as well as shared
//! code that many chips, capsules, and boards use. It also holds the Hardware
//! Interface Layer (HIL) definitions.
//!
//! Most `unsafe` code is in this kernel crate.
//!
//!
//! ## Core Kernel Visibility
//!
//! As the root crate in the Tock operating system, this crate serves multiple
//! purposes:
//!
//! 1. It includes the logic for the core kernel, including process management,
//!    grants, scheduling, etc.
//!
//! 2. It includes important interfaces for hardware and other device
//!    abstractions. These are generally in the HIL and platform folders.
//!
//! 3. It includes utility functions used elsewhere in the kernel, generally by
//!    multiple different crates such that it makes sense to have shared
//!    implementations in the core kernel crate.
//!
//! Because of these different features of the core kernel, managing visibility
//! of the various objects and functions is a bit tricky. In general, the kernel
//! crate only exposes what it absolutely needs to. However, there are three
//! cases where resources in this crate _must_ be exposed.
//!
//! 1. The shared utility functions and structs must be exposed. These are
//!    marked `pub` and are used by many other kernel crates.
//!
//!    Some utility objects and abstractions, however, expose memory unsafe
//!    behavior. These are marked as `unsafe`, and require an `unsafe` block to
//!    use them. One example of this is `StaticRef` which is used for accessing
//!    memory-mapped I/O registers. Since accessing the addresses through just a
//!    memory address is potentially very unsafe, instantiating a `StaticRef`
//!    requires an `unsafe` block.
//!
//! 2. The core kernel types generally have to be exposed as other layers of the
//!    OS need to use them. However, generally only a very small interface is
//!    exposed, and using that interface cannot compromise the overall system or
//!    the core kernel. These functions are also marked `pub`. For example, the
//!    `ProcessBuffer` abstraction must be exposed to capsules to use shared memory
//!    between a process and the kernel. However, the constructor is not public,
//!    and the API exposed to capsules is very limited and confined by the Rust
//!    type system. The constructor and other sensitive interfaces are
//!    restricted to use only inside the kernel crate and are marked
//!    `pub(crate)`.
//!
//!    In some cases, more sensitive core kernel interfaces must be exposed. For
//!    example, the kernel exposes a function for starting the main scheduling
//!    loop in the kernel. Since board crates must be able to start this loop
//!    after all initialization is finished, the kernel loop function must be
//!    exposed and marked `pub`. However, this interface is not generally safe
//!    to use, since starting the loop a second time would compromise the
//!    stability of the overall system. It's also not necessarily memory unsafe
//!    to call the start loop function again, so we do not mark it as `unsafe`.
//!    Instead, we require that the caller hold a `Capability` to call the
//!    public but sensitive functions. More information is in `capabilities.rs`.
//!    This allows the kernel crate to still expose functions as public while
//!    restricting their use. Another example of this is the `Grant`
//!    constructor, which must be called outside of the core kernel, but should
//!    not be called except during the board setup.
//!
//! 3. Certain internal core kernel interfaces must also be exposed. These are
//!    needed for extensions of the core kernel that happen to be implemented in
//!    crates outside of the kernel crate. For example, additional
//!    implementations of `Process` may live outside of the kernel crate. To
//!    successfully implement a new `Process` requires access to certain
//!    in-core-kernel APIs, and these must be marked `pub` so that outside
//!    crates can access them.
//!
//!    These interfaces are highly sensitive, so again we require the caller
//!    hold a Capability to call them. This helps restrict their use and makes
//!    it very clear that calling them requires special permissions.
//!    Additionally, to differentiate these interfaces, which are for external
//!    extensions of core kernel functionality, from the other public but
//!    sensitive interfaces (item 2 above), we append the name `_external` to
//!    the function name.
//!
//!    One note is that there are currently very few extensions to the core
//!    kernel that live outside of the kernel crate. That means we have not
//!    necessarily created `_extern` functions for all the interfaces needed for
//!    this use case. It is likely we will have to create new interfaces as new
//!    use cases are discovered.

#![warn(unreachable_pub)]
#![no_std]

/// Kernel major version.
///
/// This is compiled with the crate to enable for checking of compatibility with
/// loaded apps. Both major and minor version constants are updated during a
/// release.
pub const KERNEL_MAJOR_VERSION: u16 = 2;
/// Kernel minor version.
///
/// This is compiled with the crate to enable for checking of compatibility with
/// loaded apps.
pub const KERNEL_MINOR_VERSION: u16 = 1;

pub mod capabilities;
pub mod collections;
pub mod component;
pub mod debug;
pub mod deferred_call;
pub mod dynamic_binary_storage;
pub mod errorcode;
pub mod grant;
pub mod hil;
pub mod introspection;
pub mod ipc;
pub mod platform;
pub mod process;
pub mod process_checker;
pub mod processbuffer;
pub mod scheduler;
pub mod storage_permissions;
pub mod syscall;
pub mod upcall;
pub mod utilities;

mod config;
mod kernel;
mod memop;
mod process_binary;
mod process_loading;
mod process_policies;
mod process_printer;
mod process_standard;
mod syscall_driver;

// Core resources exposed as `kernel::Type`.
pub use crate::errorcode::ErrorCode;
pub use crate::kernel::Kernel;
pub use crate::process::ProcessId;
pub use crate::scheduler::Scheduler;
