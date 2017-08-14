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
//! * `1`: read the ambient temperature
//! * `2`: read internal cpu temperature
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
//! You need a device that provides the `hil::sensor::TemperatureDriver` trait.
//!
//! ``rust
//! let temp = static_init!(
//!        capsules::temperature::TemperatureSensor<'static>,
//!        capsules::temperature::TemperatureSensor::new(si7021,
//!                                                 kernel::Container::create()), 96/8);
//! kernel::hil::sensor::TemperatureDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver};
use kernel::ReturnCode;
use kernel::hil;
use kernel::process::Error;

#[derive(Clone,Copy,PartialEq)]
pub enum TemperatureCommand {
    Exists,
    ReadAmbientTemperature,
    ReadCPUTemperature,
}

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

pub struct TemperatureSensor<'a> {
    driver: &'a hil::sensor::TemperatureDriver,
    apps: Container<App>,
    busy: Cell<bool>,
}

impl<'a> TemperatureSensor<'a> {
    pub fn new(driver: &'a hil::sensor::TemperatureDriver,
               container: Container<App>)
               -> TemperatureSensor<'a> {
        TemperatureSensor {
            driver: driver,
            apps: container,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self,
                       command: TemperatureCommand,
                       arg1: usize,
                       appid: AppId)
                       -> ReturnCode {
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

    fn call_driver(&self, command: TemperatureCommand, _: usize) -> ReturnCode {
        match command {
            TemperatureCommand::ReadAmbientTemperature => self.driver.read_ambient_temperature(),
            TemperatureCommand::ReadCPUTemperature => self.driver.read_cpu_temperature(),
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
}

impl<'a> hil::sensor::TemperatureClient for TemperatureSensor<'a> {
    fn callback(&self, temp_val: usize, dont_care: usize, err: ReturnCode) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| if app.subscribed {
                self.busy.set(false);
                app.subscribed = false;
                app.callback.map(|mut cb| cb.schedule(temp_val, dont_care, usize::from(err)));
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

    fn command(&self, command_num: usize, arg1: usize, appid: AppId) -> ReturnCode {
        match command_num {

            // check whether the driver exist!!
            0 => ReturnCode::SUCCESS,

            // read ambient temperature
            1 => self.enqueue_command(TemperatureCommand::ReadAmbientTemperature, arg1, appid),

            // read internal cpu temperature
            2 => self.enqueue_command(TemperatureCommand::ReadCPUTemperature, arg1, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
