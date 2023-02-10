//! KV Driver
//!

use core_capsules::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::KVSystem as usize;

use crate::kv_store::KVStore;
use core::cell::Cell;
use kernel::grant::Grant;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::hil::kv_system;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Ids for read-only allow buffers
mod ro_allow {
    // unhashed key
    pub const UNHASHED_KEY: usize = 0;
    // input value
    pub const VALUE: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const VALUE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for upcalls
mod upcalls {
    pub const VALUE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub struct KVSystemDriver<
    'a,
    K: kv_system::KVSystem<'a> + kv_system::KVSystem<'a, K = T>,
    T: 'static + kv_system::KeyType,
> {
    kv: &'a KVStore<'a, K, T>,

    active: Cell<bool>,

    apps: Grant<
        App,
        UpcallCount<{ upcalls::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    processid: OptionalCell<ProcessId>,

    data_buffer: TakeCell<'static, [u8]>,
    dest_buffer: TakeCell<'static, [u8]>,
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> KVSystemDriver<'a, K, T> {
    pub fn new(
        kv: &'a KVStore<'a, K, T>,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<{ upcalls::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> KVSystemDriver<'a, K, T> {
        KVSystemDriver {
            kv,
            active: Cell::new(false),
            apps: grant,
            processid: OptionalCell::empty(),
            data_buffer: TakeCell::new(data_buffer),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.processid.map_or(Err(ErrorCode::RESERVE), |processid| {
            self.apps
                .enter(*processid, |app, kernel_data| {
                    if let Some(operation) = app.op.get() {
                        match operation {
                            UserSpaceOp::Get => {
                                kernel_data
                                    .get_readonly_processbuffer(ro_allow::UNHASHED_KEY)
                                    .and_then(|buffer| {
                                        buffer.enter(|unhashed_key| {
                                            self.data_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                                // Determine the size of the static buffer we have
                                                let static_buffer_len =
                                                    buf.len().min(unhashed_key.len());

                                                // Copy the data into the static buffer
                                                unhashed_key[..static_buffer_len]
                                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                                Ok(())
                                            })
                                        })
                                    })
                                    .unwrap_or(Err(ErrorCode::RESERVE))?;

                                if let Some(Some(Err(e))) =
                                    self.data_buffer.take().map(|data_buffer| {
                                        self.dest_buffer.take().map(|dest_buffer| {
                                            let perms = processid
                                                .get_storage_permissions()
                                                .ok_or(ErrorCode::INVAL)?;
                                            if let Err((data, dest, e)) =
                                                self.kv.get(data_buffer, dest_buffer, perms)
                                            {
                                                self.data_buffer.replace(data);
                                                self.dest_buffer.replace(dest);
                                                return Err(e);
                                            }
                                            Ok(())
                                        })
                                    })
                                {
                                    return e;
                                }
                            }
                            UserSpaceOp::Set => {
                                kernel_data
                                    .get_readonly_processbuffer(ro_allow::UNHASHED_KEY)
                                    .and_then(|buffer| {
                                        buffer.enter(|unhashed_key| {
                                            self.data_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                                // Determine the size of the static buffer we have
                                                let static_buffer_len =
                                                    buf.len().min(unhashed_key.len());

                                                // Copy the data into the static buffer
                                                unhashed_key[..static_buffer_len]
                                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                                Ok(())
                                            })
                                        })
                                    })
                                    .unwrap_or(Err(ErrorCode::RESERVE))?;

                                let mut static_buffer_len = 0;

                                kernel_data
                                    .get_readonly_processbuffer(ro_allow::VALUE)
                                    .and_then(|buffer| {
                                        buffer.enter(|value| {
                                            self.dest_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                                // Determine the size of the static buffer we have
                                                static_buffer_len = buf.len().min(value.len());

                                                // Copy the data into the static buffer
                                                value[..static_buffer_len]
                                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                                Ok(())
                                            })
                                        })
                                    })
                                    .unwrap_or(Err(ErrorCode::RESERVE))?;

                                if let Some(Some(Err(e))) =
                                    self.data_buffer.take().map(|data_buffer| {
                                        self.dest_buffer.take().map(|dest_buffer| {
                                            let perms = processid
                                                .get_storage_permissions()
                                                .ok_or(ErrorCode::INVAL)?;
                                            if let Err((data, dest, e)) = self.kv.set(
                                                data_buffer,
                                                dest_buffer,
                                                static_buffer_len,
                                                perms,
                                            ) {
                                                self.data_buffer.replace(data);
                                                self.dest_buffer.replace(dest);
                                                return Err(e);
                                            }
                                            Ok(())
                                        })
                                    })
                                {
                                    return e;
                                }
                            }
                            UserSpaceOp::Delete => {
                                kernel_data
                                    .get_readonly_processbuffer(ro_allow::UNHASHED_KEY)
                                    .and_then(|buffer| {
                                        buffer.enter(|unhashed_key| {
                                            self.data_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                                // Determine the size of the static buffer we have
                                                let static_buffer_len =
                                                    buf.len().min(unhashed_key.len());

                                                // Copy the data into the static buffer
                                                unhashed_key[..static_buffer_len]
                                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                                Ok(())
                                            })
                                        })
                                    })
                                    .unwrap_or(Err(ErrorCode::RESERVE))?;

                                if let Some(Err(e)) = self.data_buffer.take().map(|data_buffer| {
                                    let perms = processid
                                        .get_storage_permissions()
                                        .ok_or(ErrorCode::INVAL)?;
                                    if let Err((data, e)) = self.kv.delete(data_buffer, perms) {
                                        self.data_buffer.replace(data);
                                        return Err(e);
                                    }
                                    Ok(())
                                }) {
                                    return e;
                                }
                            }
                        }
                    }

                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let started_command = appiter.enter(|app, _| {
                // If an app is already running let it complete
                if self.processid.is_some() {
                    return true;
                }

                // If this app has a pending command let's use it.
                app.pending_run_app.take().map_or(false, |processid| {
                    // Mark this driver as being in use.
                    self.processid.set(processid);
                    // Actually make the buzz happen.
                    self.run() == Ok(())
                })
            });
            if started_command {
                break;
            }
        }
    }
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> kv_system::StoreClient<T>
    for KVSystemDriver<'a, K, T>
{
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut [u8],
        ret_buf: &'static mut [u8],
    ) {
        self.data_buffer.replace(key);
        self.dest_buffer.replace(ret_buf);

        self.processid.map(move |id| {
            self.apps.enter(*id, move |app, upcalls| {
                if app.op.get().map(|op| op == UserSpaceOp::Get).is_some() {
                    if let Err(e) = result {
                        upcalls
                            .schedule_upcall(
                                upcalls::VALUE,
                                (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                            )
                            .ok();
                    } else {
                        self.dest_buffer.map(|buf| {
                            let ret = upcalls
                                .get_readwrite_processbuffer(rw_allow::VALUE)
                                .and_then(|buffer| {
                                    buffer.mut_enter(|data| {
                                        // Determine the size of the static buffer we have
                                        let static_buffer_len = buf.len();
                                        let data_len = data.len();

                                        if data_len < static_buffer_len {
                                            data.copy_from_slice(&buf[..data_len]);
                                        } else {
                                            data[..static_buffer_len].copy_from_slice(&buf);
                                        }
                                        Ok(())
                                    })
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE));

                            if ret == Err(ErrorCode::RESERVE) {
                                upcalls
                                    .schedule_upcall(
                                        upcalls::VALUE,
                                        (kernel::errorcode::into_statuscode(ret.into()), 0, 0),
                                    )
                                    .ok();
                            } else {
                                upcalls.schedule_upcall(upcalls::VALUE, (0, 0, 0)).ok();
                            }
                        });

                        self.processid.clear();
                    }
                }
            })
        });
    }

    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut [u8],
        value: &'static mut [u8],
    ) {
        self.data_buffer.replace(key);
        self.dest_buffer.replace(value);

        self.processid.map(move |id| {
            self.apps.enter(*id, move |app, upcalls| {
                if app.op.get().map(|op| op == UserSpaceOp::Set).is_some() {
                    if let Err(e) = result {
                        upcalls
                            .schedule_upcall(
                                upcalls::VALUE,
                                (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                            )
                            .ok();
                    } else {
                        upcalls.schedule_upcall(upcalls::VALUE, (0, 0, 0)).ok();

                        self.processid.clear();
                    }
                }
            })
        });
    }

    fn delete_complete(&self, result: Result<(), ErrorCode>, key: &'static mut [u8]) {
        self.data_buffer.replace(key);

        self.processid.map(move |id| {
            self.apps.enter(*id, move |app, upcalls| {
                if app.op.get().map(|op| op == UserSpaceOp::Delete).is_some() {
                    if let Err(e) = result {
                        upcalls
                            .schedule_upcall(
                                upcalls::VALUE,
                                (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                            )
                            .ok();
                    } else {
                        upcalls.schedule_upcall(upcalls::VALUE, (0, 0, 0)).ok();

                        self.processid.clear();
                    }
                }
            })
        });
    }
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> SyscallDriver
    for KVSystemDriver<'a, K, T>
{
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let match_or_empty_or_nonexistant = self.processid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the KV Store.

            // If the KV Store is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the KV Store is not active, then
            // we need to verify that that application still exists, and remove
            // it as owner if not.
            if self.active.get() {
                owning_app == &processid
            } else {
                // Check the app still exists.
                //
                // If the `.enter()` succeeds, then the app is still valid, and
                // we can check if the owning app matches the one that called
                // the command. If the `.enter()` fails, then the owning app no
                // longer exists and we return `true` to signify the
                // "or_nonexistant" case.
                self.apps
                    .enter(*owning_app, |_, _| owning_app == &processid)
                    .unwrap_or(true)
            }
        });

        match command_num {
            // check if present
            0 => CommandReturn::success(),

            // get, set, delete
            1 | 2 | 3 => {
                if match_or_empty_or_nonexistant {
                    self.processid.set(processid);
                    let _ = self.apps.enter(processid, |app, _| match command_num {
                        1 => app.op.set(Some(UserSpaceOp::Get)),
                        2 => app.op.set(Some(UserSpaceOp::Set)),
                        3 => app.op.set(Some(UserSpaceOp::Delete)),
                        _ => {}
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.processid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(processid, |app, _| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(processid);
                                match command_num {
                                    1 => app.op.set(Some(UserSpaceOp::Get)),
                                    2 => app.op.set(Some(UserSpaceOp::Set)),
                                    3 => app.op.set(Some(UserSpaceOp::Delete)),
                                    _ => {}
                                }
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

#[derive(Copy, Clone, PartialEq)]
enum UserSpaceOp {
    Get,
    Set,
    Delete,
}

#[derive(Default)]
pub struct App {
    pending_run_app: Option<ProcessId>,
    op: Cell<Option<UserSpaceOp>>,
}
