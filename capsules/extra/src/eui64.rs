// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Provides an EUI-64 (Extended Unique Identifier) interface for userspace.

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Eui64 as usize;
pub const EUI64_BUF_SIZE: usize = 8;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::WriteableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Ids for read-write allow buffers
mod rw_allow {
    /// Buffer to hold and pass the EUI-64 value.
    pub const EUI64: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

mod ro_allow {
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 0;
}

/// IDs for subscribed upcalls.
mod upcall {
    /// Number of upcalls.
    pub const COUNT: u8 = 0;
}

#[derive(Default)]
pub struct App {}
pub struct Eui64 {
    eui64: &'static [u8; EUI64_BUF_SIZE],
    app: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl Eui64 {
    pub fn new(
        eui64: &'static [u8; EUI64_BUF_SIZE],
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Eui64 {
        Eui64 {
            eui64: eui64,
            app: grant,
        }
    }
}

impl SyscallDriver for Eui64 {
    /// Control the Eui64.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Obtain EUI64 - placing the EUI64 in a previously allowed read/write buffer.
    fn command(&self, command_num: usize, _: usize, _: usize, pid: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => self
                .app
                .enter(pid, |_, grant_data| {
                    grant_data
                        .get_readwrite_processbuffer(rw_allow::EUI64)
                        .and_then(|read| {
                            read.mut_enter(|buf| {
                                // Confirm that a read write buffer has been allowed (i.e. check
                                // that the buffer len is not 0) and that the allowed buffer
                                // is large enough for the EUI64.
                                if buf.len() < EUI64_BUF_SIZE {
                                    return CommandReturn::failure(ErrorCode::INVAL);
                                }
                                buf[0..EUI64_BUF_SIZE].copy_from_slice(self.eui64);
                                CommandReturn::success()
                            })
                        })
                        .unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
                })
                .unwrap_or_else(|err| CommandReturn::failure(err.into())),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.app.enter(processid, |_, _| {})
    }
}
