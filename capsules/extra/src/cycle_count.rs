// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides a cycle counter interface for userspace.
//!
//! Usage
//! -----
//!
//! This capsule is intended for debug purposes. However, to ensure that use
//! by multiple apps does not lead to innacurate results, basic virtualization
//! is implemented: only the first app to start the cycle counter can start or
//! stop or reset the counter. Other apps are restricted to reading the counter
//! (which can be useful for debugging the time required by cross-process routines).

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::CycleCount as usize;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{hil, ErrorCode, ProcessId};

#[derive(Default)]
pub struct App;

pub struct CycleCount<'a, P: hil::hw_debug::CycleCounter> {
    counters: &'a P,
    apps: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    controlling_app: OptionalCell<ProcessId>,
}

impl<'a, P: hil::hw_debug::CycleCounter> CycleCount<'a, P> {
    pub fn new(
        counters: &'a P,
        grant: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Self {
        Self {
            counters,
            apps: grant,
            controlling_app: OptionalCell::empty(),
        }
    }
}

impl<P: hil::hw_debug::CycleCounter> SyscallDriver for CycleCount<'_, P> {
    /// Control the CycleCount system.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Start the cycle counter.
    /// - `2`: Get current cycle count.
    /// - `3`: Reset and stop the cycle counter.
    /// - `4`: Stop the cycle counter.
    fn command(
        &self,
        command_num: usize,
        _data: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let try_claim_driver = || {
            let match_or_empty_or_nonexistant =
                self.controlling_app.map_or(true, |controlling_app| {
                    self.apps
                        .enter(controlling_app, |_, _| controlling_app == processid)
                        .unwrap_or(true)
                });
            if match_or_empty_or_nonexistant {
                self.controlling_app.set(processid);
                true
            } else {
                false
            }
        };
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                if try_claim_driver() {
                    self.counters.start();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            }
            2 => CommandReturn::success_u64(self.counters.count()),
            3 => {
                if try_claim_driver() {
                    self.counters.reset();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            }
            4 => {
                if try_claim_driver() {
                    self.counters.stop();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
