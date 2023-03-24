//! SyscallDriver for the AMS CCS811 an ultra-low power digital gas sensor
//! solution which integrates a metal oxide (MOX) gas sensor to detect a wide
//! range of Volatile Organic Compounds (VOCs) for indoor air quality
//! monitoring using the I2C bus.
//!
//! <https://cdn.sparkfun.com/assets/learn_tutorials/1/4/3/CCS811_Datasheet-DS000459.pdf>
//!

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{AirQualityClient, AirQualityDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

const STATUS: u8 = 0x00;
const MEAS_MODE: u8 = 0x01;
const ALG_RESULT_DATA: u8 = 0x02;
#[allow(dead_code)]
const RAW_DATA: u8 = 0x03;
#[allow(dead_code)]
const ENV_DATA: u8 = 0x05;
#[allow(dead_code)]
const NTC: u8 = 0x06;
#[allow(dead_code)]
const THRESHOLDS: u8 = 0x10;
#[allow(dead_code)]
const BASELINE: u8 = 0x11;
const HW_ID: u8 = 0x20;
#[allow(dead_code)]
const HW_VERSION: u8 = 0x21;
#[allow(dead_code)]
const FW_BOOT_VERSION: u8 = 0x23;
#[allow(dead_code)]
const FW_APP_VERSION: u8 = 0x24;
#[allow(dead_code)]
const ERROR_ID: u8 = 0xE0;
const APP_START: u8 = 0xF4;
const SW_RESET: u8 = 0xFF;

#[derive(Clone, Copy, PartialEq)]
enum DeviceState {
    Identify,
    Reset,
    StatusCheck,
    StartApp,
    Normal,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Setup,
    SetEnv,
    CO2,
    TVOC,
}

pub struct Ccs811<'a> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a dyn I2CDevice,
    client: OptionalCell<&'a dyn AirQualityClient>,
    state: Cell<DeviceState>,
    op: Cell<Operation>,

    /// Deferred caller for deferring client callbacks.
    deferred_call: DeferredCall,
    deferred_count: Cell<usize>,
}

impl<'a> Ccs811<'a> {
    pub fn new(i2c: &'a dyn I2CDevice, buffer: &'static mut [u8]) -> Self {
        Self {
            buffer: TakeCell::new(buffer),
            i2c,
            client: OptionalCell::empty(),
            state: Cell::new(DeviceState::Identify),
            op: Cell::new(Operation::Setup),
            deferred_call: DeferredCall::new(),
            deferred_count: Cell::new(0),
        }
    }

    pub fn startup(&self) {
        self.buffer.take().map(|buffer| {
            if self.state.get() == DeviceState::Identify {
                // Read the ID buffer
                buffer[0] = HW_ID;
                self.i2c.write_read(buffer, 1, 1).unwrap();
            }
        });
    }
}

impl<'a> AirQualityDriver<'a> for Ccs811<'a> {
    fn set_client(&self, client: &'a dyn AirQualityClient) {
        self.client.set(client);
    }

    fn specify_environment(
        &self,
        temp: Option<i32>,
        humidity: Option<u32>,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            // Set the default values of 50% humidity and 25 degrees Celsius
            buffer[0] = 0x05;
            buffer[1] = 0x64;
            buffer[2] = 0x00;
            buffer[3] = 0x64;
            buffer[4] = 0x00;

            // Copy in our calibration data
            if let Some(hum) = humidity {
                buffer[1] = hum as u8 * 2;
            }
            if let Some(t) = temp {
                if t < -25 {
                    buffer[3] = 0;
                } else {
                    buffer[3] = (t as u8 + 25) * 2;
                }
            }

            self.op.set(Operation::SetEnv);
            self.i2c.write(buffer, 5).unwrap();
        });

        Ok(())
    }

    fn read_co2(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            buffer[0] = ALG_RESULT_DATA;

            self.op.set(Operation::CO2);
            self.i2c.write_read(buffer, 1, 6).unwrap();
        });

        Ok(())
    }

    fn read_tvoc(&self) -> Result<(), ErrorCode> {
        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map(|buffer| {
            buffer[0] = ALG_RESULT_DATA;

            self.op.set(Operation::TVOC);
            self.i2c.write_read(buffer, 1, 6).unwrap();
        });

        Ok(())
    }
}

impl<'a> I2CClient for Ccs811<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if status.is_err() {
            match self.op.get() {
                Operation::None | Operation::Setup => (),
                Operation::SetEnv => {
                    self.client
                        .map(|client| client.environment_specified(Err(ErrorCode::FAIL)));
                }
                Operation::CO2 => {
                    self.client
                        .map(|client| client.co2_data_available(Err(ErrorCode::FAIL)));
                }
                Operation::TVOC => {
                    self.client
                        .map(|client| client.tvoc_data_available(Err(ErrorCode::FAIL)));
                }
            }
            self.buffer.replace(buffer);
            self.op.set(Operation::None);
            return;
        }

        match self.state.get() {
            DeviceState::Identify => {
                if buffer[0] != 0x81 {
                    // We don't have the correct ID, this isn't the correct device
                    // Just stop here
                    self.buffer.replace(buffer);
                    return;
                }

                buffer[0] = SW_RESET;
                buffer[1] = 0x11;
                buffer[2] = 0xE5;
                buffer[3] = 0x72;
                buffer[4] = 0x8A;
                self.i2c.write(buffer, 5).unwrap();
                self.state.set(DeviceState::Reset);
            }
            DeviceState::Reset => {
                self.deferred_call.set();
                self.buffer.replace(buffer);
            }
            DeviceState::StatusCheck => {
                if buffer[0] & 0x01 == 0x01 {
                    self.buffer.replace(buffer);
                    return;
                }

                if buffer[0] & 0x04 == 0x04 {
                    self.buffer.replace(buffer);
                    return;
                }

                buffer[0] = APP_START;
                self.i2c.write(buffer, 1).unwrap();
                self.state.set(DeviceState::StartApp);
            }
            DeviceState::StartApp => {
                buffer[0] = MEAS_MODE;
                // Drive mode: 1 - Constant power mode, IAQ measurement every second
                // Interrupt data ready: 0 - Interrupt generation is disabled
                // Interrup Threshold: 0 - Interrupt mode operates normally
                buffer[1] = (1 << 4) | (0 << 3) | (0 << 2);
                self.i2c.write(buffer, 2).unwrap();

                self.state.set(DeviceState::Normal);
            }
            DeviceState::Normal => {
                match self.op.get() {
                    Operation::None => (),
                    Operation::Setup => {
                        self.buffer.replace(buffer);
                        self.deferred_call.set();
                        return;
                    }
                    Operation::SetEnv => {
                        self.client
                            .map(|client| client.environment_specified(Ok(())));
                    }
                    Operation::CO2 => {
                        let co2 = (buffer[0] as u32) << 8 | buffer[1] as u32;
                        let status = buffer[4];
                        let _error_id = buffer[5];

                        if status & 0x01 == 0x01 {
                            self.client
                                .map(|client| client.co2_data_available(Err(ErrorCode::FAIL)));
                        }

                        self.client.map(|client| client.co2_data_available(Ok(co2)));
                    }
                    Operation::TVOC => {
                        let tvoc = (buffer[2] as u32) << 8 | buffer[3] as u32;
                        let status = buffer[4];
                        let _error_id = buffer[5];

                        if status & 0x01 == 0x01 {
                            self.client
                                .map(|client| client.tvoc_data_available(Err(ErrorCode::FAIL)));
                        }

                        self.client
                            .map(|client| client.tvoc_data_available(Ok(tvoc)));
                    }
                }
                self.buffer.replace(buffer);
                self.op.set(Operation::None);
            }
        }
    }
}

impl<'a> DeferredCallClient for Ccs811<'a> {
    fn handle_deferred_call(&self) {
        if self.deferred_count.get() > 1000 {
            match self.state.get() {
                DeviceState::Reset => {
                    self.buffer.take().map(|buffer| {
                        buffer[0] = STATUS;
                        self.i2c.write_read(buffer, 1, 1).unwrap();

                        self.state.set(DeviceState::StatusCheck);
                    });
                }
                DeviceState::Normal => {
                    self.op.set(Operation::None);
                }
                _ => unreachable!(),
            }
        } else {
            self.deferred_count.set(self.deferred_count.get() + 1);
            self.deferred_call.set();
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
