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
//! ```rust
//! # use kernel::static_init;
//!
//! let bmm170_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x10));
//! let bmm170 = static_init!(
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

pub struct BMM150<'a, I: i2c::I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    ninedof_client: OptionalCell<&'a dyn NineDofClient>,
    state: Cell<State>,
    pending_data: Cell<bool>,
}

enum State {
    
}