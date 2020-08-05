use kernel::common::StaticRef;
use lowrisc::usbdev::{Usb, UsbRegisters};

pub static mut USB: Usb = Usb::new(USB0_BASE);

const USB0_BASE: StaticRef<UsbRegisters> =
    unsafe { StaticRef::new(0x4015_0000 as *const UsbRegisters) };
