//! SyscallDriver for the STMicro HTS221 relative humidity and temperature sensor
//! using the I2C bus.
//!
//! <https://www.st.com/en/mems-and-sensors/hts221.html>
//!
//! > The HTS221 is an ultra-compact sensor for relative humidity and
//! > temperature. It includes a sensing element and a mixed signal ASIC
//! > to provide the measurement information through digital serial
//! > interfaces. The sensing element consists of a polymer dielectric
//! > planar capacitor structure capable of detecting relative humidity
//! > variations and is manufactured using a dedicated ST process.
//!
//! Driver Semantics
//! ----------------
//!
//! This driver exposes the HTS221's temperature and humidity functionality
//! via the [TemperatureDriver] and [HumidityDriver] HIL interfaces. The driver _does not_
//! attempt to support multiple concurrent requests for temperature or multiple concurrent
//! requests for humidity, but _does_ support a concurrent request each for temperature and
//! humidity. It is not the role of this driver to provide virtualization to multiple clients, but
//! it does provide virtualization to allow it to be used from both a temperature and humidity
//! driver.
//!
//! Specifically, the implementation _always_ reads both temperature and humidity (the
//! chip always provides both anyway). If the driver receives a request for either temperature or humidity while a
//! request for the other is outstanding, both will be returned to their respective clients when
//! the I2C transaction is completed, rather than performing two separate transactions.
//!
//! Polling for data readiness
//! --------------------------
//!
//! The HTS221 has a data-ready line that can provide an interrupt when temperature and humidity
//! data is ready. However, the primary board (the Nano 33 BLE Sense) this driver was developed for does not connect that
//! line to the MCU and, typically, data is ready within a couple read/write I2C transactions. So,
//! the driver **polls** the status register instead. This is probably not optimal from an energy
//! perspective, so should a use case for an interrupt driven interface arise, some brave soul
//! should modify the driver to support both.
//!
//! Limitations
//! -----------
//!
//! The driver uses floating point math to adjust readings based on the calibration registers.
//! This is accurate and matches the chip's datasheet's recommendation, but could increase code
//! size significantly in platforms that do not have hardware support for floating point
//! operations.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let hts221_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x5f));
//! let hts221 = static_init!(
//!     capsules::hts221::Hts221<'static>,
//!     capsules::hts221::Hts221::new(hts221_i2c,
//!         &mut capsules::hts221::BUFFER));
//! hts221_i2c.set_client(hts221);
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{HumidityClient, HumidityDriver, TemperatureClient, TemperatureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

const REG_AUTO_INCREMENT: u8 = 1 << 7;
const CTRL_REG1: u8 = 0x20;
const STATUS_REG: u8 = 0x27;
const HUMID0_REG: u8 = 0x28;
const CALIB_REG_1ST: u8 = 0x30;

#[derive(Copy, Clone, Debug)]
struct CalibrationData {
    temp_slope: f32,
    temp_intercept: f32,
    humidity_slope: f32,
    humidity_intercept: f32,
}

pub struct Hts221<'a> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a dyn I2CDevice,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
    humidity_client: OptionalCell<&'a dyn HumidityClient>,
    state: Cell<State>,
    pending_temperature: Cell<bool>,
    pending_humidity: Cell<bool>,
}

impl<'a> Hts221<'a> {
    pub fn new(i2c: &'a dyn I2CDevice, buffer: &'static mut [u8]) -> Self {
        Hts221 {
            buffer: TakeCell::new(buffer),
            i2c,
            temperature_client: OptionalCell::empty(),
            humidity_client: OptionalCell::empty(),
            state: Cell::new(State::Reset),
            pending_temperature: Cell::new(false),
            pending_humidity: Cell::new(false),
        }
    }

    // Helper method to kick off a reading for both temperature and humidity.
    //
    // There are three cases:
    //   1. There is no calibration data available yet ([State::Reset])
    //   2. There is calibration data already ([State::Idle])
    //   3. There is a reading already taking place
    fn start_reading(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c.enable();
                match self.state.get() {
                    State::Reset => {
                        buffer[0] = REG_AUTO_INCREMENT | CALIB_REG_1ST;

                        if let Err((_error, buffer)) = self.i2c.write_read(buffer, 1, 16) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::Calibrating);
                        }
                    }
                    State::Idle(calibration_data, _, _) => {
                        buffer[0] = REG_AUTO_INCREMENT | CTRL_REG1;
                        buffer[1] = 1 << 2 | 1 << 7; // BDU + PD
                        buffer[2] = 1; // ONE SHOT

                        if let Err((_error, buffer)) = self.i2c.write_read(buffer, 1, 16) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::InitiateReading(calibration_data));
                        }
                    }
                    _ => {} // Should really never happen since we only have `buffer` available in the above two states
                }
            })
            .ok_or(ErrorCode::BUSY)
    }
}

impl<'a> TemperatureDriver<'a> for Hts221<'a> {
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

impl<'a> HumidityDriver<'a> for Hts221<'a> {
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
    Reset,
    Calibrating,
    InitiateReading(CalibrationData),
    CheckStatus(CalibrationData),
    Read(CalibrationData),
    Idle(CalibrationData, usize, usize),
}

impl<'a> I2CClient for Hts221<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if status.is_err() {
            self.state.set(State::Idle(
                CalibrationData {
                    temp_slope: 0.0,
                    temp_intercept: 0.0,
                    humidity_slope: 0.0,
                    humidity_intercept: 0.0,
                },
                0,
                0,
            ));
            self.buffer.replace(buffer);
            self.temperature_client.map(|client| client.callback(0));
            self.humidity_client.map(|client| client.callback(0));
            return;
        }
        match status {
            Ok(()) => {
                match self.state.get() {
                    State::Calibrating => {
                        let h0rh = buffer[0] as f32;
                        let h1rh = buffer[1] as f32;
                        let h0t0out = ((buffer[6] as i16) | ((buffer[7] as i16) << 8)) as f32;
                        let h1t0out = ((buffer[10] as i16) | ((buffer[11] as i16) << 8)) as f32;

                        let humidity_slope = (h1rh - h0rh) / (2.0 * (h1t0out - h0t0out));
                        let humidity_intercept = (h0rh / 2.0) - humidity_slope * h0t0out;

                        let t0deg_c =
                            ((buffer[2] as i16) | (((buffer[5] & 0b11) as i16) << 8)) as f32;
                        let t1deg_c =
                            ((buffer[3] as i16) | (((buffer[5] & 0b1100) as i16) << 6)) as f32;

                        let t0out = ((buffer[12] as i16) | ((buffer[13] as i16) << 8)) as f32;
                        let t1out = ((buffer[14] as i16) | ((buffer[15] as i16) << 8)) as f32;

                        let temp_slope = (t1deg_c - t0deg_c) / (8.0 * (t1out - t0out));
                        let temp_intercept = (t0deg_c / 8.0) - temp_slope * t0out;

                        buffer[0] = REG_AUTO_INCREMENT | CTRL_REG1;
                        buffer[1] = 1 << 2 | 1 << 7; // BDU + PD
                        buffer[2] = 1; // ONE SHOT

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 3) {
                            self.state.set(State::Idle(
                                CalibrationData {
                                    temp_slope: 0.0,
                                    temp_intercept: 0.0,
                                    humidity_slope: 0.0,
                                    humidity_intercept: 0.0,
                                },
                                0,
                                0,
                            ));
                            self.buffer.replace(buffer);
                            self.temperature_client.map(|client| client.callback(0));
                            self.humidity_client.map(|client| client.callback(0));
                        } else {
                            self.state.set(State::InitiateReading(CalibrationData {
                                temp_slope,
                                temp_intercept,
                                humidity_slope,
                                humidity_intercept,
                            }));
                        }
                    }
                    State::InitiateReading(calibration_data) => {
                        buffer[0] = STATUS_REG;

                        if let Err((_error, buffer)) = self.i2c.write_read(buffer, 1, 1) {
                            self.state.set(State::Idle(
                                CalibrationData {
                                    temp_slope: 0.0,
                                    temp_intercept: 0.0,
                                    humidity_slope: 0.0,
                                    humidity_intercept: 0.0,
                                },
                                0,
                                0,
                            ));
                            self.buffer.replace(buffer);
                            self.temperature_client.map(|client| client.callback(0));
                            self.humidity_client.map(|client| client.callback(0));
                        } else {
                            self.state.set(State::CheckStatus(calibration_data));
                        }
                    }
                    State::CheckStatus(calibration_data) => {
                        if buffer[0] & 0b11 == 0b11 {
                            buffer[0] = REG_AUTO_INCREMENT | HUMID0_REG;

                            if let Err((_error, buffer)) = self.i2c.write_read(buffer, 1, 4) {
                                self.state.set(State::Idle(
                                    CalibrationData {
                                        temp_slope: 0.0,
                                        temp_intercept: 0.0,
                                        humidity_slope: 0.0,
                                        humidity_intercept: 0.0,
                                    },
                                    0,
                                    0,
                                ));
                                self.buffer.replace(buffer);
                                self.temperature_client.map(|client| client.callback(0));
                                self.humidity_client.map(|client| client.callback(0));
                            } else {
                                self.state.set(State::Read(calibration_data));
                            }
                        } else {
                            buffer[0] = STATUS_REG;

                            if let Err((_error, buffer)) = self.i2c.write_read(buffer, 1, 1) {
                                self.state.set(State::Idle(
                                    CalibrationData {
                                        temp_slope: 0.0,
                                        temp_intercept: 0.0,
                                        humidity_slope: 0.0,
                                        humidity_intercept: 0.0,
                                    },
                                    0,
                                    0,
                                ));
                                self.buffer.replace(buffer);
                                self.temperature_client.map(|client| client.callback(0));
                                self.humidity_client.map(|client| client.callback(0));
                            }
                        }
                    }
                    State::Read(calibration_data) => {
                        let humidity_raw = ((buffer[0] as i16) | ((buffer[1] as i16) << 8)) as f32;
                        let humidity = ((humidity_raw * calibration_data.humidity_slope
                            + calibration_data.humidity_intercept)
                            * 100.0) as usize;

                        let temperature_raw =
                            ((buffer[2] as i16) | ((buffer[3] as i16) << 8)) as f32;
                        let temperature = ((temperature_raw * calibration_data.temp_slope
                            + calibration_data.temp_intercept)
                            * 100.0) as usize;
                        buffer[0] = CTRL_REG1;
                        // TODO(alevy): this is a workaround for a bug. We should be able to turn
                        // off the the sensor between transactions, and turn it back on (as is done
                        // in [start_reading]), but doing so seems not to work and the sensor's
                        // Status register never updates to read after the first transaction. For
                        // now, leave it on and waste 2uA.
                        buffer[1] = 1 << 7; // Leave PD bit on

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 2) {
                            self.state.set(State::Idle(
                                CalibrationData {
                                    temp_slope: 0.0,
                                    temp_intercept: 0.0,
                                    humidity_slope: 0.0,
                                    humidity_intercept: 0.0,
                                },
                                0,
                                0,
                            ));
                            self.buffer.replace(buffer);
                            self.temperature_client.map(|client| client.callback(0));
                            self.humidity_client.map(|client| client.callback(0));
                        } else {
                            self.state
                                .set(State::Idle(calibration_data, temperature, humidity));
                        }
                    }
                    State::Idle(_, temperature, humidity) => {
                        self.buffer.replace(buffer);
                        self.i2c.disable();
                        if self.pending_temperature.get() {
                            self.pending_temperature.set(false);
                            self.temperature_client
                                .map(|client| client.callback(temperature));
                        }
                        if self.pending_humidity.get() {
                            self.pending_humidity.set(false);
                            self.humidity_client.map(|client| client.callback(humidity));
                        }
                    }
                    State::Reset => {} // should never happen
                }
            }
            _ => {
                kernel::debug!("Oops, some sort of error {:?}", status);
            }
        }
    }
}
