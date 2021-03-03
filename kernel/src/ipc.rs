//! Inter-process communication mechanism for Tock.
//!
//! This is a special syscall driver that allows userspace applications to
//! share memory.

use crate::callback::{AppId, Callback, ProcessCallbackFactory};
use crate::capabilities::MemoryAllocationCapability;
use crate::grant::{Grant, GrantDefault};
use crate::mem::Read;
use crate::process;
use crate::sched::Kernel;
use crate::{CommandReturn, Driver, ErrorCode, ReadOnlyAppSlice, ReadWriteAppSlice};

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
struct IPCData<const NUM_PROCS: usize> {
    /// An array of app slices that this application has shared with other
    /// applications.
    shared_memory: [ReadWriteAppSlice; NUM_PROCS],
    search_slice: ReadOnlyAppSlice,
    /// An array of callbacks this process has registered to receive callbacks
    /// from other services.
    client_callbacks: [Callback; NUM_PROCS],
    /// The callback setup by a service. Each process can only be one service.
    callback: Callback,
}

impl<const NUM_PROCS: usize> GrantDefault for IPCData<NUM_PROCS> {
    fn grant_default(_process_id: AppId, cb_factory: &mut ProcessCallbackFactory) -> Self {
        const DEFAULT_RW_APP_SLICE: ReadWriteAppSlice = ReadWriteAppSlice::const_default();
        unsafe {
            use core::mem::MaybeUninit;
            // need this unless we use a macro to initialize the variable length
            // array because each initial value is different.
            let mut array_hack: MaybeUninit<[Callback; NUM_PROCS]> = MaybeUninit::uninit();

            let service_cb = cb_factory.build_callback(0).unwrap();
            let mut ptr_i = array_hack.as_mut_ptr() as *mut Callback;
            for i in 0..NUM_PROCS {
                ptr_i.write(cb_factory.build_callback(i as u32 + 1).unwrap());
                ptr_i = ptr_i.add(1);
            }

            Self {
                shared_memory: [DEFAULT_RW_APP_SLICE; NUM_PROCS],
                search_slice: ReadOnlyAppSlice::default(),
                client_callbacks: array_hack.assume_init(),
                callback: service_cb,
            }
        }
    }
}

/// The IPC mechanism struct.
pub struct IPC<const NUM_PROCS: usize> {
    /// The grant regions for each process that holds the per-process IPC data.
    data: Grant<IPCData<NUM_PROCS>>,
}

impl<const NUM_PROCS: usize> IPC<NUM_PROCS> {
    pub fn new(
        kernel: &'static Kernel,
        driver_num: u32,
        capability: &dyn MemoryAllocationCapability,
    ) -> Self {
        Self {
            data: kernel.create_grant(driver_num, capability),
        }
    }

    /// Schedule an IPC callback for a process. This is called by the main
    /// scheduler loop if an IPC task was queued for the process.
    pub(crate) unsafe fn schedule_callback(
        &self,
        schedule_on: AppId,
        called_from: AppId,
        cb_type: IPCCallbackType,
    ) -> Result<(), process::Error> {
        self.data
            .enter(schedule_on, |mydata, _| {
                let mut with_callback = |f: &dyn Fn(&mut Callback)| {
                    match cb_type {
                        IPCCallbackType::Service => f(&mut mydata.callback),
                        IPCCallbackType::Client => match called_from.index() {
                            Some(i) => f(mydata.client_callbacks.get_mut(i).unwrap()),
                            None => panic!("Invalid app issued IPC request"), //TODO: return Error instead
                        },
                    };
                };

                self.data.enter(called_from, |called_from_data, _| {
                    // If the other app shared a buffer with us, make
                    // sure we have access to that slice and then call
                    // the callback. If no slice was shared then just
                    // call the callback.
                    match schedule_on.index() {
                        Some(i) => {
                            if i >= called_from_data.shared_memory.len() {
                                return;
                            }

                            match called_from_data.shared_memory.get(i) {
                                Some(slice) => {
                                    self.data
                                        .kernel
                                        .process_map_or(None, schedule_on, |process| {
                                            process.add_mpu_region(
                                                slice.ptr(),
                                                slice.len(),
                                                slice.len(),
                                            )
                                        });

                                    with_callback(&|cb: &mut Callback| {
                                        cb.schedule(
                                            called_from.id() + 1,
                                            crate::mem::Read::len(slice),
                                            crate::mem::Read::ptr(slice) as usize,
                                        );
                                    });
                                }
                                None => {
                                    with_callback(&|cb: &mut Callback| {
                                        cb.schedule(called_from.id() + 1, 0, 0);
                                    });
                                }
                            }
                        }
                        None => {}
                    }
                })
            })
            .and_then(|x| x)
    }
}

impl<const NUM_PROCS: usize> Driver for IPC<NUM_PROCS> {
    /// subscribe enables processes using IPC to register callbacks that fire
    /// when notify() is called.
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            // subscribe(0)
            //
            // Subscribe with subscribe_num == 0 is how a process registers
            // itself as an IPC service. Each process can only register as a
            // single IPC service. The identifier for the IPC service is the
            // application name stored in the TBF header of the application.
            // The callback that is passed to subscribe is called when another
            // process notifies the server process.
            0 => {
                let res = self
                    .data
                    .enter(app_id, |data, _| {
                        core::mem::swap(&mut data.callback, &mut callback);
                    })
                    .map_err(Into::into);

                match res {
                    Ok(()) => Ok(callback),
                    Err(e) => Err((callback, e)),
                }
            }

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

                // This type annotation is here for documentation, it's not actually necessary
                let result: Result<Result<(), ErrorCode>, process::Error> =
                    self.data.enter(app_id, |data, _| {
                        match otherapp.map_or(None, |oa| oa.index()) {
                            Some(i) => {
                                if i >= NUM_PROCS {
                                    Err(ErrorCode::INVAL)
                                } else {
                                    core::mem::swap(&mut data.client_callbacks[i], &mut callback);
                                    Ok(())
                                }
                            }
                            None => Err(ErrorCode::INVAL),
                        }
                    });

                match result {
                    Ok(Ok(())) => Ok(callback),
                    Ok(Err(err)) => Err((callback, err)),
                    Err(process_error) => Err((callback, process_error.into())),
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

    /// Initiates a service discovery or notifies a client or service.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check, always returns SUCCESS
    /// - `1`: Perform discovery on the package name passed to `allow_readonly`. Returns the
    ///        service descriptor if the service is found, otherwise returns an error.
    /// - `2`: Notify a service previously discovered to have the service descriptor in
    ///        `target_id`. Returns an error if `target_id` refers to an invalid service or the
    ///        notify fails to enqueue.
    /// - `3`: Notify a client with descriptor `target_id`, typically in response to a previous
    ///        notify from the client. Returns an error if `target_id` refers to an invalid client
    ///        or the notify fails to enqueue.
    fn command(
        &self,
        command_number: usize,
        target_id: usize,
        _: usize,
        appid: AppId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 =>
            /* Discover */
            {
                self.data
                    .enter(appid, |data, _| {
                        data.search_slice.map_or(
                            CommandReturn::failure(ErrorCode::INVAL),
                            |slice| {
                                self.data
                                    .kernel
                                    .process_until(|p| {
                                        let s = p.get_process_name().as_bytes();
                                        // are slices equal?
                                        if s.len() == slice.len()
                                            && s.iter().zip(slice.iter()).all(|(c1, c2)| c1 == c2)
                                        {
                                            Some(CommandReturn::success_u32(
                                                p.appid().id() as u32 + 1,
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(CommandReturn::failure(ErrorCode::NODEVICE))
                            },
                        )
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }
            2 =>
            /* Service notify */
            {
                let cb_type = IPCCallbackType::Service;
                let app_identifier = target_id - 1;

                self.data
                    .kernel
                    .lookup_app_by_identifier(app_identifier)
                    .map_or(CommandReturn::failure(ErrorCode::INVAL), |otherapp| {
                        self.data.kernel.process_map_or(
                            CommandReturn::failure(ErrorCode::INVAL),
                            otherapp,
                            |target| {
                                let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                                match ret {
                                    true => CommandReturn::success(),
                                    false => CommandReturn::failure(ErrorCode::FAIL),
                                }
                            },
                        )
                    })
            }
            3 =>
            /* Client notify */
            {
                let cb_type = IPCCallbackType::Client;
                let app_identifier = target_id - 1;

                self.data
                    .kernel
                    .lookup_app_by_identifier(app_identifier)
                    .map_or(CommandReturn::failure(ErrorCode::INVAL), |otherapp| {
                        self.data.kernel.process_map_or(
                            CommandReturn::failure(ErrorCode::INVAL),
                            otherapp,
                            |target| {
                                let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                                match ret {
                                    true => CommandReturn::success(),
                                    false => CommandReturn::failure(ErrorCode::FAIL),
                                }
                            },
                        )
                    })
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    /// allow_readonly with subdriver number `0` stores the provided buffer for service discovery.
    /// The buffer should contain the package name of a process that exports an IPC service.
    fn allow_readonly(
        &self,
        appid: AppId,
        subdriver: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        if subdriver == 0 {
            // Package name for discovery
            let res = self.data.enter(appid, |data, _| {
                core::mem::swap(&mut data.search_slice, &mut slice);
            });
            match res {
                Ok(_) => Ok(slice),
                Err(e) => Err((slice, e.into())),
            }
        } else {
            Err((slice, ErrorCode::NOSUPPORT))
        }
    }

    /// allow_readwrite enables processes to discover IPC services on the platform or
    /// share buffers with existing services.
    ///
    /// If allow is called with target_id >= 1, it is a share command where the
    /// application is explicitly sharing a slice with an IPC service (as
    /// specified by the target_id). allow() simply allows both processes to
    /// access the buffer, it does not signal the service.
    ///
    /// target_id == 0 is currently unsupported and reserved for future use.
    fn allow_readwrite(
        &self,
        appid: AppId,
        target_id: usize,
        mut slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        if target_id == 0 {
            Err((slice, ErrorCode::NOSUPPORT))
        } else {
            match self.data.enter(appid, |data, _| {
                // Lookup the index of the app based on the passed in
                // identifier. This also let's us check that the other app is
                // actually valid.
                let app_identifier = target_id - 1;
                let otherapp = self.data.kernel.lookup_app_by_identifier(app_identifier);
                if let Some(oa) = otherapp {
                    if let Some(i) = oa.index() {
                        if let Some(smem) = data.shared_memory.get_mut(i) {
                            core::mem::swap(smem, &mut slice);
                            Ok(())
                        } else {
                            Err(ErrorCode::INVAL)
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    }
                } else {
                    Err(ErrorCode::BUSY)
                }
            }) {
                Ok(_) => Ok(slice),
                Err(e) => Err((slice, e.into())),
            }
        }
    }
}
