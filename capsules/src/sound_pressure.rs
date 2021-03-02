//! Provides userspace with access to sound_pressure sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a sound_pressure sensor reading.
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
//! * `1`: read the sound_pressure
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
//! You need a device that provides the `hil::sensors::SoundPressure` trait.
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_sound_pressure = board_kernel.create_grant(&grant_cap);
//!
//! let temp = static_init!(
//!        capsules::sound_pressure::SoundPressureSensor<'static>,
//!        capsules::sound_pressure::SoundPressureSensor::new(si7021,
//!                                                 board_kernel.create_grant(&grant_cap)));
//!
//! kernel::hil::sensors::SoundPressure::set_client(si7021, temp);
//! ```

use core::cell::Cell;
use core::convert::TryFrom;
use core::mem;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::SoundPressure as usize;

#[derive(GrantDefault)]
pub struct App {
    #[subscribe_num = 0]
    callback: Callback,
    subscribed: bool,
    enable: bool,
}

pub struct SoundPressureSensor<'a> {
    driver: &'a dyn hil::sensors::SoundPressure<'a>,
    apps: Grant<App>,
    busy: Cell<bool>,
}

impl<'a> SoundPressureSensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::SoundPressure<'a>,
        grant: Grant<App>,
    ) -> SoundPressureSensor<'a> {
        SoundPressureSensor {
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
                    let res = self.driver.read_sound_pressure();
                    if let Ok(err) = ErrorCode::try_from(res) {
                        CommandReturn::failure(err)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }

    fn enable(&self) {
        let mut enable = false;
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.enable {
                    enable = true;
                }
            });
            if enable {
                self.driver.enable();
            } else {
                self.driver.disable();
            }
        }
    }
}

impl hil::sensors::SoundPressureClient for SoundPressureSensor<'_> {
    fn callback(&self, ret: ReturnCode, sound_val: u8) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                if app.subscribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    if ret == ReturnCode::SUCCESS {
                        app.callback.schedule(sound_val.into(), 0, 0);
                    }
                }
            });
        }
    }
}

impl Driver for SoundPressureSensor<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            // subscribe to sound_pressure reading with callback
            0 => {
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
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> CommandReturn {
        match command_num {
            // check whether the driver exists!!
            0 => CommandReturn::success(),

            // read sound_pressure
            1 => self.enqueue_command(appid),

            // enable
            2 => {
                let res = self
                    .apps
                    .enter(appid, |app, _| {
                        app.enable = true;
                        CommandReturn::success()
                    })
                    .map_err(ErrorCode::from);
                if let Err(e) = res {
                    CommandReturn::failure(e)
                } else {
                    self.enable();
                    CommandReturn::success()
                }
            }

            // disable
            3 => {
                let res = self
                    .apps
                    .enter(appid, |app, _| {
                        app.enable = false;
                        CommandReturn::success()
                    })
                    .map_err(ErrorCode::from);
                if let Err(e) = res {
                    CommandReturn::failure(e)
                } else {
                    self.enable();
                    CommandReturn::success()
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
