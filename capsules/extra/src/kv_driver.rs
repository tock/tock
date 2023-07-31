// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! KV Store Userspace Driver.
//!
//! Provides userspace access to key-value store. Access is restricted based on
//! `StoragePermissions` so processes must have the required permissions in
//! their TBF headers to use this interface.

use capsules_core::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::KVSystem as usize;

use crate::kv_store;
use crate::kv_store::KVStore;
use core::cmp;
use kernel::errorcode;
use kernel::grant::Grant;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::hil::kv_system;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
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

#[derive(Copy, Clone, PartialEq)]
enum UserSpaceOp {
    Get,
    Set,
    Delete,
}

/// Contents of the grant for each app.
#[derive(Default)]
pub struct App {
    op: OptionalCell<UserSpaceOp>,
}

/// Capsule that provides userspace access to a key-value store.
pub struct KVStoreDriver<
    'a,
    K: kv_system::KVSystem<'a> + kv_system::KVSystem<'a, K = T>,
    T: 'static + kv_system::KeyType,
> {
    /// Underlying k-v store implementation.
    kv: &'a KVStore<'a, K, T>,
    /// Grant storage for each app.
    apps: Grant<
        App,
        UpcallCount<{ upcalls::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    /// App that is actively using the k-v store.
    processid: OptionalCell<ProcessId>,
    /// Key buffer.
    data_buffer: TakeCell<'static, [u8]>,
    /// Value buffer.
    dest_buffer: TakeCell<'static, [u8]>,
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> KVStoreDriver<'a, K, T> {
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
    ) -> KVStoreDriver<'a, K, T> {
        KVStoreDriver {
            kv,
            apps: grant,
            processid: OptionalCell::empty(),
            data_buffer: TakeCell::new(data_buffer),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.processid.map_or(Err(ErrorCode::RESERVE), |processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    let unhashed_key_len = if app.op.is_some() {
                        // For all operations we need to copy in the unhashed
                        // key.
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::UNHASHED_KEY)
                            .and_then(|buffer| {
                                buffer.enter(|unhashed_key| {
                                    self.data_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                        // Determine the size of the static
                                        // buffer we have and copy the contents.
                                        let static_buffer_len = buf.len().min(unhashed_key.len());
                                        unhashed_key[..static_buffer_len]
                                            .copy_to_slice(&mut buf[..static_buffer_len]);

                                        Ok(static_buffer_len)
                                    })
                                })
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE))?
                    } else {
                        0
                    };

                    match app.op.get() {
                        Some(UserSpaceOp::Get) => {
                            if let Some(Some(Err(e))) = self.data_buffer.take().map(|data_buffer| {
                                self.dest_buffer.take().map(|dest_buffer| {
                                    let perms = processid
                                        .get_storage_permissions()
                                        .ok_or(ErrorCode::INVAL)?;

                                    let mut unhashed_key = SubSliceMut::new(data_buffer);
                                    unhashed_key.slice(..unhashed_key_len);

                                    let value = SubSliceMut::new(dest_buffer);

                                    if let Err((data, dest, e)) =
                                        self.kv.get(unhashed_key, value, perms)
                                    {
                                        self.data_buffer.replace(data.take());
                                        self.dest_buffer.replace(dest.take());
                                        return Err(e);
                                    }
                                    Ok(())
                                })
                            }) {
                                return e;
                            }
                        }
                        Some(UserSpaceOp::Set) => {
                            let value_len = kernel_data
                                .get_readonly_processbuffer(ro_allow::VALUE)
                                .and_then(|buffer| {
                                    buffer.enter(|value| {
                                        self.dest_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                            // Determine the size of the static
                                            // buffer we have for the value and
                                            // copy the contents.
                                            let header_size = self.kv.header_size();
                                            let copy_len =
                                                (buf.len() - header_size).min(value.len());
                                            value[..copy_len].copy_to_slice(
                                                &mut buf[header_size..(copy_len + header_size)],
                                            );

                                            Ok(copy_len)
                                        })
                                    })
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))?;

                            if let Some(Some(Err(e))) = self.data_buffer.take().map(|data_buffer| {
                                self.dest_buffer.take().map(|dest_buffer| {
                                    let perms = processid
                                        .get_storage_permissions()
                                        .ok_or(ErrorCode::INVAL)?;

                                    let mut unhashed_key = SubSliceMut::new(data_buffer);
                                    unhashed_key.slice(..unhashed_key_len);

                                    // Make sure we provide a value buffer with
                                    // space for the tock kv header at the
                                    // front.
                                    let header_size = self.kv.header_size();
                                    let mut value = SubSliceMut::new(dest_buffer);
                                    value.slice(..(value_len + header_size));

                                    if let Err((data, dest, e)) =
                                        self.kv.set(unhashed_key, value, perms)
                                    {
                                        self.data_buffer.replace(data.take());
                                        self.dest_buffer.replace(dest.take());
                                        return Err(e);
                                    }
                                    Ok(())
                                })
                            }) {
                                return e;
                            }
                        }
                        Some(UserSpaceOp::Delete) => {
                            if let Some(Err(e)) = self.data_buffer.take().map(|data_buffer| {
                                let perms = processid
                                    .get_storage_permissions()
                                    .ok_or(ErrorCode::INVAL)?;

                                let mut unhashed_key = SubSliceMut::new(data_buffer);
                                unhashed_key.slice(..unhashed_key_len);

                                if let Err((data, e)) = self.kv.delete(unhashed_key, perms) {
                                    self.data_buffer.replace(data.take());
                                    return Err(e);
                                }
                                Ok(())
                            }) {
                                return e;
                            }
                        }
                        _ => {}
                    }

                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn check_queue(&self) {
        // If an app is already running let it complete.
        if self.processid.is_some() {
            return;
        }

        for appiter in self.apps.iter() {
            let processid = appiter.processid();
            let has_pending_op = appiter.enter(|app, _| {
                // If this app has a pending command let's use it.
                app.op.is_some()
            });
            let started_command = if has_pending_op {
                // Mark this driver as being in use.
                self.processid.set(processid);
                self.run() == Ok(())
            } else {
                false
            };
            if started_command {
                break;
            } else {
                self.processid.clear();
            }
        }
    }
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> kv_store::StoreClient<T>
    for KVStoreDriver<'a, K, T>
{
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.data_buffer.replace(key.take());

        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Get) {
                    app.op.clear();

                    if let Err(e) = result {
                        upcalls
                            .schedule_upcall(
                                upcalls::VALUE,
                                (errorcode::into_statuscode(e.into()), 0, 0),
                            )
                            .ok();
                    } else {
                        let value_len = value.len();
                        let ret = upcalls
                            .get_readwrite_processbuffer(rw_allow::VALUE)
                            .and_then(|buffer| {
                                buffer.mut_enter(|appslice| {
                                    let copy_len = cmp::min(value_len, appslice.len());
                                    appslice[..copy_len].copy_from_slice(&value[..copy_len]);
                                    Ok(())
                                })
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE));

                        // Signal the upcall, and return the length of the
                        // value. Userspace should be careful to check for an
                        // error and only read the portion that would fit in the
                        // buffer if the value was larger than the provided
                        // processbuffer.
                        upcalls
                            .schedule_upcall(
                                upcalls::VALUE,
                                (errorcode::into_statuscode(ret.into()), value_len, 0),
                            )
                            .ok();
                    }
                }

                self.dest_buffer.replace(value.take());
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.data_buffer.replace(key.take());
        self.dest_buffer.replace(value.take());

        // Signal the upcall and clear the requested op. We have to do a lot of
        // checking for robustness, but there is no reason this should fail.
        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Set) {
                    app.op.clear();
                    upcalls
                        .schedule_upcall(upcalls::VALUE, (errorcode::into_statuscode(result), 0, 0))
                        .ok();
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn delete_complete(&self, result: Result<(), ErrorCode>, key: SubSliceMut<'static, u8>) {
        self.data_buffer.replace(key.take());

        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Delete) {
                    app.op.clear();
                    upcalls
                        .schedule_upcall(upcalls::VALUE, (errorcode::into_statuscode(result), 0, 0))
                        .ok();
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }
}

impl<'a, K: kv_system::KVSystem<'a, K = T>, T: kv_system::KeyType> SyscallDriver
    for KVStoreDriver<'a, K, T>
{
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check if present
            0 => CommandReturn::success(),

            // get, set, delete
            1 | 2 | 3 => {
                if self.processid.is_none() {
                    // Nothing is using the KV store, so we can handle this
                    // request.
                    self.processid.set(processid);
                    let _ = self.apps.enter(processid, |app, _| match command_num {
                        1 => app.op.set(UserSpaceOp::Get),
                        2 => app.op.set(UserSpaceOp::Set),
                        3 => app.op.set(UserSpaceOp::Delete),
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
                    // There is an active app, so queue this request (if
                    // possible).
                    self.apps
                        .enter(processid, |app, _| {
                            if app.op.is_some() {
                                // No more room in the queue, nowhere to store
                                // this request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // This app has not already queued a command so
                                // we can store this.
                                match command_num {
                                    1 => app.op.set(UserSpaceOp::Get),
                                    2 => app.op.set(UserSpaceOp::Set),
                                    3 => app.op.set(UserSpaceOp::Delete),
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
