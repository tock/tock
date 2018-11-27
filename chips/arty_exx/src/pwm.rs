use kernel::common::StaticRef;
use sifive::pwm::{Pwm, PwmRegisters};

pub static mut PWM0: Pwm = Pwm::new(PWM0_BASE);

const PWM0_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x20005000 as *const PwmRegisters) };
