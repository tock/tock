//! Shared userland driver for light sensors.
//!
//! You need a device that provides the `hil::sensors::AmbientLight` trait.
//!
//! ```rust
//! # use kernel::{hil, static_init};
//!
//! let light = static_init!(
//!     capsules::ambient_light::AmbientLight<'static>,
//!     capsules::ambient_light::AmbientLight::new(isl29035,
//!         board_kernel.create_grant(&grant_cap)));
//! hil::sensors::AmbientLight::set_client(isl29035, ambient_light);
//! ```

use core::cell::Cell;

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::AmbientLight as usize;

/// Per-process metadata
#[derive(Default)]
pub struct App {
    pending: bool,
}

pub struct AmbientLight<'a> {
    sensor: &'a dyn hil::sensors::AmbientLight<'a>,
    command_pending: Cell<bool>,
    apps: Grant<App, 1>,
}

impl<'a> AmbientLight<'a> {
    pub fn new(
        sensor: &'a dyn hil::sensors::AmbientLight<'a>,
        grant: Grant<App, 1>,
    ) -> AmbientLight {
        AmbientLight {
            sensor: sensor,
            command_pending: Cell::new(false),
            apps: grant,
        }
    }

    fn enqueue_sensor_reading(&self, appid: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(appid, |app, _| {
                if app.pending {
                    Err(ErrorCode::NOMEM)
                } else {
                    app.pending = true;
                    if !self.command_pending.get() {
                        self.command_pending.set(true);
                        let _ = self.sensor.read_light_intensity();
                    }
                    Ok(())
                }
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl SyscallDriver for AmbientLight<'_> {
    // Subscribe to light intensity readings
    //
    // ### `subscribe`
    //
    // - `0`: Subscribe to light intensity readings. The callback signature is
    // `fn(lux: usize)`, where `lux` is the light intensity in lux (lx).

    /// Initiate light intensity readings
    ///
    /// Sensor readings are coalesced if processes request them concurrently. If
    /// multiple processes request have outstanding requests for a sensor
    /// reading, only one command will be issued and the result is returned to
    /// all subscribed processes.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `1`: Start a light sensor reading
    fn command(&self, command_num: usize, _: usize, _: usize, appid: ProcessId) -> CommandReturn {
        match command_num {
            0 /* check if present */ => CommandReturn::success(),
            1 => {
                let _ = self.enqueue_sensor_reading(appid);
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl hil::sensors::AmbientLightClient for AmbientLight<'_> {
    fn callback(&self, lux: usize) {
        self.command_pending.set(false);
        self.apps.each(|_, app, upcalls| {
            if app.pending {
                app.pending = false;
                upcalls.schedule_upcall(0, lux, 0, 0).ok();
            }
        });
    }
}
