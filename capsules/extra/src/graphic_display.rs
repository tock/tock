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
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

use kernel::hil::display::{GraphicsFrame, GraphicsMode, PixelFormat, Point, Rotation};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Screen as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const SHARED: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

fn screen_rotation_from(screen_rotation: usize) -> Option<Rotation> {
    match screen_rotation {
        0 => Some(Rotation::Normal),
        1 => Some(Rotation::Rotated90),
        2 => Some(Rotation::Rotated180),
        3 => Some(Rotation::Rotated270),
        _ => None,
    }
}

fn screen_pixel_format_from(screen_pixel_format: usize) -> Option<PixelFormat> {
    match screen_pixel_format {
        0 => Some(PixelFormat::Mono),
        1 => Some(PixelFormat::RGB_233),
        2 => Some(PixelFormat::RGB_565),
        3 => Some(PixelFormat::RGB_888),
        4 => Some(PixelFormat::ARGB_8888),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ScreenCommand {
    Nop,
    SetBrightness(u16),
    SetPower(bool),
    SetInvert(bool),
    SetRotation(Rotation),
    SetMode(GraphicsMode),
    // SetPixelFormat(PixelFormat),
    SetWriteFrame { origin: Point, size: GraphicsFrame },
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
    width: u16,
    height: u16,
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

pub struct GraphicDisplay<'a> {
    display: &'a dyn kernel::hil::display::GraphicDisplay<'a>,
    display_setup: Option<&'a dyn kernel::hil::display::FrameBufferSetup<'a>>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
    current_process: OptionalCell<ProcessId>,
    display_format: Cell<GraphicsMode>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> GraphicDisplay<'a> {
    pub fn new(
        display: &'a dyn kernel::hil::display::GraphicDisplay<'a>,
        display_setup: Option<&'a dyn kernel::hil::display::FrameBufferSetup<'a>>,
        buffer: &'static mut [u8],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
    ) -> GraphicDisplay<'a> {
        GraphicDisplay {
            display,
            display_setup,
            apps: grant,
            current_process: OptionalCell::empty(),
            display_format: Cell::new(display.get_mode()),
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
                if app.pending_command {
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
        let depth = pixels_in_bytes(1, self.display.get_mode().pixel_format.get_bits_per_pixel());
        (len % depth) == 0
    }

    fn call_screen(&self, command: ScreenCommand, process_id: ProcessId) -> Result<(), ErrorCode> {
        match command {
            ScreenCommand::SetBrightness(brighness) => self.display.set_brightness(brighness),
            ScreenCommand::SetPower(enabled) => self.display.set_power(enabled),
            ScreenCommand::SetInvert(enabled) => self.display.set_invert(enabled),
            ScreenCommand::SetRotation(rotation) => self.display.set_rotation(rotation),
            ScreenCommand::SetMode(graphics_mode) => {
                if let Some(display) = self.display_setup {
                    display.set_mode(graphics_mode)
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
                            app.width as usize * app.height as usize,
                            self.display_format.get().pixel_format.get_bits_per_pixel(),
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
                        self.display.write(buffer, len, false)
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
                        self.display.write(buffer, len, false)
                    } else {
                        self.buffer.replace(buffer);
                        self.display.flush()
                    }
                }),
                Err(e) => Err(e),
            },

            ScreenCommand::SetWriteFrame {
                origin: Point { x, y },
                size: GraphicsFrame { width, height },
            } => self
                .apps
                .enter(process_id, |app, _| {
                    app.write_position = 0;
                    app.width = width;
                    app.height = height;

                    self.display
                        .set_write_frame(Point { x, y }, GraphicsFrame { width, height })
                })
                .unwrap_or_else(|err| err.into()),
            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn schedule_callback(&self, data1: usize, data2: usize, data3: usize) {
        if let Some(process_id) = self.current_process.take() {
            let _ = self.apps.enter(process_id, |app, upcalls| {
                app.pending_command = false;
                upcalls.schedule_upcall(0, (data1, data2, data3)).ok();
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
        let (before, after) = self.display.get_buffer_padding();
        self.current_process.map_or_else(
            || 0,
            |process_id| {
                self.apps
                    .enter(process_id, |app, kernel_data| {
                        let position = app.write_position;
                        let mut len = app.write_len;
                        // debug!("position is {} and len is {}, (before, after) - ({}, {})", position, len, before, after);
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
                                                let mut chunks =
                                                    s.chunks(buffer_size - before - after);
                                                if let Some(chunk) = chunks.nth(chunk_number) {
                                                    for item in buffer.iter_mut().take(before) {
                                                        *item = 0x00;
                                                    }
                                                    for (i, byte) in chunk.iter().enumerate() {
                                                        if pos + after < len {
                                                            buffer[i + before] = byte.get();
                                                            pos += 1
                                                        } else {
                                                            break;
                                                        }
                                                    }
                                                    for item in buffer
                                                        .iter_mut()
                                                        .take(buffer_size)
                                                        .skip(before + chunk.len())
                                                    {
                                                        *item = 0x00;
                                                    }
                                                    app.write_len - initial_pos
                                                } else {
                                                    // stop writing
                                                    0
                                                }
                                            })
                                        })
                                        .unwrap_or(0);
                                    // debug!("in fill buffer {}", res);
                                    if res > 0 {
                                        app.write_position = pos;
                                    }
                                    res
                                }
                                ScreenCommand::Fill => {
                                    // TODO bytes per pixel
                                    len -= position;
                                    let bytes_per_pixel = pixels_in_bytes(
                                        1,
                                        self.display_format.get().pixel_format.get_bits_per_pixel(),
                                    );
                                    let mut write_len =
                                        (buffer_size - before - after) / bytes_per_pixel;
                                    if write_len > len {
                                        write_len = len
                                    };
                                    app.write_position += write_len * bytes_per_pixel;
                                    kernel_data
                                        .get_readonly_processbuffer(ro_allow::SHARED)
                                        .and_then(|shared| {
                                            shared.enter(|data| {
                                                let mut bytes = data.iter();
                                                // bytes per pixel

                                                for item in buffer.iter_mut().take(bytes_per_pixel)
                                                {
                                                    if let Some(byte) = bytes.next() {
                                                        *item = byte.get();
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
                    .unwrap_or(0)
            },
        )
    }
}

impl<'a> kernel::hil::display::ScreenClient for GraphicDisplay<'a> {
    fn command_complete(&self, r: Result<(), ErrorCode>) {
        // debug!("[display capsule] command complete received from screen client cu {:?}", r);
        self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
    }
}

impl<'a> kernel::hil::display::FrameBufferClient for GraphicDisplay<'a> {
    fn write_complete(&self, buffer: &'static mut [u8], r: Result<(), ErrorCode>) {
        // debug!("[display capsule] write complete received from client");
        let len = self.fill_next_buffer_for_write(buffer);

        if r == Ok(()) && len > 0 {
            let _ = self.display.write(buffer, len, false);
        } else {
            self.buffer.replace(buffer);
            let _ = self.display.flush();
            // self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
        }
    }

    fn command_complete(&self, r: Result<(), ErrorCode>) {
        // debug!("[display capsule] command complete received from frame buffer client");
        self.run_next_command(kernel::errorcode::into_statuscode(r), 0, 0);
    }
}

impl<'a> SyscallDriver for GraphicDisplay<'a> {
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
            1 => CommandReturn::success_u32(self.display_setup.is_some() as u32),
            // Set power
            2 => self.enqueue_command(ScreenCommand::SetPower(data1 != 0), process_id),
            // Set Brightness
            3 => self.enqueue_command(ScreenCommand::SetBrightness(data1 as u16), process_id),
            // Invert on (deprecated)
            4 => self.enqueue_command(ScreenCommand::SetInvert(true), process_id),
            // Invert off (deprecated)
            5 => self.enqueue_command(ScreenCommand::SetInvert(false), process_id),
            // Set Invert
            6 => self.enqueue_command(ScreenCommand::SetInvert(data1 != 0), process_id),

            // Get Graphics Modes count
            11 => {
                if let Some(display) = self.display_setup {
                    CommandReturn::success_u32(display.get_num_supported_modes() as u32)
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }
            // Get Graphics Mode
            12 => {
                if let Some(display) = self.display_setup {
                    match display.get_supported_mode(data1) {
                        Some(GraphicsMode {
                            frame: GraphicsFrame { width, height },
                            pixel_format,
                        }) if width > 0 && height > 0 => CommandReturn::success_u32_u32(
                            (width as u32) << 16 | (height as u32),
                            pixel_format as u32,
                        ),
                        _ => CommandReturn::failure(ErrorCode::INVAL),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                }
            }

            // Get Rotation
            21 => CommandReturn::success_u32(self.display.get_rotation() as u32),
            // Set Rotation
            22 => self.enqueue_command(
                ScreenCommand::SetRotation(screen_rotation_from(data1).unwrap_or(Rotation::Normal)),
                process_id,
            ),

            // Get Resolution
            23 => {
                let GraphicsMode {
                    frame: GraphicsFrame { width, height },
                    pixel_format: _,
                } = self.display.get_mode();
                CommandReturn::success_u32_u32(width as u32, height as u32)
            }
            // Set Resolution
            24 => {
                let GraphicsMode {
                    frame: _,
                    pixel_format,
                } = self.display.get_mode();
                self.enqueue_command(
                    ScreenCommand::SetMode(GraphicsMode {
                        frame: GraphicsFrame {
                            width: data1 as u16,
                            height: data2 as u16,
                        },
                        pixel_format,
                    }),
                    process_id,
                )
            }

            // Get pixel format
            25 => {
                let GraphicsMode {
                    frame: _,
                    pixel_format,
                } = self.display.get_mode();
                CommandReturn::success_u32(pixel_format as u32)
            }
            // Set pixel format
            26 => {
                if let Some(new_pixel_format) = screen_pixel_format_from(data1) {
                    let GraphicsMode {
                        frame,
                        pixel_format: _,
                    } = self.display.get_mode();
                    self.enqueue_command(
                        ScreenCommand::SetMode(GraphicsMode {
                            frame,
                            pixel_format: new_pixel_format,
                        }),
                        process_id,
                    )
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }

            // Set Write Frame
            100 => self.enqueue_command(
                ScreenCommand::SetWriteFrame {
                    origin: Point {
                        x: ((data1 >> 16) & 0xFFFF) as u16,
                        y: (data1 & 0xFFFF) as u16,
                    },
                    size: GraphicsFrame {
                        width: ((data2 >> 16) & 0xFFFF) as u16,
                        height: (data2 & 0xFFFF) as u16,
                    },
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
