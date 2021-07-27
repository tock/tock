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

use kernel::grant::Grant;
use kernel::hil;
use kernel::hil::screen::{ScreenPixelFormat, ScreenRotation};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

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
    SetBrightness(usize),
    InvertOn,
    InvertOff,
    GetSupportedResolutionModes,
    GetSupportedResolution(usize),
    GetSupportedPixelFormats,
    GetSupportedPixelFormat(usize),
    GetRotation,
    SetRotation(ScreenRotation),
    GetResolution,
    SetResolution {
        width: usize,
        height: usize,
    },
    GetPixelFormat,
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
    shared: ReadOnlyProcessBuffer,
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
            shared: ReadOnlyProcessBuffer::default(),
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
    apps: Grant<App, 1>,
    screen_ready: Cell<bool>,
    current_process: OptionalCell<ProcessId>,
    pixel_format: Cell<ScreenPixelFormat>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Screen<'a> {
    pub fn new(
        screen: &'a dyn hil::screen::Screen,
        screen_setup: Option<&'a dyn hil::screen::ScreenSetup>,
        buffer: &'static mut [u8],
        grant: Grant<App, 1>,
    ) -> Screen<'a> {
        Screen {
            screen: screen,
            screen_setup: screen_setup,
            apps: grant,
            current_process: OptionalCell::empty(),
            screen_ready: Cell::new(false),
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
                if self.screen_ready.get() && self.current_process.is_none() {
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

    fn call_screen(&self, command: ScreenCommand, process_id: ProcessId) -> Result<(), ErrorCode> {
        match command {
            ScreenCommand::SetBrightness(brighness) => self.screen.set_brightness(brighness),
            ScreenCommand::InvertOn => self.screen.invert_on(),
            ScreenCommand::InvertOff => self.screen.invert_off(),
            ScreenCommand::SetRotation(rotation) => {
                if let Some(screen) = self.screen_setup {
                    screen.set_rotation(rotation)
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetRotation => {
                let rotation = self.screen.get_rotation();
                self.run_next_command(
                    kernel::errorcode::into_statuscode(Ok(())),
                    rotation as usize,
                    0,
                );
                Ok(())
            }
            ScreenCommand::SetResolution { width, height } => {
                if let Some(screen) = self.screen_setup {
                    screen.set_resolution((width, height))
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetResolution => {
                let (width, height) = self.screen.get_resolution();
                self.run_next_command(kernel::errorcode::into_statuscode(Ok(())), width, height);
                Ok(())
            }
            ScreenCommand::SetPixelFormat(pixel_format) => {
                if let Some(screen) = self.screen_setup {
                    screen.set_pixel_format(pixel_format)
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetPixelFormat => {
                let pixel_format = self.screen.get_pixel_format();
                self.run_next_command(
                    kernel::errorcode::into_statuscode(Ok(())),
                    pixel_format as usize,
                    0,
                );
                Ok(())
            }
            ScreenCommand::GetSupportedResolutionModes => {
                if let Some(screen) = self.screen_setup {
                    let resolution_modes = screen.get_num_supported_resolutions();
                    self.run_next_command(
                        kernel::errorcode::into_statuscode(Ok(())),
                        resolution_modes,
                        0,
                    );
                    Ok(())
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetSupportedResolution(resolution_index) => {
                if let Some(screen) = self.screen_setup {
                    if let Some((width, height)) = screen.get_supported_resolution(resolution_index)
                    {
                        self.run_next_command(
                            kernel::errorcode::into_statuscode(if width > 0 && height > 0 {
                                Ok(())
                            } else {
                                Err(ErrorCode::INVAL)
                            }),
                            width,
                            height,
                        );
                        Ok(())
                    } else {
                        Err(ErrorCode::INVAL)
                    }
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetSupportedPixelFormats => {
                if let Some(screen) = self.screen_setup {
                    let color_modes = screen.get_num_supported_pixel_formats();
                    self.run_next_command(
                        kernel::errorcode::into_statuscode(Ok(())),
                        color_modes,
                        0,
                    );
                    Ok(())
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::GetSupportedPixelFormat(pixel_format_index) => {
                if let Some(screen) = self.screen_setup {
                    if let Some(pixel_format) =
                        screen.get_supported_pixel_format(pixel_format_index)
                    {
                        self.run_next_command(
                            kernel::errorcode::into_statuscode(Ok(())),
                            pixel_format as usize,
                            0,
                        );
                        Ok(())
                    } else {
                        Err(ErrorCode::INVAL)
                    }
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            ScreenCommand::Fill => match self
                .apps
                .enter(process_id, |app, _| {
                    // if it is larger than 0, we know it fits
                    // the size has been verified by subscribe
                    if app.shared.len() > 0 {
                        app.write_position = 0;
                        app.write_len = pixels_in_bytes(
                            app.width * app.height,
                            self.pixel_format.get().get_bits_per_pixel(),
                        );
                        Ok(())
                    } else {
                        Err(ErrorCode::NOMEM)
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
                .enter(process_id, |app, _| {
                    let len = if app.shared.len() < data_len {
                        app.shared.len()
                    } else {
                        data_len
                    };
                    if len > 0 {
                        app.write_position = 0;
                        app.write_len = len;
                        Ok(())
                    } else {
                        Err(ErrorCode::NOMEM)
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
        if !self.screen_ready.get() {
            self.screen_ready.set(true);
        } else {
            self.current_process.take().map(|process_id| {
                let _ = self.apps.enter(process_id, |app, upcalls| {
                    app.pending_command = false;
                    upcalls.schedule_upcall(0, data1, data2, data3).ok();
                });
            });
        }
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
                    .enter(*process_id, |app, _| {
                        let position = app.write_position;
                        let mut len = app.write_len;
                        if position < len {
                            let buffer_size = buffer.len();
                            let chunk_number = position / buffer_size;
                            let initial_pos = chunk_number * buffer_size;
                            let mut pos = initial_pos;
                            match app.command {
                                ScreenCommand::Write(_) => {
                                    let res = app
                                        .shared
                                        .enter(|s| {
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
                                    app.shared
                                        .enter(|data| {
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
            // Set Brightness
            3 => self.enqueue_command(ScreenCommand::SetBrightness(data1), process_id),
            // Invert On
            4 => self.enqueue_command(ScreenCommand::InvertOn, process_id),
            // Invert Off
            5 => self.enqueue_command(ScreenCommand::InvertOff, process_id),

            // Get Resolution Modes Number
            11 => self.enqueue_command(ScreenCommand::GetSupportedResolutionModes, process_id),
            // Get Resolution Mode Width and Height
            12 => self.enqueue_command(ScreenCommand::GetSupportedResolution(data1), process_id),

            // Get Color Depth Modes Number
            13 => self.enqueue_command(ScreenCommand::GetSupportedPixelFormats, process_id),
            // Get Color Depth Mode Bits per Pixel
            14 => self.enqueue_command(ScreenCommand::GetSupportedPixelFormat(data1), process_id),

            // Get Rotation
            21 => self.enqueue_command(ScreenCommand::GetRotation, process_id),
            // Set Rotation
            22 => self.enqueue_command(
                ScreenCommand::SetRotation(
                    screen_rotation_from(data1).unwrap_or(ScreenRotation::Normal),
                ),
                process_id,
            ),

            // Get Resolution
            23 => self.enqueue_command(ScreenCommand::GetResolution, process_id),
            // Set Resolution
            24 => self.enqueue_command(
                ScreenCommand::SetResolution {
                    width: data1,
                    height: data2,
                },
                process_id,
            ),

            // Get Color Depth
            25 => self.enqueue_command(ScreenCommand::GetPixelFormat, process_id),
            // Set Color Depth
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

    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut slice: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        match allow_num {
            // TODO should refuse allow while writing
            0 => {
                let res = self
                    .apps
                    .enter(process_id, |app, _| {
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

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
