// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Support for IEEE 802.15.4.

pub mod device;
pub mod framer;
pub mod mac;
pub mod virtual_mac;
pub mod xmac;

mod driver;
pub mod phy_driver;

pub use self::driver::RadioDriver;
pub use self::driver::DRIVER_NUM;
