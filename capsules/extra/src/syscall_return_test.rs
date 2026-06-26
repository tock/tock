// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Capsule for testing syscall return variants from userspace.
//!
//! Each command returns a specific [`SyscallReturn`] variant with distinct,
//! known values so userspace tests can verify correct encoding and decoding
//! of every return type.
//!
//! The capsule also supports subscribe, allow read-only, allow read-write,
//! and allow userspace-readable syscalls with subscribe_num/allow_num 0. This
//! lets userspace tests verify that success and failure returns carry back the
//! expected pointer and length values.
//!
//! For allow calls, pass a valid buffer (within app memory) for a success
//! return and an invalid pointer (e.g. 0x90) for a failure return. For
//! subscribe, pass a valid function pointer for success and an invalid one
//! (e.g. 0x90) for failure.
//!
//! ## Command numbers
//!
//!  - 0:  Success (driver presence check)
//!  - 1:  Failure(ErrorCode::FAIL)
//!  - 2:  FailureU32(ErrorCode::BUSY, 0x10000001)
//!  - 3:  FailureU32U32(ErrorCode::NOMEM, 0x20000001, 0x20000002)
//!  - 4:  FailureU64(ErrorCode::INVAL, 0x4000000000000001)
//!  - 5:  Success
//!  - 6:  SuccessU32(0x60000001)
//!  - 7:  SuccessU32U32(0x70000001, 0x70000002)
//!  - 8:  SuccessU32U32U32(0x80000001, 0x80000002, 0x80000003)
//!  - 9:  SuccessU64(0x9000000000000001)
//!  - 10: SuccessU32U64(0xA0000001, 0xA000000000000002)

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::process;
use kernel::processbuffer::UserspaceReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = capsules_core::driver::NUM::SyscallReturnTest as usize;

/// Per-process grant data. Holds the userspace-readable buffer swapped in via
/// `allow_userspace_readable`.
#[derive(Default)]
pub struct App {
    userspace_readable_buf: UserspaceReadableProcessBuffer,
}

pub struct SyscallReturnTest {
    apps: Grant<App, UpcallCount<1>, AllowRoCount<1>, AllowRwCount<1>>,
}

impl SyscallReturnTest {
    pub fn new(
        grant: Grant<App, UpcallCount<1>, AllowRoCount<1>, AllowRwCount<1>>,
    ) -> Self {
        SyscallReturnTest { apps: grant }
    }
}

impl SyscallDriver for SyscallReturnTest {
    fn command(
        &self,
        command_num: usize,
        _r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => CommandReturn::failure(ErrorCode::FAIL),
            2 => CommandReturn::failure_u32(ErrorCode::BUSY, 0x1000_0001),
            3 => CommandReturn::failure_u32_u32(ErrorCode::NOMEM, 0x2000_0001, 0x2000_0002),
            4 => CommandReturn::failure_u64(ErrorCode::INVAL, 0x4000_0000_0000_0001),
            5 => CommandReturn::success(),
            6 => CommandReturn::success_u32(0x6000_0001),
            7 => CommandReturn::success_u32_u32(0x7000_0001, 0x7000_0002),
            8 => CommandReturn::success_u32_u32_u32(0x8000_0001, 0x8000_0002, 0x8000_0003),
            9 => CommandReturn::success_u64(0x9000_0000_0000_0001),
            10 => CommandReturn::success_u32_u64(0xA000_0001, 0xA000_0000_0000_0002),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    /// Accept or reject an allow-userspace-readable buffer.
    ///
    /// `which` 0: accept — swap the incoming buffer with whatever was stored
    /// previously (initially empty) and return the old one to userspace.
    /// Any other `which` value: reject with `NOSUPPORT`.
    fn allow_userspace_readable(
        &self,
        processid: ProcessId,
        which: usize,
        mut slice: UserspaceReadableProcessBuffer,
    ) -> Result<UserspaceReadableProcessBuffer, (UserspaceReadableProcessBuffer, ErrorCode)> {
        if which == 0 {
            let res = self.apps.enter(processid, |data, _: &GrantKernelData| {
                core::mem::swap(&mut data.userspace_readable_buf, &mut slice);
            });
            match res {
                Ok(()) => Ok(slice),
                Err(e) => Err((slice, e.into())),
            }
        } else {
            Err((slice, ErrorCode::NOSUPPORT))
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}
