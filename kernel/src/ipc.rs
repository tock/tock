//! Provide capsule driver for controlling buttons on a board.  This allows for much more cross
//! platform controlling of buttons without having to know which of the GPIO pins exposed across
//! the syscall interface are buttons.

use ::{AppId, AppSlice, Container, Callback, Driver, Shared};
use ::common::queue::Queue;
use ::process;

struct IPCData {
    shared_memory: [Option<AppSlice<Shared, u8>>; 8],
    callback: Option<Callback>
}

impl Default for IPCData {
    fn default() -> IPCData {
        IPCData {
            shared_memory: [None, None, None, None, None, None, None, None],
            callback: None,
        }
    }
}

pub struct IPC {
    data: Container<IPCData>,
}

impl IPC {
    pub unsafe fn new() -> IPC {
        IPC {
            data: Container::create()
        }
    }

    pub unsafe fn schedule_callback(&self, appid: AppId, otherapp: AppId) {
        self.data.enter(appid, |mydata, _| {
            mydata.callback.map(|mut callback| {
                self.data.enter(otherapp, |otherdata , _| {
                    match otherdata.shared_memory[appid.idx()] {
                        None => { callback.schedule(appid.idx(), 0, 0) ; }
                        Some(ref slice) => {
                            slice.expose_to(appid);
                            callback.schedule(otherapp.idx(), slice.len(), slice.ptr() as usize);
                        }
                    }
                }).unwrap_or(());
            }).unwrap_or(());
        }).unwrap_or(());
    }
}

impl Driver for IPC {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 /* Service callback */ => {
                self.data.enter(callback.app_id(), |data, _| {
                    data.callback = Some(callback);
                    0
                }).unwrap_or(-2)
            },
            // default
            _ => -1,
        }
    }

    fn command(&self, target_id: usize, _: usize, appid: AppId) -> isize {
        let procs = unsafe { &mut process::PROCS };
        procs[target_id].as_mut().map(|target| {
            target.callbacks.enqueue(process::GCallback::IPCCallback(appid));
            0
        }).unwrap_or(-1)
    }

    fn allow(&self, appid: AppId, target_id: usize, slice: AppSlice<Shared, u8>) -> isize {
        self.data.enter(appid, |data, _| {
            data.shared_memory[target_id] = Some(slice);
            0
        }).unwrap_or(-2)
    }
}

