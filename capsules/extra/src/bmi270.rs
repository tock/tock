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