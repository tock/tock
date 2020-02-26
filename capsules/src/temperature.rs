//! Provides userspace with access to temperature sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a temperature sensor reading.
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
//! * `1`: read the temperature
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
//! You need a device that provides the `hil::sensors::TemperatureDriver` trait.
//!
//! ```rust
//! let temp = static_init!(
//!        capsules::temperature::TemperatureSensor<'static>,
//!        capsules::temperature::TemperatureSensor::new(si7021,
//!                                                 kernel::Grant::create()), 96/8);
//! kernel::hil::sensors::TemperatureDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(Default)]
pub struct App<'ker> {
    callback: Option<Callback<'ker>>,
    subscribed: bool,
}

pub struct TemperatureSensor<'a, 'ker> {
    driver: &'a dyn hil::sensors::TemperatureDriver,
    apps: Grant<'ker, App<'ker>>,
    busy: Cell<bool>,
}

impl TemperatureSensor<'a, 'ker> {
    pub fn new(
        driver: &'a dyn hil::sensors::TemperatureDriver,
        grant: Grant<'ker, App<'ker>>,
    ) -> TemperatureSensor<'a, 'ker> {
        TemperatureSensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, appid: AppId<'ker>) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.driver.read_temperature()
                } else {
                    ReturnCode::EBUSY
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn configure_callback(
        &self,
        callback: Option<Callback<'ker>>,
        app_id: AppId<'ker>,
    ) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl hil::sensors::TemperatureClient for TemperatureSensor<'a, 'ker> {
    fn callback(&self, temp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(temp_val, 0, 0));
                }
            });
        }
    }
}

impl Driver<'ker> for TemperatureSensor<'a, 'ker> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback<'ker>>,
        app_id: AppId<'ker>,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId<'ker>) -> ReturnCode {
        match command_num {
            // check whether the driver exists!!
            0 => ReturnCode::SUCCESS,

            // read temperature
            1 => self.enqueue_command(appid),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
