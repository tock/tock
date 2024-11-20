// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! DFRobot Gravity Rainfall sensor using the I2C bus.
//!
//! <https://wiki.dfrobot.com/SKU_SEN0575_Gravity_Rainfall_Sensor>
//! <https://github.com/DFRobot/DFRobot_RainfallSensor/tree/master>
//!

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{RainFallClient, RainFallDriver};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub const BUFFER_SIZE: usize = 4;

const PID_REGISTER: u8 = 0x00;
const TIME_RAINFALL_REGISTER: u8 = 0x0C;
const RAIN_HOUR_REGISTER: u8 = 0x26;

#[derive(Clone, Copy, PartialEq)]
enum DeviceState {
    Identify,
    Normal,
    StartRainFall(usize),
    ContinueRainFall,
    FinalRainFall,
    Broken,
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    RainFall,
}

pub struct DFRobotRainFall<'a, A: Alarm<'a>, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    rainfall_client: OptionalCell<&'a dyn RainFallClient>,
    state: Cell<DeviceState>,
    op: Cell<Operation>,
    alarm: &'a A,
}

impl<'a, A: Alarm<'a>, I: I2CDevice> DFRobotRainFall<'a, A, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], alarm: &'a A) -> Self {
        DFRobotRainFall {
            buffer: TakeCell::new(buffer),
            i2c,
            rainfall_client: OptionalCell::empty(),
            state: Cell::new(DeviceState::Identify),
            op: Cell::new(Operation::None),
            alarm,
        }
    }

    pub fn startup(&self) {
        self.buffer.take().map(|buffer| {
            if self.state.get() == DeviceState::Identify {
                // Read the version register
                buffer[0] = PID_REGISTER;
                if let Err((_e, buf)) = self.i2c.write_read(buffer, 1, 4) {
                    self.buffer.replace(buf);
                }
            } else {
                self.buffer.replace(buffer);
            }
        });
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> RainFallDriver<'a> for DFRobotRainFall<'a, A, I> {
    fn set_client(&self, client: &'a dyn RainFallClient) {
        self.rainfall_client.set(client);
    }

    fn read_rainfall(&self, hours: usize) -> Result<(), ErrorCode> {
        if self.state.get() == DeviceState::Broken {
            return Err(ErrorCode::NOSUPPORT);
        }

        if self.state.get() != DeviceState::Normal {
            return Err(ErrorCode::BUSY);
        }

        if self.op.get() != Operation::None {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::BUSY), |buffer| {
            buffer[0] = RAIN_HOUR_REGISTER;
            buffer[1] = hours as u8;

            self.op.set(Operation::RainFall);
            self.state.set(DeviceState::StartRainFall(hours));
            if let Err((e, buf)) = self.i2c.write(buffer, 2) {
                self.buffer.replace(buf);
                return Err(e.into());
            }

            Ok(())
        })
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> I2CClient for DFRobotRainFall<'a, A, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.buffer.replace(buffer);

            match self.op.get() {
                Operation::None => (),
                Operation::RainFall => {
                    self.op.set(Operation::None);

                    self.rainfall_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                }
            }

            return;
        }

        match self.state.get() {
            DeviceState::Identify => {
                let pid =
                    buffer[0] as u32 | (buffer[1] as u32) << 8 | ((buffer[3] as u32) & 0xC0) << 10;
                let vid = buffer[2] as u16 | ((buffer[3] as u16) & 0x3F) << 8;

                if vid != 0x3343 || pid != 0x100C0 {
                    self.buffer.replace(buffer);
                    self.state.set(DeviceState::Broken);
                    self.op.set(Operation::None);
                    return;
                }

                self.buffer.replace(buffer);
                self.state.set(DeviceState::Normal);
                self.op.set(Operation::None);
            }
            DeviceState::StartRainFall(_hours) => match self.op.get() {
                Operation::None => (),
                Operation::RainFall => {
                    self.state.set(DeviceState::ContinueRainFall);
                    buffer[0] = TIME_RAINFALL_REGISTER;
                    if let Err((e, buf)) = self.i2c.write(buffer, 1) {
                        self.buffer.replace(buf);
                        self.op.set(Operation::None);

                        self.rainfall_client
                            .map(|client| client.callback(Err(e.into())));
                    }
                }
            },
            DeviceState::ContinueRainFall => match self.op.get() {
                Operation::None => (),
                Operation::RainFall => {
                    self.buffer.replace(buffer);
                    let delay = self.alarm.ticks_from_us(6400);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                }
            },
            DeviceState::FinalRainFall => match self.op.get() {
                Operation::None => (),
                Operation::RainFall => {
                    let rainfall = (buffer[0] as u32
                        | (buffer[1] as u32) << 8
                        | (buffer[2] as u32) << 16
                        | (buffer[3] as u32) << 24)
                        / 10;

                    self.state.set(DeviceState::Normal);
                    self.buffer.replace(buffer);
                    self.op.set(Operation::None);

                    self.rainfall_client
                        .map(|client| client.callback(Ok(rainfall as usize)));
                }
            },
            DeviceState::Normal | DeviceState::Broken => {}
        }
    }
}

impl<'a, A: Alarm<'a>, I: I2CDevice> AlarmClient for DFRobotRainFall<'a, A, I> {
    fn alarm(&self) {
        match self.op.get() {
            Operation::None => (),
            Operation::RainFall => {
                self.state.set(DeviceState::FinalRainFall);
                self.buffer.take().map(|buffer| {
                    if let Err((e, buf)) = self.i2c.read(buffer, 4) {
                        self.buffer.replace(buf);
                        self.op.set(Operation::None);

                        self.rainfall_client
                            .map(|client| client.callback(Err(e.into())));
                    }
                });
            }
        }
    }
}
