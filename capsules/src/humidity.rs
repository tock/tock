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
//! ``rust
//! let humidity = static_init!(
//!        capsules::humidity::HumiditySensor<'static>,
//!        capsules::humidity::HumiditySensor::new(si7021,
//!                                                 kernel::Container::create()), 96/8);
//! kernel::hil::sensors::HumidityDriver::set_client(si7021, humidity);
//! ```

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver};
use kernel::ReturnCode;
use kernel::hil;

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
    driver: &'a hil::sensors::HumidityDriver,
    apps: Container<App>,
    busy: Cell<bool>,
}

impl<'a> HumiditySensor<'a> {
    pub fn new(driver: &'a hil::sensors::HumidityDriver,
               container: Container<App>)
               -> HumiditySensor<'a> {
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
            .unwrap_or_else(|err| err.into())
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
            .unwrap_or_else(|err| err.into())
    }
}

impl<'a> hil::sensors::HumidityClient for HumiditySensor<'a> {
    fn callback(&self, tmp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| if app.subscribed {
                self.busy.set(false);
                app.subscribed = false;
                app.callback.map(|mut cb| cb.schedule(tmp_val, 0, 0));
            });
        }
    }
}

impl<'a> Driver for HumiditySensor<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => self.configure_callback(callback),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, appid: AppId) -> ReturnCode {
        match command_num {

            // check whether the driver exist!!
            0 => ReturnCode::SUCCESS,

            // single humidity measurement
            1 => self.enqueue_command(HumidityCommand::ReadHumidity, arg1, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
