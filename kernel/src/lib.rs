//! Core Tock Kernel
//!
//! The kernel crate implements the core features of Tock as well as shared
//! code that many chips, capsules, and boards use. It also holds the Hardware
//! Interface Layer (HIL) definitions.
//!
//! Most `unsafe` code is in this kernel crate.

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
mod grant;
mod mem;
mod memop;
mod platform;
mod process;
mod returncode;
mod sched;
mod tbfheader;

pub use crate::callback::{AppId, Callback};
pub use crate::driver::Driver;
pub use crate::grant::Grant;
pub use crate::mem::{AppSlice, Private, Shared};
pub use crate::platform::scheduler_timer::{SchedulerTimer, VirtualSchedulerTimer};
pub use crate::platform::watchdog;
pub use crate::platform::{mpu, Chip, Platform};
pub use crate::platform::{ClockInterface, NoClockControl, NO_CLOCK_CONTROL};
pub use crate::returncode::ReturnCode;
pub use crate::sched::Kernel;

// Export only select items from the process module. To remove the name conflict
// this cannot be called `process`, so we use a shortened version. These
// functions and types are used by board files to setup the platform and setup
// processes.
/// Publicly available process-related objects.
pub mod procs {
    pub use crate::process::{
        load_processes, AlwaysRestart, Error, FaultResponse, FunctionCall, Process,
        ProcessLoadError, ProcessRestartPolicy, ProcessType, ThresholdRestart,
        ThresholdRestartThenPanic,
    };
}
