// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

mod device;
mod driver;

pub use device::{len, Security, Ssid, Wpa3Passphrase, WpaPassphrase};
pub use device::{Client, Device};

pub use driver::WifiDriver;

/// Syscall driver number.
pub const DRIVER_NUM: usize = capsules_core::driver::NUM::Wifi as usize;
