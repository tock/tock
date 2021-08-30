use kernel::utilities::StaticRef;
use lowrisc::csrng::CsRngRegisters;

pub const CSRNG_BASE: StaticRef<CsRngRegisters> =
    unsafe { StaticRef::new(0x4115_0000 as *const CsRngRegisters) };
