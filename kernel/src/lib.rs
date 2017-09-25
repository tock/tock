#![feature(asm,core_intrinsics,unique,nonzero)]
#![feature(const_fn,const_cell_new,const_unsafe_cell_new,lang_items)]
#![no_std]

pub mod common;

pub mod callback;
pub mod grant;
#[macro_use]
pub mod debug;
pub mod driver;
pub mod ipc;
pub mod mem;
pub mod memop;
pub mod returncode;
pub mod hil;

// Work around https://github.com/rust-lang-nursery/rustfmt/issues/6
// It's a little sad that we have to skip the whole module, but that's
// better than the unmaintainable pile 'o strings IMO
#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod process;

pub mod support;

mod sched;

mod syscall;
mod platform;

pub use callback::{AppId, Callback};
pub use driver::Driver;
pub use grant::Grant;
pub use mem::{AppSlice, AppPtr, Private, Shared};
pub use platform::{Chip, mpu, Platform, systick};
pub use platform::systick::SysTick;
pub use process::{Process, State};
pub use returncode::ReturnCode;

/// Main loop.
pub fn main<P: Platform, C: Chip>(platform: &P,
                                  chip: &mut C,
                                  processes: &'static mut [Option<process::Process<'static>>],
                                  ipc: &ipc::IPC) {
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

            support::atomic(|| if !chip.has_pending_interrupts() && process::processes_blocked() {
                chip.prepare_for_sleep();
                support::wfi();
            })
        };
    }
}
