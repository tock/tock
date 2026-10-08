// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Driver for the TDK InvenSense ICM-42670-P 6-Axis IMU (accelerometer +
//! gyroscope).
//!
//! <https://invensense.tdk.com/products/motion-tracking/6-axis/icm-42670-p/>

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::i2c;
use kernel::hil::sensors::{NineDof, NineDofClient};
use kernel::hil::time::{self, Alarm, ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::register_bitfields;

/// The ICM-42670-P's I2C address depends on the level its `AP_AD0` pin is
/// wired to: `0x68` when low (the common/default wiring), `0x69` when
/// high.
pub const BASE_ADDR: u8 = 0x68;
/// The alternate I2C address, when `AP_AD0` is wired high.
pub const ALT_ADDR: u8 = 0x69;

/// Register addresses used by this driver.
mod register {
    /// First (high) byte of the 6-byte accelerometer X/Y/Z burst.
    pub const ACCEL_DATA_X1: u8 = 0x0B;
    /// First (high) byte of the 6-byte gyroscope X/Y/Z burst.
    pub const GYRO_DATA_X1: u8 = 0x11;
    /// First of three consecutive registers (`PWR_MGMT0`, `GYRO_CONFIG0`,
    /// `ACCEL_CONFIG0`) written together to power on and configure both
    /// sensors.
    pub const PWR_MGMT0: u8 = 0x1F;
}

register_bitfields![u8,
    PWR_MGMT0 [
        GYRO_MODE OFFSET(2) NUMBITS(2) [
            LowNoise = 0b11,
        ],
        ACCEL_MODE OFFSET(0) NUMBITS(2) [
            LowNoise = 0b11,
        ],
    ],
    GYRO_CONFIG0 [
        GYRO_UI_FS_SEL OFFSET(5) NUMBITS(2) [],
        GYRO_ODR OFFSET(0) NUMBITS(4) [],
    ],
    ACCEL_CONFIG0 [
        ACCEL_UI_FS_SEL OFFSET(5) NUMBITS(2) [],
        ACCEL_ODR OFFSET(0) NUMBITS(4) [],
    ],
];

/// `{GYRO,ACCEL}_ODR` value for 100 Hz, the same encoding for both.
const ODR_100HZ: u8 = 0b1001;

/// `ACCEL_UI_FS_SEL` value for ±2g, the accelerometer's most sensitive
/// (best resolution) range.
const ACCEL_FS_SEL_2G: u8 = 0b11;
/// Accelerometer sensitivity at `ACCEL_FS_SEL_2G`, in LSB per g.
const ACCEL_SENSITIVITY_LSB_PER_G: i32 = 16384;

/// `GYRO_UI_FS_SEL` value for ±250 º/s, the gyroscope's most sensitive
/// (best resolution) range.
const GYRO_FS_SEL_250DPS: u8 = 0b11;
/// Gyroscope sensitivity at `GYRO_FS_SEL_250DPS`, in LSB per (º/s),
/// multiplied by 10.
///
/// 131 is exactly an integer number of LSB per º/s here, but this is kept
/// as a x10 fixed-point value since nearby ranges aren't whole numbers,
/// e.g. 65.5 LSB/(º/s) for ±500 º/s.
const GYRO_SENSITIVITY_LSB_PER_DPS_X10: i32 = 1310;

/// Time to wait after configuring the sensor before its output is valid.
///
/// The datasheet's "Accelerometer Startup Time" and "Gyroscope Start-Up
/// Time" are 10 ms and 30 ms respectively (from power-on to valid data),
/// with margin.
const STARTUP_DELAY_MS: u32 = 40;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    /// Burst-writing `PWR_MGMT0`/`GYRO_CONFIG0`/`ACCEL_CONFIG0`; waiting
    /// for the I2C write to finish. Only happens once, the first time
    /// either sensor is read.
    Configuring,
    /// Waiting out `STARTUP_DELAY_MS` after `Configuring`, before the
    /// requested reading can proceed.
    StartupDelay,
    /// Reading back the 6-byte accelerometer or gyroscope result
    /// (whichever `pending` says was requested).
    Reading,
}

#[derive(Clone, Copy, PartialEq)]
enum Reading {
    Accelerometer,
    Gyroscope,
}

/// Convert two big-endian, two's-complement bytes (as the ICM-42670-P
/// returns each axis: high byte first) into a signed 16-bit value.
fn be16(hi: u8, lo: u8) -> i16 {
    (((hi as u16) << 8) | (lo as u16)) as i16
}

pub struct Icm42670p<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> {
    i2c: &'a I,
    alarm: &'a A,
    nine_dof_client: OptionalCell<&'a dyn NineDofClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    /// Set once `PWR_MGMT0`/`GYRO_CONFIG0`/`ACCEL_CONFIG0` have been
    /// written successfully: later reads can skip straight to `Reading`.
    configured: Cell<bool>,
    /// Which reading is in progress (valid whenever `state != Idle`).
    pending: Cell<Reading>,
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> Icm42670p<'a, A, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8], alarm: &'a A) -> Icm42670p<'a, A, I> {
        Icm42670p {
            i2c,
            alarm,
            nine_dof_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            configured: Cell::new(false),
            pending: Cell::new(Reading::Accelerometer),
        }
    }

    fn start_reading(&self, reading: Reading) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }

        self.pending.set(reading);

        if self.configured.get() {
            self.start_read()
        } else {
            self.start_configure()
        }
    }

    /// Burst-write `PWR_MGMT0`, `GYRO_CONFIG0`, and `ACCEL_CONFIG0`
    /// (consecutive registers) to power on both sensors in low-noise
    /// mode at their most sensitive range and a 100 Hz output data rate.
    fn start_configure(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            self.state.set(State::Configuring);

            let mut pwr_mgmt0: LocalRegisterCopy<u8, PWR_MGMT0::Register> =
                LocalRegisterCopy::new(0);
            pwr_mgmt0.write(PWR_MGMT0::GYRO_MODE::LowNoise + PWR_MGMT0::ACCEL_MODE::LowNoise);

            let mut gyro_config0: LocalRegisterCopy<u8, GYRO_CONFIG0::Register> =
                LocalRegisterCopy::new(0);
            gyro_config0.write(
                GYRO_CONFIG0::GYRO_UI_FS_SEL.val(GYRO_FS_SEL_250DPS)
                    + GYRO_CONFIG0::GYRO_ODR.val(ODR_100HZ),
            );

            let mut accel_config0: LocalRegisterCopy<u8, ACCEL_CONFIG0::Register> =
                LocalRegisterCopy::new(0);
            accel_config0.write(
                ACCEL_CONFIG0::ACCEL_UI_FS_SEL.val(ACCEL_FS_SEL_2G)
                    + ACCEL_CONFIG0::ACCEL_ODR.val(ODR_100HZ),
            );

            buffer[0] = register::PWR_MGMT0;
            buffer[1] = pwr_mgmt0.get();
            buffer[2] = gyro_config0.get();
            buffer[3] = accel_config0.get();

            self.i2c.enable();
            match self.i2c.write(buffer, 4) {
                Ok(()) => Ok(()),
                Err((error, buffer)) => {
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                    self.state.set(State::Idle);
                    Err(error.into())
                }
            }
        })
    }

    /// Read back the 6-byte accelerometer or gyroscope result, per
    /// `self.pending`.
    fn start_read(&self) -> Result<(), ErrorCode> {
        self.buffer.take().map_or(Err(ErrorCode::NOMEM), |buffer| {
            self.state.set(State::Reading);

            buffer[0] = match self.pending.get() {
                Reading::Accelerometer => register::ACCEL_DATA_X1,
                Reading::Gyroscope => register::GYRO_DATA_X1,
            };

            self.i2c.enable();
            match self.i2c.write_read(buffer, 1, 6) {
                Ok(()) => Ok(()),
                Err((error, buffer)) => {
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                    self.state.set(State::Idle);
                    Err(error.into())
                }
            }
        })
    }

    /// Abort on an I2C error: disable the bus, return to `Idle`, and
    /// report the failure the same way a bad reading is reported (there's
    /// no error type in `NineDofClient::callback`).
    fn abort(&self, buffer: &'static mut [u8]) {
        self.buffer.replace(buffer);
        self.i2c.disable();
        self.state.set(State::Idle);
        self.nine_dof_client.map(|client| client.callback(0, 0, 0));
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> time::AlarmClient for Icm42670p<'a, A, I> {
    fn alarm(&self) {
        // After the startup delay, take the reading.
        if let Err(_error) = self.start_read() {
            self.i2c.disable();
            self.state.set(State::Idle);
            self.nine_dof_client.map(|client| client.callback(0, 0, 0));
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> i2c::I2CClient for Icm42670p<'a, A, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if status.is_err() {
            self.abort(buffer);
            return;
        }

        match self.state.get() {
            State::Configuring => {
                self.buffer.replace(buffer);
                self.configured.set(true);
                self.state.set(State::StartupDelay);
                self.alarm
                    .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(STARTUP_DELAY_MS));
            }
            State::Reading => {
                let (sensitivity, x, y, z) = match self.pending.get() {
                    Reading::Accelerometer => (
                        ACCEL_SENSITIVITY_LSB_PER_G,
                        be16(buffer[0], buffer[1]),
                        be16(buffer[2], buffer[3]),
                        be16(buffer[4], buffer[5]),
                    ),
                    Reading::Gyroscope => (
                        GYRO_SENSITIVITY_LSB_PER_DPS_X10,
                        be16(buffer[0], buffer[1]),
                        be16(buffer[2], buffer[3]),
                        be16(buffer[4], buffer[5]),
                    ),
                };
                // Accelerometer values are reported in milli-g, gyroscope
                // values in milli-degrees-per-second (the same convention
                // `lsm6dsoxtr.rs` and `fxos8700cq.rs` use); like those
                // drivers, the signed result is bit-reinterpreted (not
                // numerically converted) into the `usize` this HIL's
                // callback carries it in.
                let scale = match self.pending.get() {
                    Reading::Accelerometer => 1000,
                    Reading::Gyroscope => 10000,
                };
                let x = ((x as i32 * scale) / sensitivity) as usize;
                let y = ((y as i32 * scale) / sensitivity) as usize;
                let z = ((z as i32 * scale) / sensitivity) as usize;

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
                self.nine_dof_client.map(|client| client.callback(x, y, z));
            }
            State::Idle | State::StartupDelay => {}
        }
    }
}

impl<'a, A: Alarm<'a>, I: i2c::I2CDevice<'a>> NineDof<'a> for Icm42670p<'a, A, I> {
    fn set_client(&self, client: &'a dyn NineDofClient) {
        self.nine_dof_client.set(client);
    }

    fn read_accelerometer(&self) -> Result<(), ErrorCode> {
        self.start_reading(Reading::Accelerometer)
    }

    fn read_gyroscope(&self) -> Result<(), ErrorCode> {
        self.start_reading(Reading::Gyroscope)
    }
}
