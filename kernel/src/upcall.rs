//! Data structure for storing an upcall from the kernel to a process.

use core::ptr::NonNull;

use crate::config;
use crate::debug;
use crate::process;
use crate::process::AppId;
use crate::syscall::SyscallReturn;
use crate::ErrorCode;

/// Type to uniquely identify an upcall subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct UpcallId {
    pub driver_num: usize,
    pub subscribe_num: usize,
}

/// Type for calling an upcall in a process.
///
/// This is essentially a wrapper around a function pointer.
#[derive(Clone, Copy)]
struct ProcessUpcall {
    app_id: AppId,
    upcall_id: UpcallId,
    appdata: usize,
    fn_ptr: NonNull<*mut ()>,
}

#[derive(Clone, Copy, Default)]
pub struct Upcall {
    cb: Option<ProcessUpcall>,
}

impl Upcall {
    pub(crate) fn new(
        app_id: AppId,
        upcall_id: UpcallId,
        appdata: usize,
        fn_ptr: NonNull<*mut ()>,
    ) -> Upcall {
        Upcall {
            cb: Some(ProcessUpcall::new(app_id, upcall_id, appdata, fn_ptr)),
        }
    }

    /// Tell the scheduler to run this upcall for the process.
    ///
    /// The three arguments are passed to the upcall in userspace.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.cb.map_or(true, |mut cb| cb.schedule(r0, r1, r2))
    }

    pub(crate) fn into_subscribe_success(self) -> SyscallReturn {
        match self.cb {
            None => SyscallReturn::SubscribeSuccess(0 as *mut u8, 0),
            Some(cb) => {
                SyscallReturn::SubscribeSuccess(cb.fn_ptr.as_ptr() as *const u8, cb.appdata)
            }
        }
    }

    pub(crate) fn into_subscribe_failure(self, err: ErrorCode) -> SyscallReturn {
        match self.cb {
            None => SyscallReturn::SubscribeFailure(err, 0 as *mut u8, 0),
            Some(cb) => {
                SyscallReturn::SubscribeFailure(err, cb.fn_ptr.as_ptr() as *const u8, cb.appdata)
            }
        }
    }
}

impl ProcessUpcall {
    fn new(
        app_id: AppId,
        upcall_id: UpcallId,
        appdata: usize,
        fn_ptr: NonNull<*mut ()>,
    ) -> ProcessUpcall {
        ProcessUpcall {
            app_id,
            upcall_id,
            appdata,
            fn_ptr,
        }
    }
}

impl ProcessUpcall {
    /// Actually trigger the upcall.
    ///
    /// This will queue the `Upcall` for the associated process. It returns
    /// `false` if the queue for the process is full and the upcall could not
    /// be scheduled.
    ///
    /// The arguments (`r0-r2`) are the values passed back to the process and
    /// are specific to the individual `Driver` interfaces.
    fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        let res = self
            .app_id
            .kernel
            .process_map_or(false, self.app_id, |process| {
                process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                    source: process::FunctionCallSource::Driver(self.upcall_id),
                    argument0: r0,
                    argument1: r1,
                    argument2: r2,
                    argument3: self.appdata,
                    pc: self.fn_ptr.as_ptr() as usize,
                }))
            });
        if config::CONFIG.trace_syscalls {
            debug!(
                "[{:?}] schedule[{:#x}:{}] @{:#x}({:#x}, {:#x}, {:#x}, {:#x}) = {}",
                self.app_id,
                self.upcall_id.driver_num,
                self.upcall_id.subscribe_num,
                self.fn_ptr.as_ptr() as usize,
                r0,
                r1,
                r2,
                self.appdata,
                res
            );
        }
        res
    }
}
