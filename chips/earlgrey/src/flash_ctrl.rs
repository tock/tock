use kernel::common::StaticRef;
use lowrisc::flash_ctrl::FlashCtrlRegisters;

pub const FLASH_CTRL_BASE: StaticRef<FlashCtrlRegisters> =
    unsafe { StaticRef::new(0x4003_0000 as *const FlashCtrlRegisters) };
