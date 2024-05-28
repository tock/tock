// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Sensor Driver for the Renesas HS3003 Temperature/Humidity sensor
//! using the I2C bus.
//!
//! <https://www.renesas.com/us/en/document/dst/hs300x-datasheet>
//!
//! > The HS300x (HS3001 and HS3003) series is a highly accurate,
//! > fully calibrated relative humidity and temperature sensor.
//! > The MEMS sensor features a proprietary sensor-level protection,
//! > ensuring high reliability and long-term stability. The high
//! > accuracy, fast measurement response time, and long-term stability
//! > combined with the small package size makes the HS300x series ideal
//! > for a wide number of applications ranging from portable devices to
//! > products designed for harsh environments.
//!
//! Driver Semantics
//! ----------------
//!
//! This driver exposes the HS3003's temperature and humidity functionality via
//! the [TemperatureDriver] and [HumidityDriver] HIL interfaces. If the driver
//! receives a request for either temperature or humidity while a request for the
//! other is outstanding, both will be returned to their respective clients when
//! the I2C transaction is completed, rather than performing two separate transactions.
//!
//! Limitations
//! -----------
//!
//! The driver uses floating point math to adjust the readings. This must be
//! done and macthes the chip's datasheet. This could decrease performance
//! on platforms that don't have hardware support for floating point math.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let hs3003_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x44));
//! let hs3003 = static_init!(
//!     capsules::hs3003::Hs3003<'static>,
//!     capsules::hs3003::Hs3003::new(hs3003_i2c,
//!         &mut capsules::hs3003::BUFFER));
//! hs3003_i2c.set_client(hs3003);
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{HumidityClient, HumidityDriver, TemperatureClient, TemperatureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub struct Hs3003<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
    humidity_client: OptionalCell<&'a dyn HumidityClient>,
    state: Cell<State>,
    pending_temperature: Cell<bool>,
    pending_humidity: Cell<bool>,
}

impl<'a, I: I2CDevice> Hs3003<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8]) -> Self {
        Hs3003 {
            buffer: TakeCell::new(buffer),
            i2c,
            temperature_client: OptionalCell::empty(),
            humidity_client: OptionalCell::empty(),
            state: Cell::new(State::Sleep),
            pending_temperature: Cell::new(false),
            pending_humidity: Cell::new(false),
        }
    }

    pub fn start_reading(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c.enable();
                match self.state.get() {
                    State::Sleep => {
                        if let Err((_error, buffer)) = self.i2c.write(buffer, 1) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::InitiateReading);
                        }
                    }
                    _ => {}
                }
            })
            .ok_or(ErrorCode::BUSY)
    }
}

impl<'a, I: I2CDevice> TemperatureDriver<'a> for Hs3003<'a, I> {
    fn set_client(&self, client: &'a dyn TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.pending_temperature.set(true);
        if !self.pending_humidity.get() {
            self.start_reading()
        } else {
            Ok(())
        }
    }
}

impl<'a, I: I2CDevice> HumidityDriver<'a> for Hs3003<'a, I> {
    fn set_client(&self, client: &'a dyn HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        self.pending_humidity.set(true);
        if !self.pending_temperature.get() {
            self.start_reading()
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Sleep,
    InitiateReading,
    Read,
}

impl<'a, I: I2CDevice> I2CClient for Hs3003<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.state.set(State::Sleep);
            self.buffer.replace(buffer);
            self.temperature_client
                .map(|client| client.callback(Err(i2c_err.into())));
            self.humidity_client.map(|client| client.callback(0));
            return;
        }

        match self.state.get() {
            State::InitiateReading => {
                if let Err((i2c_err, buffer)) = self.i2c.read(buffer, 4) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.temperature_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                    self.humidity_client.map(|client| client.callback(0));
                } else {
                    self.state.set(State::Read);
                }
            }
            State::Read => {
                let humidity_raw = (((buffer[0] & 0x3F) as u16) << 8) | buffer[1] as u16;
                let humidity = ((humidity_raw as f32 / ((1 << 14) - 1) as f32) * 100.0) as usize;

                let temperature_raw = ((buffer[2] as u16) << 8) | (buffer[3] as u16 >> 2);
                // This operation follows the datasheet specification except dividing by 10. If its not done,
                // the returned value will be in the hundreds (220 instead of 22 degrees celsius).
                let temperature = ((((temperature_raw as f32 / ((1 << 14) - 1) as f32) * 165.0)
                    - 40.0)
                    / 10.0) as i32;

                self.buffer.replace(buffer);
                self.i2c.disable();
                if self.pending_temperature.get() {
                    self.pending_temperature.set(false);
                    self.temperature_client
                        .map(|client| client.callback(Ok(temperature)));
                }
                if self.pending_humidity.get() {
                    self.pending_humidity.set(false);
                    self.humidity_client.map(|client| client.callback(humidity));
                }

                self.state.set(State::Sleep);
            }
            State::Sleep => {} // should never happen
        }
    }
}
