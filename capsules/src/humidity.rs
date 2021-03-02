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
use core::mem;

use kernel::hil;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Humidity as usize;

#[derive(Clone, Copy, PartialEq)]
pub enum HumidityCommand {
    Exists,
    ReadHumidity,
}

#[derive(GrantDefault)]
pub struct App {
    callback: Callback,
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

    fn enqueue_command(
        &self,
        command: HumidityCommand,
        arg1: usize,
        appid: AppId,
    ) -> CommandReturn {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.call_driver(command, arg1)
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }

    fn call_driver(&self, command: HumidityCommand, _: usize) -> CommandReturn {
        match command {
            HumidityCommand::ReadHumidity => self.driver.read_humidity().into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn configure_callback(
        &self,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = self
            .apps
            .enter(app_id, |app, _| {
                mem::swap(&mut app.callback, &mut callback);
            })
            .map_err(ErrorCode::from);

        if let Err(e) = res {
            Err((callback, e))
        } else {
            Ok(callback)
        }
    }
}

impl hil::sensors::HumidityClient for HumiditySensor<'_> {
    fn callback(&self, tmp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.schedule(tmp_val, 0, 0);
                }
            });
        }
    }
}

impl Driver for HumiditySensor<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            // subscribe to temperature reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> CommandReturn {
        match command_num {
            // check whether the driver exist!!
            0 => CommandReturn::success(),

            // single humidity measurement
            1 => self.enqueue_command(HumidityCommand::ReadHumidity, arg1, appid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
