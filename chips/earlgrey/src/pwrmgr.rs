use kernel::utilities::StaticRef;
use lowrisc::pwrmgr::PwrMgrRegisters;

pub(crate) const PWRMGR_BASE: StaticRef<PwrMgrRegisters> =
    unsafe { StaticRef::new(0x4040_0000 as *const PwrMgrRegisters) };
