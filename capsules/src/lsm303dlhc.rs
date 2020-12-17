//! Driver for the LSM303DLHC 3D accelerometer and 3D magnetometer sensor.
//!
//! May be used with NineDof and Temperature
//!
//! I2C Interface
//!
//! <https://www.st.com/en/mems-and-sensors/lsm303dlhc.html>
//!
//! The syscall interface is described in [lsm303dlhc.md](https://github.com/tock/tock/tree/master/doc/syscalls/70006_lsm303dlhc.md)
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f3xx::i2c::I2C1)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
//!    .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));
//!
//! lsm303dlhc.configure(
//!    lsm303dlhc::Lsm303AccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303dlhc::Lsm303Scale::Scale2G,
//!    false,
//!    true,
//!    lsm303dlhc::Lsm303MagnetoDataRate::DataRate3_0Hz,
//!    lsm303dlhc::Lsm303Range::Range4_7G,
//!);
//! ```
//!
//! NideDof Example
//!
//! ```rust
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_ninedof = board_kernel.create_grant(&grant_cap);
//!
//! // use as primary NineDof Sensor
//! let ninedof = static_init!(
//!    capsules::ninedof::NineDof<'static>,
//!    capsules::ninedof::NineDof::new(lsm303dlhc, grant_ninedof)
//! );
//!
//! hil::sensors::NineDof::set_client(lsm303dlhc, ninedof);
//!
//! // use as secondary NineDof Sensor
//! let lsm303dlhc_secondary = static_init!(
//!    capsules::ninedof::NineDofNode<'static, &'static dyn hil::sensors::NineDof>,
//!    capsules::ninedof::NineDofNode::new(lsm303dlhc)
//! );
//! ninedof.add_secondary_driver(lsm303dlhc_secondary);
//! hil::sensors::NineDof::set_client(lsm303dlhc, ninedof);
//! ```
//!
//! Temperature Example
//!
//! ```rust
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_temp = board_kernel.create_grant(&grant_cap);
//!
//! lsm303dlhc.configure(
//!    lsm303dlhc::Lsm303AccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303dlhc::Lsm303Scale::Scale2G,
//!    false,
//!    true,
//!    lsm303dlhc::Lsm303MagnetoDataRate::DataRate3_0Hz,
//!    lsm303dlhc::Lsm303Range::Range4_7G,
//!);
//! let temp = static_init!(
//! capsules::temperature::TemperatureSensor<'static>,
//!     capsules::temperature::TemperatureSensor::new(lsm303dlhc, grant_temperature));
//! kernel::hil::sensors::TemperatureDriver::set_client(lsm303dlhc, temp);
//! ```
//!
//! Author: Alexandru Radovici <msg4alex@gmail.com>
//!

#![allow(non_camel_case_types)]

use core::cell::{Cell, RefCell};
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::i2c::{self, Error};
use kernel::hil::sensors;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, ReturnCode};

use crate::lsm303xx::{
    AccelerometerRegisters, Lsm303AccelDataRate, Lsm303MagnetoDataRate, Lsm303Range, Lsm303Scale,
    CTRL_REG1, CTRL_REG4, RANGE_FACTOR_X_Y, RANGE_FACTOR_Z, SCALE_FACTOR,
};

use crate::driver;

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Lsm303dlch as usize;

/// Register values
const REGISTER_AUTO_INCREMENT: u8 = 0x80;

enum_from_primitive! {
    enum MagnetometerRegisters {
        CRA_REG_M = 0x00,
        CRB_REG_M = 0x01,
        OUT_X_H_M = 0x03,
        OUT_X_L_M = 0x04,
        OUT_Z_H_M = 0x05,
        OUT_Z_L_M = 0x06,
        OUT_Y_H_M = 0x07,
        OUT_Y_L_M = 0x08,
        TEMP_OUT_H_M = 0x31,
        TEMP_OUT_L_M = 0x32,
    }
}

// Experimental
const TEMP_OFFSET: i8 = 17;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    IsPresent,
    SetPowerMode,
    SetScaleAndResolution,
    ReadAccelerationXYZ,
    SetTemperatureDataRate,
    SetRange,
    ReadTemperature,
    ReadMagnetometerXYZ,
}

pub struct Lsm303dlhcI2C<'a> {
    config_in_progress: Cell<bool>,
    i2c_accelerometer: &'a dyn i2c::I2CDevice,
    i2c_magnetometer: &'a dyn i2c::I2CDevice,
    callback: RefCell<Callback>,
    state: Cell<State>,
    accel_scale: Cell<Lsm303Scale>,
    mag_range: Cell<Lsm303Range>,
    accel_high_resolution: Cell<bool>,
    mag_data_rate: Cell<Lsm303MagnetoDataRate>,
    accel_data_rate: Cell<Lsm303AccelDataRate>,
    low_power: Cell<bool>,
    temperature: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
    nine_dof_client: OptionalCell<&'a dyn sensors::NineDofClient>,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
}

impl<'a> Lsm303dlhcI2C<'a> {
    pub fn new(
        i2c_accelerometer: &'a dyn i2c::I2CDevice,
        i2c_magnetometer: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
    ) -> Lsm303dlhcI2C<'a> {
        // setup and return struct
        Lsm303dlhcI2C {
            config_in_progress: Cell::new(false),
            i2c_accelerometer: i2c_accelerometer,
            i2c_magnetometer: i2c_magnetometer,
            callback: RefCell::new(Callback::default()),
            state: Cell::new(State::Idle),
            accel_scale: Cell::new(Lsm303Scale::Scale2G),
            mag_range: Cell::new(Lsm303Range::Range1G),
            accel_high_resolution: Cell::new(false),
            mag_data_rate: Cell::new(Lsm303MagnetoDataRate::DataRate0_75Hz),
            accel_data_rate: Cell::new(Lsm303AccelDataRate::DataRate1Hz),
            low_power: Cell::new(false),
            temperature: Cell::new(false),
            buffer: TakeCell::new(buffer),
            nine_dof_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
        }
    }

    pub fn configure(
        &self,
        accel_data_rate: Lsm303AccelDataRate,
        low_power: bool,
        accel_scale: Lsm303Scale,
        accel_high_resolution: bool,
        temperature: bool,
        mag_data_rate: Lsm303MagnetoDataRate,
        mag_range: Lsm303Range,
    ) {
        if self.state.get() == State::Idle {
            self.config_in_progress.set(true);

            self.accel_scale.set(accel_scale);
            self.accel_high_resolution.set(accel_high_resolution);
            self.temperature.set(temperature);
            self.mag_data_rate.set(mag_data_rate);
            self.mag_range.set(mag_range);
            self.accel_data_rate.set(accel_data_rate);
            self.low_power.set(low_power);

            self.set_power_mode(accel_data_rate, low_power);
        }
    }

    fn is_present(&self) {
        self.state.set(State::IsPresent);
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            buf[0] = 0x0F;
            self.i2c_magnetometer.enable();
            self.i2c_magnetometer.write_read(buf, 1, 1);
        });
    }

    fn set_power_mode(&self, data_rate: Lsm303AccelDataRate, low_power: bool) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetPowerMode);
            self.buffer.take().map(|buf| {
                buf[0] = AccelerometerRegisters::CTRL_REG1 as u8;
                buf[1] = (CTRL_REG1::ODR.val(data_rate as u8)
                    + CTRL_REG1::LPEN.val(low_power as u8)
                    + CTRL_REG1::ZEN::SET
                    + CTRL_REG1::YEN::SET
                    + CTRL_REG1::XEN::SET)
                    .value;
                self.i2c_accelerometer.enable();
                self.i2c_accelerometer.write(buf, 2);
            });
        }
    }

    fn set_scale_and_resolution(&self, scale: Lsm303Scale, high_resolution: bool) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetScaleAndResolution);
            // TODO move these in completed
            self.accel_scale.set(scale);
            self.accel_high_resolution.set(high_resolution);
            self.buffer.take().map(|buf| {
                buf[0] = AccelerometerRegisters::CTRL_REG4 as u8;
                buf[1] = (CTRL_REG4::FS.val(scale as u8)
                    + CTRL_REG4::HR.val(high_resolution as u8))
                .value;
                self.i2c_accelerometer.enable();
                self.i2c_accelerometer.write(buf, 2);
            });
        }
    }

    fn read_acceleration_xyz(&self) {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadAccelerationXYZ);
            self.buffer.take().map(|buf| {
                buf[0] = AccelerometerRegisters::OUT_X_L_A as u8 | REGISTER_AUTO_INCREMENT;
                self.i2c_accelerometer.enable();
                self.i2c_accelerometer.write_read(buf, 1, 6);
            });
        }
    }

    fn set_temperature_and_magneto_data_rate(
        &self,
        temperature: bool,
        data_rate: Lsm303MagnetoDataRate,
    ) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetTemperatureDataRate);
            self.buffer.take().map(|buf| {
                buf[0] = MagnetometerRegisters::CRA_REG_M as u8;
                buf[1] = ((data_rate as u8) << 2) | if temperature { 1 << 7 } else { 0 };
                self.i2c_magnetometer.enable();
                self.i2c_magnetometer.write(buf, 2);
            });
        }
    }

    fn set_range(&self, range: Lsm303Range) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetRange);
            // TODO move these in completed
            self.mag_range.set(range);
            self.buffer.take().map(|buf| {
                buf[0] = MagnetometerRegisters::CRB_REG_M as u8;
                buf[1] = (range as u8) << 5;
                buf[2] = 0;
                self.i2c_magnetometer.enable();
                self.i2c_magnetometer.write(buf, 3);
            });
        }
    }

    fn read_temperature(&self) {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadTemperature);
            self.buffer.take().map(|buf| {
                buf[0] = MagnetometerRegisters::TEMP_OUT_H_M as u8;
                self.i2c_magnetometer.enable();
                self.i2c_magnetometer.write_read(buf, 1, 2);
            });
        }
    }

    fn read_magnetometer_xyz(&self) {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadMagnetometerXYZ);
            self.buffer.take().map(|buf| {
                buf[0] = MagnetometerRegisters::OUT_X_H_M as u8;
                self.i2c_magnetometer.enable();
                self.i2c_magnetometer.write_read(buf, 1, 6);
            });
        }
    }
}

impl i2c::I2CClient for Lsm303dlhcI2C<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], error: Error) {
        match self.state.get() {
            State::IsPresent => {
                let present = if error == Error::CommandComplete && buffer[0] == 60 {
                    true
                } else {
                    false
                };

                self.callback
                    .borrow_mut()
                    .schedule(if present { 1 } else { 0 }, 0, 0);
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::SetPowerMode => {
                let set_power = error == Error::CommandComplete;

                self.callback
                    .borrow_mut()
                    .schedule(if set_power { 1 } else { 0 }, 0, 0);
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    self.set_scale_and_resolution(
                        self.accel_scale.get(),
                        self.accel_high_resolution.get(),
                    );
                }
            }
            State::SetScaleAndResolution => {
                let set_scale_and_resolution = error == Error::CommandComplete;

                self.callback.borrow_mut().schedule(
                    if set_scale_and_resolution { 1 } else { 0 },
                    0,
                    0,
                );
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    self.set_temperature_and_magneto_data_rate(
                        self.temperature.get(),
                        self.mag_data_rate.get(),
                    );
                }
            }
            State::ReadAccelerationXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                let values = if error == Error::CommandComplete {
                    self.nine_dof_client.map(|client| {
                        // compute using only integers
                        let scale_factor = self.accel_scale.get() as usize;
                        x = (((buffer[0] as i16 | ((buffer[1] as i16) << 8)) as i32)
                            * (SCALE_FACTOR[scale_factor] as i32)
                            * 1000
                            / 32768) as usize;
                        y = (((buffer[2] as i16 | ((buffer[3] as i16) << 8)) as i32)
                            * (SCALE_FACTOR[scale_factor] as i32)
                            * 1000
                            / 32768) as usize;
                        z = (((buffer[4] as i16 | ((buffer[5] as i16) << 8)) as i32)
                            * (SCALE_FACTOR[scale_factor] as i32)
                            * 1000
                            / 32768) as usize;
                        client.callback(x, y, z);
                    });

                    x = (buffer[0] as i16 | ((buffer[1] as i16) << 8)) as usize;
                    y = (buffer[2] as i16 | ((buffer[3] as i16) << 8)) as usize;
                    z = (buffer[4] as i16 | ((buffer[5] as i16) << 8)) as usize;
                    true
                } else {
                    self.nine_dof_client.map(|client| {
                        client.callback(0, 0, 0);
                    });
                    false
                };
                if values {
                    self.callback.borrow_mut().schedule(x, y, z);
                } else {
                    self.callback.borrow_mut().schedule(0, 0, 0);
                }
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
            }
            State::SetTemperatureDataRate => {
                let set_temperature_and_magneto_data_rate = error == Error::CommandComplete;

                self.callback.borrow_mut().schedule(
                    if set_temperature_and_magneto_data_rate {
                        1
                    } else {
                        0
                    },
                    0,
                    0,
                );
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    self.set_range(self.mag_range.get());
                }
            }
            State::SetRange => {
                let set_range = error == Error::CommandComplete;

                self.callback
                    .borrow_mut()
                    .schedule(if set_range { 1 } else { 0 }, 0, 0);
                if self.config_in_progress.get() {
                    self.config_in_progress.set(false);
                }
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::ReadTemperature => {
                let mut temp: usize = 0;
                let values = if error == Error::CommandComplete {
                    temp = ((buffer[1] as i16 | ((buffer[0] as i16) << 8)) >> 4) as usize;
                    self.temperature_client.map(|client| {
                        client.callback((temp as i16 / 8 + TEMP_OFFSET as i16) as usize);
                    });
                    true
                } else {
                    self.temperature_client.map(|client| {
                        client.callback(usize::MAX);
                    });
                    false
                };
                if values {
                    self.callback.borrow_mut().schedule(temp, 0, 0);
                } else {
                    self.callback.borrow_mut().schedule(0, 0, 0);
                }
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::ReadMagnetometerXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                let values = if error == Error::CommandComplete {
                    self.nine_dof_client.map(|client| {
                        // compute using only integers
                        let range = self.mag_range.get() as usize;
                        x = (((buffer[1] as i16 | ((buffer[0] as i16) << 8)) as i32) * 100
                            / RANGE_FACTOR_X_Y[range] as i32) as usize;
                        z = (((buffer[3] as i16 | ((buffer[2] as i16) << 8)) as i32) * 100
                            / RANGE_FACTOR_X_Y[range] as i32) as usize;
                        y = (((buffer[5] as i16 | ((buffer[4] as i16) << 8)) as i32) * 100
                            / RANGE_FACTOR_Z[range] as i32) as usize;
                        client.callback(x, y, z);
                    });

                    x = ((buffer[1] as u16 | ((buffer[0] as u16) << 8)) as i16) as usize;
                    z = ((buffer[3] as u16 | ((buffer[2] as u16) << 8)) as i16) as usize;
                    y = ((buffer[5] as u16 | ((buffer[4] as u16) << 8)) as i16) as usize;
                    true
                } else {
                    self.nine_dof_client.map(|client| {
                        client.callback(0, 0, 0);
                    });
                    false
                };
                if values {
                    self.callback.borrow_mut().schedule(x, y, z);
                } else {
                    self.callback.borrow_mut().schedule(0, 0, 0);
                }
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            _ => {
                self.i2c_magnetometer.disable();
                self.i2c_accelerometer.disable();
                self.buffer.replace(buffer);
            }
        }
    }
}

impl Driver for Lsm303dlhcI2C<'_> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        _appid: AppId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            // Check is sensor is correctly connected
            1 => {
                if self.state.get() == State::Idle {
                    self.is_present();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Accelerometer Power Mode
            2 => {
                if self.state.get() == State::Idle {
                    if let Some(data_rate) = Lsm303AccelDataRate::from_usize(data1) {
                        self.set_power_mode(data_rate, if data2 != 0 { true } else { false });
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Accelerometer Scale And Resolution
            3 => {
                if self.state.get() == State::Idle {
                    if let Some(scale) = Lsm303Scale::from_usize(data1) {
                        self.set_scale_and_resolution(scale, if data2 != 0 { true } else { false });
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Magnetometer Temperature Enable and Data Rate
            4 => {
                if self.state.get() == State::Idle {
                    if let Some(data_rate) = Lsm303MagnetoDataRate::from_usize(data1) {
                        self.set_temperature_and_magneto_data_rate(
                            if data2 != 0 { true } else { false },
                            data_rate,
                        );
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Magnetometer Range
            5 => {
                if self.state.get() == State::Idle {
                    if let Some(range) = Lsm303Range::from_usize(data1) {
                        self.set_range(range);
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Read Acceleration XYZ
            6 => {
                if self.state.get() == State::Idle {
                    self.read_acceleration_xyz();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Read Temperature
            7 => {
                if self.state.get() == State::Idle {
                    self.read_temperature();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Read Mangetometer XYZ
            8 => {
                if self.state.get() == State::Idle {
                    self.read_magnetometer_xyz();
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Callback,
        _appid: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            0 /* set the one shot callback */ => {
                Ok(self.callback.replace(callback))
            },
            // default
            _ => Err ((callback, ErrorCode::NOSUPPORT)),
        }
    }
}

impl<'a> sensors::NineDof<'a> for Lsm303dlhcI2C<'a> {
    fn set_client(&self, nine_dof_client: &'a dyn sensors::NineDofClient) {
        self.nine_dof_client.replace(nine_dof_client);
    }

    fn read_accelerometer(&self) -> ReturnCode {
        if self.state.get() == State::Idle {
            self.read_acceleration_xyz();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn read_magnetometer(&self) -> ReturnCode {
        if self.state.get() == State::Idle {
            self.read_magnetometer_xyz();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }
}

impl<'a> sensors::TemperatureDriver<'a> for Lsm303dlhcI2C<'a> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> ReturnCode {
        if self.state.get() == State::Idle {
            self.read_temperature();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }
}
