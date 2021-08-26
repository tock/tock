//! AES.

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Aes as usize;

use core::cell::Cell;
use core::mem;

use kernel::grant::Grant;
use kernel::hil::symmetric_encryption::{
    AES128Ctr, CCMClient, Client, AES128, AES128CBC, AES128CCM, AES128ECB, AES128_BLOCK_SIZE,
};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::processbuffer::{ReadWriteProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

pub struct AesDriver<'a, A: AES128<'a> + AES128CCM<'static>> {
    aes: &'a A,

    active: Cell<bool>,

    apps: Grant<App, 1>,
    appid: OptionalCell<ProcessId>,

    source_buffer: TakeCell<'static, [u8]>,
    data_copied: Cell<usize>,
    dest_buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>>
    AesDriver<'static, A>
{
    pub fn new(
        aes: &'static A,
        source_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
        grant: Grant<App, 1>,
    ) -> AesDriver<'static, A> {
        AesDriver {
            aes,
            active: Cell::new(false),
            apps: grant,
            appid: OptionalCell::empty(),
            source_buffer: TakeCell::new(source_buffer),
            data_copied: Cell::new(0),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.appid.map_or(Err(ErrorCode::RESERVE), |appid| {
            self.apps
                .enter(*appid, |app, _| {
                    self.aes.enable();
                    let ret = if let Some(op) = &app.aes_operation {
                        match op {
                            AesOperation::AES128Ctr(encrypt) => {
                                self.aes.set_mode_aes128ctr(*encrypt)
                            }
                            AesOperation::AES128CBC(encrypt) => {
                                self.aes.set_mode_aes128cbc(*encrypt)
                            }
                            AesOperation::AES128ECB(encrypt) => {
                                self.aes.set_mode_aes128ecb(*encrypt)
                            }
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    };
                    if ret.is_err() {
                        return ret;
                    }

                    app.key
                        .enter(|key| {
                            let mut static_buffer_len = 0;
                            self.source_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();

                                if static_buffer_len > key.len() {
                                    static_buffer_len = key.len()
                                }

                                // Copy the data into the static buffer
                                key[..static_buffer_len]
                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                if let Err(e) = AES128::set_key(self.aes, buf) {
                                    return Err(e);
                                }
                                Ok(())
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    app.iv
                        .enter(|iv| {
                            let mut static_buffer_len = 0;
                            self.source_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();

                                if static_buffer_len > iv.len() {
                                    static_buffer_len = iv.len()
                                }

                                // Copy the data into the static buffer
                                iv[..static_buffer_len]
                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                if let Err(e) = self.aes.set_iv(buf) {
                                    return Err(e);
                                }
                                Ok(())
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    app.source
                        .enter(|source| {
                            let mut static_buffer_len = 0;
                            self.source_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();

                                if static_buffer_len > source.len() {
                                    static_buffer_len = source.len()
                                }

                                // Copy the data into the static buffer
                                source[..static_buffer_len]
                                    .copy_to_slice(&mut buf[..static_buffer_len]);

                                self.data_copied.set(static_buffer_len);

                                Ok(())
                            })?;

                            if let Err(e) = self.calculate_output() {
                                return Err(e);
                            }
                            Ok(())
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn calculate_output(&self) -> Result<(), ErrorCode> {
        if let Some((e, source, dest)) = AES128::crypt(
            self.aes,
            Some(self.source_buffer.take().unwrap()),
            self.dest_buffer.take().unwrap(),
            0,
            AES128_BLOCK_SIZE,
        ) {
            // Error, clear the appid and data
            self.aes.disable();
            self.appid.clear();
            self.source_buffer.replace(source.unwrap());
            self.dest_buffer.replace(dest);

            return e;
        }

        Ok(())
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

impl<'a, A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>>
    Client<'static> for AesDriver<'static, A>
{
    fn crypt_done(&'a self, source: Option<&'static mut [u8]>, destination: &'static mut [u8]) {
        self.source_buffer.replace(source.unwrap());
        self.dest_buffer.replace(destination);

        self.appid.map(|id| {
            self.apps
                .enter(*id, |app, upcalls| {
                    let mut data_len = 0;
                    let mut exit = false;
                    let mut static_buffer_len = 0;

                    self.dest_buffer.map(|buf| {
                        let ret = app.dest.mut_enter(|dest| {
                            let offset = self.data_copied.get() - 16;
                            let app_len = dest.len();
                            let static_len = buf.len();

                            if app_len < static_len {
                                dest[offset..(offset + app_len)].copy_from_slice(&buf[0..app_len]);
                            } else {
                                if offset + static_len <= app_len {
                                    dest[offset..(offset + static_len)].copy_from_slice(buf);
                                }
                            }
                        });

                        if let Err(e) = ret {
                            // No data buffer, clear the appid and data
                            self.aes.disable();
                            self.appid.clear();
                            upcalls.schedule_upcall(0, (e as usize, 0, 0)).ok();
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    self.source_buffer.map(|buf| {
                        let ret = app
                            .source
                            .enter(|source| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();
                                // Determine how much data we have already copied
                                let copied_data = self.data_copied.get();

                                data_len = source.len();

                                if data_len > copied_data {
                                    let remaining_data = &source[copied_data..];
                                    let remaining_len = data_len - copied_data;

                                    if remaining_len < static_buffer_len {
                                        remaining_data.copy_to_slice(&mut buf[..remaining_len]);
                                    } else {
                                        remaining_data[..static_buffer_len].copy_to_slice(buf);
                                    }
                                }
                                Ok(())
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE));

                        if let Err(e) = ret {
                            // No data buffer, clear the appid and data
                            self.aes.disable();
                            self.appid.clear();
                            upcalls.schedule_upcall(0, (e as usize, 0, 0)).ok();
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    if static_buffer_len > 0 {
                        let copied_data = self.data_copied.get();

                        if data_len > copied_data {
                            // Update the amount of data copied
                            self.data_copied.set(copied_data + static_buffer_len);

                            if self.calculate_output().is_err() {
                                // Error, clear the appid and data
                                self.aes.disable();
                                self.appid.clear();
                                self.check_queue();
                                return;
                            }

                            // Return as we don't want to run the digest yet
                            return;
                        }
                    }

                    // If we get here we have finished all the crypto operations
                    upcalls.schedule_upcall(0, (0, 0, 0)).ok();
                    self.data_copied.set(0);
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.appid.clear();
                    }
                })
        });
    }
}

impl<'a, A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>> CCMClient
    for AesDriver<'static, A>
{
    fn crypt_done(
        &self,
        _buf: &'static mut [u8],
        _res: Result<(), ErrorCode>,
        _tag_is_valid: bool,
    ) {
        unimplemented!();
    }
}

impl<'a, A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>> SyscallDriver
    for AesDriver<'static, A>
{
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass buffer for the destination to be in.
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.dest, &mut slice);
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
            // Pass buffer for the key to be in
            0 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.key, &mut slice);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // Pass buffer for the IV to be in
            1 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.iv, &mut slice);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // Pass buffer for the source to be in
            2 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.source, &mut slice);
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
        data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        let match_or_empty_or_nonexistant = self.appid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the HMAC.

            // If the HMAC is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the HMAC is not active, then
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

        let app_match = self.appid.map_or(false, |owning_app| {
            // We have recorded that an app has ownership of the HMAC.

            // If the HMAC is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the HMAC is not active, then
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
            // check if present
            0 => CommandReturn::success(),

            // set_algorithm
            1 => self
                .apps
                .enter(appid, |app, _| match data1 {
                    0 => {
                        app.aes_operation = Some(AesOperation::AES128Ctr(data2 != 0));
                        CommandReturn::success()
                    }
                    1 => {
                        app.aes_operation = Some(AesOperation::AES128CBC(data2 != 0));
                        CommandReturn::success()
                    }
                    2 => {
                        app.aes_operation = Some(AesOperation::AES128ECB(data2 != 0));
                        CommandReturn::success()
                    }
                    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                })
                .unwrap_or_else(|err| err.into()),

            // setup
            // Copy in the key and IV and run the first encryption operation
            // This will trigger a callback
            2 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.aes.disable();
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
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // crypt
            // Generate the encrypted output
            // Multiple calls to crypt will re-use the existing state
            // This will trigger a callback
            3 => {
                if app_match {
                    self.apps
                        .enter(appid, |app, upcalls| {
                            if let Err(e) = app
                                .source
                                .enter(|source| {
                                    let mut static_buffer_len = 0;
                                    self.source_buffer.map_or(Err(ErrorCode::NOMEM), |buf| {
                                        // Determine the size of the static buffer we have
                                        static_buffer_len = buf.len();

                                        if static_buffer_len > source.len() {
                                            static_buffer_len = source.len()
                                        }

                                        // Copy the data into the static buffer
                                        source[..static_buffer_len]
                                            .copy_to_slice(&mut buf[..static_buffer_len]);

                                        self.data_copied.set(static_buffer_len);

                                        Ok(())
                                    })?;

                                    if let Err(e) = self.calculate_output() {
                                        return Err(e);
                                    }
                                    Ok(())
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))
                            {
                                upcalls
                                    .schedule_upcall(
                                        0,
                                        (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                                    )
                                    .ok();
                            }
                        })
                        .unwrap();
                    CommandReturn::success()
                } else {
                    // We don't queue this request, the user has to call
                    // `setup` first.
                    CommandReturn::failure(ErrorCode::OFF)
                }
            }

            // Finish
            // Complete the operation and reset the AES
            // This will not trigger a callback and will not process any data from userspace
            4 => {
                if app_match {
                    self.apps
                        .enter(appid, |_app, _upcalls| {
                            self.aes.disable();
                            self.appid.clear();
                        })
                        .unwrap();
                    self.check_queue();
                    CommandReturn::success()
                } else {
                    // We don't queue this request, the user has to call
                    // `setup` first.
                    CommandReturn::failure(ErrorCode::OFF)
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

enum AesOperation {
    AES128Ctr(bool),
    AES128CBC(bool),
    AES128ECB(bool),
}

#[derive(Default)]
pub struct App {
    pending_run_app: Option<ProcessId>,
    aes_operation: Option<AesOperation>,
    key: ReadOnlyProcessBuffer,
    iv: ReadOnlyProcessBuffer,
    source: ReadOnlyProcessBuffer,
    dest: ReadWriteProcessBuffer,
}
