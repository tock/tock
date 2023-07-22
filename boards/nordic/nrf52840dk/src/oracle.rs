// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Encryption Oracle

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x99999;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::{AES128Ctr, Client, AES128, AES128_BLOCK_SIZE};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

static KEY: &'static [u8; kernel::hil::symmetric_encryption::AES128_KEY_SIZE] = b"InsecureAESKey12";

#[derive(Default)]
pub struct App {
    pending_run_app: bool,
    source_offset: usize,
    dest_offset: usize,
}

/// Ids for read-only allow buffers
mod ro_allow {
    pub const IV: usize = 0;
    pub const SOURCE: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const DEST: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

pub struct OracleDriver<'a, A: AES128<'a>> {
    aes: &'a A,

    apps: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,

    source_buffer: TakeCell<'static, [u8]>,
    dest_buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: AES128<'static> + AES128Ctr> OracleDriver<'static, A> {
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
    ) -> OracleDriver<'static, A> {
        OracleDriver {
            aes,
            apps: grant,
            current_process: OptionalCell::empty(),
            source_buffer: TakeCell::new(source_buffer),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    // Setup key, iv, and plaintext in underlying AES driver and initiate crypt
    // operation
    fn run(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(processid, |app, kernel_data| {
                self.aes.enable();
                self.aes.set_mode_aes128ctr(true)?;

                // set encryption key
                self.aes.set_key(KEY)?;

                kernel_data
                    .get_readonly_processbuffer(ro_allow::IV)
                    .and_then(|iv| {
                        iv.enter(|iv| {
                            let mut static_buf =
                                [0; kernel::hil::symmetric_encryption::AES128_KEY_SIZE];
                            // Determine the size of the static buffer we have
                            let copy_len = core::cmp::min(static_buf.len(), iv.len());

                            // Clear any previous iv
                            for c in static_buf.iter_mut() {
                                *c = 0;
                            }
                            // Copy the data into the static buffer
                            iv[..copy_len].copy_to_slice(&mut static_buf[..copy_len]);

                            AES128::set_iv(self.aes, &static_buf[..copy_len])
                        })
                        .map_err(Into::into)
                    })??;

                self.source_buffer
                    .map_or(Err(ErrorCode::NOMEM), |static_buf| {
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::SOURCE)
                            .and_then(|source| {
                                Ok(source.enter(|source| {
                                    // Determine the size of the static buffer we have
                                    let copy_len = core::cmp::min(
                                        static_buf.len(),
                                        source.len() - app.source_offset,
                                    );
                                    // Clear any previous plaintext
                                    for c in static_buf.iter_mut() {
                                        *c = 0;
                                    }
                                    // Copy the data into the static buffer
                                    source[app.source_offset..(copy_len + app.source_offset)]
                                        .copy_to_slice(&mut static_buf[..copy_len]);
                                    //AES128::set_iv(self.aes, static_buf)
                                }))
                            })
                            .map_err(Into::into)
                    })??;
                self.current_process.set(processid);
                self.initiate_next_segment()
            })
            .unwrap_or(Err(ErrorCode::RESERVE))
    }

    fn initiate_next_segment(&self) -> Result<(), ErrorCode> {
        if let Some((e, source, dest)) = AES128::crypt(
            self.aes,
            Some(
                self.source_buffer
                    .take()
                    .expect("Should never be called with empty source_buffer"),
            ),
            self.dest_buffer
                .take()
                .expect("Should never be called with empty dest_buffer"),
            0,
            AES128_BLOCK_SIZE,
        ) {
            // Error, clear the processid and data
            self.aes.disable();
            self.current_process.clear();
            if let Some(source_buf) = source {
                self.source_buffer.replace(source_buf);
            }
            self.dest_buffer.replace(dest);
            e
        } else {
            Ok(())
        }
    }

    fn check_queue(&self) {
        // If an app is already running let it complete
        if self.current_process.is_some() {
            return;
        }

        for appiter in self.apps.iter() {
            let processid = appiter.processid().clone();
            if appiter.enter(|app, _| app.pending_run_app) {
                if self.run(processid).is_ok() {
                    break;
                }
            }
        }
    }
}

impl<'a, A: AES128<'static> + AES128Ctr> Client<'static> for OracleDriver<'static, A> {
    fn crypt_done(&'a self, mut source: Option<&'static mut [u8]>, destination: &'static mut [u8]) {
        // One segment of encryption/decryption complete, move to next one or
        // callback to user if done
        let source = source.take().expect("source should never be None");

        self.current_process.map(|processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    let _ = kernel_data
                        .get_readwrite_processbuffer(rw_allow::DEST)
                        .and_then(|app_dest| {
                            app_dest.mut_enter(|app_dest| {
                                let remaining_copy_len = app_dest.len() - app.dest_offset;
                                let copy_len =
                                    core::cmp::min(remaining_copy_len, destination.len());
                                app_dest[app.dest_offset..(app.dest_offset + copy_len)]
                                    .copy_from_slice(&destination[..copy_len]);
                                app.dest_offset += copy_len;
                            })
                        });
                    self.dest_buffer.replace(destination);
                    let _ = kernel_data
                        .get_readonly_processbuffer(ro_allow::SOURCE)
                        .and_then(|app_source| {
                            app_source.enter(|app_source| {
                                if app.source_offset + source.len() < app_source.len() {
                                    let remaining_copy_len = app_source.len() - app.source_offset;
                                    let copy_len = core::cmp::min(remaining_copy_len, source.len());
                                    app_source[app.source_offset..(app.source_offset + copy_len)]
                                        .copy_to_slice(&mut source[..copy_len]);
                                    app.source_offset += copy_len;
                                    let _ = self.initiate_next_segment();
                                } else {
                                    // If we get here we have finished all the crypto operations
                                    app.pending_run_app = false;
                                    self.current_process.clear();
                                    kernel_data
                                        .schedule_upcall(0, (0, app_source.len(), 0))
                                        .ok();
                                }
                            })
                        });
                })
                .map_err(|_| {
                    self.current_process.clear();
                })
        });
        self.source_buffer.replace(source);
        self.check_queue();
    }
}

impl<'a, A: AES128<'static> + AES128Ctr> SyscallDriver for OracleDriver<'static, A> {
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let ret = self
            .apps
            .enter(processid, |app, _kernel_data| {
                match command_num {
                    // check if present
                    0 => CommandReturn::success(),

                    // crypt
                    // Copy in the key and IV and run the encryption operation
                    // This will trigger a callback
                    1 => {
                        // Some app is using the storage, we must wait.
                        if app.pending_run_app {
                            // No more room in the queue, nowhere to store this
                            // request.
                            CommandReturn::failure(ErrorCode::NOMEM)
                        } else {
                            // We can store this, so lets do it.
                            app.pending_run_app = true;
                            app.source_offset = 0;
                            app.dest_offset = 0;
                            CommandReturn::success()
                        }
                    }

                    // default
                    _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                }
            })
            .unwrap_or_else(|err| err.into());
        if command_num == 1 {
            self.check_queue();
        }
        ret
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
