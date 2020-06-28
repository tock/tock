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
use kernel::common::cells::TakeCell;
use kernel::hil::gpio;
use kernel::hil::i2c::{self, Error};
use kernel::{AppId, Driver, ReturnCode};

use crate::driver;

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Ft6x06 as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 17] = [0; 17];

enum State {
    Idle,
    ReadingTouches,
}

enum_from_primitive! {
    enum Registers {
        REG_NUMTOUCHES = 0x2,
        REG_CHIPID = 0xA3,
    }
}

pub struct Ft6x06<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    // callback: OptionalCell<Callback>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Ft6x06<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
    ) -> Ft6x06<'a> {
        // setup and return struct
        interrupt_pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
        Ft6x06 {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            // callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn is_present(&self) {
        self.state.set(State::Idle);
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            buf[0] = Registers::REG_CHIPID as u8;
            self.i2c.write_read(buf, 1, 1);
        });
    }
}

impl i2c::I2CClient for Ft6x06<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        self.state.set(State::Idle);
        self.buffer.replace(buffer);
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);
    }
}

impl gpio::Client for Ft6x06<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            self.interrupt_pin.disable_interrupts();

            self.state.set(State::ReadingTouches);

            buffer[0] = 0;
            self.i2c.write_read(buffer, 1, 16);
        });
    }
}

impl touch::Touch for Ft6x06<'_> {
    fn enable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static dyn touch::TouchClient) {
        self.touch_client.replace(client);
    }
}

impl touch::Gesture for Ft6x06<'_> {
    fn set_client(&self, client: &'static dyn touch::GestureClient) {
        self.gesture_client.replace(client);
    }
}

impl touch::MultiTouch for Ft6x06<'_> {
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
                    _ => TouchStatus::Released,
                };
                let x =
                    (((buffer[offset + 2] & 0x0F) as usize) << 8) + (buffer[offset + 3] as usize);
                let y =
                    (((buffer[offset + 4] & 0x0F) as usize) << 8) + (buffer[offset + 5] as usize);
                let weight = Some(buffer[offset + 6] as usize);
                let area = Some(buffer[offset + 7] as usize);
                Some(TouchEvent {
                    status,
                    x,
                    y,
                    id: 0,
                    weight,
                    area,
                })
            } else {
                None
            }
        })
    }

    fn set_client(&self, client: &'static dyn touch::MultiTouchClient) {
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
