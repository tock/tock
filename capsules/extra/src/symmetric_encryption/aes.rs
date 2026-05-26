// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! AES.

use capsules_core::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Aes as usize;

use core::cell::Cell;
use core::marker::PhantomData;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::{
    AESCtr, AESKeySize, CCMClient, Client, GCMClient, AES, AESCBC, AESCCM, AESECB, AESGCM,
    AES_BLOCK_SIZE,
};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Ids for read-only allow buffers
mod ro_allow {
    pub const KEY: usize = 0;
    pub const IV: usize = 1;
    pub const SOURCE: usize = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 3;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const DEST: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub struct AesDriver<'a, A, K>
where
    K: AESKeySize,
    A: AES<'a, K> + AESCCM<'static, K> + AESGCM<'static, K>,
{
    aes: &'a A,

    active: Cell<bool>,

    apps: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    processid: OptionalCell<ProcessId>,

    source_buffer: TakeCell<'static, [u8]>,
    data_copied: Cell<usize>,
    dest_buffer: TakeCell<'static, [u8]>,
    _phantom: PhantomData<K>,
}

impl<
        K: AESKeySize,
        A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    > AesDriver<'static, A, K>
{
    pub fn new(
        aes: &'static A,
        source_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<1>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> AesDriver<'static, A, K> {
        AesDriver {
            aes,
            active: Cell::new(false),
            apps: grant,
            processid: OptionalCell::empty(),
            source_buffer: TakeCell::new(source_buffer),
            data_copied: Cell::new(0),
            dest_buffer: TakeCell::new(dest_buffer),

            _phantom: PhantomData::<K>,
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.processid.map_or(Err(ErrorCode::RESERVE), |processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    self.aes.enable();
                    match app.aes_operation {
                        Some(AesOperation::AESCtr(encrypt)) => self.aes.set_mode_aesctr(encrypt)?,
                        Some(AesOperation::AESCBC(encrypt)) => self.aes.set_mode_aescbc(encrypt)?,
                        Some(AesOperation::AESECB(encrypt)) => self.aes.set_mode_aesecb(encrypt)?,
                        Some(AesOperation::AESCCM(_encrypt)) => {}
                        Some(AesOperation::AESGCM(_encrypt)) => {}
                        _ => return Err(ErrorCode::INVAL),
                    }

                    kernel_data
                        .get_readonly_processbuffer(ro_allow::KEY)
                        .and_then(|key| {
                            key.enter(|key| {
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

                                    if let Some(op) = app.aes_operation.as_ref() {
                                        match op {
                                            AesOperation::AESCtr(_)
                                            | AesOperation::AESCBC(_)
                                            | AesOperation::AESECB(_) => {
                                                AES::set_key(self.aes, buf)?;
                                                Ok(())
                                            }
                                            AesOperation::AESCCM(_) => {
                                                AESCCM::set_key(self.aes, buf)?;
                                                Ok(())
                                            }
                                            AesOperation::AESGCM(_) => {
                                                AESGCM::set_key(self.aes, buf)?;
                                                Ok(())
                                            }
                                        }
                                    } else {
                                        Err(ErrorCode::FAIL)
                                    }
                                })
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    kernel_data
                        .get_readonly_processbuffer(ro_allow::IV)
                        .and_then(|iv| {
                            iv.enter(|iv| {
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

                                    if let Some(op) = app.aes_operation.as_ref() {
                                        match op {
                                            AesOperation::AESCtr(_)
                                            | AesOperation::AESCBC(_)
                                            | AesOperation::AESECB(_) => {
                                                AES::set_iv(self.aes, buf)?;
                                                Ok(())
                                            }
                                            AesOperation::AESCCM(_) => {
                                                AESCCM::set_nonce(self.aes, &buf[0..13])?;
                                                Ok(())
                                            }
                                            AesOperation::AESGCM(_) => {
                                                AESGCM::set_iv(self.aes, &buf[0..13])?;
                                                Ok(())
                                            }
                                        }
                                    } else {
                                        Err(ErrorCode::FAIL)
                                    }
                                })
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    kernel_data
                        .get_readonly_processbuffer(ro_allow::SOURCE)
                        .and_then(|source| {
                            source.enter(|source| {
                                let mut static_buffer_len = 0;

                                if let Some(op) = app.aes_operation.as_ref() {
                                    match op {
                                        AesOperation::AESCtr(_)
                                        | AesOperation::AESCBC(_)
                                        | AesOperation::AESECB(_) => {
                                            self.source_buffer.map_or(
                                                Err(ErrorCode::NOMEM),
                                                |buf| {
                                                    // Determine the size of the static buffer we have
                                                    static_buffer_len = buf.len();

                                                    if static_buffer_len > source.len() {
                                                        static_buffer_len = source.len()
                                                    }

                                                    // Copy the data into the static buffer
                                                    source[..static_buffer_len].copy_to_slice(
                                                        &mut buf[..static_buffer_len],
                                                    );

                                                    self.data_copied.set(static_buffer_len);

                                                    Ok(())
                                                },
                                            )?;
                                        }
                                        AesOperation::AESCCM(_) => {
                                            self.dest_buffer.map_or(
                                                Err(ErrorCode::NOMEM),
                                                |buf| {
                                                    // Determine the size of the static buffer we have
                                                    static_buffer_len = buf.len();

                                                    if static_buffer_len > source.len() {
                                                        static_buffer_len = source.len()
                                                    }

                                                    // Copy the data into the static buffer
                                                    source[..static_buffer_len].copy_to_slice(
                                                        &mut buf[..static_buffer_len],
                                                    );

                                                    self.data_copied.set(static_buffer_len);

                                                    Ok(())
                                                },
                                            )?;
                                        }
                                        AesOperation::AESGCM(_) => {
                                            self.dest_buffer.map_or(
                                                Err(ErrorCode::NOMEM),
                                                |buf| {
                                                    // Determine the size of the static buffer we have
                                                    static_buffer_len = buf.len();

                                                    if static_buffer_len > source.len() {
                                                        static_buffer_len = source.len()
                                                    }

                                                    // Copy the data into the static buffer
                                                    source[..static_buffer_len].copy_to_slice(
                                                        &mut buf[..static_buffer_len],
                                                    );

                                                    self.data_copied.set(static_buffer_len);

                                                    Ok(())
                                                },
                                            )?;
                                        }
                                    }

                                    self.calculate_output(
                                        op,
                                        app.aoff.get(),
                                        app.moff.get(),
                                        app.mlen.get(),
                                        app.mic_len.get(),
                                        app.confidential.get(),
                                    )?;
                                    Ok(())
                                } else {
                                    Err(ErrorCode::FAIL)
                                }
                            })
                        })
                        .unwrap_or(Err(ErrorCode::RESERVE))?;

                    Ok(())
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn calculate_output(
        &self,
        op: &AesOperation,
        aoff: usize,
        moff: usize,
        mlen: usize,
        mic_len: usize,
        confidential: bool,
    ) -> Result<(), ErrorCode> {
        match op {
            AesOperation::AESCtr(_) | AesOperation::AESCBC(_) | AesOperation::AESECB(_) => {
                if let Some(dest_buf) = self.dest_buffer.take() {
                    if let Some((e, source, dest)) = AES::crypt(
                        self.aes,
                        self.source_buffer.take(),
                        dest_buf,
                        0,
                        AES_BLOCK_SIZE,
                    ) {
                        // Error, clear the processid and data
                        self.aes.disable();
                        self.processid.clear();
                        if let Some(source_buf) = source {
                            self.source_buffer.replace(source_buf);
                        }
                        self.dest_buffer.replace(dest);

                        return e;
                    }
                } else {
                    return Err(ErrorCode::FAIL);
                }
            }
            AesOperation::AESCCM(encrypting) => {
                if let Some(buf) = self.dest_buffer.take() {
                    if let Err((e, dest)) = AESCCM::crypt(
                        self.aes,
                        buf,
                        aoff,
                        moff,
                        mlen,
                        mic_len,
                        confidential,
                        *encrypting,
                    ) {
                        // Error, clear the processid and data
                        self.aes.disable();
                        self.processid.clear();
                        self.dest_buffer.replace(dest);

                        return Err(e);
                    }
                } else {
                    return Err(ErrorCode::FAIL);
                }
            }
            AesOperation::AESGCM(encrypting) => {
                if let Some(buf) = self.dest_buffer.take() {
                    if let Err((e, dest)) =
                        AESGCM::crypt(self.aes, buf, aoff, moff, mlen, mic_len, *encrypting)
                    {
                        // Error, clear the appid and data
                        self.aes.disable();
                        self.processid.clear();
                        self.dest_buffer.replace(dest);

                        return Err(e);
                    }
                } else {
                    return Err(ErrorCode::FAIL);
                }
            }
        }

        Ok(())
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let started_command = appiter.enter(|app, _| {
                // If an app is already running let it complete
                if self.processid.is_some() {
                    return true;
                }

                // If this app has a pending command let's use it.
                app.pending_run_app.take().is_some_and(|processid| {
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

impl<
        K: AESKeySize,
        A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    > Client<'static> for AesDriver<'static, A, K>
{
    fn crypt_done(&self, source: Option<&'static mut [u8]>, destination: &'static mut [u8]) {
        if let Some(source_buf) = source {
            self.source_buffer.replace(source_buf);
        }
        self.dest_buffer.replace(destination);

        self.processid.map(|id| {
            self.apps
                .enter(id, |app, kernel_data| {
                    let mut data_len = 0;
                    let mut exit = false;
                    let mut static_buffer_len = 0;

                    let source_len = kernel_data
                        .get_readonly_processbuffer(ro_allow::SOURCE)
                        .map_or(0, |source| source.len());

                    let subtract = self
                        .source_buffer
                        .map_or(0, |buf| core::cmp::min(buf.len(), source_len));

                    self.dest_buffer.map(|buf| {
                        let ret = kernel_data
                            .get_readwrite_processbuffer(rw_allow::DEST)
                            .and_then(|dest| {
                                dest.mut_enter(|dest| {
                                    let offset = self.data_copied.get() - subtract;
                                    let app_len = dest.len();
                                    let static_len = self.source_buffer.map_or(0, |source_buf| {
                                        core::cmp::min(source_buf.len(), buf.len())
                                    });

                                    if app_len < static_len {
                                        if app_len - offset > 0 {
                                            dest[offset..app_len]
                                                .copy_from_slice(&buf[0..(app_len - offset)]);
                                        }
                                    } else {
                                        if offset + static_len <= app_len {
                                            dest[offset..(offset + static_len)]
                                                .copy_from_slice(&buf[0..static_len]);
                                        }
                                    }
                                })
                            });

                        if let Err(e) = ret {
                            // No data buffer, clear the processid and data
                            self.aes.disable();
                            self.processid.clear();
                            let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    self.source_buffer.map(|buf| {
                        let ret = kernel_data
                            .get_readonly_processbuffer(ro_allow::SOURCE)
                            .and_then(|source| {
                                source.enter(|source| {
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
                            })
                            .unwrap_or(Err(ErrorCode::RESERVE));

                        if let Err(e) = ret {
                            // No data buffer, clear the processid and data
                            self.aes.disable();
                            self.processid.clear();
                            let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
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

                            if let Some(op) = app.aes_operation.as_ref() {
                                if self
                                    .calculate_output(
                                        op,
                                        app.aoff.get(),
                                        app.moff.get(),
                                        app.mlen.get(),
                                        app.mic_len.get(),
                                        app.confidential.get(),
                                    )
                                    .is_err()
                                {
                                    // Error, clear the processid and data
                                    self.aes.disable();
                                    self.processid.clear();
                                    self.check_queue();
                                    return;
                                }
                            }

                            // Return as we don't want to run the digest yet
                            return;
                        }
                    }

                    // If we get here we have finished all the crypto operations
                    let _ = kernel_data.schedule_upcall(0, (0, self.data_copied.get(), 0));
                    self.data_copied.set(0);
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.processid.clear();
                    }
                })
        });
    }
}

impl<
        K: AESKeySize,
        A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    > CCMClient for AesDriver<'static, A, K>
{
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.dest_buffer.replace(buf);

        self.processid.map(|id| {
            self.apps
                .enter(id, |_, kernel_data| {
                    let mut exit = false;

                    if let Err(e) = res {
                        let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
                        return;
                    }

                    self.dest_buffer.map(|buf| {
                        let ret = kernel_data
                            .get_readwrite_processbuffer(rw_allow::DEST)
                            .and_then(|dest| {
                                dest.mut_enter(|dest| {
                                    let offset = self.data_copied.get()
                                        - (core::cmp::min(buf.len(), dest.len()));
                                    let app_len = dest.len();
                                    let static_len = buf.len();

                                    if app_len < static_len {
                                        if app_len - offset > 0 {
                                            dest[offset..app_len]
                                                .copy_from_slice(&buf[0..(app_len - offset)]);
                                        }
                                    } else {
                                        if offset + static_len <= app_len {
                                            dest[offset..(offset + static_len)]
                                                .copy_from_slice(&buf[0..static_len]);
                                        }
                                    }
                                })
                            });

                        if let Err(e) = ret {
                            // No data buffer, clear the processid and data
                            self.aes.disable();
                            self.processid.clear();
                            let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    // AES CCM is online only we can't send any more data in, so
                    // just report what we did to the app.
                    let _ = kernel_data
                        .schedule_upcall(0, (0, self.data_copied.get(), tag_is_valid as usize));
                    self.data_copied.set(0);
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.processid.clear();
                    }
                })
        });
    }
}

impl<
        K: AESKeySize,
        A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    > GCMClient for AesDriver<'static, A, K>
{
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.dest_buffer.replace(buf);

        self.processid.map(|id| {
            self.apps
                .enter(id, |_, kernel_data| {
                    let mut exit = false;

                    if let Err(e) = res {
                        let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
                        return;
                    }

                    self.dest_buffer.map(|buf| {
                        let ret = kernel_data
                            .get_readwrite_processbuffer(rw_allow::DEST)
                            .and_then(|dest| {
                                dest.mut_enter(|dest| {
                                    let offset = self.data_copied.get()
                                        - (core::cmp::min(buf.len(), dest.len()));
                                    let app_len = dest.len();
                                    let static_len = buf.len();

                                    if app_len < static_len {
                                        if app_len - offset > 0 {
                                            dest[offset..app_len]
                                                .copy_from_slice(&buf[0..(app_len - offset)]);
                                        }
                                    } else {
                                        if offset + static_len <= app_len {
                                            dest[offset..(offset + static_len)]
                                                .copy_from_slice(&buf[0..static_len]);
                                        }
                                    }
                                })
                            });

                        if let Err(e) = ret {
                            // No data buffer, clear the appid and data
                            self.aes.disable();
                            self.processid.clear();
                            let _ = kernel_data.schedule_upcall(0, (e as usize, 0, 0));
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    // AES GCM is online only we can't send any more data in, so
                    // just report what we did to the app.
                    let _ = kernel_data
                        .schedule_upcall(0, (0, self.data_copied.get(), tag_is_valid as usize));
                    self.data_copied.set(0);
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.processid.clear();
                    }
                })
        });
    }
}

impl<
        K: AESKeySize,
        A: AES<'static, K> + AESCtr + AESCBC + AESECB + AESCCM<'static, K> + AESGCM<'static, K>,
    > SyscallDriver for AesDriver<'static, A, K>
{
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let match_or_empty_or_nonexistant = self.processid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the HMAC.

            // If the HMAC is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the HMAC is not active, then
            // we need to verify that that application still exists, and remove
            // it as owner if not.
            if self.active.get() {
                owning_app == processid
            } else {
                // Check the app still exists.
                //
                // If the `.enter()` succeeds, then the app is still valid, and
                // we can check if the owning app matches the one that called
                // the command. If the `.enter()` fails, then the owning app no
                // longer exists and we return `true` to signify the
                // "or_nonexistant" case.
                self.apps
                    .enter(owning_app, |_, _| owning_app == processid)
                    .unwrap_or(true)
            }
        });

        let app_match = self.processid.map_or(false, |owning_app| {
            // We have recorded that an app has ownership of the HMAC.

            // If the HMAC is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the HMAC is not active, then
            // we need to verify that that application still exists, and remove
            // it as owner if not.
            if self.active.get() {
                owning_app == processid
            } else {
                // Check the app still exists.
                //
                // If the `.enter()` succeeds, then the app is still valid, and
                // we can check if the owning app matches the one that called
                // the command. If the `.enter()` fails, then the owning app no
                // longer exists and we return `true` to signify the
                // "or_nonexistant" case.
                self.apps
                    .enter(owning_app, |_, _| owning_app == processid)
                    .unwrap_or(true)
            }
        });

        // Try the commands where we want to start an operation *not* entered in
        // an app grant first.
        if match_or_empty_or_nonexistant && command_num == 2 {
            self.processid.set(processid);
            let ret = self.run();

            return if let Err(e) = ret {
                self.aes.disable();
                self.processid.clear();
                self.check_queue();
                CommandReturn::failure(e)
            } else {
                CommandReturn::success()
            };
        }

        let ret = self
            .apps
            .enter(processid, |app, kernel_data| {
                match command_num {
                    // check if present
                    0 => CommandReturn::success(),

                    // set_algorithm
                    1 => match data1 {
                        0 => {
                            app.aes_operation = Some(AesOperation::AESCtr(data2 != 0));
                            CommandReturn::success()
                        }
                        1 => {
                            app.aes_operation = Some(AesOperation::AESCBC(data2 != 0));
                            CommandReturn::success()
                        }
                        2 => {
                            app.aes_operation = Some(AesOperation::AESECB(data2 != 0));
                            CommandReturn::success()
                        }
                        3 => {
                            app.aes_operation = Some(AesOperation::AESCCM(data2 != 0));
                            CommandReturn::success()
                        }
                        4 => {
                            app.aes_operation = Some(AesOperation::AESGCM(data2 != 0));
                            CommandReturn::success()
                        }
                        _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                    },

                    // setup
                    // Copy in the key and IV and run the first encryption operation
                    // This will trigger a callback
                    2 => {
                        // Some app is using the storage, we must wait.
                        if app.pending_run_app.is_some() {
                            // No more room in the queue, nowhere to store this
                            // request.
                            CommandReturn::failure(ErrorCode::NOMEM)
                        } else {
                            // We can store this, so lets do it.
                            app.pending_run_app = Some(processid);
                            CommandReturn::success()
                        }
                    }

                    // crypt
                    // Generate the encrypted output
                    // Multiple calls to crypt will re-use the existing state
                    // This will trigger a callback
                    3 => {
                        if app_match {
                            if let Err(e) = kernel_data
                                .get_readonly_processbuffer(ro_allow::SOURCE)
                                .and_then(|source| {
                                    source.enter(|source| {
                                        let mut static_buffer_len = 0;
                                        self.source_buffer.map_or(
                                            Err(ErrorCode::NOMEM),
                                            |buf| {
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
                                            },
                                        )?;

                                        if let Some(op) = app.aes_operation.as_ref() {
                                            self.calculate_output(
                                                op,
                                                app.aoff.get(),
                                                app.moff.get(),
                                                app.mlen.get(),
                                                app.mic_len.get(),
                                                app.confidential.get(),
                                            )?;
                                            Ok(())
                                        } else {
                                            Err(ErrorCode::FAIL)
                                        }
                                    })
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))
                            {
                                let _ = kernel_data.schedule_upcall(
                                    0,
                                    (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                                );
                            }
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
                            self.aes.disable();
                            self.processid.clear();

                            CommandReturn::success()
                        } else {
                            // We don't queue this request, the user has to call
                            // `setup` first.
                            CommandReturn::failure(ErrorCode::OFF)
                        }
                    }

                    // Set aoff for CCM
                    // This will not trigger a callback and will not process any data from userspace
                    5 => {
                        app.aoff.set(data1);
                        CommandReturn::success()
                    }

                    // Set moff for CCM
                    // This will not trigger a callback and will not process any data from userspace
                    6 => {
                        app.moff.set(data1);
                        CommandReturn::success()
                    }

                    // Set mic_len for CCM
                    // This will not trigger a callback and will not process any data from userspace
                    7 => {
                        app.mic_len.set(data1);
                        CommandReturn::success()
                    }

                    // Set confidential boolean for CCM
                    // This will not trigger a callback and will not process any data from userspace
                    8 => {
                        app.confidential.set(data1 > 0);
                        CommandReturn::success()
                    }

                    // default
                    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                }
            })
            .unwrap_or_else(|err| err.into());

        if command_num == 4
            || command_num == 5
            || command_num == 6
            || command_num == 7
            || command_num == 8
        {
            self.check_queue();
        }

        ret
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

enum AesOperation {
    AESCtr(bool),
    AESCBC(bool),
    AESECB(bool),
    AESCCM(bool),
    AESGCM(bool),
}

#[derive(Default)]
pub struct App {
    pending_run_app: Option<ProcessId>,
    aes_operation: Option<AesOperation>,

    aoff: Cell<usize>,
    moff: Cell<usize>,
    mlen: Cell<usize>,
    mic_len: Cell<usize>,
    confidential: Cell<bool>,
}
