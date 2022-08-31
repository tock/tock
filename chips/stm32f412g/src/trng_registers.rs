//! True random number generator

use kernel::utilities::StaticRef;
use stm32f4xx::trng::RngRegisters;

pub const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x5006_0800 as *const RngRegisters) };
