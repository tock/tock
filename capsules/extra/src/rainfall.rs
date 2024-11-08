// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to rain fall sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a rain fall reading.
//!
//! ### `command` System Call
//!
//! The `command` system call support one argument `cmd` which is used to specify the specific
//! operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exists
//! * `1`: read rainfall
//!
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `Ok(())`:    The operation has been successful.
//! * `NOSUPPORT`: Invalid `cmd`.
//! * `NOMEM`:     Insufficient memory available.
//! * `INVAL`:     Invalid address of the buffer or other error.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::RainFallDriver` trait.

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::RainFall as usize;

#[derive(Clone, Copy, PartialEq)]
enum RainFallCommand {
    ReadRainFall,
}

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct RainFallSensor<'a, H: hil::sensors::RainFallDriver<'a>> {
    driver: &'a H,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a, H: hil::sensors::RainFallDriver<'a>> RainFallSensor<'a, H> {
    pub fn new(
        driver: &'a H,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> RainFallSensor<'a, H> {
        RainFallSensor {
            driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(
        &self,
        command: RainFallCommand,
        arg1: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                app.subscribed = true;

                if !self.busy.get() {
                    self.busy.set(true);
                    self.call_driver(command, arg1)
                } else {
                    CommandReturn::success()
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }

    fn call_driver(&self, command: RainFallCommand, hours: usize) -> CommandReturn {
        match command {
            RainFallCommand::ReadRainFall => {
                let ret = self.driver.read_rainfall(hours);
                if ret.is_err() {
                    self.busy.set(false);
                }
                ret.into()
            }
        }
    }
}

impl<'a, H: hil::sensors::RainFallDriver<'a>> hil::sensors::RainFallClient
    for RainFallSensor<'a, H>
{
    fn callback(&self, value: Result<usize, ErrorCode>) {
        self.busy.set(false);

        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;
                    match value {
                        Ok(rainfall_val) => upcalls
                            .schedule_upcall(
                                0,
                                (kernel::errorcode::into_statuscode(Ok(())), rainfall_val, 0),
                            )
                            .ok(),
                        Err(e) => upcalls
                            .schedule_upcall(0, (kernel::errorcode::into_statuscode(Err(e)), 0, 0))
                            .ok(),
                    };
                }
            });
        }
    }
}

impl<'a, H: hil::sensors::RainFallDriver<'a>> SyscallDriver for RainFallSensor<'a, H> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // driver existence check
            0 => CommandReturn::success(),

            // single rainfall measurement
            1 => self.enqueue_command(RainFallCommand::ReadRainFall, arg1, processid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
