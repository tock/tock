// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! KV Store Userspace Driver.
//!
//! Provides userspace access to key-value store. Access is restricted based on
//! `StoragePermissions` so processes must have the required permissions in
//! their TBF headers to use this interface.
//!
//! ```rust,ignore
//! +===============+
//! ||  Userspace  ||
//! +===============+
//!
//! -----Syscall Interface-----
//!
//! +-------------------------+
//! |  KV Driver (this file)  |
//! +-------------------------+
//!
//!    hil::kv::KVPermissions
//!
//! +-------------------------+
//! | Virtualizer             |
//! +-------------------------+
//!
//!    hil::kv::KVPermissions
//!
//! +-------------------------+
//! |  K-V store Permissions  |
//! +-------------------------+
//!
//!    hil::kv::KV
//!
//! +-------------------------+
//! |  K-V library            |
//! +-------------------------+
//!
//!    hil::flash
//! ```

use capsules_core::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Kv as usize;

use core::cmp;
use kernel::errorcode;
use kernel::grant::Grant;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::hil::kv;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// IDs for read-only allow buffers.
mod ro_allow {
    /// Key.
    pub const KEY: usize = 0;
    /// Input value for set/add/update.
    pub const VALUE: usize = 1;
    /// The number of RO allow buffers the kernel stores for this grant.
    pub const COUNT: u8 = 2;
}

/// IDs for read-write allow buffers.
mod rw_allow {
    /// Output value for get.
    pub const VALUE: usize = 0;
    /// The number of RW allow buffers the kernel stores for this grant.
    pub const COUNT: u8 = 1;
}

/// IDs for upcalls.
mod upcalls {
    /// Single upcall.
    pub const VALUE: usize = 0;
    /// The number of upcalls the kernel stores for this grant.
    pub const COUNT: u8 = 1;
}

#[derive(Copy, Clone, PartialEq)]
enum UserSpaceOp {
    Get,
    Set,
    Delete,
    Add,
    Update,
    GarbageCollect,
}

/// Contents of the grant for each app.
#[derive(Default)]
pub struct App {
    op: OptionalCell<UserSpaceOp>,
}

/// Capsule that provides userspace access to a key-value store.
pub struct KVStoreDriver<'a, V: kv::KVPermissions<'a>> {
    /// Underlying k-v store implementation.
    kv: &'a V,
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
    key_buffer: TakeCell<'static, [u8]>,
    /// Value buffer.
    value_buffer: TakeCell<'static, [u8]>,
}

impl<'a, V: kv::KVPermissions<'a>> KVStoreDriver<'a, V> {
    pub fn new(
        kv: &'a V,
        key_buffer: &'static mut [u8],
        value_buffer: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<{ upcalls::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> KVStoreDriver<'a, V> {
        KVStoreDriver {
            kv,
            apps: grant,
            processid: OptionalCell::empty(),
            key_buffer: TakeCell::new(key_buffer),
            value_buffer: TakeCell::new(value_buffer),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.processid.map_or(Err(ErrorCode::RESERVE), |processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    let key_len = if app.op.is_some() {
                        // For all operations we need to copy in the key.
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::KEY)
                            .and_then(|buffer| {
                                buffer.enter(|key| {
                                    self.key_buffer.map_or(Err(ErrorCode::NOMEM), |key_buf| {
                                        // Error if we cannot fit the key.
                                        if key_buf.len() < key.len() {
                                            Err(ErrorCode::SIZE)
                                        } else {
                                            key.copy_to_slice(&mut key_buf[..key.len()]);
                                            Ok(key.len())
                                        }
                                    })
                                })
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE))?
                    } else {
                        0
                    };

                    match app.op.get() {
                        Some(UserSpaceOp::Get) => {
                            if let Some(Some(e)) = self.key_buffer.take().map(|key_buf| {
                                self.value_buffer.take().map(|val_buf| {
                                    let perms = processid
                                        .get_storage_permissions()
                                        .ok_or(ErrorCode::INVAL)?;

                                    let mut key = SubSliceMut::new(key_buf);
                                    key.slice(..key_len);

                                    let value = SubSliceMut::new(val_buf);

                                    if let Err((key_ret, val_ret, e)) =
                                        self.kv.get(key, value, perms)
                                    {
                                        self.key_buffer.replace(key_ret.take());
                                        self.value_buffer.replace(val_ret.take());
                                        return Err(e);
                                    }
                                    Ok(())
                                })
                            }) {
                                return e;
                            }
                        }
                        Some(UserSpaceOp::Set)
                        | Some(UserSpaceOp::Add)
                        | Some(UserSpaceOp::Update) => {
                            let value_len = kernel_data
                                .get_readonly_processbuffer(ro_allow::VALUE)
                                .and_then(|buffer| {
                                    buffer.enter(|value| {
                                        self.value_buffer.map_or(Err(ErrorCode::NOMEM), |val_buf| {
                                            // Make sure there is room for the
                                            // Tock KV header and the value.
                                            let header_size = self.kv.header_size();
                                            let remaining_space = val_buf.len() - header_size;
                                            if remaining_space < value.len() {
                                                Err(ErrorCode::SIZE)
                                            } else {
                                                value.copy_to_slice(
                                                    &mut val_buf
                                                        [header_size..(value.len() + header_size)],
                                                );
                                                Ok(value.len())
                                            }
                                        })
                                    })
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))?;

                            if let Some(Some(e)) = self.key_buffer.take().map(|key_buf| {
                                self.value_buffer.take().map(|val_buf| {
                                    let perms = processid
                                        .get_storage_permissions()
                                        .ok_or(ErrorCode::INVAL)?;

                                    let mut key = SubSliceMut::new(key_buf);
                                    key.slice(..key_len);

                                    // Make sure we provide a value buffer with
                                    // space for the tock kv header at the
                                    // front.
                                    let header_size = self.kv.header_size();
                                    let mut value = SubSliceMut::new(val_buf);
                                    value.slice(..(value_len + header_size));

                                    if let Err((key_ret, val_ret, e)) = match app.op.get() {
                                        Some(UserSpaceOp::Set) => self.kv.set(key, value, perms),
                                        Some(UserSpaceOp::Add) => self.kv.add(key, value, perms),
                                        Some(UserSpaceOp::Update) => {
                                            self.kv.update(key, value, perms)
                                        }
                                        _ => Ok(()),
                                    } {
                                        self.key_buffer.replace(key_ret.take());
                                        self.value_buffer.replace(val_ret.take());
                                        return Err(e);
                                    }
                                    Ok(())
                                })
                            }) {
                                return e;
                            }
                        }
                        Some(UserSpaceOp::Delete) => {
                            if let Some(e) = self.key_buffer.take().map(|key_buf| {
                                let perms = processid
                                    .get_storage_permissions()
                                    .ok_or(ErrorCode::INVAL)?;

                                let mut key = SubSliceMut::new(key_buf);
                                key.slice(..key_len);

                                if let Err((key_ret, e)) = self.kv.delete(key, perms) {
                                    self.key_buffer.replace(key_ret.take());
                                    return Err(e);
                                }
                                Ok(())
                            }) {
                                return e;
                            }
                        }
                        Some(UserSpaceOp::GarbageCollect) => {
                            self.kv.garbage_collect()?;
                            return Ok(());
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

impl<'a, V: kv::KVPermissions<'a>> kv::KVClient for KVStoreDriver<'a, V> {
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.key_buffer.replace(key.take());

        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Get) {
                    app.op.clear();

                    if let Err(e) = result {
                        let _ = upcalls.schedule_upcall(
                            upcalls::VALUE,
                            (errorcode::into_statuscode(e.into()), 0, 0),
                        );
                    } else {
                        let value_len = value.len();
                        let ret = upcalls
                            .get_readwrite_processbuffer(rw_allow::VALUE)
                            .and_then(|buffer| {
                                buffer.mut_enter(|appslice| {
                                    let copy_len = cmp::min(value_len, appslice.len());
                                    appslice[..copy_len].copy_from_slice(&value[..copy_len]);
                                    if copy_len < value_len {
                                        Err(ErrorCode::SIZE)
                                    } else {
                                        Ok(())
                                    }
                                })
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE));

                        // Signal the upcall, and return the length of the
                        // value. Userspace should be careful to check for an
                        // error and only read the portion that would fit in the
                        // buffer if the value was larger than the provided
                        // processbuffer.
                        let _ = upcalls.schedule_upcall(
                            upcalls::VALUE,
                            (errorcode::into_statuscode(ret), value_len, 0),
                        );
                    }
                }

                self.value_buffer.replace(value.take());
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
        self.key_buffer.replace(key.take());
        self.value_buffer.replace(value.take());

        // Signal the upcall and clear the requested op.
        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Set) {
                    app.op.clear();
                    let _ = upcalls.schedule_upcall(
                        upcalls::VALUE,
                        (errorcode::into_statuscode(result), 0, 0),
                    );
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn add_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.key_buffer.replace(key.take());
        self.value_buffer.replace(value.take());

        // Signal the upcall and clear the requested op.
        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Add) {
                    app.op.clear();
                    let _ = upcalls.schedule_upcall(
                        upcalls::VALUE,
                        (errorcode::into_statuscode(result), 0, 0),
                    );
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn update_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.key_buffer.replace(key.take());
        self.value_buffer.replace(value.take());

        // Signal the upcall and clear the requested op.
        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Update) {
                    app.op.clear();
                    let _ = upcalls.schedule_upcall(
                        upcalls::VALUE,
                        (errorcode::into_statuscode(result), 0, 0),
                    );
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn delete_complete(&self, result: Result<(), ErrorCode>, key: SubSliceMut<'static, u8>) {
        self.key_buffer.replace(key.take());

        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::Delete) {
                    app.op.clear();
                    let _ = upcalls.schedule_upcall(
                        upcalls::VALUE,
                        (errorcode::into_statuscode(result), 0, 0),
                    );
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }

    fn garbage_collection_complete(&self, result: Result<(), ErrorCode>) {
        self.processid.map(move |id| {
            self.apps.enter(id, move |app, upcalls| {
                if app.op.contains(&UserSpaceOp::GarbageCollect) {
                    app.op.clear();
                    let _ = upcalls.schedule_upcall(
                        upcalls::VALUE,
                        (errorcode::into_statuscode(result), 0, 0),
                    );
                }
            })
        });

        // We have completed the operation so see if there is a queued operation
        // to run next.
        self.processid.clear();
        self.check_queue();
    }
}

impl<'a, V: kv::KVPermissions<'a>> SyscallDriver for KVStoreDriver<'a, V> {
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

            // get, set, delete, add, update, garbage collect
            1 | 2 | 3 | 4 | 5 | 6 => {
                if self.processid.is_none() {
                    // Nothing is using the KV store, so we can handle this
                    // request.
                    self.processid.set(processid);
                    let _ = self.apps.enter(processid, |app, _| match command_num {
                        1 => app.op.set(UserSpaceOp::Get),
                        2 => app.op.set(UserSpaceOp::Set),
                        3 => app.op.set(UserSpaceOp::Delete),
                        4 => app.op.set(UserSpaceOp::Add),
                        5 => app.op.set(UserSpaceOp::Update),
                        6 => app.op.set(UserSpaceOp::GarbageCollect),
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
                                    4 => app.op.set(UserSpaceOp::Add),
                                    5 => app.op.set(UserSpaceOp::Update),
                                    6 => app.op.set(UserSpaceOp::GarbageCollect),
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
