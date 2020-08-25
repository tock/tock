use kernel::common::StaticRef;
use lowrisc::spi::{SpiDevice, SpiDeviceRegisters};

pub static mut SPI_DEVICE: SpiDevice = SpiDevice::new(SPIDEVICE0_BASE);

const SPIDEVICE0_BASE: StaticRef<SpiDeviceRegisters> =
    unsafe { StaticRef::new(0x4002_0000 as *const SpiDeviceRegisters) };
