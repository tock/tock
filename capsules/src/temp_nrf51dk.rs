//! NRF51DK Temperature Sensor Capsule
//!
//!
//! Provides a simple driver for userspace applications to perform temperature measurements


use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::temp::{TempDriver, Client};
use kernel::process::Error;

pub static mut BUF: [u8; 64] = [0; 64];

pub struct App {
    callback: Option<Callback>,
}

impl Default for App {
    fn default() -> App {
        App { callback: None }
    }
}

pub struct Temp<'a, T: TempDriver + 'a> {
    temp: &'a T,
    apps: Container<App>,
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a, T: TempDriver + 'a> Temp<'a, T> {
    pub fn new(temp: &'a T, container: Container<App>, buf: &'static mut [u8]) -> Temp<'a, T> {
        Temp {
            temp: temp,
            apps: container,
            kernel_tx: TakeCell::new(buf),
        }
    }
}

impl<'a, E: TempDriver + 'a> Client for Temp<'a, E> {
    fn measurement_done(&self, temp: usize) -> ReturnCode {
        // panic!("CT {:?}\n", ct);
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| { app.callback.map(|mut cb| { cb.schedule(temp, 0, 0); }); });
        }
        ReturnCode::SUCCESS
    }
}


impl<'a, E: TempDriver> Driver for Temp<'a, E> {
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
    fn command(&self, command_num: usize, data: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 => {
                self.temp.take_measurement();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
