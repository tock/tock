//! Generic TemperatureSensor Interface

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver};
use kernel::ReturnCode;
use kernel::hil;
use kernel::process::Error;

#[derive(Clone,Copy,PartialEq)]
pub enum HumidityCommand {
    Exists,
    ReadHumidity,
}

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

pub struct HumiditySensor<'a> {
    driver: &'a hil::sensor::HumidityDriver,
    apps: Container<App>,
    busy: Cell<bool>,
}

impl<'a> HumiditySensor<'a> {
    
    pub fn new(driver: &'a hil::sensor::HumidityDriver, container: Container<App>) -> HumiditySensor<'a> {
        HumiditySensor {
            driver: driver,
            apps: container,
            busy: Cell::new(false),
        }
    }
    
    fn enqueue_command(&self, command: HumidityCommand, arg1: usize, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| if !self.busy.get() {
                app.subscribed = true;
                self.busy.set(true);
                self.call_driver(command, arg1)
            } else {
                ReturnCode::EBUSY
            })
        .unwrap_or_else(|err| match err {
            Error::OutOfMemory => ReturnCode::ENOMEM,
            Error::AddressOutOfBounds => ReturnCode::EINVAL,
            Error::NoSuchApp => ReturnCode::EINVAL,
        })
    }

    fn call_driver(&self, command: HumidityCommand, _: usize) -> ReturnCode {
        match command {
            HumidityCommand::ReadHumidity => self.driver.read_humidity(),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn configure_callback(&self, callback: Callback) -> ReturnCode {
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

    fn reset_callback(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                app.callback = None;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| match err {
                Error::OutOfMemory => ReturnCode::ENOMEM,
                Error::AddressOutOfBounds => ReturnCode::EINVAL,
                Error::NoSuchApp => ReturnCode::EINVAL,
            })
    }
}

impl<'a> hil::sensor::HumidityClient for HumiditySensor<'a> {
    fn callback(&self, tmp_val: usize, dont_care: usize, err: ReturnCode) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| if app.subscribed {
                self.busy.set(false);
                app.subscribed = false;
                app.callback.map(|mut cb| cb.schedule(tmp_val, dont_care, usize::from(err)));
            });
        }
    }
}

impl<'a> Driver for HumiditySensor<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => {
                self.configure_callback(callback)
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, appid: AppId) -> ReturnCode {
        match command_num {

            // check whether the driver exist!!
            0 => ReturnCode::SUCCESS,

            // single temperature measurement
            1 => self.enqueue_command(HumidityCommand::ReadHumidity, arg1, appid),

            // un-subscribe callback,
            // might be un-necessary as subscribe replaces the existing callback
            2 => {
                self.reset_callback(appid)
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
