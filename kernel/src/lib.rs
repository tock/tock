#![feature(asm,core_intrinsics,unique,nonzero,const_fn,lang_items)]
#![no_std]

pub mod common;

pub mod callback;
pub mod container;
pub mod driver;
pub mod mem;
pub mod process;
pub mod hil;

pub mod support;

mod sched;

mod syscall;
mod platform;

pub use callback::{AppId, Callback};
pub use container::Container;
pub use driver::Driver;
pub use mem::{AppSlice, AppPtr, Private, Shared};
pub use platform::{Chip, MPU, Platform, SysTick};
pub use process::{Process, State};

pub fn main<P: Platform, C: Chip>(platform: &mut P,
                                  chip: &mut C,
                                  processes: &'static mut [Option<process::Process<'static>>]) {
    let processes = unsafe {
        process::PROCS = processes;
        &mut process::PROCS
    };

    loop {
        unsafe {
            chip.service_pending_interrupts();

            let mut running_left = false;
            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    sched::do_process(platform, chip, process, AppId::new(i), &mut running_left);
                    if process.state == process::State::Running {
                        running_left = true;
                    }
                });
                if chip.has_pending_interrupts() {
                    break;
                }
            }

            support::atomic(|| {
                if !chip.has_pending_interrupts() && !running_left {
                    support::wfi();
                }
            })
        };
    }
}
