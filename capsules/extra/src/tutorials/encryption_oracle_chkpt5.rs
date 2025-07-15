// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::{AES128Ctr, Client, AES128, AES128_BLOCK_SIZE};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;
use kernel::ProcessId;

pub const DRIVER_NUM: usize = 0x99999;

pub static KEY: &[u8; kernel::hil::symmetric_encryption::AES128_KEY_SIZE] = b"InsecureAESKey12";

#[derive(Default)]
pub struct ProcessState {
    request_pending: bool,
    offset: usize,
}

/// Ids for subscribe upcalls
mod upcall {
    pub const DONE: usize = 0;
    /// The number of subscribe upcalls the kernel stores for this grant
    pub const COUNT: u8 = 1;
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

pub struct EncryptionOracleDriver<'a, A: AES128<'a> + AES128Ctr> {
    aes: &'a A,
    process_grants: Grant<
        ProcessState,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    current_process: OptionalCell<ProcessId>,
    source_buffer: TakeCell<'static, [u8]>,
    dest_buffer: TakeCell<'static, [u8]>,
    crypt_len: Cell<usize>,
}

impl<'a, A: AES128<'a> + AES128Ctr> EncryptionOracleDriver<'a, A> {
    /// Create a new instance of our encryption oracle userspace driver:
    pub fn new(
        aes: &'a A,
        source_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8],
        process_grants: Grant<
            ProcessState,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Self {
        EncryptionOracleDriver {
            process_grants,
            aes,
            current_process: OptionalCell::empty(),
            source_buffer: TakeCell::new(source_buffer),
            dest_buffer: TakeCell::new(dest_buffer),
            crypt_len: Cell::new(0),
        }
    }

    /// Return a `ProcessId` which has `request_pending` set, if there is some:
    fn next_pending(&self) -> Option<ProcessId> {
        for process_grant in self.process_grants.iter() {
            let processid = process_grant.processid();
            if process_grant.enter(|grant, _| grant.request_pending) {
                // The process to which `process_grant` belongs
                // has a request pending, return its id:
                return Some(processid);
            }
        }

        // No process with `request_pending` found:
        None
    }

    /// The run method initiates a new decryption operation or continues an
    /// existing asynchronous decryption in the context of a process.
    ///
    /// If the process-state `offset` is larger or equal to the process-provided
    /// source or destination buffer size, this indicates that we have completed
    /// the requested description operation and return an error of
    /// `ErrorCode::NOMEM`. A caller can use this as a method to check whether
    /// the descryption operation has finished.
    ///
    /// If the process-state `offset` is `0`, we will initialize the AES engine
    /// with an initialization vector (IV) provided by the application, and
    /// configure it to perform an AES128-CTR operation.
    fn run(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // If the kernel's source buffer is not present in its TakeCell,
        // this implies that we're currently running a decryption
        // operation:
        if self.source_buffer.is_none() {
            return Err(ErrorCode::BUSY);
        }

        self.process_grants
            .enter(processid, |grant, kernel_data| {
                // Get a reference to both the application-provided source
                // and destination buffers:
                let (source_processbuffer, dest_processbuffer) = kernel_data
                    .get_readonly_processbuffer(ro_allow::SOURCE)
                    .and_then(|source| {
                        kernel_data
                            .get_readwrite_processbuffer(rw_allow::DEST)
                            .map(|dest| (source, dest))
                    })?;

                // Calculate the minimum length of the source & destination
                // buffers:
                let min_processbuffer_len =
                    core::cmp::min(source_processbuffer.len(), dest_processbuffer.len());

                // If our operation-offset stored in the process grant exceeds
                // `min_buffer_len`, then our operation is finished. Return with
                // the appropriate error code.
                //
                // This check also ensures that we're never running a zero-byte
                // encryption operation.
                if grant.offset >= min_processbuffer_len {
                    return Err(ErrorCode::NOMEM);
                }

                // Perform some special initialization if this is the first
                // invocation of our AES engine as part of this decryption
                // operation:
                if grant.offset == 0 {
                    // Offset = 0, initialize the AES engine & IV:
                    self.aes.enable();

                    // Set the AES engine mode to AES128 CTR (counter) mode, and
                    // make this a decryption operation:
                    self.aes.set_mode_aes128ctr(true)?;

                    self.aes.set_key(KEY)?;

                    // Set the initialization vector:
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
                        })??;
                }

                // Our AES engine works with kernel-provided `&'static mut`
                // buffers. We copy a chunk of application's source buffer
                // contents into this kernel buffer:
                source_processbuffer.enter(|source_processbuffer| {
                    // Attempt to "take" the kernel-internal source &
                    // destination buffers from their TakeCells. We expect them
                    // to be placed back into the TakeCell after an encryption
                    // operation is done, and checked that the `source_buffer`
                    // is present above -- thus this should never fail:
                    let source_buffer = self.source_buffer.take().unwrap();
                    let dest_buffer = self.dest_buffer.take().unwrap();

                    // Determine the amount of data we pass to the AES engine,
                    // which is the minimum of the user-provided buffer space
                    // and our kernel-internal buffer capacity:
                    let data_len = core::cmp::min(source_buffer.len(), min_processbuffer_len);

                    // Now, copy this data into the kernel-internal buffer:
                    source_processbuffer[..data_len].copy_to_slice(&mut source_buffer[..data_len]);

                    // However, our AES engine requires us to pass it at least
                    // `AES128_BLOCK_SIZE` data, and have our data length be a
                    // multiple of the `AES128_BLOCK_SIZE`. We assume that our
                    // `source_buffer` holds at least a full AES128 block. Then,
                    // we can round up or down to a multiple of the
                    // `AES128_BLOCK_SIZE` as required:
                    let crypt_len = if data_len < AES128_BLOCK_SIZE {
                        AES128_BLOCK_SIZE
                    } else {
                        data_len - (data_len % AES128_BLOCK_SIZE)
                    };

                    // Save `crypt_len`, so we know how much data to copy back
                    // to the process buffer after the operation finished:
                    self.crypt_len.set(crypt_len);

                    // Set this process as active:
                    self.current_process.set(processid);

                    // Now, run the operation:
                    if let Some((e, source, dest)) =
                        AES128::crypt(self.aes, Some(source_buffer), dest_buffer, 0, crypt_len)
                    {
                        // An error occurred, clear the currently active process
                        // and replace the buffers. Reset the current process'
                        // offset:
                        grant.offset = 0;
                        self.aes.disable();
                        self.current_process.clear();
                        self.source_buffer.replace(source.unwrap());
                        self.dest_buffer.replace(dest);
                        e
                    } else {
                        Ok(())
                    }
                })?
            })
            .unwrap_or(Err(ErrorCode::RESERVE))
    }

    fn run_next_pending(&self) {
        if self.current_process.is_some() {
            return;
        }

        while let Some(processid) = self.next_pending() {
            let res = self.run(processid);

            let _ = self.process_grants.enter(processid, |grant, kernel_data| {
                grant.request_pending = false;

                if let Err(e) = res {
                    let _ = kernel_data.schedule_upcall(
                        upcall::DONE,
                        (kernel::errorcode::into_statuscode(Err(e)), 0, 0),
                    );
                }
            });
        }
    }
}

impl<'a, A: AES128<'a> + AES128Ctr> SyscallDriver for EncryptionOracleDriver<'a, A> {
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check whether the driver is present:
            0 => CommandReturn::success(),

            // Request the decryption operation:
            1 => {
                let res = self
                    .process_grants
                    .enter(processid, |grant, _kernel_data| {
                        grant.request_pending = true;
                        CommandReturn::success()
                    })
                    .unwrap_or_else(|err| err.into());

                self.run_next_pending();

                res
            }

            // Unknown command number, return a NOSUPPORT error
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.process_grants.enter(processid, |_, _| {})
    }
}

impl<'a, A: AES128<'a> + AES128Ctr> Client<'a> for EncryptionOracleDriver<'a, A> {
    fn crypt_done(&'a self, mut source: Option<&'static mut [u8]>, destination: &'static mut [u8]) {
        // One segment of encryption/decryption complete, move to next one or
        // callback to user if done.
        //
        // In either case, place our kernel-internal source buffer back:
        self.source_buffer
            .replace(source.take().expect("source should never be None"));

        // Attempt to get a reference to the current process. This should never
        // be none, given that we've just completed an operation:
        let processid = self.current_process.unwrap_or_panic();

        // Enter the process' grant, to copy the decrypted data back into the
        // output processbuffer:
        let _ = self.process_grants.enter(processid, |grant, kernel_data| {
            if let Ok(dest_processbuffer) = kernel_data.get_readwrite_processbuffer(rw_allow::DEST)
            {
                // We have decrypted `self.crypt_len` bytes, starting
                // `grant.offset` in the source processbuffer. If the
                // destination processbuffer does not have enough space,
                // truncate the data. If the buffer is smaller than the current
                // offset, don't copy anything.
                if grant.offset < dest_processbuffer.len() {
                    let copy_len = core::cmp::min(
                        self.crypt_len.get(),
                        dest_processbuffer.len() - grant.offset,
                    );
                    let _ = dest_processbuffer.mut_enter(|dest_buffer| {
                        dest_buffer[grant.offset..(grant.offset + copy_len)]
                            .copy_from_slice(&destination[..copy_len])
                    });
                    grant.offset += copy_len;
                }
            }
        });

        // Place back the kernel-internal dest buffer:
        self.dest_buffer.replace(destination);

        // Try to continue the decryption operation. This will return an error
        // of `ErrorCode::NOMEM` if there is no more data to decrypt for the
        // current process.
        if let Err(ErrorCode::NOMEM) = self.run(processid) {
            // We've completed the client's decryption request. Remove its
            // `request_pending` flag, the `currrent_process` indication,
            // and schedule an upcall accordingly:
            self.current_process.clear();

            let _ = self.process_grants.enter(processid, |grant, kernel_data| {
                grant.offset = 0;

                // Pass the encryption/decryption operation length in the 2nd
                // upcall argument. This is always the minimum of the app's
                // provided source and destination buffers:
                let len = core::cmp::min(
                    kernel_data
                        .get_readonly_processbuffer(ro_allow::SOURCE)
                        .map_or(0, |source| source.len()),
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::DEST)
                        .map_or(0, |dest| dest.len()),
                );

                let _ = kernel_data.schedule_upcall(upcall::DONE, (0, len, 0));
            });

            // Attempt to schedule another operation for a new process:
            self.run_next_pending();
        }
    }
}
