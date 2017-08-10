//! Ambient light sensor system call driver

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver, ReturnCode};
use kernel::hil;
use kernel::process::Error;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    pending: bool,
}

pub struct AmbientLight<'a> {
    sensor: &'a hil::ambient_light::AmbientLight,
    command_pending: Cell<bool>,
    apps: Container<App>,
}

impl<'a> AmbientLight<'a> {
    pub fn new(sensor: &'a hil::ambient_light::AmbientLight,
               container: Container<App>)
               -> AmbientLight {
        AmbientLight {
            sensor: sensor,
            command_pending: Cell::new(false),
            apps: container,
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
            .unwrap_or_else(|err| match err {
                Error::OutOfMemory => ReturnCode::ENOMEM,
                Error::AddressOutOfBounds => ReturnCode::EINVAL,
                Error::NoSuchApp => ReturnCode::EINVAL,
            })
    }
}

impl<'a> Driver for AmbientLight<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.apps
                    .enter(callback.app_id(), |app, _| {
                        app.callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _arg1: usize, appid: AppId) -> ReturnCode {
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

impl<'a> hil::ambient_light::AmbientLightClient for AmbientLight<'a> {
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
