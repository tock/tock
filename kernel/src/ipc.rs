//! Provide capsule driver for controlling buttons on a board.  This allows for much more cross
//! platform controlling of buttons without having to know which of the GPIO pins exposed across
//! the syscall interface are buttons.

use {AppId, AppSlice, Container, Callback, Driver, Shared};
use process;
use returncode::ReturnCode;

struct IPCData {
    shared_memory: [Option<AppSlice<Shared, u8>>; 8],
    client_callbacks: [Option<Callback>; 8],
    callback: Option<Callback>,
}

impl Default for IPCData {
    fn default() -> IPCData {
        IPCData {
            shared_memory: [None, None, None, None, None, None, None, None],
            client_callbacks: [None, None, None, None, None, None, None, None],
            callback: None,
        }
    }
}

pub struct IPC {
    data: Container<IPCData>,
}

impl IPC {
    pub unsafe fn new() -> IPC {
        IPC { data: Container::create() }
    }

    pub unsafe fn schedule_callback(&self,
                                    appid: AppId,
                                    otherapp: AppId,
                                    cb_type: process::IPCType) {
        self.data
            .enter(appid, |mydata, _| {
                let callback = match cb_type {
                    process::IPCType::Service => mydata.callback,
                    process::IPCType::Client => {
                        *mydata.client_callbacks.get(otherapp.idx()).unwrap_or(&None)
                    }
                };
                callback.map(|mut callback| {
                        self.data
                            .enter(otherapp, |otherdata, _| {
                                if appid.idx() >= otherdata.shared_memory.len() {
                                    return;
                                }
                                match otherdata.shared_memory[appid.idx()] {
                                    Some(ref slice) => {
                                        slice.expose_to(appid);
                                        callback.schedule(otherapp.idx() + 1,
                                                          slice.len(),
                                                          slice.ptr() as usize);
                                    }
                                    None => {
                                        callback.schedule(appid.idx() + 1, 0, 0);
                                    }
                                }
                            })
                            .unwrap_or(());
                    })
                    .unwrap_or(());
            })
            .unwrap_or(());
    }
}

impl Driver for IPC {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 /* Service callback */ => {
                self.data.enter(callback.app_id(), |data, _| {
                    data.callback = Some(callback);
                    ReturnCode::SUCCESS
                }).unwrap_or(ReturnCode::EBUSY)
            }
            svc_id /* Client callback */ => {
                if svc_id - 1 >= 8 {
                    ReturnCode::EINVAL /* Maximum of 8 IPC's exceeded */
                } else {
                    self.data.enter(callback.app_id(), |data, _| {
                        data.client_callbacks[svc_id - 1] = Some(callback);
                        ReturnCode::SUCCESS
                    }).unwrap_or(ReturnCode::EBUSY)
                }
            }
        }
    }

    fn command(&self, target_id: usize, client_or_svc: usize, appid: AppId) -> ReturnCode {
        let procs = unsafe { &mut process::PROCS };
        if target_id == 0 || target_id > procs.len() {
            return ReturnCode::EINVAL; /* Request to IPC to impossible process */
        }

        let cb_type = if client_or_svc == 0 {
            process::IPCType::Service
        } else {
            process::IPCType::Client
        };

        procs[target_id - 1]
            .as_mut()
            .map(|target| {
                target.schedule_ipc(appid, cb_type);
                ReturnCode::SUCCESS
            })
            .unwrap_or(ReturnCode::EINVAL) /* Request to IPC to unknown process */
    }

    fn allow(&self, appid: AppId, target_id: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        if target_id == 0 {
            if slice.len() > 0 {
                let procs = unsafe { &mut process::PROCS };
                for (i, process) in procs.iter().enumerate() {
                    match process {
                        &Some(ref p) => {
                            let s = p.package_name.as_bytes();
                            // are slices equal?
                            if s.len() == slice.len() &&
                               s.iter()
                                .zip(slice.iter())
                                .all(|(c1, c2)| c1 == c2) {
                                return ReturnCode::SuccessWithValue { value: (i as usize) + 1 };
                            }
                        }
                        &None => {}
                    }
                }
            }
            return ReturnCode::EINVAL; /* AppSlice must have non-zero length */
        }
        return self.data
            .enter(appid, |data, _| {
                data.shared_memory
                    .get_mut(target_id - 1)
                    .map(|smem| {
                        *smem = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or(ReturnCode::EINVAL) /* Target process does not exist */
            })
            .unwrap_or(ReturnCode::EBUSY);
    }
}
