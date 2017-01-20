#![feature(asm,core_intrinsics,unique,nonzero,const_fn,lang_items)]
#![no_std]

pub mod common;

pub mod callback;
pub mod container;
pub mod driver;
pub mod ipc;
pub mod mem;
pub mod process;
pub mod returncode;
pub mod hil;

pub mod support;

mod sched;

mod syscall;
mod platform;

pub use callback::{AppId, Callback};
pub use container::Container;
pub use driver::Driver;
pub use mem::{AppSlice, AppPtr, Private, Shared};
pub use platform::{Chip, mpu, Platform, systick};
pub use platform::systick::SysTick;
pub use process::{Process, State};

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

            support::atomic(|| {
                if !chip.has_pending_interrupts() && process::processes_blocked() {
                    support::wfi();
                }
            })
        };
    }
}
