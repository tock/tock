//! Driver for the LSM303AGR 3D accelerometer and 3D magnetometer sensor.
//!
//! May be used with NineDof and Temperature
//!
//! I2C Interface
//!
//! <https://www.st.com/en/mems-and-sensors/lsm303agr.html>
//!
//! The syscall interface is described in
//! [lsm303dlhc.md](https://github.com/tock/tock/tree/master/doc/syscalls/70006_lsm303dlhc.md)
//!
//! Usage
//! -----
//!
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f3xx::i2c::I2C1)
//!     .finalize(components::i2c_mux_component_helper!());
//!
//! let lsm303dlhc = components::lsm303dlhc::Lsm303agrI2CComponent::new()
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

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::i2c;
use kernel::hil::sensors;
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId};

use crate::driver;
use crate::lsm303xx::{
    AccelerometerRegisters, Lsm303AccelDataRate, Lsm303MagnetoDataRate, Lsm303Range, Lsm303Scale,
    CTRL_REG1, CTRL_REG4, RANGE_FACTOR_X_Y, RANGE_FACTOR_Z, SCALE_FACTOR,
};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Lsm303dlch as usize;

/// Register values
const REGISTER_AUTO_INCREMENT: u8 = 0x80;

enum_from_primitive! {
    pub enum AgrAccelerometerRegisters {
        TEMP_OUT_H_A = 0x0C,
        TEMP_OUT_L_A = 0x0D
    }
}

enum_from_primitive! {
    enum MagnetometerRegisters {
        CRA_REG_M = 0x60,
        CRB_REG_M = 0x61,
        OUT_X_H_M = 0x68,
        OUT_X_L_M = 0x69,
        OUT_Z_H_M = 0x6A,
        OUT_Z_L_M = 0x6B,
        OUT_Y_H_M = 0x6C,
        OUT_Y_L_M = 0x6D,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    IsPresent,
    SetPowerMode,
    SetScaleAndResolution,
    ReadAccelerationXYZ,
    SetDataRate,
    // SetTemperature,
    SetRange,
    ReadTemperature,
    ReadMagnetometerXYZ,
}

#[derive(Default)]
pub struct App {}

pub struct Lsm303agrI2C<'a> {
    config_in_progress: Cell<bool>,
    i2c_accelerometer: &'a dyn i2c::I2CDevice,
    i2c_magnetometer: &'a dyn i2c::I2CDevice,
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
    apps: Grant<App, 1>,
    owning_process: OptionalCell<ProcessId>,
}

impl<'a> Lsm303agrI2C<'a> {
    pub fn new(
        i2c_accelerometer: &'a dyn i2c::I2CDevice,
        i2c_magnetometer: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
        grant: Grant<App, 1>,
    ) -> Lsm303agrI2C<'a> {
        // setup and return struct
        Lsm303agrI2C {
            config_in_progress: Cell::new(false),
            i2c_accelerometer: i2c_accelerometer,
            i2c_magnetometer: i2c_magnetometer,
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
            apps: grant,
            owning_process: OptionalCell::empty(),
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
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.config_in_progress.set(true);

            self.accel_scale.set(accel_scale);
            self.accel_high_resolution.set(accel_high_resolution);
            self.temperature.set(temperature);
            self.mag_data_rate.set(mag_data_rate);
            self.mag_range.set(mag_range);
            self.accel_data_rate.set(accel_data_rate);
            self.low_power.set(low_power);

            self.set_power_mode(accel_data_rate, low_power)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn is_present(&self) -> Result<(), ErrorCode> {
        self.state.set(State::IsPresent);
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            // turn on i2c to send commands
            buf[0] = 0x0F;
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write_read(buf, 1, 1) {
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                Ok(())
            }
        })
    }

    fn set_power_mode(
        &self,
        data_rate: Lsm303AccelDataRate,
        low_power: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::SetPowerMode);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = AccelerometerRegisters::CTRL_REG1 as u8;
                buf[1] = (CTRL_REG1::ODR.val(data_rate as u8)
                    + CTRL_REG1::LPEN.val(low_power as u8)
                    + CTRL_REG1::ZEN::SET
                    + CTRL_REG1::YEN::SET
                    + CTRL_REG1::XEN::SET)
                    .value;
                self.i2c_accelerometer.enable();
                if let Err((error, buf)) = self.i2c_accelerometer.write(buf, 2) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_scale_and_resolution(
        &self,
        scale: Lsm303Scale,
        high_resolution: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::SetScaleAndResolution);
            // TODO move these in completed
            self.accel_scale.set(scale);
            self.accel_high_resolution.set(high_resolution);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = AccelerometerRegisters::CTRL_REG4 as u8;
                buf[1] = (CTRL_REG4::FS.val(scale as u8)
                    + CTRL_REG4::HR.val(high_resolution as u8)
                    + CTRL_REG4::BDU::SET)
                    .value;
                self.i2c_accelerometer.enable();
                if let Err((error, buf)) = self.i2c_accelerometer.write(buf, 2) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn read_acceleration_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadAccelerationXYZ);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = AccelerometerRegisters::OUT_X_L_A as u8 | REGISTER_AUTO_INCREMENT;
                self.i2c_accelerometer.enable();
                if let Err((error, buf)) = self.i2c_accelerometer.write_read(buf, 1, 6) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_magneto_data_rate(&self, data_rate: Lsm303MagnetoDataRate) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::SetDataRate);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = MagnetometerRegisters::CRA_REG_M as u8;
                buf[1] = ((data_rate as u8) << 2) | 1 << 7;
                self.i2c_magnetometer.enable();
                if let Err((error, buf)) = self.i2c_magnetometer.write(buf, 2) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_range(&self, range: Lsm303Range) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::SetRange);
            self.mag_range.set(range);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = MagnetometerRegisters::CRB_REG_M as u8;
                buf[1] = (range as u8) << 5;
                buf[2] = 0;
                self.i2c_magnetometer.enable();
                if let Err((error, buf)) = self.i2c_magnetometer.write(buf, 3) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadTemperature);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = AgrAccelerometerRegisters::TEMP_OUT_H_A as u8;
                self.i2c_accelerometer.enable();
                if let Err((error, buf)) = self.i2c_accelerometer.write_read(buf, 1, 2) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn read_magnetometer_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadMagnetometerXYZ);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = MagnetometerRegisters::OUT_X_H_M as u8;
                self.i2c_magnetometer.enable();
                if let Err((error, buf)) = self.i2c_magnetometer.write_read(buf, 1, 6) {
                    self.buffer.replace(buf);
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl i2c::I2CClient for Lsm303agrI2C<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::IsPresent => {
                let present = if status == Ok(()) && buffer[0] == 60 {
                    true
                } else {
                    false
                };
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, if present { 1 } else { 0 }, 0, 0)
                            .ok();
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::SetPowerMode => {
                let set_power = status == Ok(());
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, if set_power { 1 } else { 0 }, 0, 0)
                            .ok();
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    let _ = self.set_scale_and_resolution(
                        self.accel_scale.get(),
                        self.accel_high_resolution.get(),
                    );
                }
            }
            State::SetScaleAndResolution => {
                let set_scale_and_resolution = status == Ok(());
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, if set_scale_and_resolution { 1 } else { 0 }, 0, 0)
                            .ok();
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    let _ = self.set_magneto_data_rate(self.mag_data_rate.get());
                }
            }
            State::ReadAccelerationXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                let values = if status == Ok(()) {
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
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        if values {
                            upcalls.schedule_upcall(0, x, y, z).ok();
                        } else {
                            upcalls.schedule_upcall(0, 0, 0, 0).ok();
                        }
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
            }
            State::SetDataRate => {
                let set_magneto_data_rate = status == Ok(());
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, if set_magneto_data_rate { 1 } else { 0 }, 0, 0)
                            .ok();
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
                if self.config_in_progress.get() {
                    let _ = self.set_range(self.mag_range.get());
                }
            }
            State::SetRange => {
                let set_range = status == Ok(());
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, if set_range { 1 } else { 0 }, 0, 0)
                            .ok();
                    });
                });
                if self.config_in_progress.get() {
                    self.config_in_progress.set(false);
                }
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::ReadTemperature => {
                let mut temp: usize = 0;
                let values = if status == Ok(()) {
                    temp = (buffer[1] as u16 as i16 | ((buffer[0] as i16) << 8)) as usize;
                    self.temperature_client.map(|client| {
                        client.callback((temp as i16 / 8) as usize);
                    });
                    true
                } else {
                    self.temperature_client.map(|client| {
                        client.callback(usize::MAX);
                    });
                    false
                };
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        if values {
                            upcalls.schedule_upcall(0, temp, 0, 0).ok();
                        } else {
                            upcalls.schedule_upcall(0, 0, 0, 0).ok();
                        }
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);
            }
            State::ReadMagnetometerXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                let values = if status == Ok(()) {
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
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(*pid, |_app, upcalls| {
                        if values {
                            upcalls.schedule_upcall(0, x, y, z).ok();
                        } else {
                            upcalls.schedule_upcall(0, 0, 0, 0).ok();
                        }
                    });
                });
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

impl Driver for Lsm303agrI2C<'_> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally
            return CommandReturn::success();
        }

        let match_or_empty_or_nonexistant = self.owning_process.map_or(true, |current_process| {
            self.apps
                .enter(*current_process, |_, _| current_process == &process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.owning_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::RESERVE);
        }

        match command_num {
            // Check is sensor is correctly connected
            1 => {
                if self.state.get() == State::Idle {
                    match self.is_present() {
                        Ok(()) => CommandReturn::success(),
                        Err(error) => CommandReturn::failure(error),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Accelerometer Power Mode
            2 => {
                if self.state.get() == State::Idle {
                    if let Some(data_rate) = Lsm303AccelDataRate::from_usize(data1) {
                        match self.set_power_mode(data_rate, if data2 != 0 { true } else { false })
                        {
                            Ok(()) => CommandReturn::success(),
                            Err(error) => CommandReturn::failure(error),
                        }
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
                        match self
                            .set_scale_and_resolution(scale, if data2 != 0 { true } else { false })
                        {
                            Ok(()) => CommandReturn::success(),
                            Err(error) => CommandReturn::failure(error),
                        }
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
                        match self.set_magneto_data_rate(data_rate) {
                            Ok(()) => CommandReturn::success(),
                            Err(error) => CommandReturn::failure(error),
                        }
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
                        match self.set_range(range) {
                            Ok(()) => CommandReturn::success(),
                            Err(error) => CommandReturn::failure(error),
                        }
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a> sensors::NineDof<'a> for Lsm303agrI2C<'a> {
    fn set_client(&self, nine_dof_client: &'a dyn sensors::NineDofClient) {
        self.nine_dof_client.replace(nine_dof_client);
    }

    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        self.read_acceleration_xyz()
    }

    fn read_magnetometer(&self) -> Result<(), ErrorCode> {
        self.read_magnetometer_xyz()
    }
}

impl<'a> sensors::TemperatureDriver<'a> for Lsm303agrI2C<'a> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}
