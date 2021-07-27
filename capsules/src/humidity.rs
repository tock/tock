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
//! * `Ok(())`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `NOMEM`: No sufficient memory available.
//! * `INVAL`: Invalid address of the buffer or other error.
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
//! * `Ok(())`:    The operation has been successful.
//! * `BUSY`:      The driver is busy.
//! * `ENOSUPPORT`: Invalid `cmd`.
//! * `NOMEM`:     No sufficient memory available.
//! * `INVAL`:     Invalid address of the buffer or other error.
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

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

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
    subscribed: bool,
}

pub struct HumiditySensor<'a> {
    driver: &'a dyn hil::sensors::HumidityDriver<'a>,
    apps: Grant<App, 1>,
    busy: Cell<bool>,
}

impl<'a> HumiditySensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::HumidityDriver<'a>,
        grant: Grant<App, 1>,
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
        appid: ProcessId,
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
}

impl hil::sensors::HumidityClient for HumiditySensor<'_> {
    fn callback(&self, tmp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    upcalls.schedule_upcall(0, tmp_val, 0, 0).ok();
                }
            });
        }
    }
}

impl SyscallDriver for HumiditySensor<'_> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check whether the driver exist!!
            0 => CommandReturn::success(),

            // single humidity measurement
            1 => self.enqueue_command(HumidityCommand::ReadHumidity, arg1, appid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
