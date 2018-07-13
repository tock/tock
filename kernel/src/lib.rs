//! Core Tock Kernel
//!
//! The kernel crate implements the core features of Tock as well as shared
//! code that many chips, capsules, and boards use. It also holds the Hardware
//! Interface Layer (HIL) definitions.
//!
//! Most `unsafe` code is in this kernel crate.

#![feature(asm, core_intrinsics, unique, ptr_internals, const_fn)]
#![feature(use_extern_macros, try_from, used)]
#![feature(in_band_lifetimes, crate_visibility_modifier)]
#![warn(unreachable_pub)]
#![no_std]

extern crate tock_cells;
extern crate tock_registers;

pub use tock_registers::{register_bitfields, register_bitmasks};

#[macro_use]
pub mod common;
#[macro_use]
pub mod debug;
pub mod hil;
pub mod ipc;

mod callback;
mod driver;
mod grant;
mod mem;
mod memop;
mod platform;
mod process;
mod returncode;
mod sched;
mod syscall;
mod tbfheader;

pub use callback::{AppId, Callback};
pub use driver::Driver;
pub use grant::Grant;
pub use mem::{AppPtr, AppSlice, Private, Shared};
pub use platform::systick::SysTick;
pub use platform::{mpu, Chip, Platform};
pub use platform::{ClockInterface, NoClockControl, NO_CLOCK_CONTROL};
pub use returncode::ReturnCode;
pub use sched::Kernel;

// These symbols must be exported for the arch crate to access them.
pub use process::APP_FAULT;
pub use process::SYSCALL_FIRED;

// Export only select items from the process module. To remove the name conflict
// this cannot be called `process`, so we use a shortened version. These
// functions and types are used by board files to setup the platform and setup
// processes.
pub mod procs {
    pub use process::{load_processes, FaultResponse, Process};
}
