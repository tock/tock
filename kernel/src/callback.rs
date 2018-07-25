//! Data structure for storing a callback to userspace or kernelspace.

use core::ptr::NonNull;
use process;

/// Userspace app identifier.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AppId {
    idx: usize,
}

impl AppId {
    crate fn new(idx: usize) -> AppId {
        AppId { idx: idx }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        process::get_editable_flash_range(self.idx)
    }
}

/// Wrapper around a function pointer.
#[derive(Clone, Copy, Debug)]
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
        process::schedule(
            process::FunctionCall {
                r0: r0,
                r1: r1,
                r2: r2,
                r3: self.appdata,
                pc: self.fn_ptr.as_ptr() as usize,
            },
            self.app_id,
        )
    }
}
