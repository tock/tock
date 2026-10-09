// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! LSM9DS1 Sensor
//!
//! Driver for the LSM9DS1 3D accelerometer, 3D gyroscope and 3D
//! magnetometer sensor. This chip is used on the Arduino Nano 33 BLE
//! Sense board.
//!
//! May be used with NineDof.
//!
//! I2C Interface
//!
//! The accelerometer/gyroscope and the magnetometer live behind two
//! different I2C addresses on the same bus.
//!
//! Datasheet: <https://www.st.com/resource/en/datasheet/lsm9ds1.pdf>
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&peripherals.i2c0, None)
//!     .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
//!
//! let lsm9ds1 = components::lsm9ds1::Lsm9ds1I2CComponent::new(
//!     mux_i2c,
//!     None,
//!     None,
//!     board_kernel,
//!     capsules_extra::lsm9ds1::DRIVER_NUM,
//! )
//! .finalize(components::lsm9ds1_i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let _ = lsm9ds1.configure(
//!     capsules_extra::lsm9ds1::Lsm9ds1GyroDataRate::Lsm9ds1GyroRate119Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1GyroRange::Lsm9ds1GyroRange245Dps,
//!     capsules_extra::lsm9ds1::Lsm9ds1AccelDataRate::Lsm9ds1AccelRate119Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1AccelRange::Lsm9ds1AccelRange2G,
//!     capsules_extra::lsm9ds1::Lsm9ds1MagDataRate::Lsm9ds1MagRate20Hz,
//!     capsules_extra::lsm9ds1::Lsm9ds1MagRange::Lsm9ds1MagRange4Gauss,
//! );
//! ```
//!
//! NineDof Example
//!
//! ```rust,ignore
//! let ninedof = components::ninedof::NineDofComponent::new(
//!     board_kernel,
//!     capsules_extra::ninedof::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! )
//! .finalize(components::ninedof_component_static!(lsm9ds1));
//! ```

#![allow(non_camel_case_types)]
use capsules_core::driver;

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::i2c;
use kernel::hil::sensors;
use kernel::hil::sensors::{NineDof, NineDofClient};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::register_bitfields;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Lsm9ds1 as usize;

/// WHO_AM_I value of the accelerometer/gyroscope sub-device.
pub const CHIP_ID: u8 = 0x68;
/// WHO_AM_I value of the magnetometer sub-device.
pub const MAGNETOMETER_CHIP_ID: u8 = 0x3D;

/// Default I2C address of the accelerometer/gyroscope sub-device (SDO_AG high).
pub const ACCEL_GYRO_BASE_ADDRESS: u8 = 0x6B;
/// Default I2C address of the magnetometer sub-device (SDO_M high).
pub const MAGNETOMETER_BASE_ADDRESS: u8 = 0x1E;

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1GyroDataRate {
        Lsm9ds1GyroRateShutdown = 0,
        Lsm9ds1GyroRate14_9Hz = 1,
        Lsm9ds1GyroRate59_5Hz = 2,
        Lsm9ds1GyroRate119Hz = 3,
        Lsm9ds1GyroRate238Hz = 4,
        Lsm9ds1GyroRate476Hz = 5,
        Lsm9ds1GyroRate952Hz = 6,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1GyroRange {
        Lsm9ds1GyroRange245Dps = 0,
        Lsm9ds1GyroRange500Dps = 1,
        Lsm9ds1GyroRange2000Dps = 3,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1AccelDataRate {
        Lsm9ds1AccelRateShutdown = 0,
        Lsm9ds1AccelRate10Hz = 1,
        Lsm9ds1AccelRate50Hz = 2,
        Lsm9ds1AccelRate119Hz = 3,
        Lsm9ds1AccelRate238Hz = 4,
        Lsm9ds1AccelRate476Hz = 5,
        Lsm9ds1AccelRate952Hz = 6,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1AccelRange {
        Lsm9ds1AccelRange2G = 0,
        Lsm9ds1AccelRange16G = 1,
        Lsm9ds1AccelRange4G = 2,
        Lsm9ds1AccelRange8G = 3,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1MagDataRate {
        Lsm9ds1MagRate0_625Hz = 0,
        Lsm9ds1MagRate1_25Hz = 1,
        Lsm9ds1MagRate2_5Hz = 2,
        Lsm9ds1MagRate5Hz = 3,
        Lsm9ds1MagRate10Hz = 4,
        Lsm9ds1MagRate20Hz = 5,
        Lsm9ds1MagRate40Hz = 6,
        Lsm9ds1MagRate80Hz = 7,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm9ds1MagRange {
        Lsm9ds1MagRange4Gauss = 0,
        Lsm9ds1MagRange8Gauss = 1,
        Lsm9ds1MagRange12Gauss = 2,
        Lsm9ds1MagRange16Gauss = 3,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum AgRegisters {
        WHO_AM_I = 0x0F,
        CTRL_REG1_G = 0x10,
        CTRL_REG6_XL = 0x20,
        CTRL_REG8 = 0x22,
        OUT_X_L_G = 0x18,
        OUT_X_L_XL = 0x28,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum MagRegisters {
        WHO_AM_I_M = 0x0F,
        CTRL_REG1_M = 0x20,
        CTRL_REG2_M = 0x21,
        CTRL_REG3_M = 0x22,
        OUT_X_L_M = 0x28,
    }
}

// Sensitivities, scaled so that `raw * FACTOR / DIVISOR` yields the
// scaled reading passed to the `NineDofClient`.
//
// Accelerometer: milli-g per LSB, indexed by `Lsm9ds1AccelRange`, scaled
// by 1000 (i.e. micro-g per LSB). Result is in milli-g.
pub const SCALE_FACTOR_ACCEL: [u16; 4] = [61, 732, 122, 244];
// Gyroscope: milli-dps per LSB, indexed by `Lsm9ds1GyroRange`, scaled by
// 100. Result is in milli-dps. Index 2 is unused (reserved range value).
pub const SCALE_FACTOR_GYRO: [u16; 4] = [875, 1750, 0, 7000];
// Magnetometer: milli-gauss per LSB, indexed by `Lsm9ds1MagRange`, scaled
// by 100. Result is in milli-gauss.
pub const SCALE_FACTOR_MAG: [u16; 4] = [14, 29, 43, 58];

register_bitfields![u8,
    pub (crate) CTRL_REG1_G [
        /// Gyroscope output data rate
        ODR_G OFFSET(5) NUMBITS(3) [],
        /// Gyroscope full-scale selection
        FS_G OFFSET(3) NUMBITS(2) [],
        /// Gyroscope bandwidth selection
        BW_G OFFSET(0) NUMBITS(2) [],
    ],
    pub (crate) CTRL_REG6_XL [
        /// Accelerometer output data rate
        ODR_XL OFFSET(5) NUMBITS(3) [],
        /// Accelerometer full-scale selection
        FS_XL OFFSET(3) NUMBITS(2) [],
        /// Bandwidth selection
        BW_SCAL_ODR OFFSET(2) NUMBITS(1) [],
        /// Anti-aliasing filter bandwidth selection
        BW_XL OFFSET(0) NUMBITS(2) [],
    ],
    pub (crate) CTRL_REG8 [
        /// Reboot memory content
        BOOT OFFSET(7) NUMBITS(1) [],
        /// Block data update
        BDU OFFSET(6) NUMBITS(1) [],
        /// Register address automatically incremented
        IF_ADD_INC OFFSET(2) NUMBITS(1) [],
        /// Software reset
        SW_RESET OFFSET(0) NUMBITS(1) [],
    ],
    pub (crate) CTRL_REG1_M [
        /// Temperature compensation
        TEMP_COMP OFFSET(7) NUMBITS(1) [],
        /// X and Y axes operative mode selection
        OM OFFSET(5) NUMBITS(2) [],
        /// Output data rate selection
        DO OFFSET(2) NUMBITS(3) [],
    ],
    pub (crate) CTRL_REG2_M [
        /// Full-scale selection
        FS OFFSET(5) NUMBITS(2) [],
    ],
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Idle,
    /// Check whether the accelerometer/gyroscope sub-device is present.
    /// Used both as a standalone presence check and as the first step
    /// of [`Lsm9ds1I2C::configure`].
    CheckAgPresent,
    /// Check whether the magnetometer sub-device is present. Only
    /// reached as part of the `configure` sequence.
    CheckMagPresent,
    /// Set `BDU`/`IF_ADD_INC` on the accelerometer/gyroscope. Only
    /// reached as part of the `configure` sequence.
    SetCtrl8,
    /// Write `CTRL_REG1_G` (gyroscope data rate and range).
    SetGyroConfig,
    /// Write `CTRL_REG6_XL` (accelerometer data rate and range).
    SetAccelConfig,
    /// Write `CTRL_REG1_M` (magnetometer data rate).
    SetMagDataRate,
    /// Write `CTRL_REG2_M` (magnetometer range).
    SetMagRange,
    /// Write `CTRL_REG3_M` (magnetometer continuous-conversion mode).
    /// Only reached as part of the `configure` sequence.
    SetMagMode,
    ReadAccelerationXYZ,
    ReadGyroscopeXYZ,
    ReadMagnetometerXYZ,
}

#[derive(Default)]
pub struct App {}

pub struct Lsm9ds1I2C<'a, I: i2c::I2CDevice<'a>> {
    i2c_accel_gyro: &'a I,
    i2c_magnetometer: &'a I,
    state: Cell<State>,
    config_in_progress: Cell<bool>,
    gyro_data_rate: Cell<Lsm9ds1GyroDataRate>,
    gyro_range: Cell<Lsm9ds1GyroRange>,
    accel_data_rate: Cell<Lsm9ds1AccelDataRate>,
    accel_range: Cell<Lsm9ds1AccelRange>,
    mag_data_rate: Cell<Lsm9ds1MagDataRate>,
    mag_range: Cell<Lsm9ds1MagRange>,
    is_present_ag: Cell<bool>,
    nine_dof_client: OptionalCell<&'a dyn sensors::NineDofClient>,
    buffer: TakeCell<'static, [u8]>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    syscall_process: OptionalCell<ProcessId>,
}

impl<'a, I: i2c::I2CDevice<'a>> Lsm9ds1I2C<'a, I> {
    pub fn new(
        i2c_accel_gyro: &'a I,
        i2c_magnetometer: &'a I,
        buffer: &'static mut [u8],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Lsm9ds1I2C<'a, I> {
        Lsm9ds1I2C {
            i2c_accel_gyro,
            i2c_magnetometer,
            state: Cell::new(State::Idle),
            config_in_progress: Cell::new(false),
            gyro_data_rate: Cell::new(Lsm9ds1GyroDataRate::Lsm9ds1GyroRate119Hz),
            gyro_range: Cell::new(Lsm9ds1GyroRange::Lsm9ds1GyroRange245Dps),
            accel_data_rate: Cell::new(Lsm9ds1AccelDataRate::Lsm9ds1AccelRate119Hz),
            accel_range: Cell::new(Lsm9ds1AccelRange::Lsm9ds1AccelRange2G),
            mag_data_rate: Cell::new(Lsm9ds1MagDataRate::Lsm9ds1MagRate20Hz),
            mag_range: Cell::new(Lsm9ds1MagRange::Lsm9ds1MagRange4Gauss),
            is_present_ag: Cell::new(false),
            nine_dof_client: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            apps: grant,
            syscall_process: OptionalCell::empty(),
        }
    }

    /// Run the full initialization sequence: verify both sub-devices are
    /// present, then configure the gyroscope, accelerometer and
    /// magnetometer.
    pub fn configure(
        &self,
        gyro_data_rate: Lsm9ds1GyroDataRate,
        gyro_range: Lsm9ds1GyroRange,
        accel_data_rate: Lsm9ds1AccelDataRate,
        accel_range: Lsm9ds1AccelRange,
        mag_data_rate: Lsm9ds1MagDataRate,
        mag_range: Lsm9ds1MagRange,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.gyro_data_rate.set(gyro_data_rate);
        self.gyro_range.set(gyro_range);
        self.accel_data_rate.set(accel_data_rate);
        self.accel_range.set(accel_range);
        self.mag_data_rate.set(mag_data_rate);
        self.mag_range.set(mag_range);

        self.config_in_progress.set(true);
        if let Err(error) = self.send_is_present() {
            self.config_in_progress.set(false);
            return Err(error);
        }
        Ok(())
    }

    fn send_is_present(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgRegisters::WHO_AM_I as u8;
            self.i2c_accel_gyro.enable();
            if let Err((error, buf)) = self.i2c_accel_gyro.write_read(buf, 1, 1) {
                self.i2c_accel_gyro.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state.set(State::CheckAgPresent);
                Ok(())
            }
        })
    }

    fn set_gyro_config(
        &self,
        data_rate: Lsm9ds1GyroDataRate,
        range: Lsm9ds1GyroRange,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgRegisters::CTRL_REG1_G as u8;
            let mut reg: LocalRegisterCopy<u8, CTRL_REG1_G::Register> = LocalRegisterCopy::new(0);
            reg.modify(CTRL_REG1_G::ODR_G.val(data_rate as u8));
            reg.modify(CTRL_REG1_G::FS_G.val(range as u8));
            buf[1] = reg.get();

            self.i2c_accel_gyro.enable();
            if let Err((error, buf)) = self.i2c_accel_gyro.write(buf, 2) {
                self.i2c_accel_gyro.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.gyro_data_rate.set(data_rate);
                self.gyro_range.set(range);
                self.state.set(State::SetGyroConfig);
                Ok(())
            }
        })
    }

    fn set_accel_config(
        &self,
        data_rate: Lsm9ds1AccelDataRate,
        range: Lsm9ds1AccelRange,
    ) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgRegisters::CTRL_REG6_XL as u8;
            let mut reg: LocalRegisterCopy<u8, CTRL_REG6_XL::Register> = LocalRegisterCopy::new(0);
            reg.modify(CTRL_REG6_XL::ODR_XL.val(data_rate as u8));
            reg.modify(CTRL_REG6_XL::FS_XL.val(range as u8));
            buf[1] = reg.get();

            self.i2c_accel_gyro.enable();
            if let Err((error, buf)) = self.i2c_accel_gyro.write(buf, 2) {
                self.i2c_accel_gyro.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.accel_data_rate.set(data_rate);
                self.accel_range.set(range);
                self.state.set(State::SetAccelConfig);
                Ok(())
            }
        })
    }

    fn set_mag_data_rate(&self, data_rate: Lsm9ds1MagDataRate) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = MagRegisters::CTRL_REG1_M as u8;
            let mut reg: LocalRegisterCopy<u8, CTRL_REG1_M::Register> = LocalRegisterCopy::new(0);
            reg.modify(CTRL_REG1_M::TEMP_COMP::SET);
            reg.modify(CTRL_REG1_M::OM.val(0b11));
            reg.modify(CTRL_REG1_M::DO.val(data_rate as u8));
            buf[1] = reg.get();

            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write(buf, 2) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.mag_data_rate.set(data_rate);
                self.state.set(State::SetMagDataRate);
                Ok(())
            }
        })
    }

    fn set_mag_range(&self, range: Lsm9ds1MagRange) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = MagRegisters::CTRL_REG2_M as u8;
            let mut reg: LocalRegisterCopy<u8, CTRL_REG2_M::Register> = LocalRegisterCopy::new(0);
            reg.modify(CTRL_REG2_M::FS.val(range as u8));
            buf[1] = reg.get();

            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write(buf, 2) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.mag_range.set(range);
                self.state.set(State::SetMagRange);
                Ok(())
            }
        })
    }

    fn read_acceleration_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgRegisters::OUT_X_L_XL as u8;
            self.i2c_accel_gyro.enable();
            if let Err((error, buf)) = self.i2c_accel_gyro.write_read(buf, 1, 6) {
                self.i2c_accel_gyro.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state.set(State::ReadAccelerationXYZ);
                Ok(())
            }
        })
    }

    fn read_gyroscope_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = AgRegisters::OUT_X_L_G as u8;
            self.i2c_accel_gyro.enable();
            if let Err((error, buf)) = self.i2c_accel_gyro.write_read(buf, 1, 6) {
                self.i2c_accel_gyro.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state.set(State::ReadGyroscopeXYZ);
                Ok(())
            }
        })
    }

    fn read_magnetometer_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
            buf[0] = MagRegisters::OUT_X_L_M as u8;
            self.i2c_magnetometer.enable();
            if let Err((error, buf)) = self.i2c_magnetometer.write_read(buf, 1, 6) {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buf);
                Err(error.into())
            } else {
                self.state.set(State::ReadMagnetometerXYZ);
                Ok(())
            }
        })
    }

    /// Schedule an upcall to whichever process issued the syscall command
    /// that is currently in flight, if any. Used for the steps that can be
    /// reached either as a standalone syscall command or as part of the
    /// `configure` sequence (in which case no process is waiting).
    fn schedule_upcall(&self, status: Result<(), i2c::Error>, arg1: usize) {
        self.syscall_process.take().map(|pid| {
            let _res = self.apps.enter(pid, |_app, upcalls| {
                let _ = upcalls
                    .schedule_upcall(0, (into_statuscode(status.map_err(|e| e.into())), arg1, 0));
            });
        });
    }
}

impl<'a, I: i2c::I2CDevice<'a>> i2c::I2CClient for Lsm9ds1I2C<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::Idle => {
                // Should never happen.
                self.buffer.replace(buffer);
            }

            State::CheckAgPresent => {
                let present = status == Ok(()) && buffer[0] == CHIP_ID;
                self.is_present_ag.set(present);
                self.i2c_accel_gyro.disable();

                if self.config_in_progress.get() {
                    if present {
                        buffer[0] = MagRegisters::WHO_AM_I_M as u8;
                        self.i2c_magnetometer.enable();
                        if let Err((_error, buf)) = self.i2c_magnetometer.write_read(buffer, 1, 1) {
                            self.i2c_magnetometer.disable();
                            self.buffer.replace(buf);
                            self.config_in_progress.set(false);
                            self.state.set(State::Idle);
                        } else {
                            self.state.set(State::CheckMagPresent);
                        }
                    } else {
                        self.buffer.replace(buffer);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    }
                } else {
                    self.buffer.replace(buffer);
                    self.state.set(State::Idle);
                    self.schedule_upcall(status, usize::from(present));
                }
            }

            State::CheckMagPresent => {
                let present = status == Ok(()) && buffer[0] == MAGNETOMETER_CHIP_ID;
                self.i2c_magnetometer.disable();

                if present {
                    buffer[0] = AgRegisters::CTRL_REG8 as u8;
                    buffer[1] = (CTRL_REG8::BDU::SET + CTRL_REG8::IF_ADD_INC::SET).value;
                    self.i2c_accel_gyro.enable();
                    if let Err((_error, buf)) = self.i2c_accel_gyro.write(buffer, 2) {
                        self.i2c_accel_gyro.disable();
                        self.buffer.replace(buf);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    } else {
                        self.state.set(State::SetCtrl8);
                    }
                } else {
                    self.buffer.replace(buffer);
                    self.config_in_progress.set(false);
                    self.state.set(State::Idle);
                }
            }

            State::SetCtrl8 => {
                buffer[0] = AgRegisters::CTRL_REG1_G as u8;
                let mut reg: LocalRegisterCopy<u8, CTRL_REG1_G::Register> =
                    LocalRegisterCopy::new(0);
                reg.modify(CTRL_REG1_G::ODR_G.val(self.gyro_data_rate.get() as u8));
                reg.modify(CTRL_REG1_G::FS_G.val(self.gyro_range.get() as u8));
                buffer[1] = reg.get();

                if let Err((_error, buf)) = self.i2c_accel_gyro.write(buffer, 2) {
                    self.i2c_accel_gyro.disable();
                    self.buffer.replace(buf);
                    self.config_in_progress.set(false);
                    self.state.set(State::Idle);
                } else {
                    self.state.set(State::SetGyroConfig);
                }
            }

            State::SetGyroConfig => {
                if self.config_in_progress.get() {
                    buffer[0] = AgRegisters::CTRL_REG6_XL as u8;
                    let mut reg: LocalRegisterCopy<u8, CTRL_REG6_XL::Register> =
                        LocalRegisterCopy::new(0);
                    reg.modify(CTRL_REG6_XL::ODR_XL.val(self.accel_data_rate.get() as u8));
                    reg.modify(CTRL_REG6_XL::FS_XL.val(self.accel_range.get() as u8));
                    buffer[1] = reg.get();

                    if let Err((_error, buf)) = self.i2c_accel_gyro.write(buffer, 2) {
                        self.i2c_accel_gyro.disable();
                        self.buffer.replace(buf);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    } else {
                        self.state.set(State::SetAccelConfig);
                    }
                } else {
                    self.i2c_accel_gyro.disable();
                    self.buffer.replace(buffer);
                    self.state.set(State::Idle);
                    self.schedule_upcall(status, usize::from(status == Ok(())));
                }
            }

            State::SetAccelConfig => {
                self.i2c_accel_gyro.disable();

                if self.config_in_progress.get() {
                    buffer[0] = MagRegisters::CTRL_REG1_M as u8;
                    let mut reg: LocalRegisterCopy<u8, CTRL_REG1_M::Register> =
                        LocalRegisterCopy::new(0);
                    reg.modify(CTRL_REG1_M::TEMP_COMP::SET);
                    reg.modify(CTRL_REG1_M::OM.val(0b11));
                    reg.modify(CTRL_REG1_M::DO.val(self.mag_data_rate.get() as u8));
                    buffer[1] = reg.get();

                    self.i2c_magnetometer.enable();
                    if let Err((_error, buf)) = self.i2c_magnetometer.write(buffer, 2) {
                        self.i2c_magnetometer.disable();
                        self.buffer.replace(buf);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    } else {
                        self.state.set(State::SetMagDataRate);
                    }
                } else {
                    self.buffer.replace(buffer);
                    self.state.set(State::Idle);
                    self.schedule_upcall(status, usize::from(status == Ok(())));
                }
            }

            State::SetMagDataRate => {
                if self.config_in_progress.get() {
                    buffer[0] = MagRegisters::CTRL_REG2_M as u8;
                    let mut reg: LocalRegisterCopy<u8, CTRL_REG2_M::Register> =
                        LocalRegisterCopy::new(0);
                    reg.modify(CTRL_REG2_M::FS.val(self.mag_range.get() as u8));
                    buffer[1] = reg.get();

                    if let Err((_error, buf)) = self.i2c_magnetometer.write(buffer, 2) {
                        self.i2c_magnetometer.disable();
                        self.buffer.replace(buf);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    } else {
                        self.state.set(State::SetMagRange);
                    }
                } else {
                    self.i2c_magnetometer.disable();
                    self.buffer.replace(buffer);
                    self.state.set(State::Idle);
                    self.schedule_upcall(status, usize::from(status == Ok(())));
                }
            }

            State::SetMagRange => {
                if self.config_in_progress.get() {
                    // Set the magnetometer to continuous-conversion mode.
                    buffer[0] = MagRegisters::CTRL_REG3_M as u8;
                    buffer[1] = 0;

                    if let Err((_error, buf)) = self.i2c_magnetometer.write(buffer, 2) {
                        self.i2c_magnetometer.disable();
                        self.buffer.replace(buf);
                        self.config_in_progress.set(false);
                        self.state.set(State::Idle);
                    } else {
                        self.state.set(State::SetMagMode);
                    }
                } else {
                    self.i2c_magnetometer.disable();
                    self.buffer.replace(buffer);
                    self.state.set(State::Idle);
                    self.schedule_upcall(status, usize::from(status == Ok(())));
                }
            }

            State::SetMagMode => {
                self.i2c_magnetometer.disable();
                self.buffer.replace(buffer);
                self.config_in_progress.set(false);
                self.state.set(State::Idle);
                self.schedule_upcall(status, usize::from(status == Ok(())));
            }

            State::ReadAccelerationXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;

                self.nine_dof_client.map(|nine_dof_client| {
                    if status == Ok(()) {
                        let range = self.accel_range.get() as usize;
                        x = ((((buffer[0] as u16 | ((buffer[1] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[range] as isize)
                            / 1000) as usize;
                        y = ((((buffer[2] as u16 | ((buffer[3] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[range] as isize)
                            / 1000) as usize;
                        z = ((((buffer[4] as u16 | ((buffer[5] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[range] as isize)
                            / 1000) as usize;
                        nine_dof_client.callback(x, y, z)
                    } else {
                        nine_dof_client.callback(0, 0, 0)
                    }
                });
                self.buffer.replace(buffer);
                self.i2c_accel_gyro.disable();
                self.state.set(State::Idle);
            }

            State::ReadGyroscopeXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;

                self.nine_dof_client.map(|nine_dof_client| {
                    if status == Ok(()) {
                        let range = self.gyro_range.get() as usize;
                        x = ((((buffer[0] as u16 | ((buffer[1] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_GYRO[range] as isize)
                            / 100) as usize;
                        y = ((((buffer[2] as u16 | ((buffer[3] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_GYRO[range] as isize)
                            / 100) as usize;
                        z = ((((buffer[4] as u16 | ((buffer[5] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_GYRO[range] as isize)
                            / 100) as usize;
                        nine_dof_client.callback(x, y, z)
                    } else {
                        nine_dof_client.callback(0, 0, 0)
                    }
                });
                self.buffer.replace(buffer);
                self.i2c_accel_gyro.disable();
                self.state.set(State::Idle);
            }

            State::ReadMagnetometerXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;

                self.nine_dof_client.map(|nine_dof_client| {
                    if status == Ok(()) {
                        let range = self.mag_range.get() as usize;
                        x = ((((buffer[0] as u16 | ((buffer[1] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_MAG[range] as isize)
                            / 100) as usize;
                        y = ((((buffer[2] as u16 | ((buffer[3] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_MAG[range] as isize)
                            / 100) as usize;
                        z = ((((buffer[4] as u16 | ((buffer[5] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_MAG[range] as isize)
                            / 100) as usize;
                        nine_dof_client.callback(x, y, z)
                    } else {
                        nine_dof_client.callback(0, 0, 0)
                    }
                });
                self.buffer.replace(buffer);
                self.i2c_magnetometer.disable();
                self.state.set(State::Idle);
            }
        }
    }
}

impl<'a, I: i2c::I2CDevice<'a>> SyscallDriver for Lsm9ds1I2C<'a, I> {
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally.
            return CommandReturn::success();
        }

        match command_num {
            // Check if the sensor is correctly connected.
            1 => match self.send_is_present() {
                Ok(()) => {
                    self.syscall_process.set(process_id);
                    CommandReturn::success()
                }
                Err(error) => CommandReturn::failure(error),
            },
            // Set Gyroscope Data Rate and Range.
            2 => {
                if let Some(data_rate) = Lsm9ds1GyroDataRate::from_usize(data1) {
                    if let Some(range) = Lsm9ds1GyroRange::from_usize(data2) {
                        match self.set_gyro_config(data_rate, range) {
                            Ok(()) => {
                                self.syscall_process.set(process_id);
                                CommandReturn::success()
                            }
                            Err(error) => CommandReturn::failure(error),
                        }
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }
            // Set Accelerometer Data Rate and Range.
            3 => {
                if let Some(data_rate) = Lsm9ds1AccelDataRate::from_usize(data1) {
                    if let Some(range) = Lsm9ds1AccelRange::from_usize(data2) {
                        match self.set_accel_config(data_rate, range) {
                            Ok(()) => {
                                self.syscall_process.set(process_id);
                                CommandReturn::success()
                            }
                            Err(error) => CommandReturn::failure(error),
                        }
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }
            // Set Magnetometer Data Rate.
            4 => {
                if let Some(data_rate) = Lsm9ds1MagDataRate::from_usize(data1) {
                    match self.set_mag_data_rate(data_rate) {
                        Ok(()) => {
                            self.syscall_process.set(process_id);
                            CommandReturn::success()
                        }
                        Err(error) => CommandReturn::failure(error),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }
            // Set Magnetometer Range.
            5 => {
                if let Some(range) = Lsm9ds1MagRange::from_usize(data1) {
                    match self.set_mag_range(range) {
                        Ok(()) => {
                            self.syscall_process.set(process_id);
                            CommandReturn::success()
                        }
                        Err(error) => CommandReturn::failure(error),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, I: i2c::I2CDevice<'a>> NineDof<'a> for Lsm9ds1I2C<'a, I> {
    fn set_client(&self, nine_dof_client: &'a dyn NineDofClient) {
        self.nine_dof_client.replace(nine_dof_client);
    }

    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        self.read_acceleration_xyz()
    }

    fn read_gyroscope(&self) -> Result<(), ErrorCode> {
        self.read_gyroscope_xyz()
    }

    fn read_magnetometer(&self) -> Result<(), ErrorCode> {
        self.read_magnetometer_xyz()
    }
}
