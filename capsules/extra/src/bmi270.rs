// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Sensor Driver for the BMI270 Inertial Measurement Unit
//!
//! <https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmi270-ds000.pdf>
//!
//! > The device is a highly intergrated, low power inertial measurement unit (IMU)
//! > that combines precise acceleration and angular rate (gyroscopic) measurement
//! > with intelligent on-chip motion-triggered interrupt features.
//!
//! Driver Semantics
//! ----------------
//!
//! This driver exposes the BMI270's accelerometer and gyroscope functionality via
//! the [NineDof] HIL interface. If the driver receives a request for either acceleration
//! or gyroscopic data while a request for the other is outstanding, both will be returned
//! when the I2C transaction is completed, rather than performing two separate transactions.
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
//! ```rust
//!
//! let bmi270 = BMI270Component::new(mux_i2c, 0x68, mux_alarm).finalize(
//!     components::bmi270_component_static!(nrf52840::rtc::Rtc<'static>, nrf52840::i2c::TWI));
//! bmi270.begin_reset();
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!     .finalize(components::ninedof_component_static!(bmi270));
//! ```
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{NineDof, NineDofClient};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

/// Syscall driver number
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::NINEDOF as usize;

// Time constants
const ALARM_TIME_20000: u32 = 20000_u32;
const ALARM_TIME_450: u32 = 450_u32;

/// Register values

#[allow(dead_code)]
enum Registers {
    ChipID = 0x00,
    ErrReg = 0x02,
    Status = 0x03,
    Data0 = 0x04,
    Data1 = 0x05,
    Data2 = 0x06,
    Data3 = 0x07,
    Data4 = 0x08,
    Data5 = 0x09,
    Data6 = 0x0A,
    Data7 = 0x0B,
    Data8 = 0x0C,
    Data9 = 0x0D,
    Data10 = 0x0E,
    Data11 = 0x0F,
    Data12 = 0x10,
    Data13 = 0x11,
    Data14 = 0x12,
    Data15 = 0x13,
    Data16 = 0x14,
    Data17 = 0x15,
    Data18 = 0x16,
    Data19 = 0x17,
    SensorTime0 = 0x18,
    SensorTime1 = 0x19,
    SensorTime2 = 0x1A,
    Event = 0x1B,
    IntStatus0 = 0x1C,
    IntStatus1 = 0x1D,
    ScOut0 = 0x1E,
    ScOut1 = 0x1F,
    WrGestAct = 0x20,
    InternalStatus = 0x21,
    Temperature0 = 0x22,
    Temperature1 = 0x23,
    FifoLength0 = 0x24,
    FifoLength1 = 0x25,
    FifoData = 0x26,
    FeatPage = 0x2F,
    Features = 0x30,
    AccConf = 0x40,
    AccRange = 0x41,
    GyrConf = 0x42,
    GyrRange = 0x43,
    AuxConf = 0x44,
    FifoDowns = 0x45,
    FifoWtm0 = 0x46,
    FifoWtm1 = 0x47,
    FifoConfig0 = 0x48,
    FifoConfig1 = 0x49,
    Saturation = 0x4A,
    AuxDevId = 0x4B,
    AuxIfConf = 0x4C,
    AuxRdAddr = 0x4D,
    AuxWrAddr = 0x4E,
    AuxWrData = 0x4F,
    ErrRegMsk = 0x52,
    Int1IoCtrl = 0x53,
    Int2IoCtrl = 0x54,
    IntLatch = 0x55,
    Int1MapFeat = 0x56,
    Int2MapFeat = 0x57,
    IntMapData = 0x58,
    InitCtrl = 0x59,
    InitAddr0 = 0x5B,
    InitAddr1 = 0x5C,
    InitData = 0x5E,
    InternalError = 0x5F,
    AuxIfTrim = 0x68,
    GyrCrtConf = 0x69,
    NvmConf = 0x6A,
    IfConf = 0x6B,
    Drv = 0x6C,
    AccSelfTest = 0x6D,
    GyrSelfTestAxes = 0x6E,
    NvConf = 0x70,
    Offset0 = 0x71,
    Offset1 = 0x72,
    Offset2 = 0x73,
    Offset3 = 0x74,
    Offset4 = 0x75,
    Offset5 = 0x76,
    Offset6 = 0x77,
    PwrConf = 0x7C,
    PwrCtrl = 0x7D,
    Cmd = 0x7E,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Sleep,

    WaitingForAlarm(u32),
    InitWriteConfig,
    InitDone,

    CheckStatus,

    ConfAccelRange,
    ConfGyro,
    ConfGyroRange,
    ConfPower,
    CheckConf,
    Enable,
    InitRead,
    Read,
    Done,
    Idle,
}

pub struct BMI270<'a, A: Alarm<'a>, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    config_file: TakeCell<'static, [u8]>,
    i2c: &'a I,
    alarm: &'a A,
    ninedof_client: OptionalCell<&'a dyn NineDofClient>,
    state: Cell<State>,
    pending_gyro: Cell<bool>,
    pending_accel: Cell<bool>,
}

impl<'a, A: Alarm<'a>, I: I2CDevice> BMI270<'a, A, I> {
    pub fn new(
        i2c_bus: &'a I,
        alarm: &'a A,
        buffer: &'static mut [u8],
        config_file: &'static mut [u8],
    ) -> BMI270<'a, A, I> {
        BMI270 {
            buffer: TakeCell::new(buffer),
            config_file: TakeCell::new(config_file),
            i2c: i2c_bus,
            alarm: alarm,
            ninedof_client: OptionalCell::empty(),
            state: Cell::new(State::Sleep),
            pending_gyro: Cell::new(false),
            pending_accel: Cell::new(false),
        }
    }

    fn start_measurement(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c.enable();
                match self.state.get() {
                    State::Sleep => {
                        buffer[0] = Registers::PwrConf as u8;
                        buffer[1] = 0x00_u8;

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 2) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::WaitingForAlarm(ALARM_TIME_450));
                        }
                    }
                    State::Idle => {
                        buffer[0] = Registers::Data8 as u8;

                        if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 1) {
                            self.state.set(State::Sleep);
                            self.buffer.replace(buffer);
                            self.ninedof_client
                                .map(|client| client.callback(i2c_err as usize, 0, 0));
                        } else {
                            self.state.set(State::Read);
                        }
                    }
                    _ => {}
                }
            })
            .ok_or(ErrorCode::FAIL)
    }

    fn handle_alarm(&self) {
        let _ = self.buffer.take().map(|buffer| match self.state.get() {
            State::WaitingForAlarm(us) => {
                if us == ALARM_TIME_450 {
                    buffer[0] = Registers::InitCtrl as u8;
                    buffer[1] = 0x00_u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::InitWriteConfig);
                    }
                } else if us == ALARM_TIME_20000 {
                    buffer[0] = Registers::InternalStatus as u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write_read(buffer, 1, 1) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::CheckStatus);
                    }
                }
            }
            _ => {}
        });
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> NineDof<'a> for BMI270<'a, A, I> {
    fn set_client(&self, client: &'a dyn NineDofClient) {
        self.ninedof_client.set(client);
    }

    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        if !self.pending_accel.get() {
            self.pending_accel.set(true);
            if self.pending_gyro.get() {
                Ok(())
            } else {
                self.start_measurement()
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn read_gyroscope(&self) -> Result<(), ErrorCode> {
        if !self.pending_gyro.get() {
            self.pending_gyro.set(true);
            if self.pending_accel.get() {
                Ok(())
            } else {
                self.start_measurement()
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> AlarmClient for BMI270<'a, A, I> {
    fn alarm(&self) {
        self.handle_alarm()
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> I2CClient for BMI270<'a, A, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            match self.state.get() {
                State::InitWriteConfig => {
                    self.config_file.replace(buffer);
                }
                _ => {
                    self.buffer.replace(buffer);
                }
            }
            self.state.set(State::Sleep);
            self.ninedof_client
                .map(|client| client.callback(i2c_err as usize, 0, 0));
            return;
        }

        match self.state.get() {
            State::WaitingForAlarm(us) => {
                self.buffer.replace(buffer);
                let delay = self.alarm.ticks_from_us(us);
                self.alarm.set_alarm(self.alarm.now(), delay);
            }
            State::InitWriteConfig => {
                self.config_file.take().map(|config_file| {
                    if let Err((i2c_err, buffer)) = self.i2c.write(config_file, config_file.len()) {
                        self.state.set(State::Sleep);
                        self.config_file.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::InitDone);
                    }
                });
            }
            State::InitDone => {
                self.config_file.replace(buffer);
                self.buffer.take().map(|buffer| {
                    buffer[0] = Registers::InitCtrl as u8;
                    buffer[1] = 0x01_u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::WaitingForAlarm(ALARM_TIME_20000));
                    }
                });
            }
            State::CheckStatus => {
                if buffer[0] == 1 {
                    buffer[0] = Registers::AccConf as u8;
                    buffer[1] = 0xA8_u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::ConfAccelRange);
                    }
                } else {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                    self.ninedof_client.map(|client| client.callback(1, 0, 0));
                }
            }
            State::ConfAccelRange => {
                buffer[0] = Registers::AccRange as u8;
                buffer[1] = 0x01_u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::ConfGyro);
                }
            }
            State::ConfGyro => {
                buffer[0] = Registers::GyrConf as u8;
                buffer[1] = 0xA8_u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::ConfGyroRange);
                }
            }
            State::ConfGyroRange => {
                buffer[0] = Registers::GyrRange as u8;
                buffer[1] = 0x00_u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::ConfPower);
                }
            }
            State::ConfPower => {
                buffer[0] = Registers::PwrConf as u8;
                buffer[1] = 0x02_u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::CheckConf);
                }
            }
            State::CheckConf => {
                buffer[0] = Registers::Event as u8;

                if let Err((i2c_err, buffer)) = self.i2c.write_read(buffer, 1, 1) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::Enable);
                }
            }
            State::Enable => {
                if buffer[0] == 0 {
                    buffer[0] = Registers::PwrCtrl as u8;
                    buffer[1] = 0x06_u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::InitRead);
                    }
                } else {
                    buffer[0] = Registers::AccConf as u8;
                    buffer[1] = 0xA8_u8;

                    if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 2) {
                        self.state.set(State::Sleep);
                        self.buffer.replace(buffer);
                        self.ninedof_client
                            .map(|client| client.callback(i2c_err as usize, 0, 0));
                    } else {
                        self.state.set(State::ConfAccelRange);
                    }
                }
            }
            State::InitRead => {
                buffer[0] = Registers::Data8 as u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 1) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::Read);
                }
            }
            State::Read => {
                if let Err((i2c_err, buffer)) = self.i2c.read(buffer, 12) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::Done);
                }
            }
            State::Done => {
                let gravity_earth = 9.80665_f32;
                let half_scale = 32768.0;

                let accel_data_x = ((buffer[1] as i16) << 8) | (buffer[0] as i16);
                let accel_data_y = ((buffer[3] as i16) << 8) | (buffer[2] as i16);
                let accel_data_z = ((buffer[5] as i16) << 8) | (buffer[4] as i16);

                let accel_x = (gravity_earth * accel_data_x as f32 * 4.0) / half_scale;
                let accel_y = (gravity_earth * accel_data_y as f32 * 4.0) / half_scale;
                let accel_z = (gravity_earth * accel_data_z as f32 * 4.0) / half_scale;

                let gyro_data_x = ((buffer[7] as i16) << 8) | (buffer[6] as i16);
                let gyro_data_y = ((buffer[9] as i16) << 8) | (buffer[8] as i16);
                let gyro_data_z = ((buffer[11] as i16) << 8) | (buffer[10] as i16);

                let gyro_x = (2000.0 / half_scale) * gyro_data_x as f32;
                let gyro_y = (2000.0 / half_scale) * gyro_data_y as f32;
                let gyro_z = (2000.0 / half_scale) * gyro_data_z as f32;

                self.buffer.replace(buffer);
                self.i2c.disable();
                if self.pending_accel.get() {
                    self.pending_accel.set(false);
                    self.ninedof_client.map(|client| {
                        client.callback(accel_x as usize, accel_y as usize, accel_z as usize)
                    });
                }
                if self.pending_gyro.get() {
                    self.pending_gyro.set(false);
                    self.ninedof_client.map(|client| {
                        client.callback(gyro_x as usize, gyro_y as usize, gyro_z as usize)
                    });
                }

                self.state.set(State::Idle);
            }
            _ => {} // should never happen
        }
    }
}
