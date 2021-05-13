//! Data structure for storing an upcall from the kernel to a process.

use core::ptr::NonNull;

use crate::config;
use crate::debug;
use crate::process;
use crate::ProcessId;

/// Type to uniquely identify an upcall subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct UpcallId {
    pub driver_num: u32,
    pub subscribe_num: u32,
}

/// Type for calling an upcall in a process.
///
/// This is essentially a wrapper around a function pointer with
/// associated process data.
pub struct Upcall {
    pub(crate) app_id: ProcessId,
    pub(crate) upcall_id: UpcallId,
    pub(crate) appdata: usize,
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
}

/// Factory for creating process-bound null-[`Upcall`] instances
///
/// Passing an instance of this struct to a capsule after process
/// initialization allows the capsule to exchange the default
/// [`Upcall`] instances (for example in it's
/// [`Grant`](crate::Grant) regions) for process-bound instances,
/// which are legal to be returned as part of a call to
/// [`subscribe`](crate::Driver::subscribe).
///
/// This struct can guarantee that per process and driver, at most one
/// [`Upcall`] for each `subscribe_num` (`subdriver_num`)
/// exists. This has the implication of requiring the requested
/// `subscribe_num` Upcalls to be strictly increasing.
pub struct ProcessUpcallFactory {
    process: ProcessId,
    driver_num: u32,
    next_subscribe_num: u32,
}

impl ProcessUpcallFactory {
    pub(crate) fn new(process: ProcessId, driver_num: u32) -> Self {
        ProcessUpcallFactory {
            process,
            driver_num,
            next_subscribe_num: 0,
        }
    }

    pub fn build_upcall(&mut self, subscribe_num: u32) -> Option<Upcall> {
        if subscribe_num >= self.next_subscribe_num {
            self.next_subscribe_num = subscribe_num + 1;

            let upcall_id = UpcallId {
                driver_num: self.driver_num,
                subscribe_num,
            };

            Some(Upcall::new(
                self.process,
                upcall_id,
                0,    // Default appdata value
                None, // No fnptr, this is a null-callback
            ))
        } else {
            None
        }
    }
}
