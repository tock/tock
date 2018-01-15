//! Data structure for storing a callback to userspace or kernelspace.

use core::nonzero::NonZero;
use process;

/// Userspace app identifier.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AppId {
    idx: usize,
}

/// The kernel can masquerade as an app. IDs >= this value are the kernel.
/// These IDs are used to identify which kernel container is being accessed.
const KERNEL_APPID_BOUNDARY: usize = 100;

impl AppId {
    pub fn new(idx: usize) -> AppId {
        AppId { idx: idx }
    }

    pub const fn kernel_new(idx: usize) -> AppId {
        AppId { idx: idx }
    }

    pub const fn is_kernel(self) -> bool {
        self.idx >= KERNEL_APPID_BOUNDARY
    }

    pub const fn is_kernel_idx(idx: usize) -> bool {
        idx >= KERNEL_APPID_BOUNDARY
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        process::get_editable_flash_range(self.idx)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RustOrRawFnPtr {
    Raw {
        ptr: NonZero<*mut ()>,
    },
    Rust {
        func: fn(usize, usize, usize, usize),
    },
}

/// Wrapper around a function pointer.
#[derive(Clone, Copy, Debug)]
pub struct Callback {
    app_id: AppId,
    appdata: usize,
    fn_ptr: RustOrRawFnPtr,
}

impl Callback {
    pub fn new(appid: AppId, appdata: usize, fn_ptr: NonZero<*mut ()>) -> Callback {
        Callback {
            app_id: appid,
            appdata: appdata,
            fn_ptr: RustOrRawFnPtr::Raw { ptr: fn_ptr },
        }
    }

    pub const fn kernel_new(appid: AppId, fn_ptr: fn(usize, usize, usize, usize)) -> Callback {
        Callback {
            app_id: appid,
            appdata: 0,
            fn_ptr: RustOrRawFnPtr::Rust { func: fn_ptr },
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        if self.app_id.is_kernel() {
            let fn_ptr = match self.fn_ptr {
                RustOrRawFnPtr::Raw { ptr } => {
                    panic!("Attempt to rust_call a raw function pointer: ptr {:?}", ptr)
                }
                RustOrRawFnPtr::Rust { func } => func,
            };
            fn_ptr(r0, r1, r2, self.appdata);
            true
        } else {
            let fn_ptr = match self.fn_ptr {
                RustOrRawFnPtr::Raw { ptr } => ptr,
                RustOrRawFnPtr::Rust { func } => {
                    panic!("Attempt to schedule rust function: func {:?}", func)
                }
            };
            process::schedule(
                process::FunctionCall {
                    r0: r0,
                    r1: r1,
                    r2: r2,
                    r3: self.appdata,
                    pc: fn_ptr.get() as usize,
                },
                self.app_id,
            )
        }
    }

    pub fn app_id(&self) -> AppId {
        self.app_id
    }
}
