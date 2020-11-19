//! Inter-process communication mechanism for Tock.
//!
//! This is a special syscall driver that allows userspace applications to
//! share memory.

use crate::callback::{AppId, Callback};
use crate::capabilities::MemoryAllocationCapability;
use crate::driver::LegacyDriver;
use crate::grant::Grant;
use crate::mem::{AppSlice, SharedReadWrite};
use crate::process;
use crate::returncode::ReturnCode;
use crate::sched::Kernel;

/// Syscall number
pub const DRIVER_NUM: usize = 0x10000;

/// Enum to mark which type of callback is scheduled for the IPC mechanism.
#[derive(Copy, Clone, Debug)]
pub enum IPCCallbackType {
    /// Indicates that the callback is for the service callback handler this
    /// process has setup.
    Service,
    /// Indicates that the callback is from a different service app and will
    /// call one of the client callbacks setup by this process.
    Client,
}

/// State that is stored in each process's grant region to support IPC.
#[derive(Default)]
struct IPCData {
    /// An array of app slices that this application has shared with other
    /// applications.
    shared_memory: [Option<AppSlice<SharedReadWrite, u8>>; 8],
    /// An array of callbacks this process has registered to receive callbacks
    /// from other services.
    client_callbacks: [Option<Callback>; 8],
    /// The callback setup by a service. Each process can only be one service.
    callback: Option<Callback>,
}

/// The IPC mechanism struct.
pub struct IPC {
    /// The grant regions for each process that holds the per-process IPC data.
    data: Grant<IPCData>,
}

impl IPC {
    pub fn new(kernel: &'static Kernel, capability: &dyn MemoryAllocationCapability) -> IPC {
        IPC {
            data: kernel.create_grant(capability),
        }
    }

    /// Schedule an IPC callback for a process. This is called by the main
    /// scheduler loop if an IPC task was queued for the process.
    pub(crate) unsafe fn schedule_callback(
        &self,
        appid: AppId,
        otherapp: AppId,
        cb_type: IPCCallbackType,
    ) {
        self.data
            .enter(appid, |mydata, _| {
                let callback = match cb_type {
                    IPCCallbackType::Service => mydata.callback,
                    IPCCallbackType::Client => match otherapp.index() {
                        Some(i) => *mydata.client_callbacks.get(i).unwrap_or(&None),
                        None => None,
                    },
                };
                callback.map_or((), |mut callback| {
                    self.data
                        .enter(otherapp, |otherdata, _| {
                            // If the other app shared a buffer with us, make
                            // sure we have access to that slice and then call
                            // the callback. If no slice was shared then just
                            // call the callback.
                            match appid.index() {
                                Some(i) => {
                                    if i >= otherdata.shared_memory.len() {
                                        return;
                                    }

                                    match otherdata.shared_memory[i] {
                                        Some(ref slice) => {
                                            slice.expose_to(appid);
                                            callback.schedule(
                                                otherapp.id() + 1,
                                                slice.len(),
                                                slice.ptr() as usize,
                                            );
                                        }
                                        None => {
                                            callback.schedule(otherapp.id() + 1, 0, 0);
                                        }
                                    }
                                }
                                None => {}
                            }
                        })
                        .unwrap_or(());
                });
            })
            .unwrap_or(());
    }
}

// TODO: Write a Tock 2.0 driver implementation
impl LegacyDriver for IPC {
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
                })
                .unwrap_or(ReturnCode::EBUSY),

            // subscribe(>=1)
            //
            // Subscribe with subscribe_num >= 1 is how a client registers
            // a callback for a given service. The service number (passed
            // here as subscribe_num) is returned from the allow() call.
            // Once subscribed, the client will receive callbacks when the
            // service process calls notify_client().
            svc_id => {
                // The app passes in a number which is the app identifier of the
                // other app (shifted by one).
                let app_identifier = svc_id - 1;
                // We first have to see if that identifier corresponds to a
                // valid application by asking the kernel to do a lookup for us.
                let otherapp = self.data.kernel.lookup_app_by_identifier(app_identifier);

                self.data
                    .enter(app_id, |data, _| {
                        match otherapp.map_or(None, |oa| oa.index()) {
                            Some(i) => {
                                if i > 8 {
                                    ReturnCode::EINVAL
                                } else {
                                    data.client_callbacks[i] = callback;
                                    ReturnCode::SUCCESS
                                }
                            }
                            None => ReturnCode::EINVAL,
                        }
                    })
                    .unwrap_or(ReturnCode::EBUSY)
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
            IPCCallbackType::Service
        } else {
            IPCCallbackType::Client
        };

        let app_identifier = target_id - 1;

        self.data
            .kernel
            .lookup_app_by_identifier(app_identifier)
            .map_or(ReturnCode::EINVAL, |otherapp| {
                self.data
                    .kernel
                    .process_map_or(ReturnCode::EINVAL, otherapp, |target| {
                        let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                        match ret {
                            true => ReturnCode::SUCCESS,
                            false => ReturnCode::FAIL,
                        }
                    })
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
    fn allow_readwrite(
        &self,
        appid: AppId,
        target_id: usize,
        slice: Option<AppSlice<SharedReadWrite, u8>>,
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
                                value: (p.appid().id() as usize) + 1,
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
        self.data
            .enter(appid, |data, _| {
                // Lookup the index of the app based on the passed in
                // identifier. This also let's us check that the other app is
                // actually valid.
                let app_identifier = target_id - 1;
                let otherapp = self.data.kernel.lookup_app_by_identifier(app_identifier);

                match otherapp.map_or(None, |oa| oa.index()) {
                    Some(i) => {
                        data.shared_memory.get_mut(i).map_or(
                            ReturnCode::EINVAL, /* Target process does not exist */
                            |smem| {
                                *smem = slice;
                                ReturnCode::SUCCESS
                            },
                        )
                    }
                    None => ReturnCode::EINVAL,
                }
            })
            .unwrap_or(ReturnCode::EBUSY)
    }
}
