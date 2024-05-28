// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Sensor Driver for the LPS22HB MEMS nano pressure sensor
//! using the I2C bus.
//!
//! <https://www.st.com/resource/en/datasheet/lps22hb.pdf>
//!
//! > The LPS22HB is an ultra-compact piezoresistive absolute
//! > pressure sensor which functions as a digital output barometer.
//! > The device comprises a sensing element and an IC interface
//! > which communicates through I2C or SPI from the sensing element
//! > to the application.
//!
//! Driver Semantics
//! ----------------
//!
//! This driver exposes the LPS22HB's pressure functionality via
//! the [PressureDriver] HIL interface. It doesn't support handling
//! multiple concurrent pressure requests.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let lps22hb_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x5C));
//! let lps22hb = static_init!(
//!     capsules::lps22hb::Lps22hb<'static>,
//!     capsules::lps22hb::Lps22hb::new(lps22hb_i2c,
//!         &mut capsules::lps22hb::BUFFER));
//! lps22hb_i2c.set_client(lps22hb);
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{PressureClient, PressureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Pressure as usize;

/// Register values

const REGISTER_AUTO_INCREMENT: u8 = 0x80;
const CTRL_REG1_ONE_SHOT: u8 = 0x00;

#[allow(dead_code)]
enum Registers {
    IntCfgReg = 0x0B,
    ThsPL = 0x0C,
    ThsPH = 0x0D,
    WhoAmI = 0x0F,
    CtrlReg1 = 0x10,
    CtrlReg2 = 0x11,
    CtrlReg3 = 0x12,
    FifoCtrl = 0x14,
    RefPXl = 0x15,
    RefPL = 0x16,
    RefPH = 0x17,
    RpdsL = 0x18,
    RpdsH = 0x19,
    ResConf = 0x1A,
    IntSourceReg = 0x25,
    FifoStatus = 0x26,
    StatusReg = 0x27,
    PressOutXl = 0x28,
    PressOutL = 0x29,
    PressOutH = 0x2A,
    TempOutL = 0x2B,
    TempOutH = 0x2C,
    LpfpRes = 0x33,
}

pub struct Lps22hb<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c_bus: &'a I,
    pressure_client: OptionalCell<&'a dyn PressureClient>,
    pending_pressure: Cell<bool>,
    state: Cell<State>,
}

impl<'a, I: I2CDevice> Lps22hb<'a, I> {
    pub fn new(i2c_bus: &'a I, buffer: &'static mut [u8]) -> Lps22hb<'a, I> {
        Lps22hb {
            buffer: TakeCell::new(buffer),
            i2c_bus: i2c_bus,
            pressure_client: OptionalCell::empty(),
            pending_pressure: Cell::new(false),
            state: Cell::new(State::Sleep),
        }
    }

    fn start_measurement(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c_bus.enable();
                match self.state.get() {
                    State::Sleep => {
                        buffer[0] = Registers::WhoAmI as u8;

                        if let Err((_error, buffer)) = self.i2c_bus.write_read(buffer, 1, 1) {
                            self.buffer.replace(buffer);
                            self.i2c_bus.disable();
                        } else {
                            self.state.set(State::PowerOn);
                        }
                    }
                    State::Idle => {
                        buffer[0] = Registers::CtrlReg2 as u8;
                        buffer[1] = 0x11_u8;

                        if let Err((_error, buffer)) = self.i2c_bus.write(buffer, 2) {
                            self.buffer.replace(buffer);
                            self.i2c_bus.disable();
                        } else {
                            self.state.set(State::Status);
                        }
                    }
                    _ => {}
                }
            })
            .ok_or(ErrorCode::FAIL)
    }
}

impl<'a, I: I2CDevice> PressureDriver<'a> for Lps22hb<'a, I> {
    fn set_client(&self, client: &'a dyn PressureClient) {
        self.pressure_client.set(client);
    }

    fn read_atmospheric_pressure(&self) -> Result<(), ErrorCode> {
        if !self.pending_pressure.get() {
            self.pending_pressure.set(true);
            self.start_measurement()
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Sleep,
    PowerOn,
    Idle,
    ConfOut,
    Status,
    ReadMeasurementInit,
    ReadMeasurement,
    GotMeasurement,
}

impl<'a, I: I2CDevice> I2CClient for Lps22hb<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.state.set(State::Idle);
            self.buffer.replace(buffer);
            self.pressure_client
                .map(|client| client.callback(Err(i2c_err.into())));
            return;
        }

        match self.state.get() {
            State::PowerOn => {
                if buffer[0] == 0xB1 {
                    buffer[0] = Registers::CtrlReg1 as u8;
                    buffer[1] = CTRL_REG1_ONE_SHOT;

                    if let Err((i2c_err, buffer)) = self.i2c_bus.write(buffer, 2) {
                        self.state.set(State::Idle);
                        self.buffer.replace(buffer);
                        self.pressure_client
                            .map(|client| client.callback(Err(i2c_err.into())));
                    } else {
                        self.state.set(State::ConfOut);
                    }
                } else {
                    self.state.set(State::Sleep);
                }
            }
            State::ConfOut => {
                buffer[0] = Registers::CtrlReg2 as u8;
                buffer[1] = 0x11_u8;

                if let Err((i2c_err, buffer)) = self.i2c_bus.write(buffer, 2) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.pressure_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                } else {
                    self.state.set(State::Status);
                }
            }
            State::Status => {
                buffer[0] = Registers::CtrlReg2 as u8;

                if let Err((i2c_err, buffer)) = self.i2c_bus.write_read(buffer, 1, 1) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.pressure_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                } else {
                    self.state.set(State::ReadMeasurementInit);
                }
            }
            State::ReadMeasurementInit => {
                if buffer[0] == 0x10 {
                    buffer[0] = Registers::PressOutXl as u8 | REGISTER_AUTO_INCREMENT;

                    if let Err((i2c_err, buffer)) = self.i2c_bus.write(buffer, 1) {
                        self.state.set(State::Idle);
                        self.buffer.replace(buffer);
                        self.pressure_client
                            .map(|client| client.callback(Err(i2c_err.into())));
                    } else {
                        self.state.set(State::ReadMeasurement);
                    }
                } else {
                    buffer[0] = Registers::CtrlReg2 as u8;

                    if let Err((i2c_err, buffer)) = self.i2c_bus.write_read(buffer, 1, 1) {
                        self.state.set(State::Idle);
                        self.buffer.replace(buffer);
                        self.pressure_client
                            .map(|client| client.callback(Err(i2c_err.into())));
                    } else {
                        self.state.set(State::ReadMeasurementInit);
                    }
                }
            }
            State::ReadMeasurement => {
                if let Err((i2c_err, buffer)) = self.i2c_bus.read(buffer, 3) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buffer);
                    self.pressure_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                } else {
                    self.state.set(State::GotMeasurement);
                }
            }
            State::GotMeasurement => {
                let pressure =
                    (((buffer[2] as u32) << 16) | ((buffer[1] as u32) << 8) | (buffer[0] as u32))
                        / 4096;

                self.buffer.replace(buffer);
                self.i2c_bus.disable();
                if self.pending_pressure.get() {
                    self.pending_pressure.set(false);
                    self.pressure_client
                        .map(|client| client.callback(Ok(pressure)));
                }

                self.state.set(State::Idle);
            }
            State::Sleep => {}
            State::Idle => {}
        }
    }
}
