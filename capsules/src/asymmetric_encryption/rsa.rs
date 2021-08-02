//! RSA

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Rsa as usize;

use crate::asymmetric_encryption::rsa_key::RSA2048Keys;
use core::cell::Cell;
use core::mem;
use kernel::errorcode::into_statuscode;
use kernel::grant::Grant;
use kernel::hil::public_key_crypto::{self, Operation, RsaCryptoBuffers, RsaKey};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::processbuffer::{ReadWriteProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};

use kernel::{ErrorCode, ProcessId};

enum RsaOperation {
    PKCS1v15,
}

#[derive(Copy, Clone, PartialEq)]
enum UserSpaceOp {
    Encrypt,
    Verify,
    Decrypt,
    Sign,
}

pub struct RsaDriver<'a, E: public_key_crypto::RsaCrypto<'a>> {
    rsa: &'a E,

    active: Cell<bool>,

    apps: Grant<App, 1>,
    appid: OptionalCell<ProcessId>,

    source_buffer: TakeCell<'static, [u8]>,
    data_copied: Cell<usize>,
    dest_buffer: TakeCell<'static, [u8]>,
    rsa_keys: TakeCell<'static, dyn RsaKey<'static>>,
}

impl<'a, E: public_key_crypto::RsaCrypto<'a> + public_key_crypto::RSAPKCS1v15> RsaDriver<'a, E> {
    pub fn new(
        rsa: &'a E,
        source_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
        rsa_keys: &'static mut RSA2048Keys,
        grant: Grant<App, 1>,
    ) -> Self {
        Self {
            rsa,
            active: Cell::new(false),
            apps: grant,
            appid: OptionalCell::empty(),
            source_buffer: TakeCell::new(source_buffer),
            data_copied: Cell::new(0),
            dest_buffer: TakeCell::new(dest_buffer),
            rsa_keys: TakeCell::new(rsa_keys),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.appid.map_or(Err(ErrorCode::RESERVE), |appid| {
            self.apps
                .enter(*appid, |app, _| {
                    let ret = if let Some(op) = &app.rsa_operation {
                        match op {
                            RsaOperation::PKCS1v15 => self.rsa.set_mode_rsa_pkcs1v15(),
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    };
                    if ret.is_err() {
                        return ret;
                    }

                    app.source
                        .enter(|source| {
                            let mut static_buffer_len = 0;
                            self.source_buffer.map(|source_static| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = source_static.len();

                                if static_buffer_len > source.len() {
                                    static_buffer_len = source.len()
                                }

                                self.data_copied.set(static_buffer_len);

                                // Copy the source into the static buffer
                                source[..static_buffer_len]
                                    .copy_to_slice(&mut source_static[..static_buffer_len]);
                            });

                            app.key
                                .enter(|key| {
                                    self.rsa_keys.map(|rsa_keys| {
                                        let mut temp: [u8; 256] = [0; 256];
                                        key.copy_to_slice(&mut temp);
                                        rsa_keys.import_public_key(&temp[..]).unwrap();
                                    });

                                    // Add the data from the static buffer to the RSA
                                    let ret = match app.op.get().unwrap() {
                                        UserSpaceOp::Encrypt => {
                                            self.rsa.encrypt(RsaCryptoBuffers {
                                                source: self.source_buffer.take().unwrap(),
                                                dest: self.dest_buffer.take().unwrap(),
                                                key: self.rsa_keys.take().unwrap(),
                                            })
                                        }
                                        UserSpaceOp::Verify => {
                                            app.dest.enter(|dest| {
                                                let mut static_buffer_len = 0;
                                                self.dest_buffer.map(|dest_static| {
                                                    // Determine the size of the static buffer we have
                                                    static_buffer_len = dest_static.len();

                                                    if static_buffer_len > dest.len() {
                                                        static_buffer_len = dest.len()
                                                    }

                                                    self.data_copied.set(static_buffer_len);

                                                    // Copy the dest into the static buffer
                                                    dest[..static_buffer_len].copy_to_slice(
                                                        &mut dest_static[..static_buffer_len],
                                                    );
                                                });
                                            })?;

                                            self.rsa.verify(RsaCryptoBuffers {
                                                source: self.source_buffer.take().unwrap(),
                                                dest: self.dest_buffer.take().unwrap(),
                                                key: self.rsa_keys.take().unwrap(),
                                            })
                                        }
                                        UserSpaceOp::Decrypt => {
                                            self.rsa.decrypt(RsaCryptoBuffers {
                                                source: self.source_buffer.take().unwrap(),
                                                dest: self.dest_buffer.take().unwrap(),
                                                key: self.rsa_keys.take().unwrap(),
                                            })
                                        }
                                        UserSpaceOp::Sign => self.rsa.sign(RsaCryptoBuffers {
                                            source: self.source_buffer.take().unwrap(),
                                            dest: self.dest_buffer.take().unwrap(),
                                            key: self.rsa_keys.take().unwrap(),
                                        }),
                                    };

                                    match ret {
                                        Ok(()) => Ok(()),
                                        Err(e) => {
                                            self.source_buffer.replace(e.1.source);
                                            self.dest_buffer.replace(e.1.dest);
                                            self.rsa_keys.replace(e.1.key);
                                            Err(e.0)
                                        }
                                    }
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let started_command = appiter.enter(|app, _| {
                // If an app is already running let it complete
                if self.appid.is_some() {
                    return true;
                }

                // If this app has a pending command let's use it.
                app.pending_run_app.take().map_or(false, |appid| {
                    // Mark this driver as being in use.
                    self.appid.set(appid);
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

impl<'a, E: public_key_crypto::RsaCrypto<'a> + public_key_crypto::RSAPKCS1v15>
    public_key_crypto::Client<'a, RsaCryptoBuffers> for RsaDriver<'a, E>
{
    fn operation_done(
        &'a self,
        result: Result<bool, ErrorCode>,
        op: Operation,
        buffers: RsaCryptoBuffers,
    ) {
        self.appid.map(|id| {
            self.apps
                .enter(*id, |app, upcalls| {
                    self.rsa.clear_data();

                    let _ = app.dest.mut_enter(|dest| {
                        let app_len = dest.len();
                        let static_len = buffers.dest.len();

                        if app_len < static_len {
                            dest.copy_from_slice(&buffers.dest[0..app_len]);
                        } else {
                            dest[0..static_len].copy_from_slice(buffers.dest);
                        }
                    });

                    match result {
                        Ok(equal) => upcalls
                            .schedule_upcall(0, (0, op as usize, equal as usize))
                            .ok(),
                        Err(e) => upcalls
                            .schedule_upcall(0, (into_statuscode(e.into()), op as usize, 0))
                            .ok(),
                    };

                    // Clear the current appid as it has finished running
                    self.appid.clear();
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.appid.clear();
                    }
                })
        });

        self.source_buffer.replace(buffers.source);
        self.dest_buffer.replace(buffers.dest);
        self.rsa_keys.replace(buffers.key);

        self.check_queue();
    }
}

impl<'a, E: public_key_crypto::RsaCrypto<'a> + public_key_crypto::RSAPKCS1v15> SyscallDriver
    for RsaDriver<'a, E>
{
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass buffer for the dest to be in.
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut slice, &mut app.dest);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // default
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    fn allow_readonly(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass buffer for the source to be in.
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut slice, &mut app.source);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // Pass buffer for the key to be in.
            1 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut slice, &mut app.key);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // default
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        let match_or_empty_or_nonexistant = self.appid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the RSA.

            // If the RSA is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the RSA is not active, then
            // we need to verify that that application still exists, and remove
            // it as owner if not.
            if self.active.get() {
                owning_app == &appid
            } else {
                // Check the app still exists.
                //
                // If the `.enter()` succeeds, then the app is still valid, and
                // we can check if the owning app matches the one that called
                // the command. If the `.enter()` fails, then the owning app no
                // longer exists and we return `true` to signify the
                // "or_nonexistant" case.
                self.apps
                    .enter(*owning_app, |_, _| owning_app == &appid)
                    .unwrap_or(true)
            }
        });

        match command_num {
            // set_algorithm
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        match data1 {
                            // PKCS1v15
                            0 => {
                                app.rsa_operation = Some(RsaOperation::PKCS1v15);
                                CommandReturn::success()
                            }
                            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // encrypt
            1 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Encrypt));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.rsa.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app, _| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.op.set(Some(UserSpaceOp::Encrypt));
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // verify
            2 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Verify));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.rsa.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app, _| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.op.set(Some(UserSpaceOp::Verify));
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // decrypt
            3 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Decrypt));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.rsa.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app, _| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.op.set(Some(UserSpaceOp::Decrypt));
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // sign
            4 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Sign));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.rsa.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app, _| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.op.set(Some(UserSpaceOp::Sign));
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

#[derive(Default)]
pub struct App {
    pending_run_app: Option<ProcessId>,
    rsa_operation: Option<RsaOperation>,
    op: Cell<Option<UserSpaceOp>>,
    source: ReadOnlyProcessBuffer,
    dest: ReadWriteProcessBuffer,
    key: ReadOnlyProcessBuffer,
}
