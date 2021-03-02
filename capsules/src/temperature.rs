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
//! # use kernel::static_init;
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_temperature = board_kernel.create_grant(&grant_cap);
//!
//! let temp = static_init!(
//!        capsules::temperature::TemperatureSensor<'static>,
//!        capsules::temperature::TemperatureSensor::new(si7021,
//!                                                 board_kernel.create_grant(&grant_cap)));
//!
//! kernel::hil::sensors::TemperatureDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use core::convert::TryFrom;
use core::mem;
use kernel::hil;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(GrantDefault)]
pub struct App {
    #[subscribe_num = 0]
    callback: Callback,
    subscribed: bool,
}

pub struct TemperatureSensor<'a> {
    driver: &'a dyn hil::sensors::TemperatureDriver<'a>,
    apps: Grant<App>,
    busy: Cell<bool>,
}

impl<'a> TemperatureSensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::TemperatureDriver<'a>,
        grant: Grant<App>,
    ) -> TemperatureSensor<'a> {
        TemperatureSensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, appid: AppId) -> CommandReturn {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    let rcode = self.driver.read_temperature();
                    let eres = ErrorCode::try_from(rcode);
                    match eres {
                        Ok(ecode) => CommandReturn::failure(ecode),
                        _ => CommandReturn::success(),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
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

impl hil::sensors::TemperatureClient for TemperatureSensor<'_> {
    fn callback(&self, temp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.schedule(temp_val, 0, 0);
                }
            });
        }
    }
}

impl Driver for TemperatureSensor<'_> {
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

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> CommandReturn {
        match command_num {
            // check whether the driver exists!!
            0 => CommandReturn::success(),

            // read temperature
            1 => self.enqueue_command(appid),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
