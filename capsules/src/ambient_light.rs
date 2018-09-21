//! Shared userland driver for light sensors.
//!
//! You need a device that provides the `hil::sensors::AmbientLight` trait.
//!
//! ```rust
//! let light = static_init!(
//!     capsules::sensors::AmbientLight<'static>,
//!     capsules::sensors::AmbientLight::new(isl29035,
//!         kernel::Grant::create()));
//! hil::sensors::AmbientLight::set_client(isl29035, ambient_light);
//! ```

use core::cell::Cell;
use kernel::hil;
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall number
pub const DRIVER_NUM: usize = 0x60002;

/// Per-process metadata
#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    pending: bool,
}

pub struct AmbientLight<'a> {
    sensor: &'a hil::sensors::AmbientLight,
    command_pending: Cell<bool>,
    apps: Grant<App>,
}

impl AmbientLight<'a> {
    pub fn new(sensor: &'a hil::sensors::AmbientLight, grant: Grant<App>) -> AmbientLight {
        AmbientLight {
            sensor: sensor,
            command_pending: Cell::new(false),
            apps: grant,
        }
    }

    fn enqueue_sensor_reading(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| if app.pending {
                ReturnCode::ENOMEM
            } else {
                app.pending = true;
                if !self.command_pending.get() {
                    self.command_pending.set(true);
                    self.sensor.read_light_intensity();
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl Driver for AmbientLight<'a> {
    /// Subscribe to light intensity readings
    ///
    /// ### `subscribe`
    ///
    /// - `0`: Subscribe to light intensity readings. The callback signature is
    /// `fn(lux: usize)`, where `lux` is the light intensity in lux (lx).
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.apps
                    .enter(app_id, |app, _| {
                        app.callback = callback;
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
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
    fn command(&self, command_num: usize, _arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 => {
                self.enqueue_sensor_reading(appid);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl hil::sensors::AmbientLightClient for AmbientLight<'a> {
    fn callback(&self, lux: usize) {
        self.command_pending.set(false);
        self.apps.each(|app| if app.pending {
            app.pending = false;
            if let Some(mut callback) = app.callback {
                callback.schedule(lux, 0, 0);
            }
        });
    }
}
