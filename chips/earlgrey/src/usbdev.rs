use kernel::common::StaticRef;
pub use lowrisc::usbdev::Usb;
use lowrisc::usbdev::UsbRegisters;

pub const USB0_BASE: StaticRef<UsbRegisters> =
    unsafe { StaticRef::new(0x4015_0000 as *const UsbRegisters) };
