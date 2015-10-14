#![feature(core_str_ext,core_slice_ext,const_fn,no_std,raw,core_char_ext,unique,slice_bytes)]
#![no_main]
#![no_std]

extern crate common;
extern crate support;
extern crate hil;
extern crate process;
extern crate platform;

mod apps;
mod sched;

pub mod syscall;

#[no_mangle]
pub extern fn main() {
    use process::Process;
    use process::AppId;

    let mut platform = unsafe {
        platform::init()
    };

    let app1 = unsafe { Process::create(apps::app::_start).unwrap() };

    let processes = unsafe {
        process::process::PROCS = [Some(app1)];
        &mut process::process::PROCS
    };

    loop {
        unsafe {
            platform.service_pending_interrupts();

            for (i, p) in processes.iter_mut().enumerate() {
                p.as_mut().map(|process| {
                    sched::do_process(platform, process, AppId::new(i));
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

