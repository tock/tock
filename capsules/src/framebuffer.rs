//! Provides userspace with access to the frame buffer.
//!
//! Usage
//! -----
//!
//! You need a screen that provides the `hil::framebuffer::Screen` trait.
//!
//! ```rust
//!
//! let framebuffer =
//!     components::framebuffer::FramebufferComponent::new(board_kernel, tft).finalize();
//! ```

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::hil::framebuffer::{ScreenPixelFormat, ScreenRotation};
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Framebuffer as usize;

#[derive(Clone, Copy, PartialEq)]
enum FramebufferCommand {
    Nop,
    On,
    Off,
    InvertOn,
    InvertOff,
    GetSupportedResolutionModes,
    GetSupportedResolution,
    GetSupportedPixelFormats,
    GetSupportedPixelFormat,
    GetRotation,
    SetRotation,
    GetResolution,
    SetResolution,
    GetPixelFormat,
    SetPixelFormat,
    Write,
    Fill,
}

fn pixels_in_bytes(pixels: usize, bits_per_pixel: usize) -> usize {
    let bytes = pixels * bits_per_pixel / 8;
    if pixels * bits_per_pixel % 8 != 0 {
        bytes + 1
    } else {
        bytes
    }
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
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            write_len: 0,
            write_position: 0,
        }
    }
}

pub struct Framebuffer<'a> {
    screen: &'a dyn hil::framebuffer::Screen,
    screen_setup: Option<&'a dyn hil::framebuffer::ScreenSetup>,
    apps: Grant<App>,
    screen_ready: Cell<bool>,
    current_app: OptionalCell<AppId>,
    pixel_format: Cell<ScreenPixelFormat>,
}

impl Framebuffer<'a> {
    pub fn new(
        screen: &'a dyn hil::framebuffer::Screen,
        screen_setup: Option<&'a dyn hil::framebuffer::ScreenSetup>,
        grant: Grant<App>,
    ) -> Framebuffer<'a> {
        Framebuffer {
            screen: screen,
            screen_setup: screen_setup,
            apps: grant,
            current_app: OptionalCell::empty(),
            screen_ready: Cell::new(false),
            pixel_format: Cell::new(screen.get_pixel_format()),
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
                if self.screen_ready.get() && self.current_app.is_none() {
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

    fn call_screen(
        &self,
        command: FramebufferCommand,
        data1: usize,
        data2: usize,
        appid: AppId,
    ) -> ReturnCode {
        match command {
            FramebufferCommand::On => self.screen.on(),
            FramebufferCommand::Off => self.screen.off(),
            FramebufferCommand::InvertOn => self.screen.invert_on(),
            FramebufferCommand::InvertOff => self.screen.invert_off(),
            FramebufferCommand::SetRotation => {
                if let Some(screen) = self.screen_setup {
                    screen.set_rotation(
                        ScreenRotation::from_usize(data1).unwrap_or(ScreenRotation::Normal),
                    )
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::GetRotation => {
                let rotation = self.screen.get_rotation();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), rotation as usize, 0);
                ReturnCode::SUCCESS
            }
            FramebufferCommand::SetResolution => {
                if let Some(screen) = self.screen_setup {
                    screen.set_resolution((data1, data2))
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::GetResolution => {
                let (width, height) = self.screen.get_resolution();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), width, height);
                ReturnCode::SUCCESS
            }
            FramebufferCommand::SetPixelFormat => {
                if let Some(pixel_format) = ScreenPixelFormat::from_usize(data1) {
                    if let Some(screen) = self.screen_setup {
                        screen.set_pixel_format(pixel_format)
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                } else {
                    ReturnCode::EINVAL
                }
            }
            FramebufferCommand::GetPixelFormat => {
                let pixel_format = self.screen.get_pixel_format();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), pixel_format as usize, 0);
                ReturnCode::SUCCESS
            }
            FramebufferCommand::GetSupportedResolutionModes => {
                if let Some(screen) = self.screen_setup {
                    let resolution_modes = screen.get_num_supported_resolutions();
                    self.run_next_command(usize::from(ReturnCode::SUCCESS), resolution_modes, 0);
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::GetSupportedResolution => {
                if let Some(screen) = self.screen_setup {
                    if let Some((width, height)) = screen.get_supported_resolution(data1) {
                        self.run_next_command(
                            usize::from(if width > 0 && height > 0 {
                                ReturnCode::SUCCESS
                            } else {
                                ReturnCode::EINVAL
                            }),
                            width,
                            height,
                        );
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::GetSupportedPixelFormats => {
                if let Some(screen) = self.screen_setup {
                    let color_modes = screen.get_num_supported_pixel_formats();
                    self.run_next_command(usize::from(ReturnCode::SUCCESS), color_modes, 0);
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::GetSupportedPixelFormat => {
                if let Some(screen) = self.screen_setup {
                    if let Some(pixel_format) = screen.get_supported_pixel_format(data1) {
                        self.run_next_command(
                            usize::from(ReturnCode::SUCCESS),
                            pixel_format as usize,
                            0,
                        );
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            FramebufferCommand::Fill => self
                .apps
                .enter(appid, |app, _| {
                    if app.shared.is_some() {
                        app.write_position = 0;
                        app.write_len = pixels_in_bytes(
                            app.width * app.height,
                            self.pixel_format.get().get_bits_per_pixel(),
                        );
                        self.screen.write(app.x, app.y, app.width, app.height)
                    } else {
                        ReturnCode::ENOMEM
                    }
                })
                .unwrap_or_else(|err| err.into()),
            FramebufferCommand::Write => self
                .apps
                .enter(appid, |app, _| {
                    let len = if let Some(ref shared) = app.shared {
                        shared.len()
                    } else {
                        0
                    };
                    if len > 0 {
                        app.write_position = 0;
                        app.write_len = len;
                        self.screen.write(app.x, app.y, app.width, app.height)
                    } else {
                        ReturnCode::ENOMEM
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn run_next_command(&self, data1: usize, data2: usize, data3: usize) {
        if !self.screen_ready.get() {
            self.screen_ready.set(true);
        } else {
            self.current_app.take().map(|appid| {
                let _ = self.apps.enter(appid, |app, _| {
                    app.pending_command = false;
                    app.callback.map(|mut cb| {
                        cb.schedule(data1, data2, data3);
                    });
                });
            });
        }

        // Check if there are any pending events.
        for app in self.apps.iter() {
            let started_command = app.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(app.appid());
                    self.call_screen(app.command, app.data1, app.data2, app.appid())
                        == ReturnCode::SUCCESS
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

impl hil::framebuffer::ScreenClient for Framebuffer<'a> {
    fn fill_next_buffer_for_write(&self, buffer: &'b mut [u8]) -> usize {
        self.current_app.map_or_else(
            || 0,
            |appid| {
                self.apps
                    .enter(*appid, |app, _| {
                        let position = app.write_position;
                        let mut len = app.write_len;
                        if position < len {
                            let buffer_size = buffer.len();
                            if app.command == FramebufferCommand::Write {
                                if let Some(ref mut s) = app.shared {
                                    let mut chunks = s.chunks(buffer_size);
                                    let chunk_number = position / buffer_size;
                                    let initial_pos = chunk_number * buffer_size;

                                    let mut pos = initial_pos;
                                    if let Some(chunk) = chunks.nth(chunk_number) {
                                        for (i, byte) in chunk.iter().enumerate() {
                                            if pos < len {
                                                buffer[i] = *byte;
                                                pos = pos + 1
                                            } else {
                                                break;
                                            }
                                        }
                                        app.write_position = pos;
                                        app.write_len - initial_pos
                                    } else {
                                        // stop writing
                                        0
                                    }
                                } else {
                                    // TODO should panic or report an error?
                                    panic!("framebuffer has no slice to send");
                                }
                            } else if app.command == FramebufferCommand::Fill {
                                // TODO bytes per pixel
                                len = len - position;
                                let bytes_per_pixel = pixels_in_bytes(
                                    1,
                                    self.pixel_format.get().get_bits_per_pixel(),
                                );
                                let mut write_len = buffer_size / bytes_per_pixel;
                                if write_len > len {
                                    write_len = len
                                };
                                if let Some(ref mut s) = app.shared {
                                    let mut bytes = s.iter();
                                    // bytes per pixel
                                    for i in 0..bytes_per_pixel {
                                        if let Some(byte) = bytes.next() {
                                            buffer[i] = *byte;
                                        }
                                    }
                                    for i in 1..write_len {
                                        // bytes per pixel
                                        for j in 0..bytes_per_pixel {
                                            buffer[bytes_per_pixel * i + j] = buffer[j]
                                        }
                                    }
                                } else {
                                    // TODO should panic or report an error?
                                    panic!("framebuffer has no slice to send");
                                }
                                app.write_position = app.write_position + write_len * 2;
                                write_len * 2
                            } else {
                                // unknown command
                                // stop writing
                                0
                            }
                        } else {
                            0
                        }
                    })
                    .unwrap_or_else(|_| 0)
            },
        )
    }
    fn command_complete(&self, r: ReturnCode) {
        self.run_next_command(usize::from(r), 0, 0);
    }

    fn screen_is_ready(&self) {
        self.run_next_command(usize::from(ReturnCode::SUCCESS), 0, 0);
    }
}

impl hil::framebuffer::ScreenSetupClient for Framebuffer<'a> {
    fn command_complete(&self, r: ReturnCode) {
        self.run_next_command(usize::from(r), 0, 0);
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
            // Does it have the screen setup
            1 => ReturnCode::SuccessWithValue {
                value: self.screen_setup.is_some() as usize,
            },
            // On
            2 => self.enqueue_command(FramebufferCommand::On, 0, 0, appid),
            // Off
            3 => self.enqueue_command(FramebufferCommand::Off, 0, 0, appid),
            // Invert On
            4 => self.enqueue_command(FramebufferCommand::InvertOn, 0, 0, appid),
            // Invert Off
            5 => self.enqueue_command(FramebufferCommand::InvertOff, 0, 0, appid),

            // Get Resolution Modes Number
            11 => {
                self.enqueue_command(FramebufferCommand::GetSupportedResolutionModes, 0, 0, appid)
            }
            // Get Resolution Mode Width and Height
            12 => self.enqueue_command(FramebufferCommand::GetSupportedResolution, data1, 0, appid),

            // Get Color Depth Modes Number
            13 => self.enqueue_command(FramebufferCommand::GetSupportedPixelFormats, 0, 0, appid),
            // Get Color Depth Mode Bits per Pixel
            14 => {
                self.enqueue_command(FramebufferCommand::GetSupportedPixelFormat, data1, 0, appid)
            }

            // Get Rotation
            21 => self.enqueue_command(FramebufferCommand::GetRotation, 0, 0, appid),
            // Set Rotation
            22 => self.enqueue_command(FramebufferCommand::SetRotation, data1, 0, appid),

            // Get Resolution
            23 => self.enqueue_command(FramebufferCommand::GetResolution, 0, 0, appid),
            // Set Resolution
            24 => self.enqueue_command(FramebufferCommand::SetResolution, data1, data2, appid),

            // Get Color Depth
            25 => self.enqueue_command(FramebufferCommand::GetPixelFormat, 0, 0, appid),
            // Set Color Depth
            26 => self.enqueue_command(FramebufferCommand::SetPixelFormat, data1, 0, appid),

            // Set Write Window
            100 => self
                .apps
                .enter(appid, |app, _| {
                    app.write_position = 0;
                    app.x = (data1 >> 16) & 0xFFFF;
                    app.y = data1 & 0xFFFF;
                    app.width = (data2 >> 16) & 0xFFFF;
                    app.height = data2 & 0xFFFF;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            // Write
            200 => self.enqueue_command(FramebufferCommand::Write, data1, data2, appid),
            // Fill
            300 => self.enqueue_command(FramebufferCommand::Fill, data1, data2, appid),

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
                    let depth =
                        pixels_in_bytes(1, self.screen.get_pixel_format().get_bits_per_pixel());
                    let len = if let Some(ref s) = slice { s.len() } else { 0 };
                    // allow only if the slice length is a a multiple of color depth
                    if len == 0 || (len > 0 && (len % depth == 0)) {
                        app.shared = slice;
                        app.write_position = 0;
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
