use kernel::common::StaticRef;
use lowrisc::hmac::{Hmac, HmacRegisters};

pub static mut HMAC: Hmac = Hmac::new(HMAC0_BASE);

const HMAC0_BASE: StaticRef<HmacRegisters> =
    unsafe { StaticRef::new(0x4012_0000 as *const HmacRegisters) };
