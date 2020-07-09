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

// use core::convert::From;
use core::mem;
use kernel::debug;
use kernel::hil;
use kernel::hil::screen::ScreenRotation;
use kernel::hil::touch::{GestureEvent, TouchEvent, TouchStatus};
use kernel::ReturnCode;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Touch as usize;

pub struct App {
    touch_callback: Option<Callback>,
    gesture_callback: Option<Callback>,
    multi_touch_callback: Option<Callback>,
    events_buffer: Option<AppSlice<Shared, u8>>,
    ack: bool,
    dropped_events: usize,
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
            apps: grant,
        }
    }

    /// Updates the (x, y) pf the touch event based on the
    /// screen rotation (if there si a screen)
    fn update_rotation(&self, touch_event: &mut TouchEvent) {
        if let Some(screen) = self.screen {
            let rotation = screen.get_rotation();
            let (mut width, mut height) = screen.get_resolution();

            let (x, y) = match rotation {
                ScreenRotation::Rotated270 => {
                    mem::swap(&mut width, &mut height);
                    (touch_event.y, height - touch_event.x)
                }
                ScreenRotation::Rotated180 => (width - touch_event.x, height - touch_event.y),
                ScreenRotation::Rotated90 => {
                    mem::swap(&mut width, &mut height);
                    (width - touch_event.y, touch_event.x)
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
        debug!(
            "touch {:?} x {} y {} area {:?} weight {:?}",
            event.status, event.x, event.y, event.area, event.weight
        );
        for app in self.apps.iter() {
            app.enter(|app, _| {
                app.touch_callback.map(|mut callback| {
                    let event_id = match event.status {
                        TouchStatus::Released => 0,
                        TouchStatus::Pressed => 1,
                    };
                    callback.schedule(event.x, event.y, event_id);
                })
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
        debug!("{} touch(es)", len);
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
                                let event_status = match event.status {
                                    TouchStatus::Released => 0,
                                    TouchStatus::Pressed => 1,
                                };
                                debug!(
                                    " multitouch {:?} x {} y {} area {:?} weight {:?}",
                                    event.status, event.x, event.y, event.area, event.weight
                                );
                                // one touch entry is 8 bytes long
                                let offset = event_index * 8;
                                if buffer.len() > event_index + 8 {
                                    buffer.as_mut()[offset] = event.id as u8;
                                    buffer.as_mut()[offset + 1] = event_status as u8;
                                    buffer.as_mut()[offset + 2] = ((event.x & 0xFFFF) >> 8) as u8;
                                    buffer.as_mut()[offset + 3] = (event.x & 0xFF) as u8;
                                    buffer.as_mut()[offset + 4] = ((event.y & 0xFFFF) >> 8) as u8;
                                    buffer.as_mut()[offset + 5] = (event.y & 0xFF) as u8;
                                    buffer.as_mut()[offset + 6] = if let Some(area) = event.area {
                                        area as u8
                                    } else {
                                        0
                                    };
                                    buffer.as_mut()[offset + 7] = if let Some(weight) = event.weight
                                    {
                                        weight as u8
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
        debug!("gesture {:?}", event);
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

impl<'a> Driver for Touch<'a> {
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // allow a buffer for the multi touch
            // buffer data format
            //  0         1           2                  4                  6           7             8         ...
            // +---------+-----------+------------------+------------------+-----------+-------------+--------- ...
            // | id (u8) | type (u8) | x (u16)          | y (u16)          | area (u8) | weight (u8) |          ...
            // +---------+-----------+------------------+------------------+-----------+-------------+--------- ...
            // | Touch 0                                                                             | Touch 1  ...
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
                    ReturnCode::SUCCESS
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

            // Touch Enable
            1 => {
                if let Some(touch) = self.touch {
                    touch.enable()
                } else if let Some(multi_touch) = self.multi_touch {
                    multi_touch.enable()
                } else {
                    ReturnCode::ENODEVICE
                }
            }
            // Touch Disable
            2 => {
                if let Some(touch) = self.touch {
                    touch.disable()
                } else if let Some(multi_touch) = self.multi_touch {
                    multi_touch.disable()
                } else {
                    ReturnCode::ENODEVICE
                }
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
