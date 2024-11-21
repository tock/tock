// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chirp I2C Soil moisture sensor using the I2C bus.
//!
//! <https://www.tindie.com/products/2330/>
//! <https://github.com/Miceuz/i2c-moisture-sensor/blob/master/README.md>
//!

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{MoistureClient, MoistureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub const BUFFER_SIZE: usize = 2;

const GET_CAPACITANCE: u8 = 0x00;
#[allow(dead_code)]
const SET_ADDRESS: u8 = 0x01;
#[allow(dead_code)]
const GET_ADDRESS: u8 = 0x02;
#[allow(dead_code)]
const MEASURE_LIGHT: u8 = 0x03;
#[allow(dead_code)]
const GET_LIGHT: u8 = 0x04;
#[allow(dead_code)]
const GET_TEMPERATURE: u8 = 0x05;
#[allow(dead_code)]
const RESET: u8 = 0x06;
const GET_VERSION: u8 = 0x07;
#[allow(dead_code)]
const SLEEP: u8 = 0x08;
#[allow(dead_code)]
const GET_BUSY: u8 = 0x09;

#[derive(Clone, Copy, PartialEq)]
enum DeviceState {
    Identify,
    Normal,
    StartMoisture,
    FinalMoisture,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Moisture,
}

pub struct ChirpI2cMoisture<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    moisture_client: OptionalCell<&'a dyn MoistureClient>,
    state: Cell<DeviceState>,
    op: Cell<Operation>,
}

impl<'a, I: I2CDevice> ChirpI2cMoisture<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8]) -> Self {
        ChirpI2cMoisture {
            buffer: TakeCell::new(buffer),
            i2c,
            moisture_client: OptionalCell::empty(),
            state: Cell::new(DeviceState::Identify),
            op: Cell::new(Operation::None),
        }
    }

    pub fn initialise(&self) {
        self.buffer.take().map(|buffer| {
            if self.state.get() == DeviceState::Identify {
                // Read the version register
                buffer[0] = GET_VERSION;
                if let Err((_e, buf)) = self.i2c.write_read(buffer, 1, 1) {
                    self.buffer.replace(buf);
                }
            } else {
                self.buffer.replace(buffer);
            }
        });
    }
}

impl<'a, I: I2CDevice> MoistureDriver<'a> for ChirpI2cMoisture<'a, I> {
    fn set_client(&self, client: &'a dyn MoistureClient) {
        self.moisture_client.set(client);
    }

    fn read_moisture(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            buffer[0] = GET_CAPACITANCE;

            self.op.set(Operation::Moisture);
            self.state.set(DeviceState::StartMoisture);
            if let Err((e, buf)) = self.i2c.write_read(buffer, 1, 2) {
                self.buffer.replace(buf);
                return Err(e.into());
            }

            Ok(())
        })
    }
}

impl<I: I2CDevice> I2CClient for ChirpI2cMoisture<'_, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.buffer.replace(buffer);

            match self.op.get() {
                Operation::None => (),
                Operation::Moisture => {
                    self.op.set(Operation::None);

                    self.moisture_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                }
            }

            return;
        }

        match self.state.get() {
            DeviceState::Identify => {
                if buffer[0] < 0x22 {
                    // We don't have the correct version, this isn't the correct device
                    // Just stop here
                    self.buffer.replace(buffer);
                    return;
                }

                self.buffer.replace(buffer);
                self.state.set(DeviceState::Normal);
                self.op.set(Operation::None);
            }
            DeviceState::StartMoisture => match self.op.get() {
                Operation::None => (),
                Operation::Moisture => {
                    self.state.set(DeviceState::FinalMoisture);
                    buffer[0] = GET_CAPACITANCE;
                    if let Err((e, buf)) = self.i2c.write_read(buffer, 1, 2) {
                        self.buffer.replace(buf);
                        self.op.set(Operation::None);

                        self.moisture_client
                            .map(|client| client.callback(Err(e.into())));
                    }
                }
            },
            DeviceState::FinalMoisture => {
                match self.op.get() {
                    Operation::None => (),
                    Operation::Moisture => {
                        let capacitance = (((buffer[0] as u32) << 8) | (buffer[1] as u32)) as f32;

                        // 247 is the capacitance in air
                        // 510 is the capacitance in water
                        // Use those to calculate the moisture percentage, which is rougly linear
                        // https://github.com/Miceuz/i2c-moisture-sensor/blob/master/README.md#how-to-interpret-the-readings
                        // Note that this gives moisture in hundredths of a percent
                        let moisture_content = ((capacitance - 247.0) / (510.0 - 247.0)) * 10000.0;

                        self.state.set(DeviceState::Normal);
                        self.buffer.replace(buffer);
                        self.op.set(Operation::None);

                        self.moisture_client
                            .map(|client| client.callback(Ok(moisture_content as usize)));
                    }
                }
            }
            DeviceState::Normal => {}
        }
    }
}
