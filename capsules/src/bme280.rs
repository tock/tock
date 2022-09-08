//! SyscallDriver for the Bosch BME280 Combined humidity and pressure
//! sensor using the I2C bus.
//!
//! <https://cdn.sparkfun.com/assets/learn_tutorials/4/1/9/BST-BME280_DS001-10.pdf>
//!

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{HumidityClient, HumidityDriver, TemperatureClient, TemperatureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

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

#[derive(Clone, Copy, PartialEq)]
enum DeviceState {
    Identify,
    CalibrationLow,
    CalibrationHigh,
    Probe,
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

#[derive(Clone, Copy, PartialEq)]
struct CalibrationData {
    temp1: u16,
    temp2: u16,
    temp3: u16,

    press1: u16,
    press2: u16,
    press3: u16,
    press4: u16,
    press5: u16,
    press6: u16,
    press7: u16,
    press8: u16,
    press9: u16,

    hum1: u16,
    hum2: u16,
    hum3: u16,
    hum4: u16,
    hum5: u16,
    hum6: u16,
}

impl Default for CalibrationData {
    fn default() -> Self {
        CalibrationData {
            temp1: 0,
            temp2: 0,
            temp3: 0,

            press1: 0,
            press2: 0,
            press3: 0,
            press4: 0,
            press5: 0,
            press6: 0,
            press7: 0,
            press8: 0,
            press9: 0,

            hum1: 0,
            hum2: 0,
            hum3: 0,
            hum4: 0,
            hum5: 0,
            hum6: 0,
        }
    }
}

pub struct Bme280<'a> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a dyn I2CDevice,
    calibration: Cell<CalibrationData>,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
    humidity_client: OptionalCell<&'a dyn HumidityClient>,
    state: Cell<DeviceState>,
    op: Cell<Operation>,
    t_fine: Cell<usize>,
}

impl<'a> Bme280<'a> {
    pub fn new(i2c: &'a dyn I2CDevice, buffer: &'static mut [u8]) -> Self {
        Bme280 {
            buffer: TakeCell::new(buffer),
            i2c,
            calibration: Cell::new(CalibrationData::default()),
            temperature_client: OptionalCell::empty(),
            humidity_client: OptionalCell::empty(),
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

impl<'a> TemperatureDriver<'a> for Bme280<'a> {
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

impl<'a> HumidityDriver<'a> for Bme280<'a> {
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

impl<'a> I2CClient for Bme280<'a> {
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
                    unimplemented!();
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
                calib.temp1 = buffer[0] as u16 | (buffer[1] as u16) << 8;
                calib.temp2 = buffer[2] as u16 | (buffer[3] as u16) << 8;
                calib.temp3 = buffer[4] as u16 | (buffer[5] as u16) << 8;
                calib.press1 = buffer[6] as u16 | (buffer[7] as u16) << 8;
                calib.press2 = buffer[8] as u16 | (buffer[9] as u16) << 8;
                calib.press3 = buffer[10] as u16 | (buffer[11] as u16) << 8;
                calib.press4 = buffer[12] as u16 | (buffer[13] as u16) << 8;
                calib.press5 = buffer[14] as u16 | (buffer[15] as u16) << 8;
                calib.press6 = buffer[16] as u16 | (buffer[17] as u16) << 8;
                calib.press7 = buffer[18] as u16 | (buffer[19] as u16) << 8;
                calib.press8 = buffer[20] as u16 | (buffer[21] as u16) << 8;
                calib.press9 = buffer[22] as u16 | (buffer[23] as u16) << 8;
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

                buffer[0] = CTRL_MEAS;
                self.i2c.write_read(buffer, 1, 1).unwrap();
                self.state.set(DeviceState::Probe);
            }
            DeviceState::Probe => {
                if buffer[0] & 0x11 == 0 {
                    // We are in sleep mode, setup the device
                    // Set oversampling to 1
                    buffer[0] = CTRL_HUM;
                    buffer[1] = 1;
                    self.i2c.write_read(buffer, 2, 1).unwrap();

                    self.state.set(DeviceState::Start);
                } else {
                    // Everything is already setup, just start
                    self.state.set(DeviceState::Normal);
                    self.buffer.replace(buffer);
                }
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
                        let adc_temperature = (buffer[0] as usize) << 12
                            | (buffer[1] as usize) << 4
                            | (((buffer[2] as usize) >> 4) & 0x0F);

                        if adc_temperature == 0 {
                            // We got a misread, try again
                            self.buffer.replace(buffer);
                            self.op.set(Operation::None);
                            let _ = self.read_temperature();
                            return;
                        }

                        let var1 = (((adc_temperature >> 3) - ((calib.temp1 as usize) << 1))
                            * (calib.temp2 as usize))
                            >> 11;
                        let var2 = (((((adc_temperature >> 4) - (calib.temp1 as usize))
                            * ((adc_temperature >> 4) - (calib.temp1 as usize)))
                            >> 12)
                            * (calib.temp3 as usize))
                            >> 14;

                        self.t_fine.set(var1 + var2);

                        let temperature = ((self.t_fine.get() * 5 + 128) >> 8) / 100;

                        self.temperature_client
                            .map(|client| client.callback(Ok(temperature as i32)));
                    }
                    Operation::Pressure => {
                        unimplemented!();
                    }
                    Operation::Humidity => {
                        let calib = self.calibration.get();
                        let adc_hum = (buffer[0] as usize) << 8 | buffer[1] as usize;

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
                            - ((calib.hum4 as usize) << 20)
                            - ((calib.hum5 as usize) * t_fine_offset))
                            + 16384)
                            >> 15)
                            * (((((((t_fine_offset * (calib.hum6 as usize)) >> 10)
                                * (((t_fine_offset * (calib.hum3 as usize)) >> 11) + 32768))
                                >> 10)
                                + 2097152)
                                * (calib.hum2 as usize)
                                + 8192)
                                >> 14);
                        let var2 = var1
                            - (((((var1 >> 15) * (var1 >> 15)) >> 7) * (calib.hum1 as usize)) >> 4);

                        let var6 = if var2 > 419430400 { 419430400 } else { var2 };

                        let hum = (var6 >> 12) / 1024;

                        self.humidity_client.map(|client| client.callback(hum));
                    }
                }
                self.buffer.replace(buffer);
                self.op.set(Operation::None);
            }
        }
    }
}
