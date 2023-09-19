// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::ProcessId;

pub static KEY: &[u8; kernel::hil::symmetric_encryption::AES128_KEY_SIZE] = b"InsecureAESKey12";

#[derive(Default)]
pub struct ProcessState {
    request_pending: bool,
}

pub struct EncryptionOracleDriver {
    process_grants: Grant<ProcessState, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
}

impl EncryptionOracleDriver {
    /// Create a new instance of our encryption oracle userspace driver:
    pub fn new(
        process_grants: Grant<ProcessState, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Self {
        EncryptionOracleDriver {
            process_grants: process_grants,
        }
    }
}

impl SyscallDriver for EncryptionOracleDriver {
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
