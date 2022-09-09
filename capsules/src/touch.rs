//! Provides userspace with access to the touch panel.
//!
//! Usage
//! -----
//!
//! You need a touch that provides the `hil::touch::Touch` trait.
//! An optional gesture client and a screen can be connected to it.
//!
//! ```rust
//! let touch =
//!     components::touch::TouchComponent::new(board_kernel, ts, Some(ts), Some(screen)).finalize(());
//! ```

use core::cell::Cell;
use core::mem;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::hil::screen::ScreenRotation;
use kernel::hil::touch::{GestureEvent, TouchClient, TouchEvent, TouchStatus};
use kernel::processbuffer::WriteableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Touch as usize;

/// Ids for read-write allow buffers
mod rw_allow {
    /// Allow a buffer for the multi touch. See header for format
    // Buffer data format
    //  0         1           2              4              6           7             8          ...
    // +---------+-----------+--------------+--------------+-----------+---------------+-------- ...
    // | id (u8) | type (u8) | x (u16)      | y (u16)      | size (u8) | pressure (u8) |         ...
    // +---------+-----------+--------------+--------------+-----------+---------------+-------- ...
    // | Touch 0                                                                       | Touch 1 ...
    pub const EVENTS: usize = 2;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 3;
}

fn touch_status_to_number(status: &TouchStatus) -> usize {
    match status {
        TouchStatus::Released => 0,
        TouchStatus::Pressed => 1,
        TouchStatus::Moved => 2,
        TouchStatus::Unstarted => 3,
    }
}

pub struct App {
    ack: bool,
    dropped_events: usize,
    x: u16,
    y: u16,
    status: usize,
    touch_enable: bool,
    multi_touch_enable: bool,
}

impl Default for App {
    fn default() -> App {
        App {
            ack: true,
            dropped_events: 0,
            x: 0,
            y: 0,
            status: touch_status_to_number(&TouchStatus::Unstarted),
            touch_enable: false,
            multi_touch_enable: false,
        }
    }
}

pub struct Touch<'a> {
    touch: Option<&'a dyn hil::touch::Touch<'a>>,
    multi_touch: Option<&'a dyn hil::touch::MultiTouch<'a>>,
    /// Screen under the touch panel
    /// Most of the touch panels have a screen that can be rotated
    /// 90 deg (clockwise), 180 deg (upside-down), 270 deg(clockwise).
    /// The touch gets the rotation from the screen and
    /// updates the touch (x, y) position
    screen: Option<&'a dyn hil::screen::Screen>,
    apps: Grant<App, UpcallCount<3>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    screen_rotation_offset: Cell<ScreenRotation>,
}

impl<'a> Touch<'a> {
    pub fn new(
        touch: Option<&'a dyn hil::touch::Touch<'a>>,
        multi_touch: Option<&'a dyn hil::touch::MultiTouch<'a>>,
        screen: Option<&'a dyn hil::screen::Screen>,
        grant: Grant<App, UpcallCount<3>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    ) -> Touch<'a> {
        Touch {
            touch: touch,
            multi_touch: multi_touch,
            screen: screen,
            screen_rotation_offset: Cell::new(ScreenRotation::Normal),
            apps: grant,
        }
    }

    pub fn set_screen_rotation_offset(&self, screen_rotation_offset: ScreenRotation) {
        self.screen_rotation_offset.set(screen_rotation_offset);
    }

    fn touch_enable(&self) -> Result<(), ErrorCode> {
        let mut enabled = false;
        for app in self.apps.iter() {
            if app.enter(|app, _| if app.touch_enable { true } else { false }) {
                enabled = true;
                break;
            }
        }
        self.touch.map_or_else(
            || {
                // if there is no single touch device
                // try to use the multi touch device and
                // report only a single touch
                self.multi_touch
                    .map_or(Err(ErrorCode::NODEVICE), |multi_touch| {
                        if enabled {
                            multi_touch.enable()
                        } else {
                            multi_touch.disable()
                        }
                    })
            },
            |touch| {
                if enabled {
                    touch.enable()
                } else {
                    touch.disable()
                }
            },
        )
    }

    fn multi_touch_enable(&self) -> Result<(), ErrorCode> {
        let mut enabled = false;
        for app in self.apps.iter() {
            if app.enter(|app, _| if app.multi_touch_enable { true } else { false }) {
                enabled = true;
                break;
            }
        }
        self.multi_touch
            .map_or(Err(ErrorCode::NODEVICE), |multi_touch| {
                if enabled {
                    multi_touch.enable()
                } else {
                    multi_touch.disable()
                }
            })
    }

    /// Updates the (x, y) pf the touch event based on the
    /// screen rotation (if there si a screen)
    fn update_rotation(&self, touch_event: &mut TouchEvent) {
        if let Some(screen) = self.screen {
            let rotation = screen.get_rotation() + self.screen_rotation_offset.get();
            let (mut width, mut height) = screen.get_resolution();

            let (x, y) = match rotation {
                ScreenRotation::Rotated90 => {
                    mem::swap(&mut width, &mut height);
                    (touch_event.y, height as u16 - touch_event.x)
                }
                ScreenRotation::Rotated180 => {
                    (width as u16 - touch_event.x, height as u16 - touch_event.y)
                }
                ScreenRotation::Rotated270 => {
                    mem::swap(&mut width, &mut height);
                    (width as u16 - touch_event.y as u16, touch_event.x)
                }
                _ => (touch_event.x, touch_event.y),
            };

            touch_event.x = x;
            touch_event.y = y;
        }
    }
}

impl<'a> hil::touch::TouchClient for Touch<'a> {
    fn touch_event(&self, mut event: TouchEvent) {
        // update rotation if there is a screen attached
        self.update_rotation(&mut event);
        // debug!(
        //     "touch {:?} x {} y {} size {:?} pressure {:?}",
        //     event.status, event.x, event.y, event.size, event.pressure
        // );
        for app in self.apps.iter() {
            app.enter(|app, kernel_data| {
                let event_status = touch_status_to_number(&event.status);
                if app.x != event.x || app.y != event.y || app.status != event_status {
                    app.x = event.x;
                    app.y = event.y;
                    app.status = event_status;

                    let pressure_size = match event.pressure {
                        Some(pressure) => (pressure as usize) << 16,
                        None => 0,
                    } | match event.size {
                        Some(size) => size as usize,
                        None => 0,
                    };
                    kernel_data
                        .schedule_upcall(
                            0,
                            (
                                event_status,
                                (event.x as usize) << 16 | event.y as usize,
                                pressure_size,
                            ),
                        )
                        .ok();
                }
            });
        }
    }
}

impl<'a> hil::touch::MultiTouchClient for Touch<'a> {
    fn touch_events(&self, touch_events: &[TouchEvent], num_events: usize) {
        let len = if touch_events.len() < num_events {
            touch_events.len()
        } else {
            num_events
        };
        if len > 0 {
            // report the first touch as single touch
            // for applications that only use single touches
            self.touch_event(touch_events[0]);
            // debug!("{} touch(es)", len);
            for app in self.apps.iter() {
                app.enter(|app, kernel_data| {
                    if app.ack {
                        app.dropped_events = 0;

                        let num = kernel_data
                            .get_readwrite_processbuffer(rw_allow::EVENTS)
                            .and_then(|events| {
                                events.mut_enter(|buffer| {
                                    let num = if buffer.len() / 8 < len {
                                        buffer.len() / 8
                                    } else {
                                        len
                                    };

                                    for event_index in 0..num {
                                        let mut event = touch_events[event_index].clone();
                                        self.update_rotation(&mut event);
                                        let event_status = touch_status_to_number(&event.status);
                                        // debug!(
                                        //     " multitouch {:?} x {} y {} size {:?} pressure {:?}",
                                        //     event.status, event.x, event.y, event.size, event.pressure
                                        // );
                                        // one touch entry is 8 bytes long
                                        let offset = event_index * 8;
                                        if buffer.len() > event_index + 8 {
                                            buffer[offset].set(event.id as u8);
                                            buffer[offset + 1].set(event_status as u8);
                                            buffer[offset + 2].set((event.x & 0xFF) as u8);
                                            buffer[offset + 3].set(((event.x & 0xFFFF) >> 8) as u8);
                                            buffer[offset + 4].set((event.y & 0xFF) as u8);
                                            buffer[offset + 5].set(((event.y & 0xFFFF) >> 8) as u8);
                                            buffer[offset + 6].set(
                                                if let Some(size) = event.size {
                                                    size as u8
                                                } else {
                                                    0
                                                },
                                            );
                                            buffer[offset + 7].set(
                                                if let Some(pressure) = event.pressure {
                                                    pressure as u8
                                                } else {
                                                    0
                                                },
                                            );
                                        } else {
                                            break;
                                        }
                                    }
                                    num
                                })
                            })
                            .unwrap_or(0);
                        let dropped_events = app.dropped_events;
                        if num > 0 {
                            app.ack = false;
                            kernel_data
                                .schedule_upcall(
                                    2,
                                    (num, dropped_events, if num < len { len - num } else { 0 }),
                                )
                                .ok();
                        }
                    } else {
                        app.dropped_events = app.dropped_events + 1;
                    }
                });
            }
        }
    }
}

impl<'a> hil::touch::GestureClient for Touch<'a> {
    fn gesture_event(&self, event: GestureEvent) {
        for app in self.apps.iter() {
            app.enter(|_app, kernel_data| {
                let gesture_id = match event {
                    GestureEvent::SwipeUp => 1,
                    GestureEvent::SwipeDown => 2,
                    GestureEvent::SwipeLeft => 3,
                    GestureEvent::SwipeRight => 4,
                    GestureEvent::ZoomIn => 5,
                    GestureEvent::ZoomOut => 6,
                };
                kernel_data.schedule_upcall(1, (gesture_id, 0, 0)).ok();
            });
        }
    }
}

impl<'a> SyscallDriver for Touch<'a> {
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 =>
            // This driver exists.
            {
                CommandReturn::success()
            }

            // touch enable
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.touch_enable = true;
                    })
                    .unwrap_or(());
                let _ = self.touch_enable();
                CommandReturn::success()
            }

            // touch disable
            2 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.touch_enable = false;
                    })
                    .unwrap_or(());
                let _ = self.touch_enable();
                CommandReturn::success()
            }

            // multi touch ack
            10 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.ack = true;
                    })
                    .unwrap_or(());
                CommandReturn::success()
            }

            // multi touch enable
            11 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.multi_touch_enable = true;
                    })
                    .unwrap_or(());
                let _ = self.multi_touch_enable();
                CommandReturn::success()
            }

            // multi touch disable
            12 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.multi_touch_enable = false;
                    })
                    .unwrap_or(());
                let _ = self.multi_touch_enable();
                CommandReturn::success()
            }

            // number of touches
            100 => {
                let num_touches = if let Some(multi_touch) = self.multi_touch {
                    multi_touch.get_num_touches()
                } else {
                    if self.touch.is_some() {
                        1
                    } else {
                        0
                    }
                };
                CommandReturn::success_u32(num_touches as u32)
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
