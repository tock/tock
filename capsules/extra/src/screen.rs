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

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::hil::screen::{ScreenPixelFormat, ScreenRotation};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::Screen as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const SHARED: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

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
    SetBrightness(usize),
    SetPower(bool),
    SetInvert(bool),
    SetRotation(ScreenRotation),
    SetResolution {
        width: usize,
        height: usize,
    },
    SetPixelFormat(ScreenPixelFormat),
    SetWriteFrame {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
    Write(usize),
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
    pending_command: bool,
    write_position: usize,
    write_len: usize,
    command: ScreenCommand,
    width: usize,
    height: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            pending_command: false,
            command: ScreenCommand::Nop,
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
    apps: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
    current_process: OptionalCell<ProcessId>,
    pixel_format: Cell<ScreenPixelFormat>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Screen<'a> {
    pub fn new(
        screen: &'a dyn hil::screen::Screen,
        screen_setup: Option<&'a dyn hil::screen::ScreenSetup>,
        buffer: &'static mut [u8],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
    ) -> Screen<'a> {
        Screen {
            screen: screen,
            screen_setup: screen_setup,
            apps: grant,
            current_process: OptionalCell::empty(),
            pixel_format: Cell::new(screen.get_pixel_format()),
            buffer: TakeCell::new(buffer),
        }
    }

    // Check to see if we are doing something. If not,
    // go ahead and do this command. If so, this is queued
    // and will be run when the pending command completes.
    fn enqueue_command(&self, command: ScreenCommand, process_id: ProcessId) -> CommandReturn {
        match self
            .apps
            .enter(process_id, |app, _| {
                if app.pending_command == true {
                    CommandReturn::failure(ErrorCode::BUSY)
                } else {
                    app.pending_command = true;
                    app.command = command;
                    app.write_position = 0;
                    CommandReturn::success()
                }
            })
            .map_err(ErrorCode::from)
        {
            Err(e) => CommandReturn::failure(e),
            Ok(r) => {
                if self.current_process.is_none() {
                    self.current_process.set(process_id);
                    let r = self.call_screen(command, process_id);
                    if r != Ok(()) {
                        self.current_process.clear();
                    }
                    CommandReturn::from(r)
                } else {
                    r
                }
            }
        }
    }

    fn is_len_multiple_color_depth(&self, len: usize) -> bool {
        let depth = pixels_in_bytes(1, self.screen.get_pixel_format().get_bits_per_pixel());
        (len % depth) == 0
    }

    fn call_screen(&self, command: ScreenCommand, process_id: ProcessId) -> Result<(), ErrorCode> {
        match command {
            ScreenCommand::SetBrightness(brighness) => self.screen.set_brightness(brighness),
            ScreenCommand::SetPower(enabled) => self.screen.set_power(enabled),
            ScreenCommand::SetInvert(enabled) => self.screen.set_invert(enabled),
            ScreenCommand::SetRotation(rotation) => {
                if let Some(screen) = self.screen_setup {
                    screen.set_rotation(rotation)
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::SetResolution { width, height } => {
                if let Some(screen) = self.screen_setup {
                    screen.set_resolution((width, height))
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::SetPixelFormat(pixel_format) => {
                if let Some(screen) = self.screen_setup {
                    screen.set_pixel_format(pixel_format)
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::Fill => match self
                .apps
                .enter(process_id, |app, kernel_data| {
                    let len = kernel_data
                        .get_readonly_processbuffer(ro_allow::SHARED)
                        .map_or(0, |shared| shared.len());
                    // Ensure we have a buffer that is the correct size
                    if len == 0 {
                        Err(ErrorCode::NOMEM)
                    } else if !self.is_len_multiple_color_depth(len) {
                        Err(ErrorCode::INVAL)
                    } else {
                        app.write_position = 0;
                        app.write_len = pixels_in_bytes(
                            app.width * app.height,
                            self.pixel_format.get().get_bits_per_pixel(),
                        );
                        Ok(())
                    }
                })
                .unwrap_or_else(|err| err.into())
            {
                Err(e) => Err(e),
                Ok(()) => self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
                    let len = self.fill_next_buffer_for_write(buffer);
                    if len > 0 {
                        self.screen.write(buffer, len)
                    } else {
                        self.buffer.replace(buffer);
                        self.run_next_command(kernel::errorcode::into_statuscode(Ok(())), 0, 0);
                        Ok(())
                    }
                }),
            },

            ScreenCommand::Write(data_len) => match self
                .apps
                .enter(process_id, |app, kernel_data| {
                    let len = kernel_data
                        .get_readonly_processbuffer(ro_allow::SHARED)
                        .map_or(0, |shared| shared.len())
                        .min(data_len);
                    // Ensure we have a buffer that is the correct size
                    if len == 0 {
                        Err(ErrorCode::NOMEM)
                    } else if !self.is_len_multiple_color_depth(len) {
                        Err(ErrorCode::INVAL)
                    } else {
                        app.write_position = 0;
                        app.write_len = len;
                        Ok(())
                    }
                })
                .unwrap_or_else(|err| err.into())
            {
                Ok(()) => self.buffer.take().map_or(Err(ErrorCode::FAIL), |buffer| {
                    let len = self.fill_next_buffer_for_write(buffer);
                    if len > 0 {
                        self.screen.write(buffer, len)
                    } else {
                        self.buffer.replace(buffer);
                        self.run_next_command(kernel::errorcode::into_statuscode(Ok(())), 0, 0);
                        Ok(())
                    }
                }),
                Err(e) => Err(e),
            },
            ScreenCommand::SetWriteFrame {
                x,
                y,
                width,
                height,
            } => self
                .apps
                .enter(process_id, |app, _| {
                    app.write_position = 0;
                    app.width = width;
                    app.height = height;

                    self.screen.set_write_frame(x, y, width, height)
                })
                .unwrap_or_else(|err| err.into()),
            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn schedule_callback(&self, data1: usize, data2: usize, data3: usize) {
        self.current_process.take().map(|process_id| {
            let _ = self.apps.enter(process_id, |app, upcalls| {
                app.pending_command = false;
                upcalls.schedule_upcall(0, (data1, data2, data3)).ok();
            });
        });
    }

    fn run_next_command(&self, data1: usize, data2: usize, data3: usize) {
        self.schedule_callback(data1, data2, data3);

        let mut command = ScreenCommand::Nop;

        // Check if there are any pending events.
        for app in self.apps.iter() {
            let process_id = app.processid();
            let start_command = app.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    command = app.command;
                    self.current_process.set(process_id);
                    true
                } else {
                    false
                }
            });
            if start_command {
                match self.call_screen(command, process_id) {
                    Err(err) => {
                        self.current_process.clear();
                        self.schedule_callback(kernel::errorcode::into_statuscode(Err(err)), 0, 0);
                    }
                    Ok(()) => {
                        break;
                    }
                }
            }
        }
    }

    fn fill_next_buffer_for_write(&self, buffer: &mut [u8]) -> usize {
        self.current_process.map_or_else(
            || 0,
            |process_id| {
                self.apps
                    .enter(*process_id, |app, kernel_data| {
                        let position = app.write_position;
                        let mut len = app.write_len;
                        if position < len {
                            let buffer_size = buffer.len();
                            let chunk_number = position / buffer_size;
                            let initial_pos = chunk_number * buffer_size;
                            let mut pos = initial_pos;
                            match app.command {
                                ScreenCommand::Write(_) => {
                                    let res = kernel_data
                                        .get_readonly_processbuffer(ro_allow::SHARED)
                                        .and_then(|shared| {
                                            shared.enter(|s| {
                                                let mut chunks = s.chunks(buffer_size);
                                                if let Some(chunk) = chunks.nth(chunk_number) {
                                                    for (i, byte) in chunk.iter().enumerate() {
                                                        if pos < len {
                                                            buffer[i] = byte.get();
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
                                            })
                                        })
                                        .unwrap_or(0);
                                    if res > 0 {
                                        app.write_position = pos;
                                    }
                                    res
                                }
                                ScreenCommand::Fill => {
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
                                    kernel_data
                                        .get_readonly_processbuffer(ro_allow::SHARED)
                                        .and_then(|shared| {
                                            shared.enter(|data| {
                                                let mut bytes = data.iter();
                                                // bytes per pixel
                                                for i in 0..bytes_per_pixel {
                                                    if let Some(byte) = bytes.next() {
                                                        buffer[i] = byte.get();
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
                                        })
                                        .unwrap_or(0)
                                }
                                _ => 0,
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
    fn command_complete(&self, r: Result<(), ErrorCode>) {
        self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
    }

    fn write_complete(&self, buffer: &'static mut [u8], r: Result<(), ErrorCode>) {
        let len = self.fill_next_buffer_for_write(buffer);

        if r == Ok(()) && len > 0 {
            let _ = self.screen.write_continue(buffer, len);
        } else {
            self.buffer.replace(buffer);
            self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
        }
    }

    fn screen_is_ready(&self) {
        self.run_next_command(kernel::errorcode::into_statuscode(Ok(())), 0, 0);
    }
}

impl<'a> hil::screen::ScreenSetupClient for Screen<'a> {
    fn command_complete(&self, r: Result<(), ErrorCode>) {
        self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
    }
}

impl<'a> SyscallDriver for Screen<'a> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 =>
            // This driver exists.
            {
                CommandReturn::success()
            }
            // Does it have the screen setup
            1 => CommandReturn::success_u32(self.screen_setup.is_some() as u32),
            // Set power
            2 => self.enqueue_command(ScreenCommand::SetPower(data1 != 0), process_id),
            // Set Brightness
            3 => self.enqueue_command(ScreenCommand::SetBrightness(data1), process_id),
            // Invert on (deprecated)
            4 => self.enqueue_command(ScreenCommand::SetInvert(true), process_id),
            // Invert off (deprecated)
            5 => self.enqueue_command(ScreenCommand::SetInvert(false), process_id),
            // Set Invert
            6 => self.enqueue_command(ScreenCommand::SetInvert(data1 != 0), process_id),

            // Get Resolution Modes count
            11 => {
                if let Some(screen) = self.screen_setup {
                    CommandReturn::success_u32(screen.get_num_supported_resolutions() as u32)
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }
            // Get Resolution Mode Width and Height
            12 => {
                if let Some(screen) = self.screen_setup {
                    match screen.get_supported_resolution(data1) {
                        Some((width, height)) if width > 0 && height > 0 => {
                            CommandReturn::success_u32_u32(width as u32, height as u32)
                        }
                        _ => CommandReturn::failure(ErrorCode::INVAL),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }

            // Get pixel format Modes count
            13 => {
                if let Some(screen) = self.screen_setup {
                    CommandReturn::success_u32(screen.get_num_supported_pixel_formats() as u32)
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }
            // Get supported pixel format
            14 => {
                if let Some(screen) = self.screen_setup {
                    match screen.get_supported_pixel_format(data1) {
                        Some(pixel_format) => CommandReturn::success_u32(pixel_format as u32),
                        _ => CommandReturn::failure(ErrorCode::INVAL),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }

            // Get Rotation
            21 => CommandReturn::success_u32(self.screen.get_rotation() as u32),
            // Set Rotation
            22 => self.enqueue_command(
                ScreenCommand::SetRotation(
                    screen_rotation_from(data1).unwrap_or(ScreenRotation::Normal),
                ),
                process_id,
            ),

            // Get Resolution
            23 => {
                let (width, height) = self.screen.get_resolution();
                CommandReturn::success_u32_u32(width as u32, height as u32)
            }
            // Set Resolution
            24 => self.enqueue_command(
                ScreenCommand::SetResolution {
                    width: data1,
                    height: data2,
                },
                process_id,
            ),

            // Get pixel format
            25 => CommandReturn::success_u32(self.screen.get_pixel_format() as u32),
            // Set pixel format
            26 => {
                if let Some(pixel_format) = screen_pixel_format_from(data1) {
                    self.enqueue_command(ScreenCommand::SetPixelFormat(pixel_format), process_id)
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }

            // Set Write Frame
            100 => self.enqueue_command(
                ScreenCommand::SetWriteFrame {
                    x: (data1 >> 16) & 0xFFFF,
                    y: data1 & 0xFFFF,
                    width: (data2 >> 16) & 0xFFFF,
                    height: data2 & 0xFFFF,
                },
                process_id,
            ),
            // Write
            200 => self.enqueue_command(ScreenCommand::Write(data1), process_id),
            // Fill
            300 => self.enqueue_command(ScreenCommand::Fill, process_id),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
