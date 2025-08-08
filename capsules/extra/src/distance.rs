// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Provides userspace with access to distance sensor.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a distance sensor reading.
//! The `subscribe` call return codes indicate the following:
//!
//! * `Ok(())`: the callback has been successfully been configured.
//! * `ENOSUPPORT`: Invalid `subscribe_number`.
//! * `NOMEM`: No sufficient memory available.
//! * `INVAL`: Invalid address of the buffer or other error.
//!
//! ### `command` System Call
//!
//! The `command` system call supports one argument `cmd` which is used to
//! specify the specific operation. Currently, the following commands are supported:
//!
//! * `0`: check whether the driver exists.
//! * `1`: read the distance.
//! * `2`: get the minimum distance that the sensor can measure based on the datasheet, in millimeters.
//! * `3`: get the maximum distance that the sensor can measure based on the datasheet, in millimeters.
//!
//! The possible returns from the `command` system call indicate the following:
//!
//! * `Ok(())`: The operation has been successful.
//! * `NOACK`: No acknowledgment was received from the sensor during distance measurement.
//! * `INVAL`: Invalid measurement, such as when the object is out of range or no valid echo is received.
//! * `ENOSUPPORT`: Invalid `cmd`.
//! * `NOMEM`: Insufficient memory available.
//! * `INVAL`: Invalid address of the buffer or other error.
//!
//! The upcall has the following parameters:
//!
//! * `0`: Indicates a successful distance measurement, with the second parameter containing the distance, in millimeters.
//! * Non-zero: Indicates an error, with the first parameter containing the error code, and the second parameter being `0`.
//!
//! Components for the distance sensor.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::Distance` trait.
//! Here is an example of how to set up a distance sensor with the HC-SR04.
//!
//! ```rust,ignore
//! use components::hcsr04::HcSr04Component;

//! let trig_pin = peripherals.pins.get_pin(RPGpio::GPIO4);
//! let echo_pin = peripherals.pins.get_pin(RPGpio::GPIO5);
//!
//! let distance_sensor = components::hcsr04::HcSr04Component::new(
//!     mux_alarm,
//!     trig_pin,
//!     echo_pin
//! ).finalize(components::hcsr04_component_static!());
//!
//! distance_sensor.set_client(distance_sensor_client);
//! ```

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Distance as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct DistanceSensor<'a, T: hil::sensors::Distance<'a>> {
    driver: &'a T,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a, T: hil::sensors::Distance<'a>> DistanceSensor<'a, T> {
    pub fn new(
        driver: &'a T,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> DistanceSensor<'a, T> {
        DistanceSensor {
            driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, processid: ProcessId) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                // Unconditionally mark this client as subscribed so it will get
                // a callback when we get the distance reading.
                app.subscribed = true;

                // If we do not already have an ongoing read, start one now.
                if !self.busy.get() {
                    self.busy.set(true);
                    match self.driver.read_distance() {
                        Ok(()) => CommandReturn::success(),
                        Err(e) => {
                            self.busy.set(false);
                            app.subscribed = false;
                            CommandReturn::failure(e)
                        }
                    }
                } else {
                    // Just return success and we will get the upcall when the
                    // distance read is ready.
                    CommandReturn::success()
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}

impl<'a, T: hil::sensors::Distance<'a>> hil::sensors::DistanceClient for DistanceSensor<'a, T> {
    fn callback(&self, distance_val: Result<u32, ErrorCode>) {
        // We completed the operation so we clear the busy flag in case we get
        // another measurement request.
        self.busy.set(false);

        // Return the distance reading or an error to any waiting client.
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false; // Clear the subscribed flag.
                    match distance_val {
                        Ok(distance) => {
                            let _ = upcalls.schedule_upcall(0, (0, distance as usize, 0));
                        }
                        Err(e) => {
                            let _ = upcalls.schedule_upcall(0, (e as usize, 0, 0));
                        }
                    }
                }
            });
        }
    }
}

impl<'a, T: hil::sensors::Distance<'a>> SyscallDriver for DistanceSensor<'a, T> {
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => {
                // Driver existence check.
                CommandReturn::success()
            }
            1 => {
                // Read distance.
                self.enqueue_command(processid)
            }
            2 => {
                // Get minimum distance.
                CommandReturn::success_u32(self.driver.get_minimum_distance())
            }
            3 => {
                // Get maximum distance.
                CommandReturn::success_u32(self.driver.get_maximum_distance())
            }
            _ => {
                // Command not supported.
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}
