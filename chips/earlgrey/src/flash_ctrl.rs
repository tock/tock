use kernel::common::StaticRef;
use lowrisc::flash_ctrl::{FlashCtrl, FlashCtrlRegisters};

pub static mut FLASH_CTRL: FlashCtrl = FlashCtrl::new(FLASH_CTRL_BASE);

const FLASH_CTRL_BASE: StaticRef<FlashCtrlRegisters> =
    unsafe { StaticRef::new(0x4003_0000 as *const FlashCtrlRegisters) };
