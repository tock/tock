//! NRF51DK Temperature Sensor Capsule
//!
//!
//! Provides a simple driver for userspace applications to perform temperature measurements


use core::cell::Cell;
use kernel::{AppId, Container, Callback, Driver, ReturnCode};
use kernel::hil::temperature::{TemperatureDriver, Client};
use kernel::process::Error;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

pub struct Temperature<'a, T: TemperatureDriver + 'a> {
    temp: &'a T,
    apps: Container<App>,
    busy: Cell<bool>,
}

impl<'a, T: TemperatureDriver + 'a> Temperature<'a, T> {
    pub fn new(temp: &'a T, container: Container<App>) -> Temperature<'a, T> {
        Temperature {
            temp: temp,
            apps: container,
            busy: Cell::new(false),
        }
    }
}

impl<'a, E: TemperatureDriver + 'a> Client for Temperature<'a, E> {
    fn measurement_done(&self, temp: usize) -> ReturnCode {
        for cntr in self.apps.iter() {
            self.busy.set(false);
            cntr.enter(|app, _| if app.subscribed {
                app.subscribed = false;
                app.callback.map(|mut cb| cb.schedule(temp, 0, 0));
            });
        }
        ReturnCode::SUCCESS
    }
}


impl<'a, E: TemperatureDriver> Driver for Temperature<'a, E> {
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
    fn command(&self, command_num: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| app.subscribed = true)
                    .unwrap_or(());

                if !self.busy.get() {
                    self.busy.set(true);
                    self.temp.take_measurement();
                }
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
