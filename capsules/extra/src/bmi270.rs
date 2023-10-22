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
//! # use kernel::static_init;
//!
//! let bmi270_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x68));
//! let bmi270 = static_init!(
//!     capsules::bmi270::BMI270<'static>,
//!     capsules::bmi270::BMI270::new(bmi270_i2c,
//!         &mut capsules::bmi270::BUFFER));
//! bmi270_i2c.set_client(bmi270);
//! ```

use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{NineDofClient, NineDof};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::NINEDOF as usize;

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