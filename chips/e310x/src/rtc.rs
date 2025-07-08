// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RTC instantiation.

use kernel::utilities::StaticRef;
use sifive::rtc::RtcRegisters;

pub const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x1000_0040 as *const RtcRegisters) };
