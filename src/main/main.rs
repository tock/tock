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
    use process::Process;
    use process::AppId;

    let processes = unsafe {
        process::process::PROCS = [Process::create(&_sapps)];
        &mut process::process::PROCS
    };

    let mut platform = unsafe {
        platform::init()
    };

    loop {
        unsafe {
            platform.service_pending_interrupts();

            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    // Reserve 0 to mean "kernel", apps start at 1
                    sched::do_process(platform, process, AppId::new(i+1));
                });
            }

            support::atomic(|| {
                if !platform.has_pending_interrupts() {
                    support::wfi();
                }
            })
        };
    }
}

