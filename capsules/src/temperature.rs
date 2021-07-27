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
//! * `1`: read the temperature
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

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Temperature as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
}

pub struct TemperatureSensor<'a> {
    driver: &'a dyn hil::sensors::TemperatureDriver<'a>,
    apps: Grant<App, 1>,
    busy: Cell<bool>,
}

impl<'a> TemperatureSensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::TemperatureDriver<'a>,
        grant: Grant<App, 1>,
    ) -> TemperatureSensor<'a> {
        TemperatureSensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, appid: ProcessId) -> CommandReturn {
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
}

impl hil::sensors::TemperatureClient for TemperatureSensor<'_> {
    fn callback(&self, temp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    upcalls.schedule_upcall(0, temp_val, 0, 0).ok();
                }
            });
        }
    }
}

impl SyscallDriver for TemperatureSensor<'_> {
    fn command(&self, command_num: usize, _: usize, _: usize, appid: ProcessId) -> CommandReturn {
        match command_num {
            // check whether the driver exists!!
            0 => CommandReturn::success(),

            // read temperature
            1 => self.enqueue_command(appid),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
