//! Provides userspace with virtualized access to 9DOF sensors.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::NineDof` trait.
//!
//! ```rust
//! # use kernel::{hil, static_init};
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_ninedof = board_kernel.create_grant(&grant_cap);
//!
//! let ninedof = static_init!(
//!     capsules::ninedof::NineDof<'static>,
//!     capsules::ninedof::NineDof::new(fxos8700, board_kernel.create_grant(&grant_ninedof));
//!
//! hil::sensors::NineDof::set_client(fxos8700, ninedof);
//! ```

use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::NINEDOF as usize;

#[derive(Clone, Copy, PartialEq)]
pub enum NineDofCommand {
    Exists,
    ReadAccelerometer,
    ReadMagnetometer,
    ReadGyroscope,
}

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    command: NineDofCommand,
    arg1: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            pending_command: false,
            command: NineDofCommand::Exists,
            arg1: 0,
        }
    }
}

pub struct NineDof<'a> {
    driver: &'a dyn hil::sensors::NineDof,
    apps: Grant<App>,
    current_app: OptionalCell<AppId>,
}

impl NineDof<'a> {
    pub fn new(driver: &'a dyn hil::sensors::NineDof, grant: Grant<App>) -> NineDof<'a> {
        NineDof {
            driver: driver,
            apps: grant,
            current_app: OptionalCell::empty(),
        }
    }

    // Check so see if we are doing something. If not,
    // go ahead and do this command. If so, this is queued
    // and will be run when the pending command completes.
    fn enqueue_command(&self, command: NineDofCommand, arg1: usize, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if self.current_app.is_none() {
                    self.current_app.set(appid);
                    self.call_driver(command, arg1)
                } else {
                    if app.pending_command == true {
                        ReturnCode::ENOMEM
                    } else {
                        app.pending_command = true;
                        app.command = command;
                        app.arg1 = arg1;
                        ReturnCode::SUCCESS
                    }
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn call_driver(&self, command: NineDofCommand, _: usize) -> ReturnCode {
        match command {
            NineDofCommand::ReadAccelerometer => self.driver.read_accelerometer(),
            NineDofCommand::ReadMagnetometer => self.driver.read_magnetometer(),
            NineDofCommand::ReadGyroscope => self.driver.read_gyroscope(),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl hil::sensors::NineDofClient for NineDof<'a> {
    fn callback(&self, arg1: usize, arg2: usize, arg3: usize) {
        // Notify the current application that the command finished.
        // Also keep track of what just finished to see if we can re-use
        // the result.
        let mut finished_command = NineDofCommand::Exists;
        let mut finished_command_arg = 0;
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.pending_command = false;
                finished_command = app.command;
                finished_command_arg = app.arg1;
                app.callback.map(|mut cb| {
                    cb.schedule(arg1, arg2, arg3);
                });
            });
        });

        // Check if there are any pending events.
        for cntr in self.apps.iter() {
            let started_command = cntr.enter(|app, _| {
                if app.pending_command
                    && app.command == finished_command
                    && app.arg1 == finished_command_arg
                {
                    // Don't bother re-issuing this command, just use
                    // the existing result.
                    app.pending_command = false;
                    app.callback.map(|mut cb| {
                        cb.schedule(arg1, arg2, arg3);
                    });
                    false
                } else if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(app.appid());
                    self.call_driver(app.command, app.arg1) == ReturnCode::SUCCESS
                } else {
                    false
                }
            });
            if started_command {
                break;
            }
        }
    }
}

impl Driver for NineDof<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    app.callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 =>
            /* This driver exists. */
            {
                ReturnCode::SUCCESS
            }

            // Single acceleration reading.
            1 => self.enqueue_command(NineDofCommand::ReadAccelerometer, arg1, appid),

            // Single magnetometer reading.
            100 => self.enqueue_command(NineDofCommand::ReadMagnetometer, arg1, appid),

            // Single gyroscope reading.
            200 => self.enqueue_command(NineDofCommand::ReadGyroscope, arg1, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
