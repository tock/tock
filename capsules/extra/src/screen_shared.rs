// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Shares a screen among multiple userspace processes.
//!
//! The screen can be split into multiple regions, and regions are assigned to
//! processes by AppID.
//!
//! Boards should create an array of `AppScreenRegion` objects that assign apps
//! to specific regions (frames) within the screen.
//!
//! ```rust,ignore
//! AppScreenRegion {
//!     app_id: kernel::process:ShortId::new(id),
//!     frame: Frame {
//!         x: 0,
//!         y: 0,
//!         width: 8,
//!         height: 16,
//!     }
//! }
//! ```
//!
//! This driver uses a subset of the API from `Screen`. It does not support any
//! screen config settings (brightness, invert) as those operations affect the
//! entire screen.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Screen as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const SHARED: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Clone, Copy, PartialEq)]
enum ScreenCommand {
    WriteSetFrame,
    WriteBuffer,
}

fn pixels_in_bytes(pixels: usize, bits_per_pixel: usize) -> usize {
    let bytes = pixels * bits_per_pixel / 8;
    if pixels * bits_per_pixel % 8 != 0 {
        bytes + 1
    } else {
        bytes
    }
}

/// Rectangular region of a screen.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Frame {
    /// X coordinate of the upper left corner of the frame.
    x: usize,
    /// Y coordinate of the upper left corner of the frame.
    y: usize,
    /// Width of the frame.
    width: usize,
    /// Height of the frame.
    height: usize,
}

pub struct AppScreenRegion {
    app_id: kernel::process::ShortId,
    frame: Frame,
}

impl AppScreenRegion {
    pub fn new(
        app_id: kernel::process::ShortId,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            app_id,
            frame: Frame {
                x,
                y,
                width,
                height,
            },
        }
    }
}

#[derive(Default)]
pub struct App {
    /// The app has requested some screen operation, or `None()` if idle.
    command: Option<ScreenCommand>,
    /// The current frame the app is using.
    frame: Frame,
}

/// A userspace driver that allows multiple apps to use the same screen.
///
/// Each app is given a pre-set rectangular region of the screen to use.
pub struct ScreenShared<'a, S: hil::screen::Screen<'a>> {
    /// Underlying screen driver to use.
    screen: &'a S,

    /// Grant region for apps using the screen.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,

    /// Static allocations of screen regions for each app.
    apps_regions: &'a [AppScreenRegion],

    /// The process currently executing a command on the screen.
    current_process: OptionalCell<ProcessId>,

    /// Internal buffer for write commands.
    buffer: TakeCell<'static, [u8]>,
}

impl<'a, S: hil::screen::Screen<'a>> ScreenShared<'a, S> {
    pub fn new(
        screen: &'a S,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<{ ro_allow::COUNT }>, AllowRwCount<0>>,
        buffer: &'static mut [u8],
        apps_regions: &'a [AppScreenRegion],
    ) -> ScreenShared<'a, S> {
        ScreenShared {
            screen: screen,
            apps: grant,
            current_process: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            apps_regions,
        }
    }

    // Enqueue a command for the given app.
    fn enqueue_command(&self, command: ScreenCommand, process_id: ProcessId) -> CommandReturn {
        let ret = self
            .apps
            .enter(process_id, |app, _| {
                if app.command.is_some() {
                    Err(ErrorCode::BUSY)
                } else {
                    app.command = Some(command);
                    Ok(())
                }
            })
            .map_err(ErrorCode::from)
            .and_then(|r| r)
            .into();

        if self.current_process.is_none() {
            self.run_next_command();
        }

        ret
    }

    /// Calculate the frame within the entire screen that the app is currently
    /// trying to use. This is the `app_frame` within the app's allocated
    /// `app_screen_region`.
    fn calculate_absolute_frame(&self, app_screen_region_frame: Frame, app_frame: Frame) -> Frame {
        // x and y are sums
        let mut absolute_x = app_screen_region_frame.x + app_frame.x;
        let mut absolute_y = app_screen_region_frame.y + app_frame.y;
        // width and height are simply the app_frame width and height.
        let mut absolute_w = app_frame.width;
        let mut absolute_h = app_frame.height;

        // Make sure that the calculate frame is within the allocated region.
        absolute_x = core::cmp::min(
            app_screen_region_frame.x + app_screen_region_frame.width,
            absolute_x,
        );
        absolute_y = core::cmp::min(
            app_screen_region_frame.y + app_screen_region_frame.height,
            absolute_y,
        );
        absolute_w = core::cmp::min(
            app_screen_region_frame.x + app_screen_region_frame.width - absolute_x,
            absolute_w,
        );
        absolute_h = core::cmp::min(
            app_screen_region_frame.y + app_screen_region_frame.height - absolute_y,
            absolute_h,
        );

        Frame {
            x: absolute_x,
            y: absolute_y,
            width: absolute_w,
            height: absolute_h,
        }
    }

    fn call_screen(
        &self,
        process_id: ProcessId,
        app_screen_region_frame: Frame,
    ) -> Result<(), ErrorCode> {
        self.apps
            .enter(process_id, |app, kernel_data| {
                match app.command {
                    Some(ScreenCommand::WriteSetFrame) => {
                        let absolute_frame =
                            self.calculate_absolute_frame(app_screen_region_frame, app.frame);

                        app.command = Some(ScreenCommand::WriteBuffer);
                        self.screen
                            .set_write_frame(
                                absolute_frame.x,
                                absolute_frame.y,
                                absolute_frame.width,
                                absolute_frame.height,
                            )
                            .map_err(|e| {
                                app.command = None;
                                e
                            })
                    }
                    Some(ScreenCommand::WriteBuffer) => {
                        app.command = None;
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::SHARED)
                            .map(|allow_buf| {
                                let len = allow_buf.len();

                                if len == 0 {
                                    Err(ErrorCode::NOMEM)
                                } else if !self.is_len_multiple_color_depth(len) {
                                    Err(ErrorCode::INVAL)
                                } else {
                                    // All good, copy buffer.

                                    self.buffer.take().map_or(Err(ErrorCode::FAIL), |buffer| {
                                        let copy_len =
                                            core::cmp::min(buffer.len(), allow_buf.len());
                                        allow_buf.enter(|ab| {
                                            // buffer[..copy_len].copy_from_slice(ab[..copy_len]);
                                            ab[..copy_len].copy_to_slice(&mut buffer[..copy_len])
                                        })?;

                                        // Send to screen.
                                        let mut data = SubSliceMut::new(buffer);
                                        data.slice(..copy_len);
                                        self.screen.write(data, false)
                                    })
                                }
                            })
                            .map_err(ErrorCode::from)
                            .and_then(|r| r)
                    }
                    _ => Err(ErrorCode::NOSUPPORT),
                }
            })
            .map_err(ErrorCode::from)
            .and_then(|r| r)
    }

    fn schedule_callback(&self, process_id: ProcessId, data1: usize, data2: usize, data3: usize) {
        let _ = self.apps.enter(process_id, |_app, kernel_data| {
            kernel_data.schedule_upcall(0, (data1, data2, data3)).ok();
        });
    }

    fn get_app_screen_region_frame(&self, process_id: ProcessId) -> Option<Frame> {
        let short_id = process_id.short_app_id();

        for app_screen_region in self.apps_regions {
            if short_id == app_screen_region.app_id {
                return Some(app_screen_region.frame);
            }
        }
        None
    }

    fn run_next_command(&self) {
        let ran_cmd = self.current_process.map_or(false, |process_id| {
            let app_region_frame = self.get_app_screen_region_frame(process_id);

            app_region_frame.map_or(false, |frame| {
                let r = self.call_screen(process_id, frame);
                if r.is_err() {
                    // We were unable to run the screen operation meaning we
                    // will not get a callback and we need to report the error.
                    self.current_process.take().map(|process_id| {
                        self.schedule_callback(
                            process_id,
                            kernel::errorcode::into_statuscode(r),
                            0,
                            0,
                        );
                    });
                    false
                } else {
                    true
                }
            })
        });

        if !ran_cmd {
            // Check if there are any pending events.
            for app in self.apps.iter() {
                let process_id = app.processid();

                // Check if this process has both a pending command and is
                // allocated a region on the screen.
                let frame_maybe = app.enter(|app, _| {
                    if app.command.is_some() {
                        self.get_app_screen_region_frame(process_id)
                    } else {
                        None
                    }
                });

                // If we have a candidate, try to execute the screen operation.
                if frame_maybe.is_some() {
                    match frame_maybe {
                        Some(frame) => {
                            // Reserve the screen for this process and execute
                            // the operation.
                            self.current_process.set(process_id);
                            match self.call_screen(process_id, frame) {
                                Ok(()) => {
                                    // Everything is good, stop looking for apps
                                    // to execute.
                                    break;
                                }
                                Err(err) => {
                                    // Could not run the screen command.
                                    // Un-reserve the screen and do an upcall
                                    // with the bad news.
                                    self.current_process.clear();
                                    self.schedule_callback(
                                        process_id,
                                        kernel::errorcode::into_statuscode(Err(err)),
                                        0,
                                        0,
                                    );
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }

    fn is_len_multiple_color_depth(&self, len: usize) -> bool {
        let depth = pixels_in_bytes(1, self.screen.get_pixel_format().get_bits_per_pixel());
        (len % depth) == 0
    }
}

impl<'a, S: hil::screen::Screen<'a>> hil::screen::ScreenClient for ScreenShared<'a, S> {
    fn command_complete(&self, r: Result<(), ErrorCode>) {
        if r.is_err() {
            self.current_process.take().map(|process_id| {
                self.schedule_callback(process_id, kernel::errorcode::into_statuscode(r), 0, 0);
            });
        }

        self.run_next_command();
    }

    fn write_complete(&self, data: SubSliceMut<'static, u8>, r: Result<(), ErrorCode>) {
        self.buffer.replace(data.take());

        // Notify that the write is finished.
        self.current_process.take().map(|process_id| {
            self.schedule_callback(process_id, kernel::errorcode::into_statuscode(r), 0, 0);
        });

        self.run_next_command();
    }

    fn screen_is_ready(&self) {
        self.run_next_command();
    }
}

impl<'a, S: hil::screen::Screen<'a>> SyscallDriver for ScreenShared<'a, S> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Driver existence check
            0 => CommandReturn::success(),

            // Get Rotation
            21 => CommandReturn::success_u32(self.screen.get_rotation() as u32),

            // Get Resolution
            23 => match self.get_app_screen_region_frame(process_id) {
                Some(frame) => {
                    CommandReturn::success_u32_u32(frame.width as u32, frame.height as u32)
                }
                None => CommandReturn::failure(ErrorCode::NOSUPPORT),
            },

            // Get pixel format
            25 => CommandReturn::success_u32(self.screen.get_pixel_format() as u32),

            // Set Write Frame
            100 => {
                let frame = Frame {
                    x: (data1 >> 16) & 0xFFFF,
                    y: data1 & 0xFFFF,
                    width: (data2 >> 16) & 0xFFFF,
                    height: data2 & 0xFFFF,
                };

                self.apps
                    .enter(process_id, |app, kernel_data| {
                        app.frame = frame;

                        // Just issue upcall.
                        let _ = kernel_data
                            .schedule_upcall(0, (kernel::errorcode::into_statuscode(Ok(())), 0, 0));
                    })
                    .map_err(ErrorCode::from)
                    .into()
            }

            // Write
            200 => {
                // First check if this app has any screen real estate allocated.
                // If not, return error.
                if self.get_app_screen_region_frame(process_id).is_none() {
                    CommandReturn::failure(ErrorCode::NOSUPPORT)
                } else {
                    self.enqueue_command(ScreenCommand::WriteSetFrame, process_id)
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
