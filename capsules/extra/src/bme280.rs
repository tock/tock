// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for the Bosch BME280 Combined humidity and pressure
//! sensor using the I2C bus.
//!
//! <https://cdn.sparkfun.com/assets/learn_tutorials/4/1/9/BST-BME280_DS001-10.pdf>
//!

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{
    HumidityClient, HumidityDriver, PressureClient, PressureDriver, TemperatureClient,
    TemperatureDriver,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};

const HUM_MSB: u8 = 0xFD;
const TEMP_MSB: u8 = 0xFA;
#[allow(dead_code)]
const PRESS_MSB: u8 = 0xF7;
#[allow(dead_code)]
const CONFIG: u8 = 0xF5;
const CTRL_MEAS: u8 = 0xF4;
#[allow(dead_code)]
const STATUS: u8 = 0xF3;
const CTRL_HUM: u8 = 0xF2;
#[allow(dead_code)]
const CALIB41: u8 = 0xF0;
const CALIB26: u8 = 0xE1;
#[allow(dead_code)]
const RESET: u8 = 0xE0;
const ID: u8 = 0xD0;
#[allow(dead_code)]
const CALIB25: u8 = 0xA1;
const CALIB00: u8 = 0x88;
// Raw 20-bit value returned when pressure measurement is skipped.
const SKIPPED_PRESSURE_READING: i32 = 0x80000;

#[derive(Clone, Copy, PartialEq)]
enum DeviceState {
    Identify,
    CalibrationLow,
    CalibrationHigh,
    Start,
    Normal,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Temp,
    Pressure,
    Humidity,
}

#[derive(Clone, Copy, PartialEq, Default)]
struct CalibrationData {
    temp1: u16,
    temp2: u16,
    temp3: u16,

    press1: u16,
    press2: i16,
    press3: i16,
    press4: i16,
    press5: i16,
    press6: i16,
    press7: i16,
    press8: i16,
    press9: i16,

    hum1: u16,
    hum2: u16,
    hum3: u16,
    hum4: u16,
    hum5: u16,
    hum6: u16,
}

pub struct Bme280<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    calibration: Cell<CalibrationData>,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
    humidity_client: OptionalCell<&'a dyn HumidityClient>,
    pressure_client: OptionalCell<&'a dyn PressureClient>,
    state: Cell<DeviceState>,
    op: Cell<Operation>,
    t_fine: Cell<i32>,
}

impl<'a, I: I2CDevice> Bme280<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8]) -> Self {
        Bme280 {
            buffer: TakeCell::new(buffer),
            i2c,
            calibration: Cell::new(CalibrationData::default()),
            temperature_client: OptionalCell::empty(),
            humidity_client: OptionalCell::empty(),
            pressure_client: OptionalCell::empty(),
            state: Cell::new(DeviceState::Identify),
            op: Cell::new(Operation::None),
            t_fine: Cell::new(0),
        }
    }

    pub fn startup(&self) {
        self.buffer.take().map(|buffer| {
            if self.state.get() == DeviceState::Identify {
                // Read the ID buffer
                buffer[0] = ID;
                self.i2c.write_read(buffer, 1, 1).unwrap();
            }
        });
    }
}

impl<'a, I: I2CDevice> TemperatureDriver<'a> for Bme280<'a, I> {
    fn set_client(&self, client: &'a dyn TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            buffer[0] = TEMP_MSB;

            self.op.set(Operation::Temp);
            self.i2c.write_read(buffer, 1, 3).unwrap();
        });

        Ok(())
    }
}

impl<'a, I: I2CDevice> HumidityDriver<'a> for Bme280<'a, I> {
    fn set_client(&self, client: &'a dyn HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            buffer[0] = HUM_MSB;

            self.op.set(Operation::Humidity);
            self.i2c.write_read(buffer, 1, 3).unwrap();
        });

        Ok(())
    }
}

impl<'a, I: I2CDevice> PressureDriver<'a> for Bme280<'a, I> {
    fn set_client(&self, client: &'a dyn PressureClient) {
        self.pressure_client.set(client);
    }

    fn read_atmospheric_pressure(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            buffer[0] = PRESS_MSB;

            self.op.set(Operation::Pressure);
            self.i2c.write_read(buffer, 1, 3).unwrap();
        });

        Ok(())
    }
}

impl<I: I2CDevice> I2CClient for Bme280<'_, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            // We have no way to report an error, so just return a bogus value
            match self.op.get() {
                Operation::None => (),
                Operation::Temp => {
                    self.temperature_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                }
                Operation::Pressure => {
                    self.pressure_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                }
                Operation::Humidity => {
                    self.humidity_client.map(|client| client.callback(0));
                }
            }
            self.buffer.replace(buffer);
            self.op.set(Operation::None);
            return;
        }

        match self.state.get() {
            DeviceState::Identify => {
                if buffer[0] != 0x60 {
                    // We don't have the correct ID, this isn't the correct device
                    // Just stop here
                    self.buffer.replace(buffer);
                    return;
                }

                buffer[0] = CALIB00;
                self.i2c.write_read(buffer, 1, 26).unwrap();
                self.state.set(DeviceState::CalibrationLow);
            }
            DeviceState::CalibrationLow => {
                let mut calib = self.calibration.take();
                //TODO: Use Rust's built in u16 and i16 from_le_bytes(buffer[0], buffer[1]);
                calib.temp1 = buffer[0] as u16 | (buffer[1] as u16) << 8;
                calib.temp2 = buffer[2] as u16 | (buffer[3] as u16) << 8;
                calib.temp3 = buffer[4] as u16 | (buffer[5] as u16) << 8;
                calib.press1 = buffer[6] as u16 | (buffer[7] as u16) << 8;
                calib.press2 = buffer[8] as i16 | (buffer[9] as i16) << 8;
                calib.press3 = buffer[10] as i16 | (buffer[11] as i16) << 8;
                calib.press4 = buffer[12] as i16 | (buffer[13] as i16) << 8;
                calib.press5 = buffer[14] as i16 | (buffer[15] as i16) << 8;
                calib.press6 = buffer[16] as i16 | (buffer[17] as i16) << 8;
                calib.press7 = buffer[18] as i16 | (buffer[19] as i16) << 8;
                calib.press8 = buffer[20] as i16 | (buffer[21] as i16) << 8;
                calib.press9 = buffer[22] as i16 | (buffer[23] as i16) << 8;
                calib.hum1 = buffer[25] as u16;
                self.calibration.set(calib);

                if calib.temp1 == 0 || calib.temp2 == 0 || calib.temp3 == 0 {
                    // We received stale calibration data, let's try again

                    buffer[0] = CALIB00;
                    self.i2c.write_read(buffer, 1, 26).unwrap();
                    self.state.set(DeviceState::CalibrationLow);
                    return;
                }

                buffer[0] = CALIB26;
                self.i2c.write_read(buffer, 1, 8).unwrap();

                self.state.set(DeviceState::CalibrationHigh);
            }
            DeviceState::CalibrationHigh => {
                let mut calib = self.calibration.take();
                calib.hum2 = buffer[0] as u16 | (buffer[1] as u16) << 8;
                calib.hum3 = buffer[3] as u16;
                calib.hum4 = buffer[4] as u16 | (buffer[5] as u16) << 4;
                calib.hum5 = (buffer[6] as u16 >> 4) | (buffer[7] as u16) << 4;
                calib.hum6 = buffer[8] as u16;
                self.calibration.set(calib);

                // Set humidity oversampling to 1
                buffer[0] = CTRL_HUM;
                buffer[1] = 1;
                self.i2c.write(buffer, 2).unwrap();
                self.state.set(DeviceState::Start);
            }
            DeviceState::Start => {
                // Set the mode to normal and set oversampling to 1
                buffer[0] = CTRL_MEAS;
                buffer[1] = 0x11 | 1 << 5 | 1 << 2;
                self.i2c.write(buffer, 2).unwrap();

                self.state.set(DeviceState::Normal);
            }
            DeviceState::Normal => {
                match self.op.get() {
                    Operation::None => (),
                    Operation::Temp => {
                        let calib = self.calibration.get();

                        let adc_temperature: i32 = ((buffer[0] as usize) << 12
                            | (buffer[1] as usize) << 4
                            | (((buffer[2] as usize) >> 4) & 0x0F))
                            as i32;

                        if adc_temperature == 0 {
                            // We got a misread, try again
                            self.buffer.replace(buffer);
                            self.op.set(Operation::None);
                            let _ = self.read_temperature();
                            return;
                        }

                        let var1 = (((adc_temperature >> 3) - ((calib.temp1 as i32) << 1))
                            * (calib.temp2 as i32))
                            >> 11;
                        let var2 = (((((adc_temperature >> 4) - (calib.temp1 as i32))
                            * ((adc_temperature >> 4) - (calib.temp1 as i32)))
                            >> 12)
                            * (calib.temp3 as i32))
                            >> 14;

                        self.t_fine.set(var1 + var2);

                        let temperature = (self.t_fine.get() * 5 + 128) >> 8;

                        self.temperature_client
                            .map(|client| client.callback(Ok(temperature)));
                    }
                    Operation::Pressure => {
                        let calib = self.calibration.get();
                        let adc_pressure: i32 = ((buffer[0] as usize) << 12
                            | (buffer[1] as usize) << 4
                            | (buffer[2] as usize) >> 4)
                            as i32;

                        if adc_pressure == SKIPPED_PRESSURE_READING {
                            self.buffer.replace(buffer);
                            self.op.set(Operation::None);
                            self.pressure_client
                                .map(|client| client.callback(Err(ErrorCode::FAIL)));
                            return;
                        }

                        // This is straight from the datasheet (Page 25/60)
                        let mut var1: i64;
                        let mut var2: i64;
                        let mut p: i64;

                        var1 = self.t_fine.get() as i64 - 128_000;
                        var2 = var1 * var1 * (calib.press6 as i64);
                        var2 += (var1 * (calib.press5 as i64)) << 17;
                        var2 += (calib.press4 as i64) << 35;
                        var1 = ((var1 * var1 * calib.press3 as i64) >> 8)
                            + ((var1 * calib.press2 as i64) << 12);
                        var1 = ((((1_i64) << 47) + var1) * (calib.press1 as i64)) >> 33;

                        // Avoid divide by zero fault
                        // Spec sheet returns 0 to client here
                        if var1 == 0 {
                            self.buffer.replace(buffer);
                            self.op.set(Operation::None);
                            self.pressure_client.map(|client| client.callback(Ok(0)));
                            return;
                        }

                        p = 1_048_576 - adc_pressure as i64;
                        p = (((p << 31) - var2) * 3125) / var1;
                        var1 = (calib.press9 as i64 * (p >> 13) * (p >> 13)) >> 25;
                        var2 = (calib.press8 as i64 * p) >> 19;
                        p = ((p + var1 + var2) >> 8) + ((calib.press7 as i64) << 4);

                        // p is Q24.8 Pa but we expect to return in hPa
                        let pressure_hpa = p / 25_600;

                        self.pressure_client
                            .map(|client| client.callback(Ok(pressure_hpa as u32)));
                    }
                    Operation::Humidity => {
                        let calib = self.calibration.get();
                        let adc_hum = (((buffer[0] as u32) << 8) | (buffer[1] as u32)) as i32;

                        if adc_hum == 0 {
                            // We got a misread, try again
                            self.buffer.replace(buffer);
                            self.op.set(Operation::None);
                            let _ = self.read_humidity();
                            return;
                        }

                        let t_fine_offset = self.t_fine.get() - 76800;

                        // This is straight from the datasheet
                        let var1 = ((((adc_hum << 14)
                            - ((calib.hum4 as i32) << 20)
                            - ((calib.hum5 as i32) * t_fine_offset))
                            + 16384)
                            >> 15)
                            * (((((((t_fine_offset * (calib.hum6 as i32)) >> 10)
                                * (((t_fine_offset * (calib.hum3 as i32)) >> 11) + 32768))
                                >> 10)
                                + 2097152)
                                * (calib.hum2 as i32)
                                + 8192)
                                >> 14);
                        let var2 = var1
                            - (((((var1 >> 15) * (var1 >> 15)) >> 7) * (calib.hum1 as i32)) >> 4);

                        let var3 = if var2 < 0 { 0 } else { var2 };
                        let var6 = if var3 > 419430400 { 419430400 } else { var3 };

                        let hum = (((var6 >> 12) * 100) / 1024) as usize;

                        self.humidity_client.map(|client| client.callback(hum));
                    }
                }
                self.buffer.replace(buffer);
                self.op.set(Operation::None);
            }
        }
    }
}
