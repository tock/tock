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
//!
//! The possible returns from the `command` system call indicate the following:
//!
//! * `Ok(())`: The operation has been successful.
//! * `ENOSUPPORT`: Invalid `cmd`.
//! * `NOMEM`: Insufficient memory available.
//! * `INVAL`: Invalid address of the buffer or other error.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::Distance` trait.
//! Here is an example of how to set up a distance sensor with the HC-SR04.
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let trig_pin = peripherals.pins.get_pin();
//! let echo_pin = peripherals.pins.get_pin();
//! echo_pin.make_input();
//! trig_pin.make_output();
//!
//! let virtual_alarm_hc_sr04 = static_init!(
//!     VirtualMuxAlarm<'static, RPTimer>,
//!     VirtualMuxAlarm::new(mux_alarm)
//! );
//! virtual_alarm_hc_sr04.setup();
//!
//! let hc_sr04 = static_init!(
//!     hc_sr04::HcSr04<VirtualMuxAlarm<'static, RPTimer>>,
//!     hc_sr04::HcSr04::new(trig_pin, echo_pin, virtual_alarm_hc_sr04)
//! );
//! virtual_alarm_hc_sr04.set_alarm_client(hc_sr04);
//!
//! echo_pin.set_client(hc_sr04);
//!
//! let distance_sensor = static_init!(
//!     distance::DistanceSensor<
//!         'static,
//!         hc_sr04::HcSr04<'static, VirtualMuxAlarm<'static, RPTimer>>,
//!     >,
//!     distance::DistanceSensor::new(
//!         hc_sr04,
//!         board_kernel.create_grant(distance::DRIVER_NUM, &memory_allocation_capability)
//!     )
//! );
//! hc_sr04.set_client(distance_sensor);
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
            driver: driver,
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
                        Err(e) => CommandReturn::failure(e),
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

        // Return the distance reading to any waiting client.
        if let Ok(distance_val) = distance_val {
            for cntr in self.apps.iter() {
                cntr.enter(|app, upcalls| {
                    if app.subscribed {
                        app.subscribed = false;
                        upcalls
                            .schedule_upcall(0, (distance_val as usize, 0, 0))
                            .ok();
                    }
                });
            }
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
