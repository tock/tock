use kernel::common::StaticRef;
use sifive::pwm::{Pwm, PwmRegisters};

pub static mut PWM0: Pwm = Pwm::new(PWM0_BASE);
pub static mut PWM1: Pwm = Pwm::new(PWM1_BASE);
pub static mut PWM2: Pwm = Pwm::new(PWM2_BASE);


const PWM0_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10015000 as *const PwmRegisters) };
const PWM1_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10025000 as *const PwmRegisters) };
const PWM2_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x10035000 as *const PwmRegisters) };
