//! Provides userspace with access to the screen.
//!
//! Usage
//! -----
//!
//! You need a screen that provides the `hil::screen::Screen` trait.
//!
//! ```rust
//! let screen =
//!     components::screen::ScreenComponent::new(board_kernel, tft).finalize();
//! ```

use core::cell::Cell;
use core::convert::From;
use core::mem;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::screen::{ScreenPixelFormat, ScreenRotation};
use kernel::{
    AppId, Callback, CommandReturn, Driver, ErrorCode, Grant, GrantDefault, ProcessCallbackFactory,
    Read, ReadOnlyAppSlice, ReturnCode,
};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Screen as usize;

fn screen_rotation_from(screen_rotation: usize) -> Option<ScreenRotation> {
    match screen_rotation {
        0 => Some(ScreenRotation::Normal),
        1 => Some(ScreenRotation::Rotated90),
        2 => Some(ScreenRotation::Rotated180),
        3 => Some(ScreenRotation::Rotated270),
        _ => None,
    }
}

fn screen_pixel_format_from(screen_pixel_format: usize) -> Option<ScreenPixelFormat> {
    match screen_pixel_format {
        0 => Some(ScreenPixelFormat::Mono),
        1 => Some(ScreenPixelFormat::RGB_233),
        2 => Some(ScreenPixelFormat::RGB_565),
        3 => Some(ScreenPixelFormat::RGB_888),
        4 => Some(ScreenPixelFormat::ARGB_8888),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ScreenCommand {
    Nop,
    SetBrightness,
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
    SetWriteFrame,
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
    callback: Callback,
    pending_command: bool,
    shared: ReadOnlyAppSlice,
    write_position: usize,
    write_len: usize,
    command: ScreenCommand,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    data1: usize,
    data2: usize,
}

impl GrantDefault for App {
    fn grant_default(_process_id: AppId, _cb_factory: &mut ProcessCallbackFactory) -> App {
        App {
            callback: Callback::default(),
            pending_command: false,
            shared: ReadOnlyAppSlice::default(),
            command: ScreenCommand::Nop,
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

pub struct Screen<'a> {
    screen: &'a dyn hil::screen::Screen,
    screen_setup: Option<&'a dyn hil::screen::ScreenSetup>,
    apps: Grant<App>,
    screen_ready: Cell<bool>,
    current_app: OptionalCell<AppId>,
    pixel_format: Cell<ScreenPixelFormat>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Screen<'a> {
    pub fn new(
        screen: &'a dyn hil::screen::Screen,
        screen_setup: Option<&'a dyn hil::screen::ScreenSetup>,
        buffer: &'static mut [u8],
        grant: Grant<App>,
    ) -> Screen<'a> {
        Screen {
            screen: screen,
            screen_setup: screen_setup,
            apps: grant,
            current_app: OptionalCell::empty(),
            screen_ready: Cell::new(false),
            pixel_format: Cell::new(screen.get_pixel_format()),
            buffer: TakeCell::new(buffer),
        }
    }

    // Check to see if we are doing something. If not,
    // go ahead and do this command. If so, this is queued
    // and will be run when the pending command completes.
    fn enqueue_command(
        &self,
        command: ScreenCommand,
        data1: usize,
        data2: usize,
        appid: AppId,
    ) -> CommandReturn {
        let res = self
            .apps
            .enter(appid, |app, _| {
                if self.screen_ready.get() && self.current_app.is_none() {
                    self.current_app.set(appid);
                    app.command = command;
                    let r = self.call_screen(command, data1, data2, appid);
                    if r != ReturnCode::SUCCESS {
                        self.current_app.clear();
                    }
                    CommandReturn::from(r)
                } else {
                    if app.pending_command == true {
                        CommandReturn::failure(ErrorCode::BUSY)
                    } else {
                        app.pending_command = true;
                        app.command = command;
                        app.write_position = 0;
                        app.data1 = data1;
                        app.data2 = data2;
                        CommandReturn::success()
                    }
                }
            })
            .map_err(ErrorCode::from);
        match res {
            Err(e) => CommandReturn::failure(e),
            Ok(r) => r,
        }
    }

    fn call_screen(
        &self,
        command: ScreenCommand,
        data1: usize,
        data2: usize,
        appid: AppId,
    ) -> ReturnCode {
        match command {
            ScreenCommand::SetBrightness => self.screen.set_brightness(data1),
            ScreenCommand::InvertOn => self.screen.invert_on(),
            ScreenCommand::InvertOff => self.screen.invert_off(),
            ScreenCommand::SetRotation => {
                if let Some(screen) = self.screen_setup {
                    screen
                        .set_rotation(screen_rotation_from(data1).unwrap_or(ScreenRotation::Normal))
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            ScreenCommand::GetRotation => {
                let rotation = self.screen.get_rotation();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), rotation as usize, 0);
                ReturnCode::SUCCESS
            }
            ScreenCommand::SetResolution => {
                if let Some(screen) = self.screen_setup {
                    screen.set_resolution((data1, data2))
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            ScreenCommand::GetResolution => {
                let (width, height) = self.screen.get_resolution();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), width, height);
                ReturnCode::SUCCESS
            }
            ScreenCommand::SetPixelFormat => {
                if let Some(pixel_format) = screen_pixel_format_from(data1) {
                    if let Some(screen) = self.screen_setup {
                        screen.set_pixel_format(pixel_format)
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                } else {
                    ReturnCode::EINVAL
                }
            }
            ScreenCommand::GetPixelFormat => {
                let pixel_format = self.screen.get_pixel_format();
                self.run_next_command(usize::from(ReturnCode::SUCCESS), pixel_format as usize, 0);
                ReturnCode::SUCCESS
            }
            ScreenCommand::GetSupportedResolutionModes => {
                if let Some(screen) = self.screen_setup {
                    let resolution_modes = screen.get_num_supported_resolutions();
                    self.run_next_command(usize::from(ReturnCode::SUCCESS), resolution_modes, 0);
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            ScreenCommand::GetSupportedResolution => {
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
            ScreenCommand::GetSupportedPixelFormats => {
                if let Some(screen) = self.screen_setup {
                    let color_modes = screen.get_num_supported_pixel_formats();
                    self.run_next_command(usize::from(ReturnCode::SUCCESS), color_modes, 0);
                    ReturnCode::SUCCESS
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            ScreenCommand::GetSupportedPixelFormat => {
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
            ScreenCommand::Fill => {
                let res = self
                    .apps
                    .enter(appid, |app, _| {
                        // if it is larger than 0, we know it fits
                        // the size has been verified by subscribe
                        if app.shared.len() > 0 {
                            app.write_position = 0;
                            app.write_len = pixels_in_bytes(
                                app.width * app.height,
                                self.pixel_format.get().get_bits_per_pixel(),
                            );

                            self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
                                let len = self.fill_next_buffer_for_write(buffer);

                                if len > 0 {
                                    self.screen.write(buffer, len)
                                } else {
                                    self.buffer.replace(buffer);
                                    self.run_next_command(usize::from(ReturnCode::SUCCESS), 0, 0);
                                    ReturnCode::SUCCESS
                                }
                            })
                        } else {
                            ReturnCode::ENOMEM
                        }
                    })
                    .map_err(|err| err.into());

                match res {
                    Err(e) => e,
                    Ok(r) => r,
                }
            }
            ScreenCommand::Write => self
                .apps
                .enter(appid, |app, _| {
                    let len = if app.shared.len() < data1 {
                        app.shared.len()
                    } else {
                        data1
                    };
                    if len > 0 {
                        app.write_position = 0;
                        app.write_len = len;
                        self.buffer.take().map_or(ReturnCode::FAIL, |buffer| {
                            let len = self.fill_next_buffer_for_write(buffer);
                            if len > 0 {
                                self.screen.write(buffer, len)
                            } else {
                                self.buffer.replace(buffer);
                                self.run_next_command(usize::from(ReturnCode::SUCCESS), 0, 0);
                                ReturnCode::SUCCESS
                            }
                        })
                    } else {
                        ReturnCode::ENOMEM
                    }
                })
                .unwrap_or_else(|err| err.into()),
            ScreenCommand::SetWriteFrame => self
                .apps
                .enter(appid, |app, _| {
                    app.write_position = 0;
                    app.x = (data1 >> 16) & 0xFFFF;
                    app.y = data1 & 0xFFFF;
                    app.width = (data2 >> 16) & 0xFFFF;
                    app.height = data2 & 0xFFFF;
                    self.screen
                        .set_write_frame(app.x, app.y, app.width, app.height)
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
                    app.callback.schedule(data1, data2, data3);
                });
            });
        }

        // Check if there are any pending events.
        for app in self.apps.iter() {
            let started_command = app.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(app.appid());
                    let r = self.call_screen(app.command, app.data1, app.data2, app.appid());
                    if r != ReturnCode::SUCCESS {
                        self.current_app.clear();
                    }
                    r == ReturnCode::SUCCESS
                } else {
                    false
                }
            });
            if started_command {
                break;
            }
        }
    }

    fn fill_next_buffer_for_write<'b>(&self, buffer: &'b mut [u8]) -> usize {
        self.current_app.map_or_else(
            || 0,
            |appid| {
                self.apps
                    .enter(*appid, |app, _| {
                        let position = app.write_position;
                        let mut len = app.write_len;
                        if position < len {
                            let buffer_size = buffer.len();
                            let chunk_number = position / buffer_size;
                            let initial_pos = chunk_number * buffer_size;
                            let mut pos = initial_pos;
                            if app.command == ScreenCommand::Write {
                                let res = app.shared.map_or(0, |s| {
                                    let mut chunks = s.chunks(buffer_size);
                                    if let Some(chunk) = chunks.nth(chunk_number) {
                                        for (i, byte) in chunk.iter().enumerate() {
                                            if pos < len {
                                                buffer[i] = *byte;
                                                pos = pos + 1
                                            } else {
                                                break;
                                            }
                                        }
                                        app.write_len - initial_pos
                                    } else {
                                        // stop writing
                                        0
                                    }
                                });
                                if res > 0 {
                                    app.write_position = pos;
                                }
                                res
                            } else if app.command == ScreenCommand::Fill {
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
                                app.write_position =
                                    app.write_position + write_len * bytes_per_pixel;
                                app.shared.map_or(0, |data| {
                                    let mut bytes = data.iter();
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
                                    write_len * bytes_per_pixel
                                })
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
}

impl<'a> hil::screen::ScreenClient for Screen<'a> {
    fn command_complete(&self, r: ReturnCode) {
        self.run_next_command(usize::from(r), 0, 0);
    }

    fn write_complete(&self, buffer: &'static mut [u8], r: ReturnCode) {
        let len = self.fill_next_buffer_for_write(buffer);

        if r == ReturnCode::SUCCESS && len > 0 {
            self.screen.write_continue(buffer, len);
        } else {
            self.buffer.replace(buffer);
            self.run_next_command(usize::from(r), 0, 0);
        }
    }

    fn screen_is_ready(&self) {
        self.run_next_command(usize::from(ReturnCode::SUCCESS), 0, 0);
    }
}

impl<'a> hil::screen::ScreenSetupClient for Screen<'a> {
    fn command_complete(&self, r: ReturnCode) {
        self.run_next_command(usize::from(r), 0, 0);
    }
}

impl<'a> Driver for Screen<'a> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    mem::swap(&mut app.callback, &mut callback);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };
        if let Err(e) = res {
            Err((callback, e))
        } else {
            Ok(callback)
        }
    }

    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        appid: AppId,
    ) -> CommandReturn {
        match command_num {
            0 =>
            // This driver exists.
            {
                CommandReturn::success()
            }
            // Does it have the screen setup
            1 => CommandReturn::success_u32(self.screen_setup.is_some() as u32),
            // Set Brightness
            3 => self.enqueue_command(ScreenCommand::SetBrightness, data1, 0, appid),
            // Invert On
            4 => self.enqueue_command(ScreenCommand::InvertOn, 0, 0, appid),
            // Invert Off
            5 => self.enqueue_command(ScreenCommand::InvertOff, 0, 0, appid),

            // Get Resolution Modes Number
            11 => self.enqueue_command(ScreenCommand::GetSupportedResolutionModes, 0, 0, appid),
            // Get Resolution Mode Width and Height
            12 => self.enqueue_command(ScreenCommand::GetSupportedResolution, data1, 0, appid),

            // Get Color Depth Modes Number
            13 => self.enqueue_command(ScreenCommand::GetSupportedPixelFormats, 0, 0, appid),
            // Get Color Depth Mode Bits per Pixel
            14 => self.enqueue_command(ScreenCommand::GetSupportedPixelFormat, data1, 0, appid),

            // Get Rotation
            21 => self.enqueue_command(ScreenCommand::GetRotation, 0, 0, appid),
            // Set Rotation
            22 => self.enqueue_command(ScreenCommand::SetRotation, data1, 0, appid),

            // Get Resolution
            23 => self.enqueue_command(ScreenCommand::GetResolution, 0, 0, appid),
            // Set Resolution
            24 => self.enqueue_command(ScreenCommand::SetResolution, data1, data2, appid),

            // Get Color Depth
            25 => self.enqueue_command(ScreenCommand::GetPixelFormat, 0, 0, appid),
            // Set Color Depth
            26 => self.enqueue_command(ScreenCommand::SetPixelFormat, data1, 0, appid),

            // Set Write Frame
            100 => self.enqueue_command(ScreenCommand::SetWriteFrame, data1, data2, appid),
            // Write
            200 => self.enqueue_command(ScreenCommand::Write, data1, data2, appid),
            // Fill
            300 => self.enqueue_command(ScreenCommand::Fill, data1, data2, appid),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allow_readonly(
        &self,
        appid: AppId,
        allow_num: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        match allow_num {
            // TODO should refuse allow while writing
            0 => {
                let res = self
                    .apps
                    .enter(appid, |app, _| {
                        let depth =
                            pixels_in_bytes(1, self.screen.get_pixel_format().get_bits_per_pixel());
                        let len = slice.len();
                        // allow only if the slice length is a a multiple of color depth
                        if len == 0 || (len > 0 && (len % depth == 0)) {
                            app.write_position = 0;
                            mem::swap(&mut app.shared, &mut slice);
                            Ok(())
                        } else {
                            Err(ErrorCode::INVAL)
                        }
                    })
                    .map_err(ErrorCode::from);
                match res {
                    Err(e) => Err((slice, e)),
                    Ok(_) => Ok(slice),
                }
            }
            _ => Err((slice, ErrorCode::NOSUPPORT)),
        }
    }
}
