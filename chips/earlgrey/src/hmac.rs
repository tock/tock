use kernel::common::StaticRef;
use lowrisc::hmac::HmacRegisters;

pub const HMAC0_BASE: StaticRef<HmacRegisters> =
    unsafe { StaticRef::new(0x4012_0000 as *const HmacRegisters) };
