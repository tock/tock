//! AES.

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Aes as usize;

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::{
    AES128Ctr, CCMClient, Client, AES128, AES128CBC, AES128CCM, AES128ECB, AES128_BLOCK_SIZE,
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

pub struct AesDriver<'a, A: AES128<'a> + AES128CCM<'static>> {
    aes: &'a A,

    active: Cell<bool>,

    apps: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
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
        grant: Grant<
            App,
            UpcallCount<1>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
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
                .enter(*appid, |app, kernel_data| {
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
                            AesOperation::AES128CCM(_encrypt) => Ok(()),
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    };
                    if ret.is_err() {
                        return ret;
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
                                            AesOperation::AES128Ctr(_)
                                            | AesOperation::AES128CBC(_)
                                            | AesOperation::AES128ECB(_) => {
                                                if let Err(e) = AES128::set_key(self.aes, buf) {
                                                    return Err(e);
                                                }
                                                Ok(())
                                            }
                                            AesOperation::AES128CCM(_) => {
                                                if let Err(e) = AES128CCM::set_key(self.aes, buf) {
                                                    return Err(e);
                                                }
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
                                            AesOperation::AES128Ctr(_)
                                            | AesOperation::AES128CBC(_)
                                            | AesOperation::AES128ECB(_) => {
                                                if let Err(e) = self.aes.set_iv(buf) {
                                                    return Err(e);
                                                }
                                                Ok(())
                                            }
                                            AesOperation::AES128CCM(_) => {
                                                if let Err(e) = self.aes.set_nonce(&buf[0..13]) {
                                                    return Err(e);
                                                }
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
                                        AesOperation::AES128Ctr(_)
                                        | AesOperation::AES128CBC(_)
                                        | AesOperation::AES128ECB(_) => {
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
                                        AesOperation::AES128CCM(_) => {
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

                                    if let Err(e) = self.calculate_output(
                                        op,
                                        app.aoff.get(),
                                        app.moff.get(),
                                        app.mlen.get(),
                                        app.mic_len.get(),
                                        app.confidential.get(),
                                    ) {
                                        return Err(e);
                                    }
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
            AesOperation::AES128Ctr(_)
            | AesOperation::AES128CBC(_)
            | AesOperation::AES128ECB(_) => {
                if let Some(dest_buf) = self.dest_buffer.take() {
                    if let Some((e, source, dest)) = AES128::crypt(
                        self.aes,
                        self.source_buffer.take(),
                        dest_buf,
                        0,
                        AES128_BLOCK_SIZE,
                    ) {
                        // Error, clear the appid and data
                        self.aes.disable();
                        self.appid.clear();
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
            AesOperation::AES128CCM(encrypting) => {
                if let Some(buf) = self.dest_buffer.take() {
                    if let Err((e, dest)) = AES128CCM::crypt(
                        self.aes,
                        buf,
                        aoff,
                        moff,
                        mlen,
                        mic_len,
                        confidential,
                        *encrypting,
                    ) {
                        // Error, clear the appid and data
                        self.aes.disable();
                        self.appid.clear();
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
        if let Some(source_buf) = source {
            self.source_buffer.replace(source_buf);
        }
        self.dest_buffer.replace(destination);

        self.appid.map(|id| {
            self.apps
                .enter(*id, |app, kernel_data| {
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
                            // No data buffer, clear the appid and data
                            self.aes.disable();
                            self.appid.clear();
                            kernel_data.schedule_upcall(0, (e as usize, 0, 0)).ok();
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
                            // No data buffer, clear the appid and data
                            self.aes.disable();
                            self.appid.clear();
                            kernel_data.schedule_upcall(0, (e as usize, 0, 0)).ok();
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
                                    // Error, clear the appid and data
                                    self.aes.disable();
                                    self.appid.clear();
                                    self.check_queue();
                                    return;
                                }
                            }

                            // Return as we don't want to run the digest yet
                            return;
                        }
                    }

                    // If we get here we have finished all the crypto operations
                    kernel_data
                        .schedule_upcall(0, (0, self.data_copied.get(), 0))
                        .ok();
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
    fn crypt_done(&self, buf: &'static mut [u8], res: Result<(), ErrorCode>, tag_is_valid: bool) {
        self.dest_buffer.replace(buf);

        self.appid.map(|id| {
            self.apps
                .enter(*id, |_, kernel_data| {
                    let mut exit = false;

                    if let Err(e) = res {
                        kernel_data.schedule_upcall(0, (e as usize, 0, 0)).ok();
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
                            self.appid.clear();
                            kernel_data.schedule_upcall(0, (e as usize, 0, 0)).ok();
                            exit = true;
                        }
                    });

                    if exit {
                        return;
                    }

                    // AES CCM is online only we can't send any more data in, so
                    // just report what we did to the app.
                    kernel_data
                        .schedule_upcall(0, (0, self.data_copied.get(), tag_is_valid as usize))
                        .ok();
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

impl<'a, A: AES128<'static> + AES128Ctr + AES128CBC + AES128ECB + AES128CCM<'static>> SyscallDriver
    for AesDriver<'static, A>
{
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

        // Try the commands where we want to start an operation *not* entered in
        // an app grant first.
        if match_or_empty_or_nonexistant && command_num == 2 {
            self.appid.set(appid);
            let ret = self.run();

            return if let Err(e) = ret {
                self.aes.disable();
                self.appid.clear();
                self.check_queue();
                CommandReturn::failure(e)
            } else {
                CommandReturn::success()
            };
        }

        let ret = self
            .apps
            .enter(appid, |app, kernel_data| {
                match command_num {
                    // check if present
                    0 => CommandReturn::success(),

                    // set_algorithm
                    1 => match data1 {
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
                        3 => {
                            app.aes_operation = Some(AesOperation::AES128CCM(data2 != 0));
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
                            app.pending_run_app = Some(appid);
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
                                            if let Err(e) = self.calculate_output(
                                                op,
                                                app.aoff.get(),
                                                app.moff.get(),
                                                app.mlen.get(),
                                                app.mic_len.get(),
                                                app.confidential.get(),
                                            ) {
                                                return Err(e);
                                            }
                                            Ok(())
                                        } else {
                                            Err(ErrorCode::FAIL)
                                        }
                                    })
                                })
                                .unwrap_or(Err(ErrorCode::RESERVE))
                            {
                                kernel_data
                                    .schedule_upcall(
                                        0,
                                        (kernel::errorcode::into_statuscode(e.into()), 0, 0),
                                    )
                                    .ok();
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
                            self.appid.clear();

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
    AES128Ctr(bool),
    AES128CBC(bool),
    AES128ECB(bool),
    AES128CCM(bool),
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
