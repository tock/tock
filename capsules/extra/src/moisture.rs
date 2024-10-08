// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to moisture sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a moisture reading.
//!
//! ### `command` System Call
//!
//! The `command` system call support one argument `cmd` which is used to specify the specific
//! operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exists
//! * `1`: read moisture
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
//! You need a device that provides the `hil::sensors::MoistureDriver` trait.
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let moisture = static_init!(
//!        capsules::moisture::MoistureSensor<'static>,
//!        capsules::moisture::MoistureSensor::new(si7021,
//!                                                board_kernel.create_grant(&grant_cap)));
//! kernel::hil::sensors::MoistureDriver::set_client(si7021, moisture);
//! ```

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Moisture as usize;

#[derive(Clone, Copy, PartialEq)]
enum MoistureCommand {
    ReadMoisture,
}

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct MoistureSensor<'a, H: hil::sensors::MoistureDriver<'a>> {
    driver: &'a H,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a, H: hil::sensors::MoistureDriver<'a>> MoistureSensor<'a, H> {
    pub fn new(
        driver: &'a H,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> MoistureSensor<'a, H> {
        MoistureSensor {
            driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(
        &self,
        command: MoistureCommand,
        _arg1: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                app.subscribed = true;

                if !self.busy.get() {
                    self.busy.set(true);
                    self.call_driver(command)
                } else {
                    CommandReturn::success()
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }

    fn call_driver(&self, command: MoistureCommand) -> CommandReturn {
        match command {
            MoistureCommand::ReadMoisture => {
                let ret = self.driver.read_moisture();
                if ret.is_err() {
                    self.busy.set(false);
                }
                ret.into()
            }
        }
    }
}

impl<'a, H: hil::sensors::MoistureDriver<'a>> hil::sensors::MoistureClient
    for MoistureSensor<'a, H>
{
    fn callback(&self, value: Result<usize, ErrorCode>) {
        self.busy.set(false);

        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;
                    match value {
                        Ok(moisture_val) => upcalls
                            .schedule_upcall(
                                0,
                                (kernel::errorcode::into_statuscode(Ok(())), moisture_val, 0),
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

impl<'a, H: hil::sensors::MoistureDriver<'a>> SyscallDriver for MoistureSensor<'a, H> {
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

            // single moisture measurement
            1 => self.enqueue_command(MoistureCommand::ReadMoisture, arg1, processid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
