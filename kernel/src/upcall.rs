//! Data structure for storing an upcall from the kernel to a process.

use core::ptr::NonNull;

use crate::config;
use crate::debug;
use crate::process;
use crate::process::ProcessId;
use crate::syscall::SyscallReturn;
use crate::ErrorCode;

/// Type to uniquely identify an upcall subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct UpcallId {
    pub driver_num: usize,
    pub subscribe_num: usize, //TODO: use u32?
}

/// Type for calling an upcall in a process.
///
/// This is essentially a wrapper around a function pointer with
/// associated process data.
pub struct Upcall {
    /// The ProcessId of the process that issued this upcall
    pub(crate) app_id: ProcessId,

    /// A unique identifier of this particular upcall, representing the
    /// driver_num and subdriver_num used to submit it.
    pub(crate) upcall_id: UpcallId,

    /// The application data passed by the app when `subscribe()` was called
    pub(crate) appdata: usize,

    /// A pointer to the first instruction of a function in the app
    /// associated with app_id.
    /// If this value is `None`, this is a null upcall, which cannot actually be
    /// scheduled. An `Upcall` can be null when it is first created,
    /// or after an app unsubscribes from an upcall.
    pub(crate) fn_ptr: Option<NonNull<()>>,
}

impl Upcall {
    pub(crate) fn new(
        app_id: ProcessId,
        upcall_id: UpcallId,
        appdata: usize,
        fn_ptr: Option<NonNull<()>>,
    ) -> Upcall {
        Upcall {
            app_id,
            upcall_id,
            appdata,
            fn_ptr,
        }
    }

    /// Schedule the upcall
    ///
    /// This will queue the [`Upcall`] for the associated process. It
    /// returns `false` if the queue for the process is full and the
    /// upcall could not be scheduled or this is a null upcall.
    ///
    /// The arguments (`r0-r2`) are the values passed back to the process and
    /// are specific to the individual `Driver` interfaces.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        let res = self.fn_ptr.map_or(false, |fp| {
            self.app_id
                .kernel
                .process_map_or(false, self.app_id, |process| {
                    process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                        source: process::FunctionCallSource::Driver(self.upcall_id),
                        argument0: r0,
                        argument1: r1,
                        argument2: r2,
                        argument3: self.appdata,
                        pc: fp.as_ptr() as usize,
                    }))
                })
        });

        if config::CONFIG.trace_syscalls {
            debug!(
                "[{:?}] schedule[{:#x}:{}] @{:#x}({:#x}, {:#x}, {:#x}, {:#x}) = {}",
                self.app_id,
                self.upcall_id.driver_num,
                self.upcall_id.subscribe_num,
                self.fn_ptr.map_or(0x0 as *mut (), |fp| fp.as_ptr()) as usize,
                r0,
                r1,
                r2,
                self.appdata,
                res
            );
        }
        res
    }

    pub(crate) fn into_subscribe_success(self) -> SyscallReturn {
        match self.fn_ptr {
            None => SyscallReturn::SubscribeSuccess(0 as *mut (), self.appdata),
            Some(fp) => SyscallReturn::SubscribeSuccess(fp.as_ptr(), self.appdata),
        }
    }

    pub(crate) fn into_subscribe_failure(self, err: ErrorCode) -> SyscallReturn {
        match self.fn_ptr {
            None => SyscallReturn::SubscribeFailure(err, 0 as *mut (), self.appdata),
            Some(fp) => SyscallReturn::SubscribeFailure(err, fp.as_ptr(), self.appdata),
        }
    }
}
