//! Core Tock Kernel
//!
//! The kernel crate implements the core features of Tock as well as shared
//! code that many chips, capsules, and boards use. It also holds the Hardware
//! Interface Layer (HIL) definitions.
//!
//! Most `unsafe` code is in this kernel crate.

#![feature(asm, core_intrinsics, unique, nonzero, ptr_internals)]
#![feature(const_fn, const_cell_new, const_unsafe_cell_new, lang_items)]
#![feature(nonnull_cast)]
#![no_std]

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

pub use callback::{AppId, Callback};
pub use driver::Driver;
pub use grant::Grant;
pub use mem::{AppPtr, AppSlice, Private, Shared};
pub use platform::systick::SysTick;
pub use platform::{mpu, Chip, Platform};
pub use platform::{ClockInterface, NoClockControl, NO_CLOCK_CONTROL};
pub use returncode::ReturnCode;

// Export only select items from the process module. To remove the name conflict
// this cannot be called `process`, so we use a shortened version. These
// functions and types are used by board files to setup the platform and setup
// processes.
pub mod procs {
    pub use process::{load_processes, FaultResponse, Process};
}

/// Main loop.
pub fn main<P: Platform, C: Chip>(
    platform: &P,
    chip: &mut C,
    processes: &'static mut [Option<&mut process::Process<'static>>],
    ipc: Option<&ipc::IPC>,
) {
    let processes = unsafe {
        process::PROCS = processes;
        &mut process::PROCS
    };

    loop {
        unsafe {
            chip.service_pending_interrupts();

            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    sched::do_process(platform, chip, process, callback::AppId::new(i), ipc);
                });
                if chip.has_pending_interrupts() {
                    break;
                }
            }

            chip.atomic(|| {
                if !chip.has_pending_interrupts() && process::processes_blocked() {
                    chip.sleep();
                }
            });
        };
    }
}
