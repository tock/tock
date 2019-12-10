//! Data structure for storing a callback to userspace or kernelspace.

use core::fmt;
use core::ptr::NonNull;

use crate::config;
use crate::debug;
use crate::process;
use crate::sched::Kernel;

/// Userspace app identifier.
#[derive(Clone, Copy)]
pub struct AppId {
    crate kernel: &'static Kernel,
    idx: usize,
}

impl PartialEq for AppId {
    fn eq(&self, other: &AppId) -> bool {
        self.idx == other.idx
    }
}

impl Eq for AppId {}

impl fmt::Debug for AppId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.idx)
    }
}

impl AppId {
    crate fn new(kernel: &'static Kernel, idx: usize) -> AppId {
        AppId {
            kernel: kernel,
            idx: idx,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Returns the full address of the start and end of the flash region that
    /// the app owns and can write to. This includes the app's code and data and
    /// any padding at the end of the app. It does not include the TBF header,
    /// or any space that the kernel is using for any potential bookkeeping.
    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        self.kernel.process_map_or((0, 0), self.idx, |process| {
            let start = process.flash_non_protected_start() as usize;
            let end = process.flash_end() as usize;
            (start, end)
        })
    }
}

/// Type to uniquely identify a callback subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CallbackId {
    pub driver_num: usize,
    pub subscribe_num: usize,
}

/// Type for calling a callback in a process.
///
/// This is essentially a wrapper around a function pointer.
#[derive(Clone, Copy)]
pub struct Callback {
    app_id: AppId,
    callback_id: CallbackId,
    appdata: usize,
    fn_ptr: NonNull<*mut ()>,
}

impl Callback {
    crate fn new(
        app_id: AppId,
        callback_id: CallbackId,
        appdata: usize,
        fn_ptr: NonNull<*mut ()>,
    ) -> Callback {
        Callback {
            app_id,
            callback_id,
            appdata,
            fn_ptr,
        }
    }

    /// Actually trigger the callback.
    ///
    /// This will queue the `Callback` for the associated process. It returns
    /// `false` if the queue for the process is full and the callback could not
    /// be scheduled.
    ///
    /// The arguments (`r0-r2`) are the values passed back to the process and
    /// are specific to the individual `Driver` interfaces.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        let res = self
            .app_id
            .kernel
            .process_map_or(false, self.app_id.idx(), |process| {
                process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                    source: process::FunctionCallSource::Driver(self.callback_id),
                    argument0: r0,
                    argument1: r1,
                    argument2: r2,
                    argument3: self.appdata,
                    pc: self.fn_ptr.as_ptr() as usize,
                }))
            });
        if config::CONFIG.trace_syscalls {
            debug!(
                "[{}] schedule[{:x}:{:x}] @{:x}({:x}, {:x}, {:x}, {:x}) = {}",
                self.app_id.idx(),
                self.callback_id.driver_num,
                self.callback_id.subscribe_num,
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
