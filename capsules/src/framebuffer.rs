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
use kernel::debug;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Framebuffer as usize;

#[derive(Clone, Copy, PartialEq)]
enum FramebufferCommand {
    Nop,
    Write,
    Fill
}

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    shared: Option<AppSlice<Shared, u8>>,
    write_position: usize,
    write_len: usize,
    command: FramebufferCommand,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    data1: usize,
    data2: usize
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
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            write_len: 0,
            write_position: 0
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
                    app.command = command;
                    self.call_screen(command, data1, data2, appid)
                } else {
                    if app.pending_command == true {
                        ReturnCode::EBUSY
                    } else {
                        app.pending_command = true;
                        app.command = command;
                        app.write_position = 0;
                        app.data1 = data1;
                        app.data2 = data2;
                        ReturnCode::SUCCESS
                    }
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn call_screen(&self, command: FramebufferCommand, data1: usize, data2: usize, appid:AppId) -> ReturnCode {
        match command {
            FramebufferCommand::Fill => {
                self
                .apps
                .enter(appid, |app, _| {
                    if app.shared.is_some() {
                        app.write_position = 0;
                        app.write_len = app.width * app.height * 2;
                        debug!("fill len {}", data1);
                        self.screen.write_slice (app.x, app.y, app.width, app.height)
                    }
                    else
                    {
                        ReturnCode::ENOMEM
                    }
                })
                .unwrap_or_else(|err| err.into())
            }
            FramebufferCommand::Write => {
                self
                .apps
                .enter(appid, |app, _| {
                    if app.shared.is_some() {
                        app.write_position = 0;
                        app.write_len = data1;
                        debug!("write len {}", data1);
                        self.screen.write_slice (app.x, app.y, app.width, app.height)
                    }
                    else
                    {
                        ReturnCode::ENOMEM
                    }
                })
                .unwrap_or_else(|err| err.into())
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl hil::framebuffer::ScreenClient for Framebuffer<'a> {
    fn fill_buffer_for_write_slice (&self, buffer:&'b mut [u8]) -> usize {
        self.current_app.map_or_else (|| 0, |appid| {
            self.apps.enter (*appid, |app, _| {
                let position = app.write_position;
                debug! ("fill buffer {}", position);
                let mut len = app.write_len;
                debug! ("fill len {}", len);
                if position < len {
                    debug! ("position < len");
                    let buffer_size = buffer.len ();
                    if app.command == FramebufferCommand::Write {
                        debug! ("command write");
                        if let Some (ref mut s) = app.shared {
                            debug! ("shared");
                            let mut chunks = s.chunks (buffer_size);
                            let chunk_number = position / buffer_size;
                            let initial_pos = chunk_number*buffer_size;
                            
                            let mut pos = initial_pos;
                            if let Some (chunk) = chunks.nth (chunk_number) {
                                for (i, byte) in chunk.iter().enumerate() {
                                    if pos < len {
                                        buffer[i] = *byte;
                                        pos = pos + 1
                                    }
                                    else
                                    {
                                        break;
                                    }
                                }
                                app.write_position = pos;
                                app.write_len - initial_pos
                            }
                            else
                            {
                                // stop writing
                                0
                            }
                        }
                        else
                        {
                            // TODO should panic or report an error?
                            panic! ("framebuffer has no slice to send");
                        }
                    }
                    else if app.command == FramebufferCommand::Fill {
                        // TODO bytes per pixel
                        len = len - position;
                        let bytes_per_pixel = 2;
                        let mut write_len = buffer_size / bytes_per_pixel;
                        if write_len > len { write_len = len };
                        if let Some (ref mut s) = app.shared {
                            let mut bytes = s.iter ();
                            // bytes per pixel
                            for i in 0..2 {
                                if let Some (byte) = bytes.next() {
                                    buffer[i] = *byte;
                                }
                            }
                            for i in 1..write_len {
                                // bytes per pixel
                                for j in 0..2 {
                                    buffer[2*i+j] = buffer[j]
                                }
                            }
                        }
                        else
                        {
                            // TODO should panic or report an error?
                            panic! ("framebuffer has no slice to send");
                        }
                        app.write_position = app.write_position + write_len*2;
                        write_len*2
                    }
                    else
                    {
                        // unknown command
                        // stop writing
                        debug! ("unknown command");
                        0
                    }
                }
                else {
                    0
                }
            }).unwrap_or_else(|err| 0)
        })
    }
    fn write_complete(&self, buffer: Option<&'static mut [u8]>, _r: ReturnCode) {
        debug! ("write complete");
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.pending_command = false;
                app.callback.map(|mut cb| {
                    cb.schedule(0, 0, 0);
                });
            });
        });

        // Check if there are any pending events.
        for app in self.apps.iter() {
            let started_command = app.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(app.appid());
                    self.call_screen(app.command, app.data1, app.data2, app.appid()) == ReturnCode::SUCCESS
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

            // Set Window
            100 => {
                self
                .apps
                .enter(appid, |app, _| {
                    app.write_position = 0;
                    app.x = (data1 >> 16) & 0xFFFF;
                    app.y = data1 & 0xFFFF;
                    app.width = (data2 >> 16) & 0xFFFF;
                    app.height = data2 & 0xFFFF;
                    debug!("x {} y {} width {} height {}", app.x, app.y, app.width, app.height);
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into())
            }
            // Write
            200 => self.enqueue_command (FramebufferCommand::Write, data1, data2, appid),
            
            // Fill
            300 => self.enqueue_command (FramebufferCommand::Fill, data1, data2, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // TODO should refuse allow while writing
            0 => self
                .apps
                .enter(appid, |app, _| {
                    app.shared = slice;
                    app.write_position = 0;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
