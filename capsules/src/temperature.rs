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
//! ``rust
//! let temp = static_init!(
//!        capsules::temperature::TemperatureSensor<'static>,
//!        capsules::temperature::TemperatureSensor::new(si7021,
//!                                                 kernel::Grant::create()), 96/8);
//! kernel::hil::sensors::TemperatureDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use kernel::{AppId, Callback, Grant, Driver};
use kernel::ReturnCode;
use kernel::hil;

/// Syscall number
pub const DRIVER_NUM: usize = 0x60000;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

pub struct TemperatureSensor<'a> {
    driver: &'a hil::sensors::TemperatureDriver,
    apps: Grant<App>,
    busy: Cell<bool>,
}

impl<'a> TemperatureSensor<'a> {
    pub fn new(driver: &'a hil::sensors::TemperatureDriver,
               grant: Grant<App>)
               -> TemperatureSensor<'a> {
        TemperatureSensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| if !self.busy.get() {
                app.subscribed = true;
                self.busy.set(true);
                self.driver.read_temperature()
            } else {
                ReturnCode::EBUSY
            })
            .unwrap_or_else(|err| err.into())
    }

    fn configure_callback(&self, callback: Callback) -> ReturnCode {
        self.apps
            .enter(callback.app_id(), |app, _| {
                app.callback = Some(callback);
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl<'a> hil::sensors::TemperatureClient for TemperatureSensor<'a> {
    fn callback(&self, temp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| if app.subscribed {
                self.busy.set(false);
                app.subscribed = false;
                app.callback.map(|mut cb| cb.schedule(temp_val, 0, 0));
            });
        }
    }
}

impl<'a> Driver for TemperatureSensor<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => self.configure_callback(callback),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {

            // check whether the driver exists!!
            0 => ReturnCode::SUCCESS,

            // read temperature
            1 => self.enqueue_command(appid),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
