#![crate_name = "main"]
#![crate_type = "rlib"]
#![feature(const_fn)]
#![no_std]

extern crate common;
extern crate support;
extern crate hil;
extern crate process;

mod sched;

mod syscall;
mod platform;

pub use platform::{Chip, MPU, Platform, SysTick};

extern {
    /// Beginning of the ROM region containing app images.
    static _sapps : u8;
}

pub fn main<P: Platform, C: Chip>(platform: &mut P, chip: &mut C) {
    use process::AppId;

    let processes = unsafe {
        process::process::load_processes(&_sapps)
    };

    loop {
        unsafe {
            chip.service_pending_interrupts();

            let mut running_left = false;
            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    sched::do_process(platform, chip, process, AppId::new(i));
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

