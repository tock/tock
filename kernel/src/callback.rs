use core::nonzero::NonZero;
use process;

#[derive(Clone,Copy)]
pub struct AppId {
    idx: usize,
}

impl AppId {
    pub fn new(idx: usize) -> AppId {
        AppId { idx: idx }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Copy)]
pub struct Callback {
    app_id: AppId,
    appdata: usize,
    fn_ptr: NonZero<*mut ()>,
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
        process::schedule(process::FunctionCall {
                              r0: r0,
                              r1: r1,
                              r2: r2,
                              r3: self.appdata,
                              pc: *self.fn_ptr as usize,
                          },
                          self.app_id)
    }

    pub fn app_id(&self) -> AppId {
        self.app_id
    }
}
