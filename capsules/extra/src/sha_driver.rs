// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SHA Userspace Driver

use capsules_core::driver;
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::digest;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Sha as usize;

/// Upcalls for SHA operations completing.
mod upcall {
    pub const HASH: usize = 0;
    pub const VERIFY: usize = 1;
    pub const COUNT: u8 = 2;
}

/// Ids for read-only allow buffers
mod ro_allow {
    pub const DATA: usize = 0;
    pub const COMPARE: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const DEST: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Copy, Clone, PartialEq)]
enum AppOp {
    Hash,
    Verify,
}

#[derive(Default)]
pub struct App {
    sha_algorithm: ShaAlgorithm,
    operation: OptionalCell<AppOp>,
    data_offset: usize,
}

#[derive(Default)]
enum ShaAlgorithm {
    #[default]
    Sha256,
    // Sha384,
    // Sha512,
}

pub struct ShaDriver<'a, H: digest::Digest<'a, DIGEST_LEN>, const DIGEST_LEN: usize> {
    /// Underlying hasher to use for the SHA operations.
    sha: &'a H,

    /// Virtualized capsule that supports a single operation per app.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    /// The process currently using the SHA hasher.
    processid: OptionalCell<ProcessId>,

    /// Buffer to hold the data we are copying to the SHA hasher.
    data_buffer: TakeCell<'static, [u8]>,

    /// Buffer to hold the output of the SHA hasher, or a hash to compare for a
    /// verify operation.
    dest_buffer: TakeCell<'static, [u8; DIGEST_LEN]>,
}

impl<'a, H: digest::Digest<'a, DIGEST_LEN> + digest::Sha256, const DIGEST_LEN: usize>
    ShaDriver<'a, H, DIGEST_LEN>
{
    pub fn new(
        sha: &'a H,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8; DIGEST_LEN],
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> ShaDriver<'a, H, DIGEST_LEN> {
        ShaDriver {
            sha,
            apps: grant,
            processid: OptionalCell::empty(),
            data_buffer: TakeCell::new(data_buffer),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    fn run(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Save this process as the active process.
        self.processid.set(processid);

        self.apps
            .enter(processid, |app, kernel_data| {
                // First, set the operation of the underlying hasher.
                match app.sha_algorithm {
                    ShaAlgorithm::Sha256 => self.sha.set_mode_sha256()?,
                    // ShaAlgorithm::Sha384 => self.sha.set_mode_sha384()?,
                    // ShaAlgorithm::Sha512 => self.sha.set_mode_sha512()?,
                }

                // Now, start copying data from the allowed buffer into our `data_buffer`
                // and then share that data with the underlying hasher.
                kernel_data
                    .get_readonly_processbuffer(ro_allow::DATA)
                    .and_then(|data| {
                        data.enter(|data| {
                            self.data_buffer.take().map_or(Err(ErrorCode::FAIL), |buf| {
                                // Copy as much data as we have or as much as we can fit in our
                                // kernel buffer.
                                let copy_len = core::cmp::min(data.len(), buf.len());
                                let _ =
                                    data[0..copy_len].copy_to_slice_or_err(&mut buf[0..copy_len]);

                                // Save how far into the buffer we are.
                                app.data_offset = copy_len;

                                // Add data to the hasher.
                                let mut lease_buf = SubSliceMut::new(buf);
                                lease_buf.slice(0..copy_len);
                                if let Err((e, buf)) = self.sha.add_mut_data(lease_buf) {
                                    self.data_buffer.replace(buf.take());
                                    Err(e)
                                } else {
                                    Ok(())
                                }
                            })
                        })
                    })
                    .unwrap_or(Err(ErrorCode::RESERVE))
            })
            .unwrap_or_else(|err| Err(err.into()))
    }

    fn check_queue(&self) -> Result<(), ErrorCode> {
        // Check if there is already something using the SHA hasher.
        if self.processid.is_some() {
            // Something is using the hasher. That is fine, we have nothing to do,
            // pending operations will run later.
            Ok(())
        } else {
            let ready_app = self.apps.iter().find_map(|appiter| {
                let possible_process = appiter.processid();
                let ready = appiter.enter(|app, _| app.operation.is_some());
                if ready {
                    Some(possible_process)
                } else {
                    None
                }
            });

            if let Some(ready_app) = ready_app {
                self.run(ready_app)
            } else {
                // Nothing to do
                Ok(())
            }
        }
    }

    // Check queue, but instead of returning an error, trigger an upcall.
    fn check_queue_async(&self) {
        if let Err(e) = self.check_queue() {
            self.processid.take().map(|processid| {
                let _ = self.apps.enter(processid, |app, kernel_data| {
                    let upcall_num = match app.operation.get() {
                        Some(AppOp::Hash) | None => upcall::HASH,
                        Some(AppOp::Verify) => upcall::VERIFY,
                    };
                    app.operation.clear();

                    let _ =
                        kernel_data.schedule_upcall(upcall_num, (into_statuscode(e.into()), 0, 0));
                });
            });
        }
    }
}

impl<'a, H: digest::Digest<'a, DIGEST_LEN> + digest::Sha256, const DIGEST_LEN: usize>
    digest::ClientData<DIGEST_LEN> for ShaDriver<'a, H, DIGEST_LEN>
{
    // Because data needs to be copied from a userspace buffer into a kernel (RAM) one,
    // we always pass mut data; this callback should never be invoked.
    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {}

    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        // Unconditionally return our kernel buffer.
        self.data_buffer.replace(data.take());

        // Continue with the active process. If there is more data to add, do that.
        // If all data has been added, then do the requested operation.
        self.processid.map(|processid| {
            self.apps
                .enter(processid, |app, kernel_data| {
                    // Check if we have more data to copy.
                    let res = kernel_data
                        .get_readonly_processbuffer(ro_allow::DATA)
                        .and_then(|data| {
                            data.enter(|data| {
                                let remaining = data.len() - app.data_offset;

                                if remaining > 0 {
                                    // More data to add.
                                    self.data_buffer.take().map_or(Err(ErrorCode::FAIL), |buf| {
                                        let copy_len = core::cmp::min(remaining, buf.len());
                                        let src_start = app.data_offset;
                                        let src_end = src_start + copy_len;

                                        let _ = data[src_start..src_end]
                                            .copy_to_slice_or_err(&mut buf[0..copy_len]);

                                        // Save how far into the buffer we are.
                                        app.data_offset = src_end;

                                        // Add data to the hasher.
                                        let mut lease_buf = SubSliceMut::new(buf);
                                        lease_buf.slice(0..copy_len);
                                        self.sha.add_mut_data(lease_buf).and(Ok(true)).map_err(
                                            |(e, buf)| {
                                                self.sha.clear_data();
                                                self.processid.clear();
                                                self.data_buffer.replace(buf.take());
                                                e
                                            },
                                        )
                                    })
                                } else {
                                    Ok(false)
                                }
                            })
                        })
                        .unwrap_or_else(|err| err.into());

                    // If we did have more data to copy, we will get `Ok(true)` and we
                    // have nothing more to do. If we did not have more data to copy, we
                    // will get `Ok(false)` and can move to the hash or verify
                    // operation. If we got an error, we do an upcall to the app.
                    let _ = match res {
                        Ok(false) => {
                            match app.operation.get() {
                                Some(AppOp::Hash) => {
                                    // No more data to copy. Run the hash.
                                    self.dest_buffer.take().map_or(Err(ErrorCode::FAIL), |buf| {
                                        self.sha.run(buf).map_err(|(e, buf)| {
                                            // Error, clear the processid and data
                                            self.sha.clear_data();
                                            self.processid.clear();
                                            self.dest_buffer.replace(buf);
                                            e
                                        })
                                    })
                                }
                                Some(AppOp::Verify) => {
                                    // Copy to compare buffer.
                                    kernel_data
                                        .get_readonly_processbuffer(ro_allow::COMPARE)
                                        .and_then(|compare| {
                                            compare.enter(|compare| {
                                                if compare.len() == DIGEST_LEN {
                                                    self.dest_buffer.take().map_or(
                                                        Err(ErrorCode::FAIL),
                                                        |buf| {
                                                            let _ =
                                                                compare.copy_to_slice_or_err(buf);

                                                            self.sha.verify(buf).map_err(
                                                                |(e, buf)| {
                                                                    // Error, clear the processid and data
                                                                    self.sha.clear_data();
                                                                    self.processid.clear();
                                                                    self.dest_buffer.replace(buf);
                                                                    e
                                                                },
                                                            )
                                                        },
                                                    )
                                                } else {
                                                    Err(ErrorCode::NOMEM)
                                                }
                                            })
                                        })
                                        .unwrap_or_else(|err| err.into())
                                }

                                _ => Ok(()),
                            }
                        }
                        Ok(true) => Ok(()),
                        Err(e) => Err(e),
                    };
                    if let Err(e) = res {
                        // Notify the process.
                        let upcall_num = match app.operation.get() {
                            Some(AppOp::Hash) | None => upcall::HASH,
                            Some(AppOp::Verify) => upcall::VERIFY,
                        };
                        let _ = kernel_data
                            .schedule_upcall(upcall_num, (into_statuscode(e.into()), 0, 0));
                    }
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.sha.clear_data();
                        self.processid.clear();
                    }
                })
        });

        // Check for more work to do.
        self.check_queue_async();
    }
}

impl<'a, H: digest::Digest<'a, DIGEST_LEN> + digest::Sha256, const DIGEST_LEN: usize>
    digest::ClientHash<DIGEST_LEN> for ShaDriver<'a, H, DIGEST_LEN>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; DIGEST_LEN]) {
        // Clear the underlying hasher.
        self.sha.clear_data();

        // Do our best to copy the digest to the app.
        //
        // If the app is gone, or didn't give us a `DIGEST_LEN` buffer, we won't
        // be able to copy the buffer. If the app still exists it will get an
        // upcall either way.
        self.processid.map(|processid| {
            let _ = self.apps.enter(processid, |app, kernel_data| {
                // Mark app operation as completed.
                app.operation.clear();

                let res = result.and_then(|()| {
                    // Do our best to copy to the app's buffer. The app MUST have given
                    // us a `DIGEST_LEN` length buffer to copy to. If not, the app won't
                    // get the digest.
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::DEST)
                        .and_then(|dest| {
                            dest.mut_enter(|dest| {
                                if dest.len() == DIGEST_LEN {
                                    let _ = dest.copy_from_slice_or_err(digest);
                                    Ok(())
                                } else {
                                    Err(ErrorCode::NOMEM)
                                }
                            })
                        })
                        .unwrap_or_else(|err| err.into())
                });

                // Notify the app the operation has finished.
                let _ = kernel_data.schedule_upcall(upcall::HASH, (into_statuscode(res), 0, 0));
            });
        });

        // Unconditionally clear the current app. Either, the app still exists
        // and we did the upcall, or the app is gone and we need to reset.
        self.processid.clear();

        // Be sure to replace our buffer.
        self.dest_buffer.replace(digest);

        // Check for more work to do.
        self.check_queue_async();
    }
}

impl<'a, H: digest::Digest<'a, DIGEST_LEN> + digest::Sha256, const DIGEST_LEN: usize>
    digest::ClientVerify<DIGEST_LEN> for ShaDriver<'a, H, DIGEST_LEN>
{
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        compare: &'static mut [u8; DIGEST_LEN],
    ) {
        // Clear the underlying hasher.
        self.sha.clear_data();

        // Notify the app
        self.processid.map(|processid| {
            let _ = self.apps.enter(processid, |app, kernel_data| {
                // Mark app operation as completed.
                app.operation.clear();

                // Notify the app the operation has finished.
                let arg = match result {
                    Ok(equal) => (into_statuscode(Ok(())), equal as usize, 0),
                    Err(e) => (into_statuscode(e.into()), 0, 0),
                };
                let _ = kernel_data.schedule_upcall(upcall::VERIFY, arg);
            });
        });

        // Unconditionally clear the current app. Either, the app still exists
        // and we did the upcall, or the app is gone and we need to reset.
        self.processid.clear();

        // Be sure to replace our buffer.
        self.dest_buffer.replace(compare);

        // Check for more work to do.
        if let Err(e) = self.check_queue() {
            self.processid.take().map(|processid| {
                let _ = self.apps.enter(processid, |app, kernel_data| {
                    let upcall_num = match app.operation.get() {
                        Some(AppOp::Hash) | None => upcall::HASH,
                        Some(AppOp::Verify) => upcall::VERIFY,
                    };
                    app.operation.clear();

                    let _ =
                        kernel_data.schedule_upcall(upcall_num, (into_statuscode(e.into()), 0, 0));
                });
            });
        }
    }
}

impl<'a, H: digest::Digest<'a, DIGEST_LEN> + digest::Sha256, const DIGEST_LEN: usize> SyscallDriver
    for ShaDriver<'a, H, DIGEST_LEN>
{
    /// Setup and run a SHA hash.
    ///
    /// We expect userspace to setup buffers for the data, and either the
    /// generated hash or a hash to compare with. These buffers must be
    /// allocated and specified to the kernel with allow calls.
    ///
    /// We expect userspace not to change the value while running. If userspace
    /// changes the value we have no guarantee of what is passed to the
    /// hardware. This isn't a security issue, it will just prove the requesting
    /// app with invalid data.
    ///
    /// The driver will take care of clearing data from the underlying
    /// implementation by calling the `clear_data()` function when the
    /// `hash_complete()` callback is called or if an error is encountered.
    ///
    /// ### `command_num`
    ///
    /// - `0`: driver check
    /// - `1`: set_algorithm
    /// - `2`: hash
    /// - `3`: verify
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check if present
            0 => CommandReturn::success(),

            // set_algorithm
            1 => {
                self.apps
                    .enter(processid, |app, _kernel_data| {
                        match data1 {
                            // SHA256
                            0 => {
                                app.sha_algorithm = ShaAlgorithm::Sha256;
                                CommandReturn::success()
                            }
                            // // SHA384
                            // 1 => {
                            //     app.sha_algorithm = ShaAlgorithm::Sha384;
                            //     CommandReturn::success()
                            // }
                            // // SHA512
                            // 2 => {
                            //     app.sha_algorithm = ShaAlgorithm::Sha512;
                            //     CommandReturn::success()
                            // }
                            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // hash
            2 => {
                let res = self.apps.enter(processid, |app, _kernel_data| {
                    if app.operation.is_some() {
                        // No more room in the queue, nowhere to store this request.
                        Err(ErrorCode::NOMEM)
                    } else {
                        app.operation.set(AppOp::Hash);
                        Ok(())
                    }
                });
                match res {
                    Ok(_) => {
                        // If we were able to enqueue the operation, check if we can
                        // actually run it. If there was an error starting it return the
                        // error, otherwise return ok if the operation started successfully
                        // or was queued for later. This also ensures we are not already in
                        // the grant.
                        self.check_queue()
                            .inspect_err(|_| {
                                let _ = self.apps.enter(processid, |app, _kernel_data| {
                                    app.operation.clear();
                                });
                            })
                            .into()
                    }
                    Err(e) => e.into(),
                }
            }

            // verify
            3 => {
                let res = self.apps.enter(processid, |app, _kernel_data| {
                    if app.operation.is_some() {
                        // No more room in the queue, nowhere to store this
                        // request.
                        Err(ErrorCode::NOMEM)
                    } else {
                        app.operation.set(AppOp::Verify);
                        Ok(())
                    }
                });
                match res {
                    Ok(_) => {
                        // If we were able to enqueue the operation, check if we can
                        // actually run it. If there was an error starting it return the
                        // error, otherwise return ok if the operation started successfully
                        // or was queued for later. This also ensures we are not already in
                        // the grant.
                        self.check_queue()
                            .inspect_err(|_| {
                                let _ = self.apps.enter(processid, |app, _kernel_data| {
                                    app.operation.clear();
                                });
                            })
                            .into()
                    }
                    Err(e) => e.into(),
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
