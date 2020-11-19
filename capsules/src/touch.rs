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
use kernel::hil;
use kernel::hil::screen::ScreenRotation;
use kernel::hil::touch::{GestureEvent, TouchEvent, TouchStatus};
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Grant, LegacyDriver, SharedReadWrite};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Touch as usize;

fn touch_status_to_number(status: &TouchStatus) -> usize {
    match status {
        TouchStatus::Released => 0,
        TouchStatus::Pressed => 1,
        TouchStatus::Moved => 2,
        TouchStatus::Unstarted => 3,
    }
}

pub struct App {
    touch_callback: Option<Callback>,
    gesture_callback: Option<Callback>,
    multi_touch_callback: Option<Callback>,
    events_buffer: Option<AppSlice<SharedReadWrite, u8>>,
    ack: bool,
    dropped_events: usize,
    x: u16,
    y: u16,
    status: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            touch_callback: None,
            gesture_callback: None,
            multi_touch_callback: None,
            events_buffer: None,
            ack: true,
            dropped_events: 0,
            x: 0,
            y: 0,
            status: touch_status_to_number(&TouchStatus::Unstarted),
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
    apps: Grant<App>,
    screen_rotation_offset: Cell<ScreenRotation>,
}

impl<'a> Touch<'a> {
    pub fn new(
        touch: Option<&'a dyn hil::touch::Touch<'a>>,
        multi_touch: Option<&'a dyn hil::touch::MultiTouch<'a>>,
        screen: Option<&'a dyn hil::screen::Screen>,
        grant: Grant<App>,
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

    fn touch_enable(&self) -> ReturnCode {
        let mut enabled = false;
        for app in self.apps.iter() {
            if app.enter(|app, _| {
                if app.touch_callback.is_some() {
                    true
                } else {
                    false
                }
            }) {
                enabled = true;
                break;
            }
        }
        self.touch.map_or(ReturnCode::ENODEVICE, |touch| {
            if enabled {
                touch.enable()
            } else {
                touch.disable()
            }
        })
    }

    fn multi_touch_enable(&self) -> ReturnCode {
        let mut enabled = false;
        for app in self.apps.iter() {
            if app.enter(|app, _| {
                if app.multi_touch_callback.is_some() {
                    true
                } else {
                    false
                }
            }) {
                enabled = true;
                break;
            }
        }
        self.multi_touch
            .map_or(ReturnCode::ENODEVICE, |multi_touch| {
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
            app.enter(|app, _| {
                let event_status = touch_status_to_number(&event.status);
                if app.x != event.x || app.y != event.y || app.status != event_status {
                    app.x = event.x;
                    app.y = event.y;
                    app.status = event_status;
                    app.touch_callback.map(|mut callback| {
                        let pressure_size = match event.pressure {
                            Some(pressure) => (pressure as usize) << 16,
                            None => 0,
                        } | match event.size {
                            Some(size) => size as usize,
                            None => 0,
                        };
                        callback.schedule(
                            event_status,
                            (event.x as usize) << 16 | event.y as usize,
                            pressure_size,
                        );
                    });
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
        // debug!("{} touch(es)", len);
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if app.ack {
                    app.dropped_events = 0;
                    app.multi_touch_callback.map(|mut callback| {
                        if let Some(ref mut buffer) = app.events_buffer {
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
                                    buffer.as_mut()[offset] = event.id as u8;
                                    buffer.as_mut()[offset + 1] = event_status as u8;
                                    buffer.as_mut()[offset + 2] = ((event.x & 0xFFFF) >> 8) as u8;
                                    buffer.as_mut()[offset + 3] = (event.x & 0xFF) as u8;
                                    buffer.as_mut()[offset + 4] = ((event.y & 0xFFFF) >> 8) as u8;
                                    buffer.as_mut()[offset + 5] = (event.y & 0xFF) as u8;
                                    buffer.as_mut()[offset + 6] = if let Some(size) = event.size {
                                        size as u8
                                    } else {
                                        0
                                    };
                                    buffer.as_mut()[offset + 7] =
                                        if let Some(pressure) = event.pressure {
                                            pressure as u8
                                        } else {
                                            0
                                        };
                                } else {
                                    break;
                                }
                            }
                            callback.schedule(
                                num,
                                app.dropped_events,
                                if num < len { len - num } else { 0 },
                            );
                        }
                    });
                // app.ack = false;
                } else {
                    app.dropped_events = app.dropped_events + 1;
                }
            });
        }
    }
}

impl<'a> hil::touch::GestureClient for Touch<'a> {
    fn gesture_event(&self, event: GestureEvent) {
        // debug!("gesture {:?}", event);
        for app in self.apps.iter() {
            app.enter(|app, _| {
                app.gesture_callback.map(|mut callback| {
                    let gesture_id = match event {
                        GestureEvent::SwipeUp => 1,
                        GestureEvent::SwipeDown => 2,
                        GestureEvent::SwipeLeft => 3,
                        GestureEvent::SwipeRight => 4,
                        GestureEvent::ZoomIn => 5,
                        GestureEvent::ZoomOut => 6,
                    };
                    callback.schedule(gesture_id, 0, 0);
                })
            });
        }
    }
}

impl<'a> LegacyDriver for Touch<'a> {
    fn allow_readwrite(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<SharedReadWrite, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // allow a buffer for the multi touch
            // buffer data format
            //  0         1           2                  4                  6           7             8         ...
            // +---------+-----------+------------------+------------------+-----------+---------------+--------- ...
            // | id (u8) | type (u8) | x (u16)          | y (u16)          | size (u8) | pressure (u8) |          ...
            // +---------+-----------+------------------+------------------+-----------+---------------+--------- ...
            // | Touch 0                                                                               | Touch 1  ...
            2 => {
                if self.multi_touch.is_some() {
                    self.apps
                        .enter(appid, |app, _| {
                            app.events_buffer = slice;
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| err.into())
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to touch
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    app.touch_callback = callback;
                    self.touch_enable()
                })
                .unwrap_or_else(|err| err.into()),

            // subscribe to gestures
            1 => self
                .apps
                .enter(app_id, |app, _| {
                    app.gesture_callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),

            // subscribe to multi touch
            2 => {
                if self.multi_touch.is_some() {
                    self.apps
                        .enter(app_id, |app, _| {
                            app.multi_touch_callback = callback;
                            self.multi_touch_enable()
                        })
                        .unwrap_or_else(|err| err.into())
                } else {
                    ReturnCode::ENOSUPPORT
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        _appid: AppId,
    ) -> ReturnCode {
        match command_num {
            0 =>
            // This driver exists.
            {
                ReturnCode::SUCCESS
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
                ReturnCode::SuccessWithValue { value: num_touches }
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
