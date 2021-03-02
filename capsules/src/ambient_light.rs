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
use core::convert::TryFrom;
use core::mem;
use kernel::hil;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::AmbientLight as usize;

/// Per-process metadata
#[derive(GrantDefault)]
pub struct App {
    callback: Callback,
    pending: bool,
}

pub struct AmbientLight<'a> {
    sensor: &'a dyn hil::sensors::AmbientLight<'a>,
    command_pending: Cell<bool>,
    apps: Grant<App>,
}

impl<'a> AmbientLight<'a> {
    pub fn new(sensor: &'a dyn hil::sensors::AmbientLight<'a>, grant: Grant<App>) -> AmbientLight {
        AmbientLight {
            sensor: sensor,
            command_pending: Cell::new(false),
            apps: grant,
        }
    }

    fn enqueue_sensor_reading(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if app.pending {
                    ReturnCode::ENOMEM
                } else {
                    app.pending = true;
                    if !self.command_pending.get() {
                        self.command_pending.set(true);
                        self.sensor.read_light_intensity();
                    }
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl Driver for AmbientLight<'_> {
    /// Subscribe to light intensity readings
    ///
    /// ### `subscribe`
    ///
    /// - `0`: Subscribe to light intensity readings. The callback signature is
    /// `fn(lux: usize)`, where `lux` is the light intensity in lux (lx).
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            0 => {
                let rcode = self
                    .apps
                    .enter(app_id, |app, _| {
                        mem::swap(&mut callback, &mut app.callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into());

                let eres = ErrorCode::try_from(rcode);
                match eres {
                    Ok(ecode) => Err((callback, ecode)),
                    _ => Ok(callback),
                }
            }
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

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
    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> CommandReturn {
        match command_num {
            0 /* check if present */ => CommandReturn::success(),
            1 => {
                self.enqueue_sensor_reading(appid);
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }
}

impl hil::sensors::AmbientLightClient for AmbientLight<'_> {
    fn callback(&self, lux: usize) {
        self.command_pending.set(false);
        self.apps.each(|app| {
            if app.pending {
                app.pending = false;
                app.callback.schedule(lux, 0, 0);
            }
        });
    }
}
