//! Provides userspace with access to air quality sensors.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::AirQualityDriver` trait.
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_temperature = board_kernel.create_grant(&grant_cap);
//!
//! let temp = static_init!(
//!        capsules::temperature::AirQualitySensor<'static>,
//!        capsules::temperature::AirQualitySensor::new(si7021,
//!                                                 board_kernel.create_grant(&grant_cap)));
//!
//! kernel::hil::sensors::AirQualityDriver::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use core::convert::TryFrom;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::AirQuality as usize;

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    CO2,
    TVOC,
}

impl Default for Operation {
    fn default() -> Self {
        Operation::None
    }
}

#[derive(Default)]
pub struct App {
    operation: Operation,
}

pub struct AirQualitySensor<'a> {
    driver: &'a dyn hil::sensors::AirQualityDriver<'a>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    busy: Cell<bool>,
}

impl<'a> AirQualitySensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::AirQualityDriver<'a>,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> AirQualitySensor<'a> {
        AirQualitySensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, processid: ProcessId, op: Operation) -> CommandReturn {
        self.apps
            .enter(processid, |app, _| {
                if !self.busy.get() {
                    self.busy.set(true);
                    app.operation = op;

                    let rcode = match op {
                        Operation::None => Err(ErrorCode::FAIL),
                        Operation::CO2 => self.driver.read_co2(),
                        Operation::TVOC => self.driver.read_tvoc(),
                    };
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

impl hil::sensors::AirQualityClient for AirQualitySensor<'_> {
    fn environment_specified(&self, _result: Result<(), ErrorCode>) {
        unimplemented!();
    }

    fn co2_data_available(&self, value: Result<u32, ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.operation == Operation::CO2 {
                    value
                        .map(|co2| {
                            self.busy.set(false);
                            app.operation = Operation::None;
                            upcalls.schedule_upcall(0, (co2 as usize, 0, 0)).ok();
                        })
                        .ok();
                }
            });
        }
    }

    fn tvoc_data_available(&self, value: Result<u32, ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.operation == Operation::TVOC {
                    value
                        .map(|tvoc| {
                            self.busy.set(false);
                            app.operation = Operation::None;
                            upcalls.schedule_upcall(0, (tvoc as usize, 0, 0)).ok();
                        })
                        .ok();
                }
            });
        }
    }
}

impl SyscallDriver for AirQualitySensor<'_> {
    fn command(
        &self,
        command_num: usize,
        _: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check whether the driver exists!!
            0 => CommandReturn::success(),

            // specify temperature and humidity (TODO)
            1 => CommandReturn::failure(ErrorCode::NOSUPPORT),

            // read CO2
            2 => self.enqueue_command(processid, Operation::CO2),

            // read TVOC
            3 => self.enqueue_command(processid, Operation::TVOC),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
