use core::nonzero::NonZero;
use process;

#[derive(Clone, Copy, Debug)]
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

    pub const fn is_kernel(appid: AppId) -> bool {
        appid.idx >= KERNEL_APPID_BOUNDARY
    }

    pub const fn is_kernel_idx(appid: usize) -> bool {
        appid >= KERNEL_APPID_BOUNDARY
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Callback {
    pub app_id: AppId,
    pub appdata: usize,
    pub fn_ptr: NonZero<*mut ()>,
}

impl Callback {
    pub fn new(appid: AppId, appdata: usize, fn_ptr: NonZero<*mut ()>) -> Callback {
        Callback {
            app_id: appid,
            appdata: appdata,
            fn_ptr: fn_ptr,
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        if AppId::is_kernel(self.app_id) {
            unimplemented!();
        } else {
            process::schedule(process::FunctionCall {
                                  r0: r0,
                                  r1: r1,
                                  r2: r2,
                                  r3: self.appdata,
                                  pc: *self.fn_ptr as usize,
                              },
                              self.app_id)
        }
    }

    pub fn app_id(&self) -> AppId {
        self.app_id
    }
}
