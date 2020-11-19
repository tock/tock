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
//!    `AppSlice` abstraction must be exposed to capsules to use shared memory
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
//!    implementations of `ProcessType` may live outside of the kernel crate. To
//!    successfully implement a new `ProcessType` requires access to certain
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

#![feature(core_intrinsics, const_fn, associated_type_defaults, try_trait)]
#![warn(unreachable_pub)]
#![no_std]

pub mod capabilities;
pub mod common;
pub mod component;
pub mod debug;
pub mod hil;
pub mod introspection;
pub mod ipc;
pub mod syscall;

mod callback;
mod config;
mod driver;
mod errorcode;
mod grant;
mod mem;
mod memop;
mod platform;
mod process;
mod returncode;
mod sched;
mod tbfheader;

pub use crate::callback::{AppId, Callback};
pub use crate::driver::{
    AllowReadOnlyResult, AllowReadWriteResult, CommandResult, Driver, LegacyDriver,
};
pub use crate::errorcode::ErrorCode;
pub use crate::grant::Grant;
pub use crate::mem::{AppSlice, Private, Read, ReadWrite, SharedReadOnly, SharedReadWrite};
pub use crate::platform::scheduler_timer::{SchedulerTimer, VirtualSchedulerTimer};
pub use crate::platform::watchdog;
pub use crate::platform::{mpu, Chip, InterruptService, Platform};
pub use crate::platform::{ClockInterface, NoClockControl, NO_CLOCK_CONTROL};
pub use crate::returncode::ReturnCode;
pub use crate::sched::cooperative::{CoopProcessNode, CooperativeSched};
pub use crate::sched::mlfq::{MLFQProcessNode, MLFQSched};
pub use crate::sched::priority::PrioritySched;
pub use crate::sched::round_robin::{RoundRobinProcessNode, RoundRobinSched};
pub use crate::sched::{Kernel, Scheduler};

// Export only select items from the process module. To remove the name conflict
// this cannot be called `process`, so we use a shortened version. These
// functions and types are used by board files to setup the platform and setup
// processes.
/// Publicly available process-related objects.
pub mod procs {
    pub use crate::process::{
        load_processes, AlwaysRestart, Error, FaultResponse, FunctionCall, FunctionCallSource,
        Process, ProcessLoadError, ProcessRestartPolicy, ProcessType, State, Task,
        ThresholdRestart, ThresholdRestartThenPanic,
    };
}
