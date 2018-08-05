//! Data structure for storing a callback to userspace or kernelspace.

use core::fmt;
use core::ptr::NonNull;

use process;
use sched::Kernel;

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

    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        self.kernel.process_map_or((0, 0), self.idx, |process| {
            let start = process.flash_non_protected_start() as usize;
            let end = process.flash_end() as usize;
            (start, end)
        })
    }
}

/// Wrapper around a function pointer.
#[derive(Clone, Copy)]
pub struct Callback {
    app_id: AppId,
    appdata: usize,
    fn_ptr: NonNull<*mut ()>,
}

impl Callback {
    crate fn new(appid: AppId, appdata: usize, fn_ptr: NonNull<*mut ()>) -> Callback {
        Callback {
            app_id: appid,
            appdata: appdata,
            fn_ptr: fn_ptr,
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.app_id
            .kernel
            .process_map_or(false, self.app_id.idx(), |process| {
                process.schedule(process::FunctionCall {
                    r0: r0,
                    r1: r1,
                    r2: r2,
                    r3: self.appdata,
                    pc: self.fn_ptr.as_ptr() as usize,
                })
            })
    }
}
