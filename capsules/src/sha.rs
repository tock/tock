//! SHA
//!
//! Usage
//! -----
//!
//! ```rust
//! let sha = &earlgrey::sha::HMAC;
//!
//! let mux_sha = static_init!(MuxSha<'static, lowrisc::sha::Sha>, MuxSha::new(sha));
//! digest::Digest::set_client(&earlgrey::sha::HMAC, mux_sha);
//!
//! let virtual_sha_user = static_init!(
//!     VirtualMuxSha<'static, lowrisc::sha::Sha>,
//!     VirtualMuxSha::new(mux_sha)
//! );
//! let sha = static_init!(
//!     capsules::sha::ShaDriver<'static, VirtualMuxSha<'static, lowrisc::sha::Sha>>,
//!     capsules::sha::ShaDriver::new(
//!         virtual_sha_user,
//!         board_kernel.create_grant(&memory_allocation_cap),
//!     )
//! );
//! digest::Digest::set_client(virtual_sha_user, sha);
//! ```

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Sha as usize;

use core::cell::Cell;
use core::mem;

use kernel::grant::Grant;
use kernel::hil::digest;
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadWriteProcessBuffer};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::{ErrorCode, ProcessId};

enum ShaOperation {
    Sha256,
    Sha384,
    Sha512,
}

pub struct ShaDriver<'a, H: digest::Digest<'a, L>, const L: usize> {
    sha: &'a H,

    active: Cell<bool>,

    apps: Grant<App, 1>,
    appid: OptionalCell<ProcessId>,

    data_buffer: TakeCell<'static, [u8]>,
    data_copied: Cell<usize>,
    dest_buffer: TakeCell<'static, [u8; L]>,
}

impl<
        'a,
        H: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > ShaDriver<'a, H, L>
{
    pub fn new(
        sha: &'a H,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8; L],
        grant: Grant<App, 1>,
    ) -> ShaDriver<'a, H, L> {
        ShaDriver {
            sha: sha,
            active: Cell::new(false),
            apps: grant,
            appid: OptionalCell::empty(),
            data_buffer: TakeCell::new(data_buffer),
            data_copied: Cell::new(0),
            dest_buffer: TakeCell::new(dest_buffer),
        }
    }

    fn run(&self) -> Result<(), ErrorCode> {
        self.appid.map_or(Err(ErrorCode::RESERVE), |appid| {
            self.apps
                .enter(*appid, |app, _| {
                    let ret = if let Some(op) = &app.sha_operation {
                        match op {
                            ShaOperation::Sha256 => self.sha.set_mode_sha256(),
                            ShaOperation::Sha384 => self.sha.set_mode_sha384(),
                            ShaOperation::Sha512 => self.sha.set_mode_sha512(),
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    };
                    if ret.is_err() {
                        return ret;
                    }

                    app.data
                        .enter(|data| {
                            let mut static_buffer_len = 0;
                            self.data_buffer.map(|buf| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();

                                if static_buffer_len > data.len() {
                                    static_buffer_len = data.len()
                                }

                                self.data_copied.set(static_buffer_len);

                                // Copy the data into the static buffer
                                data[..static_buffer_len]
                                    .copy_to_slice(&mut buf[..static_buffer_len]);
                            });

                            // Add the data from the static buffer to the HMAC
                            let mut lease_buf =
                                LeasableBuffer::new(self.data_buffer.take().unwrap());
                            lease_buf.slice(0..static_buffer_len);
                            if let Err(e) = self.sha.add_data(lease_buf) {
                                self.data_buffer.replace(e.1);
                                return Err(e.0);
                            }
                            Ok(())
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

    fn calculate_digest(&self) -> Result<(), ErrorCode> {
        self.data_copied.set(0);

        if let Err(e) = self.sha.run(self.dest_buffer.take().unwrap()) {
            // Error, clear the appid and data
            self.sha.clear_data();
            self.appid.clear();
            self.dest_buffer.replace(e.1);

            return Err(e.0);
        }

        Ok(())
    }
}

impl<
        'a,
        H: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > digest::Client<'a, L> for ShaDriver<'a, H, L>
{
    fn add_data_done(&'a self, _result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        self.appid.map(move |id| {
            self.apps
                .enter(*id, move |app, upcalls| {
                    let mut data_len = 0;
                    let mut exit = false;
                    let mut static_buffer_len = 0;

                    self.data_buffer.replace(data);

                    self.data_buffer.map(|buf| {
                        let ret = app
                            .data
                            .enter(|data| {
                                // Determine the size of the static buffer we have
                                static_buffer_len = buf.len();

                                // Determine how much data we have already copied
                                let copied_data = self.data_copied.get();

                                data_len = data.len();

                                if data_len > copied_data {
                                    let remaining_data = &data[copied_data..];
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

                        if ret == Err(ErrorCode::RESERVE) {
                            // No data buffer, clear the appid and data
                            self.sha.clear_data();
                            self.appid.clear();
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

                            let mut lease_buf =
                                LeasableBuffer::new(self.data_buffer.take().unwrap());

                            // Add the data from the static buffer to the HMAC
                            if data_len < (copied_data + static_buffer_len) {
                                lease_buf.slice(..(data_len - copied_data))
                            }

                            if self.sha.add_data(lease_buf).is_err() {
                                // Error, clear the appid and data
                                self.sha.clear_data();
                                self.appid.clear();
                                return;
                            }

                            // Return as we don't want to run the digest yet
                            return;
                        }
                    }

                    // If we get here we are ready to run the digest, reset the copied data
                    if app.op.get().unwrap() == UserSpaceOp::Run {
                        if let Err(e) = self.calculate_digest() {
                            upcalls
                                .schedule_upcall(
                                    0,
                                    kernel::errorcode::into_statuscode(e.into()),
                                    0,
                                    0,
                                )
                                .ok();
                        }
                    } else {
                        upcalls.schedule_upcall(0, 0, 0, 0).ok();
                    }
                })
                .map_err(|err| {
                    if err == kernel::process::Error::NoSuchApp
                        || err == kernel::process::Error::InactiveApp
                    {
                        self.appid.clear();
                    }
                })
        });

        self.check_queue();
    }

    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]) {
        self.appid.map(|id| {
            self.apps
                .enter(*id, |app, upcalls| {
                    self.sha.clear_data();

                    let pointer = digest.as_ref()[0] as *mut u8;

                    let _ = app.dest.mut_enter(|dest| {
                        dest.copy_from_slice(digest);
                    });

                    match result {
                        Ok(_) => upcalls.schedule_upcall(0, 0, pointer as usize, 0).ok(),
                        Err(e) => upcalls
                            .schedule_upcall(
                                0,
                                kernel::errorcode::into_statuscode(e.into()),
                                pointer as usize,
                                0,
                            )
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

        self.check_queue();
        self.dest_buffer.replace(digest);
    }
}

impl<
        'a,
        H: digest::Digest<'a, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > SyscallDriver for ShaDriver<'a, H, L>
{
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        let res = match allow_num {
            // Pass buffer for the digest to be in.
            2 => self
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
            // Pass buffer for the data to be in
            1 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.data, &mut slice);
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

    /// Setup and run the HMAC hardware
    ///
    /// We expect userspace to setup buffers for the key, data and digest.
    /// These buffers must be allocated and specified to the kernel from the
    /// above allow calls.
    ///
    /// We expect userspace not to change the value while running. If userspace
    /// changes the value we have no guarentee of what is passed to the
    /// hardware. This isn't a security issue, it will just prove the requesting
    /// app with invalid data.
    ///
    /// The driver will take care of clearing data from the underlying impelemenation
    /// by calling the `clear_data()` function when the `hash_complete()` callback
    /// is called or if an error is encounted.
    ///
    /// ### `command_num`
    ///
    /// - `0`: set_algorithm
    /// - `1`: run
    /// - `2`: update
    /// - `3`: finish
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        _data2: usize,
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
            // set_algorithm
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        match data1 {
                            // SHA256
                            0 => {
                                app.sha_operation = Some(ShaOperation::Sha256);
                                CommandReturn::success()
                            }
                            // SHA384
                            1 => {
                                app.sha_operation = Some(ShaOperation::Sha384);
                                CommandReturn::success()
                            }
                            // SHA512
                            2 => {
                                app.sha_operation = Some(ShaOperation::Sha512);
                                CommandReturn::success()
                            }
                            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // run
            // Use key and data to compute hash
            // This will trigger a callback once the digest is generated
            1 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Run));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.sha.clear_data();
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
                                app.op.set(Some(UserSpaceOp::Run));
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // update
            // Input key and data, don't compute final hash yet
            // This will trigger a callback once the data has been added.
            2 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let _ = self.apps.enter(appid, |app, _| {
                        app.op.set(Some(UserSpaceOp::Update));
                    });
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.sha.clear_data();
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
                                app.op.set(Some(UserSpaceOp::Update));
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // finish
            // Compute final hash yet, useful after a update command
            3 => {
                if app_match {
                    self.apps
                        .enter(appid, |_app, upcalls| {
                            if let Err(e) = self.calculate_digest() {
                                upcalls
                                    .schedule_upcall(
                                        0,
                                        kernel::errorcode::into_statuscode(e.into()),
                                        0,
                                        0,
                                    )
                                    .ok();
                            }
                        })
                        .unwrap();
                    CommandReturn::success()
                } else {
                    // We don't queue this request, the user has to call
                    // `update` first.
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

#[derive(Copy, Clone, PartialEq)]
enum UserSpaceOp {
    Run,
    Update,
}

#[derive(Default)]
pub struct App {
    pending_run_app: Option<ProcessId>,
    sha_operation: Option<ShaOperation>,
    op: Cell<Option<UserSpaceOp>>,
    data: ReadOnlyProcessBuffer,
    dest: ReadWriteProcessBuffer,
}
