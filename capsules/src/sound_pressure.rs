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
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::SoundPressure as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
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

    fn enqueue_command(&self, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.driver.read_sound_pressure()
                } else {
                    ReturnCode::EBUSY
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
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
                        app.callback
                            .map(|mut cb| cb.schedule(sound_val.into(), 0, 0));
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
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to sound_pressure reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // check whether the driver exists!!
            0 => ReturnCode::SUCCESS,

            // read sound_pressure
            1 => self.enqueue_command(appid),

            // enable
            2 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.enable = true;
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into());
                self.enable();
                ReturnCode::SUCCESS
            }

            // disable
            3 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.enable = false;
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into());
                self.enable();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
