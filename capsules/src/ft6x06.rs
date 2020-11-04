//! Driver for the FT6x06 Touch Panel.
//!
//! I2C Interface
//!
//! <http://www.tvielectronics.com/ocart/download/controller/FT6206.pdf>
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f4xx::i2c::I2C1)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let ft6x06 = components::ft6x06::Ft6x06Component::new(
//!     stm32f412g::gpio::PinId::PG05.get_pin().as_ref().unwrap(),
//! )
//! .finalize(components::ft6x06_i2c_component_helper!(mux_i2c));
//!
//! Author: Alexandru Radovici <msg4alex@gmail.com>

#![allow(non_camel_case_types)]

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::i2c::{self, Error};
use kernel::hil::touch::{self, GestureEvent, TouchEvent, TouchStatus};
use kernel::{AppId, Driver, ReturnCode};

use crate::driver;

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Ft6x06 as usize;

pub static NO_TOUCH: TouchEvent = TouchEvent {
    id: 0,
    x: 0,
    y: 0,
    status: TouchStatus::Released,
    size: None,
    pressure: None,
};

enum State {
    Idle,
    ReadingTouches,
}

enum_from_primitive! {
    enum Registers {
        REG_GEST_ID = 0x01,
        REG_TD_STATUS = 0x02,
        REG_CHIPID = 0xA3,
    }
}

pub struct Ft6x06<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    touch_client: OptionalCell<&'a dyn touch::TouchClient>,
    gesture_client: OptionalCell<&'a dyn touch::GestureClient>,
    multi_touch_client: OptionalCell<&'a dyn touch::MultiTouchClient>,
    state: Cell<State>,
    num_touches: Cell<usize>,
    buffer: TakeCell<'static, [u8]>,
    events: TakeCell<'static, [TouchEvent]>,
}

impl<'a> Ft6x06<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
        events: &'static mut [TouchEvent],
    ) -> Ft6x06<'a> {
        // setup and return struct
        interrupt_pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
        Ft6x06 {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            touch_client: OptionalCell::empty(),
            gesture_client: OptionalCell::empty(),
            multi_touch_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            num_touches: Cell::new(0),
            buffer: TakeCell::new(buffer),
            events: TakeCell::new(events),
        }
    }
}

impl<'a> i2c::I2CClient for Ft6x06<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        self.state.set(State::Idle);
        self.num_touches.set((buffer[1] & 0x0F) as usize);
        self.touch_client.map(|client| {
            if self.num_touches.get() <= 2 {
                let status = match buffer[2] >> 6 {
                    0x00 => TouchStatus::Pressed,
                    0x01 => TouchStatus::Released,
                    0x02 => TouchStatus::Moved,
                    _ => TouchStatus::Released,
                };
                let x = (((buffer[2] & 0x0F) as u16) << 8) + (buffer[3] as u16);
                let y = (((buffer[4] & 0x0F) as u16) << 8) + (buffer[5] as u16);
                let pressure = Some(buffer[6] as u16);
                let size = Some(buffer[7] as u16);
                client.touch_event(TouchEvent {
                    status,
                    x,
                    y,
                    id: 0,
                    pressure,
                    size,
                });
            }
        });
        self.gesture_client.map(|client| {
            if self.num_touches.get() <= 2 {
                let gesture_event = match buffer[0] {
                    0x10 => Some(GestureEvent::SwipeUp),
                    0x14 => Some(GestureEvent::SwipeRight),
                    0x18 => Some(GestureEvent::SwipeDown),
                    0x1C => Some(GestureEvent::SwipeLeft),
                    0x48 => Some(GestureEvent::ZoomIn),
                    0x49 => Some(GestureEvent::ZoomOut),
                    _ => None,
                };
                if let Some(gesture) = gesture_event {
                    client.gesture_event(gesture);
                }
            }
        });
        self.multi_touch_client.map(|client| {
            if self.num_touches.get() <= 2 {
                for touch_event in 0..self.num_touches.get() {
                    let status = match buffer[touch_event * 8 + 2] >> 6 {
                        0x00 => TouchStatus::Pressed,
                        0x01 => TouchStatus::Released,
                        _ => TouchStatus::Released,
                    };
                    let x = (((buffer[touch_event * 8 + 2] & 0x0F) as u16) << 8)
                        + (buffer[touch_event * 8 + 3] as u16);
                    let y = (((buffer[touch_event * 8 + 4] & 0x0F) as u16) << 8)
                        + (buffer[touch_event * 8 + 5] as u16);
                    let pressure = Some(buffer[touch_event * 8 + 6] as u16);
                    let size = Some(buffer[touch_event * 8 + 7] as u16);
                    self.events.map(|buffer| {
                        buffer[touch_event] = TouchEvent {
                            status,
                            x,
                            y,
                            id: 0,
                            pressure,
                            size,
                        };
                    });
                }
                self.events.map(|buffer| {
                    client.touch_events(buffer, self.num_touches.get());
                });
            }
        });
        self.buffer.replace(buffer);
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);
    }
}

impl<'a> gpio::Client for Ft6x06<'a> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            self.interrupt_pin.disable_interrupts();

            self.state.set(State::ReadingTouches);

            buffer[0] = Registers::REG_GEST_ID as u8;
            self.i2c.write_read(buffer, 1, 15);
        });
    }
}

impl<'a> touch::Touch<'a> for Ft6x06<'a> {
    fn enable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'a dyn touch::TouchClient) {
        self.touch_client.replace(client);
    }
}

impl<'a> touch::Gesture<'a> for Ft6x06<'a> {
    fn set_client(&self, client: &'a dyn touch::GestureClient) {
        self.gesture_client.replace(client);
    }
}

impl<'a> touch::MultiTouch<'a> for Ft6x06<'a> {
    fn enable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn get_num_touches(&self) -> usize {
        2
    }

    fn get_touch(&self, index: usize) -> Option<TouchEvent> {
        self.buffer.map_or(None, |buffer| {
            if index <= self.num_touches.get() {
                // a touch has 7 bytes
                let offset = index * 7;
                let status = match buffer[offset + 1] >> 6 {
                    0x00 => TouchStatus::Pressed,
                    0x01 => TouchStatus::Released,
                    0x02 => TouchStatus::Moved,
                    _ => TouchStatus::Released,
                };
                let x = (((buffer[offset + 2] & 0x0F) as u16) << 8) + (buffer[offset + 3] as u16);
                let y = (((buffer[offset + 4] & 0x0F) as u16) << 8) + (buffer[offset + 5] as u16);
                let pressure = Some(buffer[offset + 6] as u16);
                let size = Some(buffer[offset + 7] as u16);
                Some(TouchEvent {
                    status,
                    x,
                    y,
                    id: 0,
                    pressure,
                    size,
                })
            } else {
                None
            }
        })
    }

    fn set_client(&self, client: &'a dyn touch::MultiTouchClient) {
        self.multi_touch_client.replace(client);
    }
}

impl Driver for Ft6x06<'_> {
    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // is driver present
            0 => ReturnCode::SUCCESS,

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
