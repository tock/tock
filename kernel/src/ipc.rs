//! Inter-process communication mechanism for Tock.
//!
//! This is a special syscall driver that allows userspace applications to
//! share memory.

/// Syscall number
pub const DRIVER_NUM: usize = 0x00010000;

use callback::{AppId, Callback};
use capabilities::MemoryAllocationCapability;
use driver::Driver;
use grant::Grant;
use mem::{AppSlice, Shared};
use process;
use returncode::ReturnCode;
use sched::Kernel;

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
    data: Grant<IPCData>,
}

impl IPC {
    pub fn new(kernel: &'static Kernel, capability: &MemoryAllocationCapability) -> IPC {
        IPC {
            data: kernel.create_grant(capability),
        }
    }

    pub unsafe fn schedule_callback(
        &self,
        appid: AppId,
        otherapp: AppId,
        cb_type: process::IPCType,
    ) {
        self.data
            .enter(appid, |mydata, _| {
                let callback = match cb_type {
                    process::IPCType::Service => mydata.callback,
                    process::IPCType::Client => {
                        *mydata.client_callbacks.get(otherapp.idx()).unwrap_or(&None)
                    }
                };
                callback
                    .map(|mut callback| {
                        self.data
                            .enter(otherapp, |otherdata, _| {
                                if appid.idx() >= otherdata.shared_memory.len() {
                                    return;
                                }
                                match otherdata.shared_memory[appid.idx()] {
                                    Some(ref slice) => {
                                        slice.expose_to(appid);
                                        callback.schedule(
                                            otherapp.idx() + 1,
                                            slice.len(),
                                            slice.ptr() as usize,
                                        );
                                    }
                                    None => {
                                        callback.schedule(otherapp.idx() + 1, 0, 0);
                                    }
                                }
                            }).unwrap_or(());
                    }).unwrap_or(());
            }).unwrap_or(());
    }
}

impl Driver for IPC {
    /// subscribe enables processes using IPC to register callbacks that fire
    /// when notify() is called.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe(0)
            //
            // Subscribe with subscribe_num == 0 is how a process registers
            // itself as an IPC service. Each process can only register as a
            // single IPC service. The identifier for the IPC service is the
            // application name stored in the TBF header of the application.
            // The callback that is passed to subscribe is called when another
            // process notifies the server process.
            0 => self
                .data
                .enter(app_id, |data, _| {
                    data.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or(ReturnCode::EBUSY),

            // subscribe(>=1)
            //
            // Subscribe with subscribe_num >= 1 is how a client registers
            // a callback for a given service. The service number (passed
            // here as subscribe_num) is returned from the allow() call.
            // Once subscribed, the client will receive callbacks when the
            // service process calls notify_client().
            svc_id => {
                if svc_id - 1 >= 8 {
                    ReturnCode::EINVAL /* Maximum of 8 IPC's exceeded */
                } else {
                    self.data
                        .enter(app_id, |data, _| {
                            data.client_callbacks[svc_id - 1] = callback;
                            ReturnCode::SUCCESS
                        }).unwrap_or(ReturnCode::EBUSY)
                }
            }
        }
    }

    /// command is how notify() is implemented.
    /// Notifying an IPC service is done by setting client_or_svc to 0,
    /// and notifying an IPC client is done by setting client_or_svc to 1.
    /// In either case, the target_id is the same number as provided in a notify
    /// callback or as returned by allow.
    ///
    /// Returns EINVAL if the other process doesn't exist.
    fn command(
        &self,
        target_id: usize,
        client_or_svc: usize,
        _: usize,
        appid: AppId,
    ) -> ReturnCode {
        let cb_type = if client_or_svc == 0 {
            process::IPCType::Service
        } else {
            process::IPCType::Client
        };

        self.data
            .kernel
            .process_map_or(ReturnCode::EINVAL, target_id - 1, |target| {
                let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                match ret {
                    true => ReturnCode::SUCCESS,
                    false => ReturnCode::FAIL,
                }
            })
    }

    /// allow enables processes to discover IPC services on the platform or
    /// share buffers with existing services.
    ///
    /// If allow is called with target_id == 0, it is an IPC service discover
    /// call. The contents of the slice should be the string name of the IPC
    /// service. If this mechanism can find that service, allow will return
    /// an ID that can be used to notify that service. Otherwise an error will
    /// be returned.
    ///
    /// If allow is called with target_id >= 1, it is a share command where the
    /// application is explicitly sharing a slice with an IPC service (as
    /// specified by the target_id). allow() simply allows both processes to
    /// access the buffer, it does not signal the service.
    fn allow(
        &self,
        appid: AppId,
        target_id: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        if target_id == 0 {
            match slice {
                Some(slice_data) => {
                    let ret = self.data.kernel.process_until(|p| {
                        let s = p.get_process_name().as_bytes();
                        // are slices equal?
                        if s.len() == slice_data.len()
                            && s.iter().zip(slice_data.iter()).all(|(c1, c2)| c1 == c2)
                        {
                            ReturnCode::SuccessWithValue {
                                value: (p.appid().idx() as usize) + 1,
                            }
                        } else {
                            ReturnCode::FAIL
                        }
                    });
                    if ret != ReturnCode::FAIL {
                        return ret;
                    }
                }
                None => {}
            }

            return ReturnCode::EINVAL; /* AppSlice must have non-zero length */
        }
        return self
            .data
            .enter(appid, |data, _| {
                data.shared_memory
                    .get_mut(target_id - 1)
                    .map(|smem| {
                        *smem = slice;
                        ReturnCode::SUCCESS
                    }).unwrap_or(ReturnCode::EINVAL) /* Target process does not exist */
            }).unwrap_or(ReturnCode::EBUSY);
    }
}
