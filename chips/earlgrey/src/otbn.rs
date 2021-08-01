use kernel::utilities::StaticRef;
use lowrisc::otbn::OtbnRegisters;

pub const OTBN_BASE: StaticRef<OtbnRegisters> =
    unsafe { StaticRef::new(0x411D_0000 as *const OtbnRegisters) };
