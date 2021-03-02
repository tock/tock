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
//!     capsules::ninedof::NineDof::new(grant_ninedof));
//! ninedof.add_driver(fxos8700);
//! hil::sensors::NineDof::set_client(fxos8700, ninedof);
//! ```

use core::mem;
use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{
    AppId, Callback, CommandReturn, Driver, ErrorCode, Grant, GrantDefault, ProcessCallbackFactory,
};

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
    callback: Callback,
    pending_command: bool,
    command: NineDofCommand,
    arg1: usize,
}

impl GrantDefault for App {
    fn grant_default(_process_id: AppId, cb_factory: &mut ProcessCallbackFactory) -> App {
        App {
            callback: cb_factory.build_callback(0).unwrap(),
            pending_command: false,
            command: NineDofCommand::Exists,
            arg1: 0,
        }
    }
}

pub struct NineDof<'a> {
    drivers: &'a [&'a dyn hil::sensors::NineDof<'a>],
    apps: Grant<App>,
    current_app: OptionalCell<AppId>,
}

impl<'a> NineDof<'a> {
    pub fn new(drivers: &'a [&'a dyn hil::sensors::NineDof<'a>], grant: Grant<App>) -> NineDof<'a> {
        NineDof {
            drivers: drivers,
            apps: grant,
            current_app: OptionalCell::empty(),
        }
    }

    // Check so see if we are doing something. If not,
    // go ahead and do this command. If so, this is queued
    // and will be run when the pending command completes.
    fn enqueue_command(&self, command: NineDofCommand, arg1: usize, appid: AppId) -> CommandReturn {
        self.apps
            .enter(appid, |app, _| {
                if self.current_app.is_none() {
                    self.current_app.set(appid);
                    let value = self.call_driver(command, arg1);
                    if value != ReturnCode::SUCCESS {
                        self.current_app.clear();
                    }
                    CommandReturn::from(value)
                } else {
                    if app.pending_command == true {
                        CommandReturn::failure(ErrorCode::BUSY)
                    } else {
                        app.pending_command = true;
                        app.command = command;
                        app.arg1 = arg1;
                        CommandReturn::success()
                    }
                }
            })
            .unwrap_or_else(|err| {
                let rcode: ReturnCode = err.into();
                CommandReturn::from(rcode)
            })
    }

    fn call_driver(&self, command: NineDofCommand, _: usize) -> ReturnCode {
        match command {
            NineDofCommand::ReadAccelerometer => {
                let mut data = ReturnCode::ENODEVICE;
                for driver in self.drivers.iter() {
                    data = driver.read_accelerometer();
                    if data == ReturnCode::SUCCESS {
                        break;
                    }
                }
                data
            }
            NineDofCommand::ReadMagnetometer => {
                let mut data = ReturnCode::ENODEVICE;
                for driver in self.drivers.iter() {
                    data = driver.read_magnetometer();
                    if data == ReturnCode::SUCCESS {
                        break;
                    }
                }
                data
            }
            NineDofCommand::ReadGyroscope => {
                let mut data = ReturnCode::ENODEVICE;
                for driver in self.drivers.iter() {
                    data = driver.read_gyroscope();
                    if data == ReturnCode::SUCCESS {
                        break;
                    }
                }
                data
            }
            _ => ReturnCode::ENOSUPPORT,
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

impl hil::sensors::NineDofClient for NineDof<'_> {
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
                app.callback.schedule(arg1, arg2, arg3);
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
                    app.callback.schedule(arg1, arg2, arg3);
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

impl Driver for NineDof<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            0 => self.configure_callback(callback, app_id),
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            // Single acceleration reading.
            1 => self.enqueue_command(NineDofCommand::ReadAccelerometer, arg1, appid),

            // Single magnetometer reading.
            100 => self.enqueue_command(NineDofCommand::ReadMagnetometer, arg1, appid),

            // Single gyroscope reading.
            200 => self.enqueue_command(NineDofCommand::ReadGyroscope, arg1, appid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
