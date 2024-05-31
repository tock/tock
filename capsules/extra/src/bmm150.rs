// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! SyscallDriver for the Bosch BMM150 geomagnetic sensor.
//!
//! <https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmm150-ds001.pdf>
//!
//! > The BMM150 is a standalone geomagnetic sensor for consumer
//! > market applications. It allows measurements of the magnetic
//! > field in three perpendicular axes. Based on Bosch’s proprietary
//! > FlipCore technology, performance and features of BMM150 are
//! > carefully tuned and perfectly match the demanding requirements of
//! > all 3-axis mobile applications such as electronic compass, navigation
//! > or augmented reality.
//!
//! //! Driver Semantics
//! ----------------
//!
//! This driver exposes the BMM150's functionality via the [NineDof] and
//! [NineDofClient] HIL interfaces. If gyroscope or accelerometer data is
//! requested, the driver will return a ErrorCode.
//!
//! //! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let bmm150_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x10));
//! let bmm150 = static_init!(
//!     capsules::bmm150::BMM150<'static>,
//!     capsules::bmm150::BMM150::new(bmm150_i2c,
//!         &mut capsules::BMM150::BUFFER));
//! bmm150_i2c.set_client(bmm150);
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{NineDof, NineDofClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

#[allow(dead_code)]
enum Registers {
    ChipID = 0x40,
    DATAxLsb = 0x42,
    DATAxMsb = 0x43,
    DATAyLsb = 0x44,
    DATAyMsb = 0x45,
    DATAzLsb = 0x46,
    DATAzMsb = 0x47,
    RHALLlsb = 0x48,
    RHALLmsb = 0x49,
    INTST = 0x4A,
    CTRL1 = 0x4B,
    CTRL2 = 0x4C,
    CTRL3 = 0x4D,
    CTRL4 = 0x4E,
    LoThres = 0x4F,
    HiThres = 0x50,
    REPXY = 0x51,
    REPZ = 0x52,
}

pub struct BMM150<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    ninedof_client: OptionalCell<&'a dyn NineDofClient>,
    state: Cell<State>,
}

impl<'a, I: I2CDevice> BMM150<'a, I> {
    pub fn new(buffer: &'static mut [u8], i2c: &'a I) -> BMM150<'a, I> {
        BMM150 {
            buffer: TakeCell::new(buffer),
            i2c: i2c,
            ninedof_client: OptionalCell::empty(),
            state: Cell::new(State::Suspend),
        }
    }

    pub fn start_measurement(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c.enable();
                match self.state.get() {
                    State::Suspend => {
                        buffer[0] = Registers::CTRL1 as u8;
                        buffer[1] = 0x1_u8;

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 2) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::PowerOn);
                        }
                    }
                    State::Sleep => {
                        buffer[0] = Registers::CTRL2 as u8;
                        buffer[1] = 0x3A_u8;

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 2) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::InitializeReading);
                        }
                    }
                    _ => {}
                }
            })
            .ok_or(ErrorCode::FAIL)
    }
}

impl<'a, I: i2c::I2CDevice> NineDof<'a> for BMM150<'a, I> {
    fn set_client(&self, client: &'a dyn NineDofClient) {
        self.ninedof_client.set(client);
    }

    fn read_magnetometer(&self) -> Result<(), ErrorCode> {
        self.start_measurement()
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Suspend,
    Sleep,
    PowerOn,
    InitializeReading,
    ReadMeasurement,
    Read,
}

impl<'a, I: I2CDevice> I2CClient for BMM150<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.state.set(State::Sleep);
            self.buffer.replace(buffer);
            self.ninedof_client
                .map(|client| client.callback(i2c_err as usize, 0, 0));
            return;
        }

        match self.state.get() {
            State::PowerOn => {
                buffer[0] = Registers::CTRL2 as u8;
                buffer[1] = 0x3A_u8;

                if let Err((error, buffer)) = self.i2c.write(buffer, 2) {
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                    self.ninedof_client
                        .map(|client| client.callback(error as usize, 0, 0));
                } else {
                    self.state.set(State::InitializeReading);
                }
            }
            State::InitializeReading => {
                buffer[0] = Registers::DATAxLsb as u8;

                if let Err((i2c_err, buffer)) = self.i2c.write(buffer, 1) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::ReadMeasurement);
                }
            }
            State::ReadMeasurement => {
                if let Err((i2c_err, buffer)) = self.i2c.read(buffer, 8) {
                    self.state.set(State::Sleep);
                    self.buffer.replace(buffer);
                    self.ninedof_client
                        .map(|client| client.callback(i2c_err as usize, 0, 0));
                } else {
                    self.state.set(State::Read);
                }
            }
            State::Read => {
                let x_axis = ((buffer[1] as i16) << 5) | ((buffer[0] as i16) >> 3);
                let y_axis = ((buffer[3] as i16) << 5) | ((buffer[2] as i16) >> 3);
                let z_axis = ((buffer[5] as i16) << 7) | ((buffer[4] as i16) >> 1);

                self.state.set(State::Sleep);
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.ninedof_client.map(|client| {
                    client.callback(x_axis as usize, y_axis as usize, z_axis as usize)
                });
            }
            State::Sleep => {}   // should never happen
            State::Suspend => {} // should never happen
        }
    }
}
