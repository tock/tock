// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to temperature sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a temperature sensor reading.
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
//! The `command` system call support one argument `cmd` which is used to
//! specify the specific operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exists
//! * `1`: read the temperature
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
//! You need a device that provides the `hil::sensors::TemperatureDriver` trait.
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_temperature = board_kernel.create_grant(&grant_cap);
//!
//! let temp = static_init!(
//!        capsules::temperature::TemperatureSensor<'static>,
//!        capsules::temperature::TemperatureSensor::new(si7021,
//!                                                 board_kernel.create_grant(&grant_cap)));
//!
//! kernel::hil::sensors::TemperatureDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct TemperatureSensor<'a, T: hil::sensors::TemperatureDriver<'a>> {
    driver: &'a T,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a, T: hil::sensors::TemperatureDriver<'a>> TemperatureSensor<'a, T> {
    pub fn new(
        driver: &'a T,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> TemperatureSensor<'a, T> {
        TemperatureSensor {
            driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, processid: ProcessId) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                // Unconditionally mark this client as subscribed so it will get
                // a callback when we get the temperature reading.
                app.subscribed = true;

                // If we do not already have an ongoing read, start one now.
                if !self.busy.get() {
                    self.busy.set(true);
                    match self.driver.read_temperature() {
                        Ok(()) => CommandReturn::success(),
                        Err(e) => CommandReturn::failure(e),
                    }
                } else {
                    // Just return success and we will get the upcall when the
                    // temperature read is ready.
                    CommandReturn::success()
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}

impl<'a, T: hil::sensors::TemperatureDriver<'a>> hil::sensors::TemperatureClient
    for TemperatureSensor<'a, T>
{
    fn callback(&self, temp_val: Result<i32, ErrorCode>) {
        // We completed the operation so we clear the busy flag in case we get
        // another measurement request.
        self.busy.set(false);

        // Return the temperature reading to any waiting client.
        if let Ok(temp_val) = temp_val {
            // TODO: forward error conditions
            for cntr in self.apps.iter() {
                cntr.enter(|app, upcalls| {
                    if app.subscribed {
                        app.subscribed = false;
                        let _ = upcalls.schedule_upcall(0, (temp_val as usize, 0, 0));
                    }
                });
            }
        }
    }
}

impl<'a, T: hil::sensors::TemperatureDriver<'a>> SyscallDriver for TemperatureSensor<'a, T> {
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // driver existence check
            0 => CommandReturn::success(),

            // read temperature
            1 => self.enqueue_command(processid),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
