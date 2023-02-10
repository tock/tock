//! LSM6DSOXTR Sensor
//!
//! Driver for the LSM6DSOXTR 3D accelerometer and 3D gyroscope sensor.
//!
//! May be used with NineDof and Temperature
//!
//! I2C Interface
//!
//! Datasheet: <https://www.digikey.sg/product-detail/en/stmicroelectronics/LSM6DSOXTR/497-18367-1-ND/9841887>
//!
//! Author: Cristiana Andrei <cristiana.andrei05@gmail.com>

#![allow(non_camel_case_types)]
use core_capsules::driver;

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
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = driver::NUM::Lsm6dsoxtr as usize;

use kernel::utilities::registers::register_bitfields;

pub const CHIP_ID: u8 = 0x6C;
pub const ACCELEROMETER_BASE_ADDRESS: u8 = 0x6A;

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXGyroDataRate {
        LSMDSOX_GYRO_RATE_SHUTDOWN = 0,
        LSM6DSOX_GYRO_RATE_12_5_HZ = 1,
        LSM6DSOX_GYRO_RATE_26_HZ = 2,
        LSM6DSOX_GYRO_RATE_52_HZ = 3,
        LSM6DSOX_GYRO_RATE_104_HZ = 4,
        LSM6DSOX_GYRO_RATE_208_HZ = 5,
        LSM6DSOX_GYRO_RATE_416_HZ = 6,
        LSM6DSOX_GYRO_RATE_833_HZ = 7,
        LSM6DSOX_GYRO_RATE_1_66k_HZ = 8,
        LSM6DSOX_GYRO_RATE_3_33K_HZ = 9,
        LSM6DSOX_GYRO_RATE_6_66K_HZ = 10
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXAccelDataRate {
        LSMDSOX_ACCEL_RATE_SHUTDOWN = 0,
        LSM6DSOX_ACCEL_RATE_12_5_HZ = 1,
        LSM6DSOX_ACCEL_RATE_26_HZ = 2,
        LSM6DSOX_ACCEL_RATE_52_HZ = 3,
        LSM6DSOX_ACCEL_RATE_104_HZ = 4,
        LSM6DSOX_ACCEL_RATE_208_HZ = 5,
        LSM6DSOX_ACCEL_RATE_416_HZ = 6,
        LSM6DSOX_ACCEL_RATE_833_HZ = 7,
        LSM6DSOX_ACCEL_RATE_1_66k_HZ = 8,
        LSM6DSOX_ACCEL_RATE_3_33K_HZ = 9,
        LSM6DSOX_ACCEL_RATE_6_66K_HZ = 10
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXAccelRange {
        LSM6DSOX_ACCEL_RANGE_2_G = 0,
        LSM6DSOX_ACCEL_RANGE_16_G = 1,
        LSM6DSOX_ACCEL_RANGE_4_G = 2,
        LSM6DSOX_ACCEL_RANGE_8_G = 3
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXTRGyroRange {
        LSM6DSOX_GYRO_RANGE_250_DPS = 0,
        LSM6DSOX_GYRO_RANGE_500_DPS = 1,
        LSM6DSOX_GYRO_RANGE_1000_DPS = 2,
        LSM6DSOX_GYRO_RANGE_2000_DPS = 3
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXTRGyroRegisters {
        CTRL2_G = 0x11,
        CTRL7_G = 0x16,
        OUT_X_L_G = 0x22,
        OUT_X_H_G = 0x23,
        OUT_Y_L_G = 0x24,
        OUT_Y_H_G = 0x25,
        OUT_Z_L_G = 0x26,
        OUT_Z_H_G = 0x27
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXTRTempRegisters {
        OUT_TEMP_L = 0x20,
        OUT_TEMP_H = 0x21
    }
}

pub const SCALE_FACTOR_ACCEL: [u16; 4] = [61, 488, 122, 244];
pub const SCALE_FACTOR_GYRO: [u16; 4] = [875, 1750, 3500, 7000];
pub const TEMP_SENSITIVITY_FACTOR: u16 = 256;

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum LSM6DSOXTRAccelRegisters {
        CTRL1_XL = 0x10,
        CTRL8_XL = 0x17,
        CTRL9_XL = 0x18,
        OUT_X_L_A = 0x28,
        OUT_X_H_A = 0x29,
        OUT_Y_L_A = 0x2A,
        OUT_Y_H_A = 0x2B,
        OUT_Z_L_A = 0x2C,
        OUT_Z_H_A = 0x2D
    }
}

register_bitfields![u8,
    pub (crate) CTRL1_XL [
        /// Output data rate
        ODR OFFSET(4) NUMBITS(4) [],

        FS OFFSET(2) NUMBITS(2) [],

        LPF OFFSET(1) NUMBITS(1) [],

    ],
];

register_bitfields![u8,
    pub (crate) CTRL2_G [
        /// Output data rate
        ODR OFFSET(4) NUMBITS(4) [],

        FS OFFSET(2) NUMBITS(2) [],

        LPF OFFSET(1) NUMBITS(1) [],

    ],
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Idle,
    IsPresent,
    ReadAccelerationXYZ,
    ReadGyroscopeXYZ,
    ReadTemperature,
    SetPowerModeAccel,
    SetPowerModeGyro,
}
#[derive(Default)]
pub struct App {}

pub struct Lsm6dsoxtrI2C<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    state: Cell<State>,
    config_in_progress: Cell<bool>,
    gyro_data_rate: Cell<LSM6DSOXGyroDataRate>,
    accel_data_rate: Cell<LSM6DSOXAccelDataRate>,
    accel_scale: Cell<LSM6DSOXAccelRange>,
    gyro_range: Cell<LSM6DSOXTRGyroRange>,
    low_power: Cell<bool>,
    temperature: Cell<bool>,
    nine_dof_client: OptionalCell<&'a dyn sensors::NineDofClient>,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    is_present: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    syscall_process: OptionalCell<ProcessId>,
}

impl<'a> Lsm6dsoxtrI2C<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Lsm6dsoxtrI2C<'a> {
        Lsm6dsoxtrI2C {
            i2c: i2c,
            state: Cell::new(State::Idle),
            config_in_progress: Cell::new(false),
            gyro_data_rate: Cell::new(LSM6DSOXGyroDataRate::LSM6DSOX_GYRO_RATE_12_5_HZ),
            accel_data_rate: Cell::new(LSM6DSOXAccelDataRate::LSM6DSOX_ACCEL_RATE_12_5_HZ),
            accel_scale: Cell::new(LSM6DSOXAccelRange::LSM6DSOX_ACCEL_RANGE_2_G),
            gyro_range: Cell::new(LSM6DSOXTRGyroRange::LSM6DSOX_GYRO_RANGE_250_DPS),
            low_power: Cell::new(false),
            temperature: Cell::new(false),
            nine_dof_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            is_present: Cell::new(false),
            buffer: TakeCell::new(buffer),
            apps: grant,
            syscall_process: OptionalCell::empty(),
        }
    }

    pub fn configure(
        &self,
        gyro_data_rate: LSM6DSOXGyroDataRate,
        accel_data_rate: LSM6DSOXAccelDataRate,
        accel_scale: LSM6DSOXAccelRange,
        gyro_range: LSM6DSOXTRGyroRange,
        low_power: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.gyro_data_rate.set(gyro_data_rate);
            self.accel_data_rate.set(accel_data_rate);
            self.accel_scale.set(accel_scale);
            self.gyro_range.set(gyro_range);
            self.low_power.set(low_power);
            self.temperature.set(true);
            if self.send_is_present() == Ok(()) {
                self.config_in_progress.set(true);
                Ok(())
            } else {
                Err(ErrorCode::NODEVICE)
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn send_is_present(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::IsPresent);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                // turn on i2c to send commands
                buf[0] = 0x0F;
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write_read(buf, 1, 1) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buf);
                    self.i2c.disable();
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn set_accelerometer_power_mode(
        &self,
        data_rate: LSM6DSOXAccelDataRate,
        low_power: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                self.state.set(State::SetPowerModeAccel);
                buf[0] = LSM6DSOXTRAccelRegisters::CTRL1_XL as u8;
                let mut reg: LocalRegisterCopy<u8, CTRL1_XL::Register> = LocalRegisterCopy::new(0);
                reg.modify(CTRL1_XL::ODR.val(data_rate as u8));
                reg.modify(CTRL1_XL::LPF.val(low_power as u8));
                reg.modify(CTRL1_XL::FS.val(0));

                buf[1] = reg.get();
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write(buf, 2) {
                    self.state.set(State::Idle);
                    self.i2c.disable();
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

    pub fn set_gyroscope_power_mode(
        &self,
        data_rate: LSM6DSOXGyroDataRate,
        low_power: bool,
    ) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                self.state.set(State::SetPowerModeGyro);
                buf[0] = LSM6DSOXTRGyroRegisters::CTRL2_G as u8;
                let mut reg: LocalRegisterCopy<u8, CTRL2_G::Register> = LocalRegisterCopy::new(0);
                reg.modify(CTRL2_G::ODR.val(data_rate as u8));
                reg.modify(CTRL2_G::LPF.val(low_power as u8));
                reg.modify(CTRL2_G::FS.val(0));

                buf[1] = reg.get();
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write(buf, 2) {
                    self.state.set(State::Idle);
                    self.i2c.disable();
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

    pub fn read_acceleration_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadAccelerationXYZ);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = LSM6DSOXTRAccelRegisters::OUT_X_L_A as u8;
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write_read(buf, 1, 6) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buf);
                    self.i2c.disable();
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn read_gyroscope_xyz(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadGyroscopeXYZ);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = LSM6DSOXTRGyroRegisters::OUT_X_L_G as u8;
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write_read(buf, 1, 6) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buf);
                    self.i2c.disable();
                    Err(error.into())
                } else {
                    Ok(())
                }
            })
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn read_temperature(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadTemperature);
            self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buf| {
                buf[0] = LSM6DSOXTRTempRegisters::OUT_TEMP_L as u8;
                self.i2c.enable();
                if let Err((error, buf)) = self.i2c.write_read(buf, 1, 6) {
                    self.state.set(State::Idle);
                    self.buffer.replace(buf);
                    self.i2c.disable();
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

impl i2c::I2CClient for Lsm6dsoxtrI2C<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::IsPresent => {
                let id = buffer[0];
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
                if status == Ok(()) && id == 108 {
                    self.is_present.set(true);
                    if self.config_in_progress.get() {
                        if let Err(_error) = self.set_accelerometer_power_mode(
                            self.accel_data_rate.get(),
                            self.low_power.get(),
                        ) {
                            self.config_in_progress.set(false);
                        }
                    }
                } else {
                    self.is_present.set(false);
                    self.config_in_progress.set(false);
                }

                self.syscall_process.take().map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(
                                0,
                                (
                                    into_statuscode(status.map_err(|i2c_error| i2c_error.into())),
                                    if self.is_present.get() { 1 } else { 0 },
                                    0,
                                ),
                            )
                            .ok();
                    });
                });
            }
            State::Idle => {
                //should never get here
            }
            State::ReadAccelerationXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;

                if status == Ok(()) {
                    self.nine_dof_client.map(|nine_dof_client| {
                        let scale_factor = self.accel_scale.get() as usize;
                        x = ((((buffer[0] as u16 + ((buffer[1] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[scale_factor] as isize)
                            / 1000) as usize;
                        y = ((((buffer[2] as u16 + ((buffer[3] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[scale_factor] as isize)
                            / 1000) as usize;

                        z = ((((buffer[4] as u16 + ((buffer[5] as u16) << 8)) as i16) as isize)
                            * (SCALE_FACTOR_ACCEL[scale_factor] as isize)
                            / 1000) as usize;
                        nine_dof_client.callback(x, y, z)
                    });
                } else {
                    self.nine_dof_client.map(|client| {
                        client.callback(0, 0, 0);
                    });
                };
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }

            State::ReadGyroscopeXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                if status == Ok(()) {
                    self.nine_dof_client.map(|nine_dof_client| {
                        let scale_factor = self.gyro_range.get() as usize;
                        x = (((buffer[0] as u16 + ((buffer[1] as u16) << 8)) as i16) as isize
                            * (SCALE_FACTOR_GYRO[scale_factor] as isize)
                            / 100) as usize;
                        y = (((buffer[2] as u16 + ((buffer[3] as u16) << 8)) as i16) as isize
                            * (SCALE_FACTOR_GYRO[scale_factor] as isize)
                            / 100) as usize;

                        z = (((buffer[4] as u16 + ((buffer[5] as u16) << 8)) as i16) as isize
                            * (SCALE_FACTOR_GYRO[scale_factor] as isize)
                            / 100) as usize;
                        nine_dof_client.callback(x, y, z)
                    });
                } else {
                    self.nine_dof_client.map(|client| {
                        client.callback(0, 0, 0);
                    });
                };
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }

            State::ReadTemperature => {
                let temperature = match status {
                    Ok(()) => Ok(((((buffer[0] as u16 + ((buffer[1] as u16) << 8)) as i16)
                        as isize
                        / (TEMP_SENSITIVITY_FACTOR as isize)
                        + 25)
                        * 100) as i32),
                    Err(i2c_error) => Err(i2c_error.into()),
                };
                self.temperature_client.map(|client| {
                    client.callback(temperature);
                });
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }

            State::SetPowerModeAccel => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
                if status == Ok(()) {
                    if self.config_in_progress.get() {
                        if let Err(_error) = self.set_gyroscope_power_mode(
                            self.gyro_data_rate.get(),
                            self.low_power.get(),
                        ) {
                            self.config_in_progress.set(false);
                        }
                    }
                } else {
                    self.config_in_progress.set(false);
                }
                self.syscall_process.take().map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(
                                0,
                                (
                                    into_statuscode(status.map_err(|i2c_error| i2c_error.into())),
                                    if status == Ok(()) { 1 } else { 0 },
                                    0,
                                ),
                            )
                            .ok();
                    });
                });
            }

            State::SetPowerModeGyro => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
                self.config_in_progress.set(false);
                self.syscall_process.take().map(|pid| {
                    let _res = self.apps.enter(pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(
                                0,
                                (
                                    into_statuscode(status.map_err(|i2c_error| i2c_error.into())),
                                    if status == Ok(()) { 1 } else { 0 },
                                    0,
                                ),
                            )
                            .ok();
                    });
                });
            }
        }
    }
}

impl SyscallDriver for Lsm6dsoxtrI2C<'_> {
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

        match command_num {
            // Check if the sensor is correctly connected
            1 => {
                if self.state.get() == State::Idle {
                    match self.send_is_present() {
                        Ok(()) => {
                            self.syscall_process.set(process_id);
                            CommandReturn::success()
                        }
                        Err(error) => CommandReturn::failure(error),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Accelerometer Power Mode
            2 => {
                if self.state.get() == State::Idle {
                    if let Some(data_rate) = LSM6DSOXAccelDataRate::from_usize(data1) {
                        match self.set_accelerometer_power_mode(
                            data_rate,
                            if data2 != 0 { true } else { false },
                        ) {
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
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Set Gyroscope Power Mode
            3 => {
                if self.state.get() == State::Idle {
                    if let Some(data_rate) = LSM6DSOXGyroDataRate::from_usize(data1) {
                        match self.set_gyroscope_power_mode(
                            data_rate,
                            if data2 != 0 { true } else { false },
                        ) {
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
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a> NineDof<'a> for Lsm6dsoxtrI2C<'a> {
    fn set_client(&self, nine_dof_client: &'a dyn NineDofClient) {
        self.nine_dof_client.replace(nine_dof_client);
    }

    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        self.read_acceleration_xyz()
    }

    fn read_gyroscope(&self) -> Result<(), ErrorCode> {
        self.read_gyroscope_xyz()
    }
}

impl<'a> sensors::TemperatureDriver<'a> for Lsm6dsoxtrI2C<'a> {
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_temperature()
    }
}
