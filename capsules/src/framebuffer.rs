//! Provides userspace with virtualized access to 9DOF sensors.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::NineDof` trait.
//!
//! ```rust
//!
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_ninedof = board_kernel.create_grant(&grant_cap);
//!
//! let ninedof = static_init!(
//!     capsules::ninedof::NineDof<'static>,
//!     capsules::ninedof::NineDof::new(fxos8700, grant_ninedof));
//! hil::sensors::NineDof::set_client(fxos8700, ninedof);
//! ```

use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Framebuffer as usize;

#[derive(Clone, Copy, PartialEq)]
enum FramebufferCommand {
    Nop,
}

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    shared: Option<AppSlice<Shared, u8>>,
    command: FramebufferCommand,
    data1: usize,
    data2: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            pending_command: false,
            shared: None,
            command: FramebufferCommand::Nop,
            data1: 0,
            data2: 0,
        }
    }
}

pub struct Framebuffer<'a> {
    screen: &'a dyn hil::framebuffer::Screen,
    apps: Grant<App>,
    current_app: OptionalCell<AppId>,
}

impl Framebuffer<'a> {
    pub fn new(screen: &'a dyn hil::framebuffer::Screen, grant: Grant<App>) -> Framebuffer<'a> {
        Framebuffer {
            screen: screen,
            apps: grant,
            current_app: OptionalCell::empty(),
        }
    }

    // Check so see if we are doing something. If not,
    // go ahead and do this command. If so, this is queued
    // and will be run when the pending command completes.
    fn enqueue_command(
        &self,
        command: FramebufferCommand,
        data1: usize,
        data2: usize,
        appid: AppId,
    ) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if self.current_app.is_none() {
                    self.current_app.set(appid);
                    self.call_screen(command, data1, data2)
                } else {
                    if app.pending_command == true {
                        ReturnCode::EBUSY
                    } else {
                        app.pending_command = true;
                        app.command = command;
                        app.data1 = data1;
                        app.data2 = data2;
                        ReturnCode::SUCCESS
                    }
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn call_screen(&self, command: FramebufferCommand, data1: usize, data2: usize) -> ReturnCode {
        match command {
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl hil::framebuffer::ScreenClient for Framebuffer<'a> {
    fn write_complete(&self, r: ReturnCode) {}
}

impl Driver for Framebuffer<'a> {
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

    fn command(&self, command_num: usize, data1: usize, data2: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 =>
            /* This driver exists. */
            {
                ReturnCode::SUCCESS
            }

            1 => {
                ReturnCode::ENOSUPPORT
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
