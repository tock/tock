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

pub mod callback;
pub mod grant;
#[macro_use]
pub mod debug;
pub mod driver;
pub mod hil;
pub mod ipc;
pub mod mem;
pub mod memop;
pub mod returncode;
pub mod component;

// Work around https://github.com/rust-lang-nursery/rustfmt/issues/6
// It's a little sad that we have to skip the whole module, but that's
// better than the unmaintainable pile 'o strings IMO
#[cfg_attr(rustfmt, rustfmt_skip)]

mod sched;

mod platform;
mod syscall;

pub use callback::{AppId, Callback};
pub use common::StaticRef;
pub use driver::Driver;
pub use grant::Grant;
pub use mem::{AppPtr, AppSlice, Private, Shared};
pub use platform::systick::SysTick;
pub use platform::{mpu, systick, Chip, Platform};
pub use platform::{ClockInterface, NoClockControl, NO_CLOCK_CONTROL};
pub use process::{Process, State};
pub use returncode::ReturnCode;

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
                    sched::do_process(platform, chip, process, AppId::new(i), ipc);
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
