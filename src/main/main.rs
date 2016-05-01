#![feature(const_fn)]
#![no_main]
#![no_std]

extern crate common;
extern crate support;
extern crate hil;
extern crate process;
extern crate platform;

mod sched;

pub mod syscall;

#[allow(improper_ctypes)]
extern {
    static _sapps : usize;
}

#[no_mangle]
pub extern fn main() {
    use process::AppId;

    let mut platform = unsafe {
        platform::init()
    };


    let processes = unsafe {
        process::process::load_processes(&_sapps)
    };

    loop {
        unsafe {
            platform.service_pending_interrupts();

            let mut running_left = false;
            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    sched::do_process(platform, process, AppId::new(i));
                    if process.state == process::State::Running {
                        running_left = true;
                    }
                });
                if platform.has_pending_interrupts() {
                    break;
                }
            }

            support::atomic(|| {
                if !platform.has_pending_interrupts() && !running_left {
                    support::wfi();
                }
            })
        };
    }
}

