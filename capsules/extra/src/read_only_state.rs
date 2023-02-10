//! Read Only State
//!
//! This capsule provides read only state to userspace applications.
//! This is similar to the Linux vDSO syscalls.
//!
//! The benefit of using these is that applications can avoid the context
//! switch overhead of traditional syscalls by just reading the value from
//! memory.
//!
//! The value will only be as accurate as the last time the application was
//! switched to by the kernel.
//!
//! The layout of the read only state in the allow region depends on the
//! version. Userspace can use `command 0` to get the version information.
//!
//! Versions are backwards compatible, that is new versions will only add
//! fields, not remove existing ones or change the order.
//!
//! ```text
//! Version 1:
//!   |-------------------------|
//!   |    Switch Count (u32)   |
//!   |-------------------------|
//!   |   Pending Tasks (u32)   |
//!   |-------------------------|
//!   |                         |
//!   |     Time Ticks (u64)    |
//!   |-------------------------|
//! ```

use core::cell::Cell;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::time::{Ticks, Time};
use kernel::platform::ContextSwitchCallback;
use kernel::process::{self, ProcessId};
use kernel::processbuffer::{UserspaceReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;

/// Syscall driver number.
pub const DRIVER_NUM: usize = core_capsules::driver::NUM::ReadOnlyState as usize;
const VERSION: u32 = 1;

pub struct ReadOnlyStateDriver<'a, T: Time> {
    timer: &'a T,

    apps: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
}

impl<'a, T: Time> ReadOnlyStateDriver<'a, T> {
    pub fn new(
        timer: &'a T,
        grant: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> ReadOnlyStateDriver<'a, T> {
        ReadOnlyStateDriver { timer, apps: grant }
    }
}

impl<'a, T: Time> ContextSwitchCallback for ReadOnlyStateDriver<'a, T> {
    fn context_switch_hook(&self, process: &dyn process::Process) {
        let processid = process.processid();
        let pending_tasks = process.pending_tasks();

        self.apps
            .enter(processid, |app, _| {
                let count = app.count.get();

                let _ = app.mem_region.mut_enter(|buf| {
                    if buf.len() >= 4 {
                        buf[0..4].copy_from_slice(&count.to_le_bytes());
                    }
                    if buf.len() >= 8 {
                        buf[4..8].copy_from_slice(&(pending_tasks as u32).to_le_bytes());
                    }
                    if buf.len() >= 16 {
                        let now = self.timer.now().into_usize() as u64;
                        buf[8..16].copy_from_slice(&now.to_le_bytes());
                    }
                });

                app.count.set(count.wrapping_add(1));
            })
            .unwrap();
    }
}

impl<'a, T: Time> SyscallDriver for ReadOnlyStateDriver<'a, T> {
    /// Specify memory regions to be used.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Allow a buffer for the kernel to stored syscall values.
    ///        This should only be read by the app and written by the capsule.
    fn allow_userspace_readable(
        &self,
        processid: ProcessId,
        which: usize,
        mut slice: UserspaceReadableProcessBuffer,
    ) -> Result<UserspaceReadableProcessBuffer, (UserspaceReadableProcessBuffer, ErrorCode)> {
        if which == 0 {
            let res = self.apps.enter(processid, |data, _| {
                core::mem::swap(&mut data.mem_region, &mut slice);
            });
            match res {
                Ok(_) => Ok(slice),
                Err(e) => Err((slice, e.into())),
            }
        } else {
            Err((slice, ErrorCode::NOSUPPORT))
        }
    }

    /// Commands for ReadOnlyStateDriver.
    ///
    /// ### `command_num`
    ///
    /// - `0`: get version
    fn command(
        &self,
        command_number: usize,
        _target_id: usize,
        _: usize,
        _processid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            // get version
            0 => CommandReturn::success_u32(VERSION),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

#[derive(Default)]
pub struct App {
    mem_region: UserspaceReadableProcessBuffer,
    count: Cell<u32>,
}
