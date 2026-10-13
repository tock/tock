// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Provides userspace with a software watchdog for applications.
//!
//! The application registers/opts-in for the software watchdog and
//! then must "tickle" the software watchdog via a Command Syscall
//! within the prenegotiated window. If the application fails to tickle
//! the watchdog before the interval expires, the capsule will attempt
//! to restart the application (succeeding if the kernel permits the
//! operation).

use capsules_core::alarm::util::Expiration;
use kernel::capabilities::ProcessRestartCapability;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::time::{Alarm, ConvertTicks, Ticks};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, Kernel, ProcessId};

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::AppSoftwareWatchdog as usize;

pub struct AppData<T: Ticks> {
    alarm_data: Option<capsules_core::alarm::util::Expiration<T>>,
    window_size: Option<u32>,
}

impl<T: Ticks> Default for AppData<T> {
    fn default() -> Self {
        Self {
            alarm_data: None,
            window_size: None,
        }
    }
}

pub struct AppSoftwareWatchdog<'a, A: Alarm<'a>, P: ProcessRestartCapability> {
    apps: Grant<AppData<A::Ticks>, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
    alarm: &'a A,
    board_kernel: &'a Kernel,
    capability: P,
}

impl<'a, A: Alarm<'a>, P: ProcessRestartCapability> AppSoftwareWatchdog<'a, A, P> {
    pub fn new(
        grant: Grant<AppData<A::Ticks>, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<0>>,
        alarm: &'a A,
        board_kernel: &'a Kernel,
        capability: P,
    ) -> Self {
        Self {
            apps: grant,
            alarm,
            board_kernel,
            capability,
        }
    }

    // Helper to iterate across all Grants/stored iterations to: find
    // the nearest expiration, arm the alarm with this expiration, and
    // if the expiration has already passed (e.g., missed checkin window),
    // restart the process.
    //
    // NOTE: This must be called whenever we alter an expiration in a grant.
    // This is uesed o properly setup the alarm to be configured for the next
    // expiration.
    fn iterate_expirations(&self) {
        let expired_handler = |_expir: capsules_core::alarm::util::Expiration<A::Ticks>,
                               pid: &ProcessId| {
            self.board_kernel.restart_process(*pid, &self.capability);
            Option::None::<()>
        };

        let next_alarm_expr = capsules_core::alarm::util::earliest_alarm::<_, _, A, _>(
            self.alarm.now(),
            self.apps.iter().filter_map(|app| {
                let process_id = app.processid();
                app.enter(|alarm_state, _upcalls| {
                    if let Some(exp) = alarm_state.alarm_data {
                        Some((exp, process_id, expired_handler))
                    } else {
                        None
                    }
                })
            }),
        )
        .unwrap_or_else(|(expir, pid, ())| Some((expir, pid)));

        // Rearm the alarm with the nearest expiration.
        if let Some((expir, pid)) = next_alarm_expr {
            // There is a chance the time has already passed, in that
            // case, don't arm the alarm, just call the expired_handler
            // and call `iterate_expirations` again to setup the next alarm.
            if expir.reference.into_u32() + expir.dt.into_u32() < self.alarm.now().into_u32() {
                expired_handler(expir, &pid);
                self.iterate_expirations();
            } else {
                self.alarm.set_alarm(expir.reference, expir.dt);
            }
        }
    }

    fn tickle(&self, pid: ProcessId) {
        let _ = self.apps.enter(pid, |app_grant, _| {
            if let Some(window_size) = app_grant.window_size {
                // Reset this expiration to next check in window.
                app_grant.alarm_data = Some(capsules_core::alarm::util::Expiration::new(
                    self.alarm.now(),
                    self.alarm.ticks_from_seconds(window_size),
                ));
            }
        });

        self.iterate_expirations();
    }
}

impl<'a, A: Alarm<'a>, P: ProcessRestartCapability> kernel::hil::time::AlarmClient
    for AppSoftwareWatchdog<'a, A, P>
{
    fn alarm(&self) {
        self.iterate_expirations();
    }
}

impl<'a, A: Alarm<'a>, P: ProcessRestartCapability> SyscallDriver
    for AppSoftwareWatchdog<'a, A, P>
{
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Driver existence check.
            0 => CommandReturn::success(),

            // Register app to software watchdog service, specifying the
            // tickle interval for the respective app (arg1).
            1 => {
                let res = self.apps.enter(processid, |app_data, _| {
                    let window_size = match u32::try_from(arg1) {
                        Ok(val) => val,
                        Err(_) => return Err(ErrorCode::INVAL),
                    };

                    app_data.window_size = Some(window_size);
                    app_data.alarm_data = Some(Expiration::new(
                        self.alarm.now(),
                        self.alarm.ticks_from_seconds(arg1 as u32),
                    ));

                    Ok(())
                });

                self.iterate_expirations();
                match res {
                    Ok(Ok(())) => CommandReturn::success(),
                    Ok(Err(code)) => CommandReturn::failure(code),
                    Err(code) => code.into(), // unable to enter app
                }
            }

            // Tickle watchdog.
            2 => {
                self.tickle(processid);
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
