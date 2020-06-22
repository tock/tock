//! Driver for the FT6202 Touch Panel.
//!
//! I2C Interface
//!
//! <http://www.tvielectronics.com/ocart/download/controller/FT6206.pdf>
//!
//! The syscall interface is described in [lsm303dlhc.md](https://github.com/tock/tock/tree/master/doc/syscalls/70006_lsm303dlhc.md)
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f4xx::i2c::I2C1)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let ft6206 = components::ft6206::Ft6206Component::new(
//!     stm32f412g::gpio::PinId::PG05.get_pin().as_ref().unwrap(),
//! )
//! .finalize(components::ft6206_i2c_component_helper!(mux_i2c));
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
pub const DRIVER_NUM: usize = driver::NUM::Ft6206 as usize;

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

pub struct Ft6206<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin,
    // callback: OptionalCell<Callback>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> Ft6206<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin,
        buffer: &'static mut [u8],
    ) -> Ft6206<'a> {
        // setup and return struct
        interrupt_pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
        Ft6206 {
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

impl i2c::I2CClient for Ft6206<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        self.state.set(State::Idle);
        self.buffer.replace(buffer);
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);
    }
}

impl gpio::Client for Ft6206<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            self.interrupt_pin.disable_interrupts();

            self.state.set(State::ReadingTouches);

            buffer[0] = 0;
            self.i2c.write_read(buffer, 1, 16);
        });
    }
}

impl Driver for Ft6206<'_> {
    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // is driver present
            0 => ReturnCode::SUCCESS,

            // on
            1 => {
                self.is_present();
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
