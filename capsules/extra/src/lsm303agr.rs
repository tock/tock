// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for the LSM303AGR 3D accelerometer and 3D magnetometer sensor.
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
//! ```rust,ignore
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
//!    lsm303dlhc::Lsm303MagnetoDataRate::DataRate3_0Hz,
//!    lsm303dlhc::Lsm303Range::Range4_7G,
//!);
//! ```
//!
//! NideDof Example
//!
//! ```rust,ignore
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
//! ```rust,ignore
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_temp = board_kernel.create_grant(&grant_cap);
//!
//! lsm303dlhc.configure(
//!    lsm303dlhc::Lsm303AccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303dlhc::Lsm303Scale::Scale2G,
//!    false,
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

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::i2c;
use kernel::hil::sensors;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

use crate::lsm303xx::{
    AccelerometerRegisters, CTRL_REG1, CTRL_REG4, Lsm303AccelDataRate, Lsm303MagnetoDataRate,
    Lsm303Range, Lsm303Scale, RANGE_FACTOR_X_Y, RANGE_FACTOR_Z, SCALE_FACTOR,
};
use capsules_core::driver;

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

#[derive(Clone, Copy, PartialEq, Debug)]
enum ConfigurationState {
    SetPowerMode,
    SetScaleAndResolution,
    SetDataRate,
    SetRange,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum MeasurementState {
    ReadTemperature,
    ReadAccelerationXYZ,
    ReadMagnetometerXYZ,
}

/// Operating state of the sensor.
#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    /// Waiting for a command.
    Idle,
    /// Check if the chip is present.
    IsPresent,
    /// Configure the sensor.
    Configure {
        /// Which particular configuration operation.
        state: ConfigurationState,
        /// Whether we are doing the full initialization sequence.
        initialize: bool,
        /// If a measurement request happens during initialization,
        /// enqueue it.
        queued: Option<MeasurementState>,
    },
    /// Take a sensor measurement.
    Measure(MeasurementState),
}

#[derive(Clone, Copy)]
struct Settings {
    accel_data_rate: Lsm303AccelDataRate,
    low_power: bool,
    accel_scale: Lsm303Scale,
    accel_high_resolution: bool,
    mag_data_rate: Lsm303MagnetoDataRate,
    mag_range: Lsm303Range,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            accel_data_rate: Lsm303AccelDataRate::DataRate1Hz,
            low_power: false,
            accel_scale: Lsm303Scale::Scale2G,
            accel_high_resolution: false,
            mag_data_rate: Lsm303MagnetoDataRate::DataRate0_75Hz,
            mag_range: Lsm303Range::Range1G,
        }
    }
}

#[derive(Default)]
pub struct App {}

pub struct Lsm303agrI2C<'a, I: i2c::I2CDevice> {
    i2c_accelerometer: &'a I,
    i2c_magnetometer: &'a I,

    state: Cell<State>,

    settings: Cell<Settings>,

    buffer: TakeCell<'static, [u8]>,

    nine_dof_client: OptionalCell<&'a dyn sensors::NineDofClient>,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,

    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    owning_process: OptionalCell<ProcessId>,
}

impl<'a, I: i2c::I2CDevice> Lsm303agrI2C<'a, I> {
    pub fn new(
        i2c_accelerometer: &'a I,
        i2c_magnetometer: &'a I,
        buffer: &'static mut [u8],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Self {
        Self {
            i2c_accelerometer,
            i2c_magnetometer,
            state: Cell::new(State::Idle),
            settings: Cell::new(Settings::default()),
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
        mag_data_rate: Lsm303MagnetoDataRate,
        mag_range: Lsm303Range,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.settings.set(Settings {
            accel_data_rate,
            low_power,
            accel_scale,
            accel_high_resolution,
            mag_data_rate,
            mag_range,
        });

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            let tx_len = self.load_power_mode(accel_data_rate, low_power, buf);

            self.i2c_accelerometer.enable();
            if let Err((error, buf)) = self.i2c_accelerometer.write(buf, tx_len) {
                self.i2c_accelerometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state.set(State::Configure {
                    state: ConfigurationState::SetPowerMode,
                    initialize: true,
                    queued: None,
                });
                Ok(())
            }
        })
    }

    fn is_present(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            // turn on i2c to send commands
            buf[0] = 0x0F;
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write_read(buf, 1, 1) {
                self.buffer.replace(buf);
                self.i2c_magnetometer.disable();
                Err(error.into())
            } else {
                self.state.set(State::IsPresent);
                Ok(())
            }
        })
    }

    fn set_power_mode(
        &self,
        data_rate: Lsm303AccelDataRate,
        low_power: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            let tx_len = self.load_power_mode(data_rate, low_power, buf);
            self.i2c_accelerometer.enable();
            if let Err((error, buf)) = self.i2c_accelerometer.write(buf, tx_len) {
                self.i2c_accelerometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                let mut settings = self.settings.get();
                settings.accel_data_rate = data_rate;
                settings.low_power = low_power;
                self.settings.set(settings);

                self.state.set(State::Configure {
                    state: ConfigurationState::SetPowerMode,
                    initialize: false,
                    queued: None,
                });
                Ok(())
            }
        })
    }

    fn load_power_mode(
        &self,
        data_rate: Lsm303AccelDataRate,
        low_power: bool,
        buffer: &mut [u8],
    ) -> usize {
        buffer[0] = AccelerometerRegisters::CTRL_REG1 as u8;
        buffer[1] = (CTRL_REG1::ODR.val(data_rate as u8)
            + CTRL_REG1::LPEN.val(low_power as u8)
            + CTRL_REG1::ZEN::SET
            + CTRL_REG1::YEN::SET
            + CTRL_REG1::XEN::SET)
            .value;
        2
    }

    fn set_scale_and_resolution(
        &self,
        scale: Lsm303Scale,
        high_resolution: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            let tx_len = self.load_scale_and_resolution(scale, high_resolution, buf);
            self.i2c_accelerometer.enable();
            if let Err((error, buf)) = self.i2c_accelerometer.write(buf, tx_len) {
                self.i2c_accelerometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                let mut settings = self.settings.get();
                settings.accel_scale = scale;
                settings.accel_high_resolution = high_resolution;
                self.settings.set(settings);

                self.state.set(State::Configure {
                    state: ConfigurationState::SetScaleAndResolution,
                    initialize: false,
                    queued: None,
                });
                Ok(())
            }
        })
    }

    fn load_scale_and_resolution(
        &self,
        scale: Lsm303Scale,
        high_resolution: bool,
        buffer: &mut [u8],
    ) -> usize {
        buffer[0] = AccelerometerRegisters::CTRL_REG4 as u8;
        buffer[1] = (CTRL_REG4::FS.val(scale as u8)
            + CTRL_REG4::HR.val(high_resolution as u8)
            + CTRL_REG4::BDU::SET)
            .value;
        2
    }

    fn set_magneto_data_rate(&self, data_rate: Lsm303MagnetoDataRate) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            let tx_len = self.load_magneto_data_rate(data_rate, buf);
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write(buf, tx_len) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                let mut settings = self.settings.get();
                settings.mag_data_rate = data_rate;
                self.settings.set(settings);

                self.state.set(State::Configure {
                    state: ConfigurationState::SetDataRate,
                    initialize: false,
                    queued: None,
                });
                Ok(())
            }
        })
    }

    fn load_magneto_data_rate(&self, data_rate: Lsm303MagnetoDataRate, buffer: &mut [u8]) -> usize {
        buffer[0] = MagnetometerRegisters::CRA_REG_M as u8;
        buffer[1] = ((data_rate as u8) << 2) | 1 << 7;
        2
    }

    fn set_range(&self, range: Lsm303Range) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            let tx_len = self.load_range(range, buf);
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write(buf, tx_len) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                let mut settings = self.settings.get();
                settings.mag_range = range;
                self.settings.set(settings);

                self.state.set(State::Configure {
                    state: ConfigurationState::SetRange,
                    initialize: false,
                    queued: None,
                });
                Ok(())
            }
        })
    }

    fn load_range(&self, range: Lsm303Range, buffer: &mut [u8]) -> usize {
        buffer[0] = MagnetometerRegisters::CRB_REG_M as u8;
        buffer[1] = (range as u8) << 5;
        buffer[2] = 0;
        3
    }

    fn read_acceleration_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            if let State::Configure {
                state,
                initialize,
                queued: None,
            } = self.state.get()
            {
                // Queue this request until after configuration finishes.
                self.state.set(State::Configure {
                    state,
                    initialize,
                    queued: Some(MeasurementState::ReadAccelerationXYZ),
                });
                return Ok(());
            } else {
                return Err(ErrorCode::BUSY);
            }
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AccelerometerRegisters::OUT_X_L_A as u8 | REGISTER_AUTO_INCREMENT;
            self.i2c_accelerometer.enable();
            if let Err((error, buf)) = self.i2c_accelerometer.write_read(buf, 1, 6) {
                self.buffer.replace(buf);
                self.i2c_accelerometer.disable();
                Err(error.into())
            } else {
                self.state
                    .set(State::Measure(MeasurementState::ReadAccelerationXYZ));
                Ok(())
            }
        })
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            if let State::Configure {
                state,
                initialize,
                queued: None,
            } = self.state.get()
            {
                // Queue this request until after configuration finishes.
                self.state.set(State::Configure {
                    state,
                    initialize,
                    queued: Some(MeasurementState::ReadTemperature),
                });
                return Ok(());
            } else {
                return Err(ErrorCode::BUSY);
            }
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgrAccelerometerRegisters::TEMP_OUT_H_A as u8;
            self.i2c_accelerometer.enable();
            if let Err((error, buf)) = self.i2c_accelerometer.write_read(buf, 1, 2) {
                self.i2c_accelerometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state
                    .set(State::Measure(MeasurementState::ReadTemperature));
                Ok(())
            }
        })
    }

    fn read_magnetometer_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            if let State::Configure {
                state,
                initialize,
                queued: None,
            } = self.state.get()
            {
                // Queue this request until after configuration finishes.
                self.state.set(State::Configure {
                    state,
                    initialize,
                    queued: Some(MeasurementState::ReadMagnetometerXYZ),
                });
                return Ok(());
            } else {
                return Err(ErrorCode::BUSY);
            }
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = MagnetometerRegisters::OUT_X_H_M as u8;
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write_read(buf, 1, 6) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state
                    .set(State::Measure(MeasurementState::ReadMagnetometerXYZ));
                Ok(())
            }
        })
    }
}

impl<I: i2c::I2CDevice> i2c::I2CClient for Lsm303agrI2C<'_, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::IsPresent => {
                let present = status.is_ok() && buffer[0] == 60;
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        let _ = upcalls.schedule_upcall(0, (usize::from(present), 0, 0));
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
            State::Configure {
                state,
                initialize,
                queued,
            } => match state {
                ConfigurationState::SetPowerMode => {
                    let set_power = status == Ok(());

                    self.owning_process.map(|pid| {
                        let _res = self.apps.enter(pid, |_app, upcalls| {
                            let _ = upcalls.schedule_upcall(0, (usize::from(set_power), 0, 0));
                        });
                    });

                    if initialize {
                        // Next step in initialization is setting the accelerometer scale
                        // and resolution.
                        let settings = self.settings.get();

                        let tx_len = self.load_scale_and_resolution(
                            settings.accel_scale,
                            settings.accel_high_resolution,
                            buffer,
                        );

                        if let Err((_error, buffer)) = self.i2c_accelerometer.write(buffer, tx_len)
                        {
                            self.state.set(State::Idle);
                            self.i2c_accelerometer.disable();
                            self.buffer.replace(buffer);
                        } else {
                            self.state.set(State::Configure {
                                state: ConfigurationState::SetScaleAndResolution,
                                initialize,
                                queued,
                            });
                        }
                    } else {
                        self.state.set(State::Idle);
                        self.i2c_accelerometer.disable();
                        self.buffer.replace(buffer);
                    }
                }

                ConfigurationState::SetScaleAndResolution => {
                    let set_scale_and_resolution = status == Ok(());

                    self.owning_process.map(|pid| {
                        let _res = self.apps.enter(pid, |_app, upcalls| {
                            let _ = upcalls
                                .schedule_upcall(0, (usize::from(set_scale_and_resolution), 0, 0));
                        });
                    });

                    self.i2c_accelerometer.disable();

                    if initialize {
                        // Next step in initialization is setting the magnetometer data
                        // rate.
                        let settings = self.settings.get();

                        let tx_len = self.load_magneto_data_rate(settings.mag_data_rate, buffer);

                        self.i2c_magnetometer.enable();
                        if let Err((_error, buffer)) = self.i2c_magnetometer.write(buffer, tx_len) {
                            self.state.set(State::Idle);
                            self.i2c_magnetometer.disable();
                            self.buffer.replace(buffer);
                        } else {
                            self.state.set(State::Configure {
                                state: ConfigurationState::SetDataRate,
                                initialize,
                                queued,
                            });
                        }
                    } else {
                        self.state.set(State::Idle);
                        self.buffer.replace(buffer);
                    }
                }

                ConfigurationState::SetDataRate => {
                    let set_magneto_data_rate = status == Ok(());

                    self.owning_process.map(|pid| {
                        let _res = self.apps.enter(pid, |_app, upcalls| {
                            let _ = upcalls
                                .schedule_upcall(0, (usize::from(set_magneto_data_rate), 0, 0));
                        });
                    });

                    if initialize {
                        // Next step in initialization is setting the magnetometer range.
                        let settings = self.settings.get();

                        let tx_len = self.load_range(settings.mag_range, buffer);

                        if let Err((_error, buf)) = self.i2c_magnetometer.write(buffer, tx_len) {
                            self.state.set(State::Idle);
                            self.i2c_magnetometer.disable();
                            self.buffer.replace(buf);
                        } else {
                            self.state.set(State::Configure {
                                state: ConfigurationState::SetRange,
                                initialize,
                                queued,
                            });
                        }
                    } else {
                        self.i2c_magnetometer.disable();
                        self.state.set(State::Idle);
                        self.buffer.replace(buffer);
                    }
                }

                ConfigurationState::SetRange => {
                    let set_range = status == Ok(());

                    self.owning_process.map(|pid| {
                        let _res = self.apps.enter(pid, |_app, upcalls| {
                            let _ = upcalls.schedule_upcall(0, (usize::from(set_range), 0, 0));
                        });
                    });

                    // Initialization is done.

                    self.buffer.replace(buffer);
                    self.i2c_magnetometer.disable();
                    self.state.set(State::Idle);

                    // Check if there was a queued measurement, and if so, take the
                    // measurement now.
                    if let Some(queued_measurement) = queued {
                        let _ = match queued_measurement {
                            MeasurementState::ReadTemperature => self.read_temperature(),
                            MeasurementState::ReadAccelerationXYZ => self.read_acceleration_xyz(),
                            MeasurementState::ReadMagnetometerXYZ => self.read_magnetometer_xyz(),
                        };
                    }
                }
            },
            State::Measure(MeasurementState::ReadAccelerationXYZ) => {
                let (x, y, z, sx, sy, sz) = if status == Ok(()) {
                    // compute using only integers
                    let scale_factor = self.settings.get().accel_scale as usize;
                    let sx = (((buffer[0] as i16 | ((buffer[1] as i16) << 8)) as i32)
                        * (SCALE_FACTOR[scale_factor] as i32)
                        * 1000
                        / 32768) as usize;
                    let sy = (((buffer[2] as i16 | ((buffer[3] as i16) << 8)) as i32)
                        * (SCALE_FACTOR[scale_factor] as i32)
                        * 1000
                        / 32768) as usize;
                    let sz = (((buffer[4] as i16 | ((buffer[5] as i16) << 8)) as i32)
                        * (SCALE_FACTOR[scale_factor] as i32)
                        * 1000
                        / 32768) as usize;

                    let x = (buffer[0] as i16 | ((buffer[1] as i16) << 8)) as usize;
                    let y = (buffer[2] as i16 | ((buffer[3] as i16) << 8)) as usize;
                    let z = (buffer[4] as i16 | ((buffer[5] as i16) << 8)) as usize;
                    (x, y, z, sx, sy, sz)
                } else {
                    (0, 0, 0, 0, 0, 0)
                };
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        let _ = upcalls.schedule_upcall(0, (x, y, z));
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);

                self.nine_dof_client.map(|client| {
                    client.callback(sx, sy, sz);
                });
            }

            State::Measure(MeasurementState::ReadTemperature) => {
                let values = match status {
                    Ok(()) => Ok((buffer[1] as u16 as i16 | ((buffer[0] as i16) << 8)) as i32 / 8),
                    Err(i2c_err) => Err(i2c_err.into()),
                };
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        if let Ok(temp) = values {
                            let _ = upcalls.schedule_upcall(0, (temp as usize, 0, 0));
                        } else {
                            let _ = upcalls.schedule_upcall(0, (0, 0, 0));
                        }
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_accelerometer.disable();
                self.state.set(State::Idle);

                self.temperature_client.map(|client| {
                    client.callback(values);
                });
            }
            State::Measure(MeasurementState::ReadMagnetometerXYZ) => {
                let (x, y, z, sx, sy, sz) = if status == Ok(()) {
                    // compute using only integers
                    let range = self.settings.get().mag_range as usize;
                    let sx = (((buffer[1] as i16 | ((buffer[0] as i16) << 8)) as i32) * 100
                        / RANGE_FACTOR_X_Y[range] as i32) as usize;
                    let sz = (((buffer[3] as i16 | ((buffer[2] as i16) << 8)) as i32) * 100
                        / RANGE_FACTOR_X_Y[range] as i32) as usize;
                    let sy = (((buffer[5] as i16 | ((buffer[4] as i16) << 8)) as i32) * 100
                        / RANGE_FACTOR_Z[range] as i32) as usize;

                    let x = ((buffer[1] as u16 | ((buffer[0] as u16) << 8)) as i16) as usize;
                    let z = ((buffer[3] as u16 | ((buffer[2] as u16) << 8)) as i16) as usize;
                    let y = ((buffer[5] as u16 | ((buffer[4] as u16) << 8)) as i16) as usize;
                    (x, y, z, sx, sy, sz)
                } else {
                    (0, 0, 0, 0, 0, 0)
                };
                self.owning_process.map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        let _ = upcalls.schedule_upcall(0, (x, y, z));
                    });
                });
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);

                self.nine_dof_client.map(|client| {
                    client.callback(sx, sy, sz);
                });
            }
            _ => {
                self.i2c_magnetometer.disable();
                self.i2c_accelerometer.disable();
                self.buffer.replace(buffer);
            }
        }
    }
}

impl<I: i2c::I2CDevice> SyscallDriver for Lsm303agrI2C<'_, I> {
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
                .enter(current_process, |_, _| current_process == process_id)
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
                        match self.set_power_mode(data_rate, data2 != 0) {
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
                        match self.set_scale_and_resolution(scale, data2 != 0) {
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

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, I: i2c::I2CDevice> sensors::NineDof<'a> for Lsm303agrI2C<'a, I> {
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

impl<'a, I: i2c::I2CDevice> sensors::TemperatureDriver<'a> for Lsm303agrI2C<'a, I> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}
