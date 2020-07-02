use kernel::common::StaticRef;
use lowrisc::pwrmgr::{PwrMgr, PwrMgrRegisters};

pub static mut PWRMGR: PwrMgr = PwrMgr::new(PWRMGR_BASE);

const PWRMGR_BASE: StaticRef<PwrMgrRegisters> =
    unsafe { StaticRef::new(0x400A_0000 as *const PwrMgrRegisters) };
