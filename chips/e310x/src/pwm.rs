//! PWM instantiation.

use kernel::common::StaticRef;
use sifive::pwm::PwmRegisters;

pub const PWM0_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10015000 as *const PwmRegisters) };
pub const PWM1_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10025000 as *const PwmRegisters) };
pub const PWM2_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10035000 as *const PwmRegisters) };
