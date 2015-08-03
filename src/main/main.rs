#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate common;
extern crate support;
extern crate hil;
extern crate process;
extern crate platform;

mod apps;

pub mod syscall;

#[no_mangle]
pub extern fn main() {
    use core::prelude::*;
    use process::Process;
    use process::AppSlice;
    use common::{Shared,Queue};

    let mut platform = unsafe {
        platform::init()
    };

    let app1 = unsafe { Process::create(apps::app1::_start).unwrap() };

    let mut processes = [Shared::new(app1)];

    loop {
        unsafe {
            platform.service_pending_interrupts();

            'sched: for process_s in processes.iter_mut() {
                let process = process_s.borrow_mut();
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
                            let driver_num = process.r0();
                            let subdriver_num = process.r1();
                            let callback_ptr = process.r2() as *mut ();

                            let res = platform.with_driver(driver_num, |driver| {
                                let callback =
                                    hil::Callback::new(process_s.borrow_mut(),
                                                       callback_ptr);
                                match driver {
                                    Some(d) => d.subscribe(subdriver_num,
                                                           callback),
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
                        Some(syscall::ALLOW) => {
                            let process_ptr = process as *mut Process<'static> as *mut ();
                            let res = platform.with_driver(process.r0(), |driver| {
                                match driver {
                                    Some(d) => {
                                        let start_addr = process.r2() as *mut u8;
                                        let size = process.r3();
                                        if process.in_exposed_bounds(start_addr, size) {
                                            let slice = AppSlice::new(start_addr as *mut u8, size, process_ptr);
                                            d.allow(process.r1(), slice)
                                        } else {
                                            -1
                                        }
                                    },
                                    None => -1
                                }
                            });
                            process.set_r0(res);
                        }
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

