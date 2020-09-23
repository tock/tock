use kernel::common::StaticRef;
use lowrisc::pwrmgr::PwrMgrRegisters;

pub(crate) const PWRMGR_BASE: StaticRef<PwrMgrRegisters> =
    unsafe { StaticRef::new(0x400A_0000 as *const PwrMgrRegisters) };
