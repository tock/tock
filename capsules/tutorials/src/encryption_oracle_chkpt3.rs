// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::symmetric_encryption::{AES128Ctr, AES128};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use kernel::ProcessId;

pub static KEY: &'static [u8; kernel::hil::symmetric_encryption::AES128_KEY_SIZE] =
    b"InsecureAESKey12";

#[derive(Default)]
pub struct ProcessState {
    request_pending: bool,
}

pub struct EncryptionOracleDriver<'a, A: AES128<'a> + AES128Ctr> {
    aes: &'a A,
    process_grants: Grant<ProcessState, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    current_process: OptionalCell<ProcessId>,
}

impl<'a, A: AES128<'a> + AES128Ctr> EncryptionOracleDriver<'a, A> {
    /// Create a new instance of our encryption oracle userspace driver:
    pub fn new(
        aes: &'a A,
        _source_buffer: &'static mut [u8],
        _dest_buffer: &'static mut [u8],
        process_grants: Grant<ProcessState, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Self {
        EncryptionOracleDriver {
            process_grants: process_grants,
            aes: aes,
            current_process: OptionalCell::empty(),
        }
    }

    /// Return either the current process (in case of an ongoing operation), or
    /// a process which has a request pending (if there is some).
    ///
    /// When returning a process which has a request pending, this method
    /// further marks this as the new current process.
    fn next_process(&self) -> Option<ProcessId> {
        if let Some(current_process) = self.current_process.extract() {
            // We already have an active current process, return its id:
            Some(current_process)
        } else {
            // We don't have an active process, search for one which has the
            // `request_pending` flag set:
            for process_grant in self.process_grants.iter() {
                let processid = process_grant.processid();
                if process_grant.enter(|grant, _| grant.request_pending) {
                    // The process to which `process_grant` belongs
                    // has a request pending, set it to be the current
                    // process and return its id:
                    self.current_process.set(processid);
                    return Some(processid);
                }
            }

            // We could not find a process with a pending request:
            None
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
            1 => self
                .process_grants
                .enter(processid, |grant, _kernel_data| {
                    grant.request_pending = true;
                    CommandReturn::success()
                })
                .unwrap_or_else(|err| err.into()),

            // Unknown command number, return a NOSUPPORT error
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.process_grants.enter(processid, |_, _| {})
    }
}
