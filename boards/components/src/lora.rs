//! Component for the Murata CMWX1ZZABZ-078 LoRa Module

use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init_half;

use capsules::lora::driver::RadioDriver;
use capsules::lora::radio::Radio;
use capsules::lora::radio::RadioConfig;

// The LoRa radio requires buffers for its SPI operations:
static mut LORA_BUF: [u8; kernel::hil::radio::MAX_BUF_SIZE] =
    [0x00; kernel::hil::radio::MAX_BUF_SIZE];
static mut LORA_REG_WRITE: [u8; 2] = [0x00; 2];
static mut LORA_REG_READ: [u8; 2] = [0x00; 2];

// Setup static space for the objects.
#[macro_export]
macro_rules! lora_component_helper {
    ($S: ty) => {{
        use capsules::lora::driver::RadioDriver;
        use capsules::lora::radio::Radio;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualSpiMasterDevice<'static, $S>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<Radio<'static>> = MaybeUninit::uninit();
        static mut BUF3: MaybeUninit<RadioDriver<'static>> = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub struct LoraComponent<S: 'static + hil::spi::SpiMaster, P: 'static + hil::gpio::Pin> {
    mux_spi: &'static MuxSpiMaster<'static, S>,
    chip_select: S::ChipSelect,
    reset_pin: &'static P,
}

impl<S: 'static + hil::spi::SpiMaster, P: 'static + hil::gpio::Pin> LoraComponent<S, P> {
    pub fn new(
        mux_spi: &'static MuxSpiMaster<'static, S>,
        chip_select: S::ChipSelect,
        reset_pin: &'static P,
    ) -> LoraComponent<S, P> {
        LoraComponent {
            mux_spi,
            chip_select,
            reset_pin,
        }
    }
}

impl<S: 'static + hil::spi::SpiMaster, P: 'static + hil::gpio::Pin> Component
    for LoraComponent<S, P>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<Radio<'static>>,
        &'static mut MaybeUninit<RadioDriver<'static>>,
    );
    type Output = &'static RadioDriver<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        // Create a SPI instance for LoRa
        let lora_spi = static_init_half!(
            static_buffer.0,
            VirtualSpiMasterDevice<'static, S>,
            VirtualSpiMasterDevice::new(self.mux_spi, self.chip_select)
        );
        let radio_device = static_init_half!(
            static_buffer.1,
            Radio<'static>,
            Radio::new(lora_spi, self.reset_pin)
        );
        radio_device.initialize(&mut LORA_BUF, &mut LORA_REG_WRITE, &mut LORA_REG_READ);
        // Create the actual radio instance
        let radio_driver = static_init_half!(
            static_buffer.2,
            RadioDriver<'static>,
            RadioDriver::new(radio_device)
        );
        lora_spi.set_client(radio_driver.device);
        radio_driver
    }
}
