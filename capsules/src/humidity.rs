//! Provides userspace with access to humidity sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a humidity reading.
//! The `subscribe`call return codes indicate the following:
//!
//! * `SUCCESS`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `ENOMEM`: No sufficient memory available.
//! * `EINVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `command` System Call
//!
//! The `command` system call support one argument `cmd` which is used to specify the specific
//! operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exist
//! * `1`: read humidity
//!
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `SUCCESS`:    The operation has been successful.
//! * `EBUSY`:      The driver is busy.
//! * `ENOSUPPORT`: Invalid `cmd`.
//! * `ENOMEM`:     No sufficient memory available.
//! * `EINVAL`:     Invalid address of the buffer or other error.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::HumidityDriver` trait.
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let humidity = static_init!(
//!        capsules::humidity::HumiditySensor<'static>,
//!        capsules::humidity::HumiditySensor::new(si7021,
//!                                                board_kernel.create_grant(&grant_cap)));
//! kernel::hil::sensors::HumidityDriver::set_client(si7021, humidity);
//! ```

use core::cell::Cell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Grant, LegacyDriver};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Humidity as usize;

#[derive(Clone, Copy, PartialEq)]
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
    driver: &'a dyn hil::sensors::HumidityDriver<'a>,
    apps: Grant<App>,
    busy: Cell<bool>,
}

impl<'a> HumiditySensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::HumidityDriver<'a>,
        grant: Grant<App>,
    ) -> HumiditySensor<'a> {
        HumiditySensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, command: HumidityCommand, arg1: usize, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.call_driver(command, arg1)
                } else {
                    ReturnCode::EBUSY
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn call_driver(&self, command: HumidityCommand, _: usize) -> ReturnCode {
        match command {
            HumidityCommand::ReadHumidity => self.driver.read_humidity(),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl hil::sensors::HumidityClient for HumiditySensor<'_> {
    fn callback(&self, tmp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(tmp_val, 0, 0));
                }
            });
        }
    }
}

impl LegacyDriver for HumiditySensor<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // check whether the driver exist!!
            0 => ReturnCode::SUCCESS,

            // single humidity measurement
            1 => self.enqueue_command(HumidityCommand::ReadHumidity, arg1, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
