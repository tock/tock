// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::registers::top_earlgrey::TOP_EARLGREY_USBDEV_BASE_ADDR;
use kernel::utilities::StaticRef;
pub use lowrisc::usbdev::Usb;
use lowrisc::usbdev::UsbRegisters;

pub const USB0_BASE: StaticRef<UsbRegisters> =
    unsafe { StaticRef::new(TOP_EARLGREY_USBDEV_BASE_ADDR as *const UsbRegisters) };
