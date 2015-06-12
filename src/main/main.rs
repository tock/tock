#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate common;
extern crate support;
extern crate hil;
extern crate platform;

mod apps;

pub mod process;
pub mod syscall;

#[no_mangle]
pub extern fn main() {
    use core::prelude::*;
    use process::Process;

    let mut platform = unsafe {
        platform::init()
    };

    let app1 = unsafe { Process::create(apps::app1::_start).unwrap() };

    let mut processes = [app1];

    loop {
        unsafe {
            platform.service_pending_interrupts();

            'sched: for process in processes.iter_mut() {
                'process: loop {
                    match process.state {
                        process::State::Running => {
                            process.switch_to();
                        }
                        process::State::Waiting => {
                            match process.callbacks.dequeue() {
                                None => { continue 'sched },
                                Some(cb) => {
                                    process.state = process::State::Running;
                                    process.switch_to_callback(cb);
                                }
                            }
                        }
                    }
                    match process.svc_number() {
                        Some(syscall::WAIT) => {
                            process.state = process::State::Waiting;
                            process.pop_syscall_stack();
                            break 'process;
                        },
                        Some(syscall::SUBSCRIBE) => {
                            let res = platform.with_driver(process.r0(), |driver| {
                                match driver {
                                    Some(d) => d.subscribe(process.r1(),
                                                                process.r2()),
                                    None => -1
                                }
                            });
                            process.set_r0(res);
                        },
                        Some(syscall::COMMAND) => {
                            let res = platform.with_driver(process.r0(), |driver| {
                                match driver {
                                    Some(d) => d.command(process.r1(),
                                                         process.r2()),
                                    None => -1
                                }
                            });
                            process.set_r0(res);
                        },
                        _ => {}
                    }
                }
            }

            support::atomic(|| {
                if !platform.has_pending_interrupts() {
                    support::wfi();
                }
            })
        };
    }
}

