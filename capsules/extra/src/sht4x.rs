// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Driver for SHT4x Temperature and Humidity Sensor

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::hil::i2c;
use kernel::hil::time::{self, Alarm, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub static BASE_ADDR: u8 = 0x44;

enum_from_primitive! {
    enum Registers {
        /// sht4x has no Clock Stretching
        /// Measurement High Repeatability
        MEASHIGHREP = 0xFD,
        /// Measurement Medium Repeatability
        MEASMEDREP = 0xF6,
        /// Measurement Low Repeatability
        MEASLOWREP = 0xE0,
        /// Read Serial Number
        READSERIALNUM = 0x89,
        /// Soft Reset
        SOFTRESET = 0x94,
        /// Activate heater with 200mW for 1s
        HEATER200MW1S = 0x39,
        /// Activate heater with 200mW for 0.1s
        HEATER200MW01S = 0x32,
        /// Activate heater with 110mW for 1s
        HEATER110MW1S = 0x2F,
        /// Activate heater with 110mW for 0.1s
        HEATER110MW01S = 0x24,
        /// Activate heater with 20mW for 1s
        HEATER20MW1S = 0x1E,
        /// Activate heater with 20mW for 0.1s
        HEATER20MW01S = 0x15,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Read,
    ReadData,
}

fn crc8(data: &[u8]) -> u8 {
    let polynomial = 0x31;
    let mut crc = 0xff;

    for x in 0..data.len() {
        crc ^= data[x];
        for _i in 0..8 {
            if (crc & 0x80) != 0 {
                crc = crc << 1 ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

pub struct SHT4x<'a, A: Alarm<'a>, I: i2c::I2CDevice> {
    i2c: &'a I,
    humidity_client: OptionalCell<&'a dyn kernel::hil::sensors::HumidityClient>,
    temperature_client: OptionalCell<&'a dyn kernel::hil::sensors::TemperatureClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    read_temp: Cell<bool>,
    read_hum: Cell<bool>,
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice> SHT4x<'a, A, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], alarm: &'a A) -> SHT4x<'a, A, I> {
        SHT4x {
            i2c: i2c,
            humidity_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            read_temp: Cell::new(false),
            read_hum: Cell::new(false),
            alarm: alarm,
        }
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        if self.read_hum.get() {
            Err(ErrorCode::BUSY)
        } else {
            if self.state.get() == State::Idle {
                let result = self.read_temp_hum();
                if result.is_ok() {
                    self.read_hum.set(true);
                }
                result
            } else {
                self.read_hum.set(true);
                Ok(())
            }
        }
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.read_temp.get() {
            Err(ErrorCode::BUSY)
        } else {
            if self.state.get() == State::Idle {
                let result = self.read_temp_hum();
                if result.is_ok() {
                    self.read_temp.set(true);
                }
                result
            } else {
                self.read_temp.set(true);
                Ok(())
            }
        }
    }

    fn read_temp_hum(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            self.state.set(State::Read);
            self.i2c.enable();

            buffer[0] = Registers::MEASHIGHREP as u8;

            let _res = self.i2c.write(buffer, 1);
            match _res {
                Ok(()) => Ok(()),
                Err((error, data)) => {
                    self.buffer.replace(data);
                    self.state.set(State::Idle);
                    self.i2c.disable();
                    Err(error.into())
                }
            }
        })
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice> time::AlarmClient for SHT4x<'a, A, I> {
    fn alarm(&self) {
        let state = self.state.get();
        match state {
            State::Read => {
                self.state.set(State::ReadData);
                self.buffer.take().map(|buffer| {
                    let _res = self.i2c.read(buffer, 6);
                });
            }
            _ => {
                // This should never happen
                panic!("SHT4x Invalid alarm!");
            }
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice> i2c::I2CClient for SHT4x<'a, A, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match status {
            Ok(()) => {
                let state = self.state.get();

                match state {
                    State::ReadData => {
                        let read_temp_res = if self.read_temp.get() {
                            self.read_temp.set(false);
                            if crc8(&buffer[0..2]) == buffer[2] {
                                let mut stemp = buffer[0] as u32;
                                stemp <<= 8;
                                stemp |= buffer[1] as u32;
                                let stemp = ((4375 * stemp) >> 14) as i32 - 4500;
                                Some(Ok(stemp))
                            } else {
                                Some(Err(ErrorCode::FAIL))
                            }
                        } else {
                            None
                        };

                        let read_hum_res = if self.read_hum.get() {
                            self.read_hum.set(false);
                            if crc8(&buffer[3..5]) == buffer[5] {
                                let mut shum = buffer[3] as u32;
                                shum <<= 8;
                                shum |= buffer[4] as u32;
                                shum = (625 * shum) >> 12;
                                Some(shum as usize)
                            } else {
                                Some(usize::MAX)
                            }
                        } else {
                            None
                        };

                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);

                        read_temp_res.map(|res| {
                            self.temperature_client.map(|cb| cb.callback(res));
                        });

                        read_hum_res.map(|res| {
                            self.humidity_client.map(|cb| cb.callback(res));
                        });
                    }
                    State::Read => {
                        self.buffer.replace(buffer);
                        let interval = self.alarm.ticks_from_ms(20);
                        self.alarm.set_alarm(self.alarm.now(), interval);
                    }
                    _ => {}
                }
            }
            Err(i2c_err) => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
                if self.read_temp.get() {
                    self.read_temp.set(false);
                    self.temperature_client
                        .map(|cb| cb.callback(Err(i2c_err.into())));
                }
                if self.read_hum.get() {
                    self.read_hum.set(false);
                    self.humidity_client.map(|cb| cb.callback(usize::MAX));
                }
            }
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice> kernel::hil::sensors::HumidityDriver<'a>
    for SHT4x<'a, A, I>
{
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        self.read_humidity()
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice> kernel::hil::sensors::TemperatureDriver<'a>
    for SHT4x<'a, A, I>
{
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}
