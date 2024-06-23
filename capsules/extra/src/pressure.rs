// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Provides userspace with access to barometer sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a barometer sensor reading.
//! The `subscribe`call return codes indicate the following:
//!
//! * `Ok(())`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `NOMEM`: No sufficient memory available.
//! * `INVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `command` System Call
//!
//! The `command` system call support one argument `cmd` which is used to specify the specific
//! operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exist
//! * `1`: read the barometer
//!
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `Ok(())`:    The operation has been successful.
//! * `BUSY`:      The driver is busy.
//! * `ENOSUPPORT`: Invalid `cmd`.
//! * `NOMEM`:     No sufficient memory available.
//! * `INVAL`:     Invalid address of the buffer or other error.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::PressureDriver` trait.
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let pressure = static_init!(
//!        capsules::temperature::PressureSensor<'static>,
//!        capsules::temperature::PressureSensor::new(si7021,
//!                                                 board_kernel.create_grant(&grant_cap)));
//!
//! kernel::hil::sensors::PressureDriver::set_client(si7021, pressure);
//! ```

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Pressure as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct PressureSensor<'a, T: hil::sensors::PressureDriver<'a>> {
    driver: &'a T,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a, T: hil::sensors::PressureDriver<'a>> PressureSensor<'a, T> {
    pub fn new(
        driver: &'a T,
        apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> PressureSensor<'a, T> {
        PressureSensor {
            driver: driver,
            apps: apps,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, processid: ProcessId) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                app.subscribed = true;
                if !self.busy.get() {
                    let res = self.driver.read_atmospheric_pressure();
                    if let Ok(err) = ErrorCode::try_from(res) {
                        CommandReturn::failure(err)
                    } else {
                        self.busy.set(true);
                        CommandReturn::success()
                    }
                } else {
                    CommandReturn::success()
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}

impl<'a, T: hil::sensors::PressureDriver<'a>> hil::sensors::PressureClient
    for PressureSensor<'a, T>
{
    fn callback(&self, pressure: Result<u32, ErrorCode>) {
        self.busy.set(false);
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;
                    let result = match pressure {
                        Ok(pressure_value) => (
                            kernel::errorcode::into_statuscode(Ok(())),
                            pressure_value as usize,
                            0,
                        ),
                        Err(err) => (kernel::errorcode::into_statuscode(Err(err)), 0, 0),
                    };
                    upcalls.schedule_upcall(0, result).ok();
                }
            })
        }
    }
}

impl<'a, T: hil::sensors::PressureDriver<'a>> SyscallDriver for PressureSensor<'a, T> {
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => self.enqueue_command(process_id),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}
