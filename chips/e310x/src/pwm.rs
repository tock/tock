// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! PWM instantiation.

use kernel::utilities::StaticRef;
use sifive::pwm::PwmRegisters;

pub const PWM0_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10015000 as *const PwmRegisters) };
pub const PWM1_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10025000 as *const PwmRegisters) };
pub const PWM2_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10035000 as *const PwmRegisters) };
