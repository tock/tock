//! Components for the FM25CL FRAM chip.
//!
//! Uses a SPI Interface.
//!
//! Usage
//! -----
//! ```rust
//! let fm25cl = components::fm25cl::Fm25clComponent::new(spi_mux, stm32f429zi::gpio::PinId::PE03)
//!     .finalize(components::fm25cl_component_static!(stm32f429zi::spi::Spi));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use extra_capsules::fm25cl::FM25CL;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::hil::spi::SpiMasterDevice;

#[macro_export]
macro_rules! fm25cl_component_static {
    ($S:ty $(,)?) => {{
        let txbuffer = kernel::static_buf!([u8; extra_capsules::fm25cl::BUF_LEN]);
        let rxbuffer = kernel::static_buf!([u8; extra_capsules::fm25cl::BUF_LEN]);

        let spi =
            kernel::static_buf!(core_capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S>);
        let fm25cl = kernel::static_buf!(
            extra_capsules::fm25cl::FM25CL<
                'static,
                core_capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
            >
        );

        (spi, fm25cl, txbuffer, rxbuffer)
    };};
}

pub struct Fm25clComponent<S: 'static + spi::SpiMaster> {
    spi_mux: &'static MuxSpiMaster<'static, S>,
    chip_select: S::ChipSelect,
}

impl<S: 'static + spi::SpiMaster> Fm25clComponent<S> {
    pub fn new(
        spi_mux: &'static MuxSpiMaster<'static, S>,
        chip_select: S::ChipSelect,
    ) -> Fm25clComponent<S> {
        Fm25clComponent {
            spi_mux,
            chip_select,
        }
    }
}

impl<S: 'static + spi::SpiMaster> Component for Fm25clComponent<S> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<FM25CL<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut MaybeUninit<[u8; extra_capsules::fm25cl::BUF_LEN]>,
        &'static mut MaybeUninit<[u8; extra_capsules::fm25cl::BUF_LEN]>,
    );
    type Output = &'static FM25CL<'static, VirtualSpiMasterDevice<'static, S>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let spi_device = static_buffer
            .0
            .write(VirtualSpiMasterDevice::new(self.spi_mux, self.chip_select));
        spi_device.setup();

        let txbuffer = static_buffer.2.write([0; extra_capsules::fm25cl::BUF_LEN]);
        let rxbuffer = static_buffer.3.write([0; extra_capsules::fm25cl::BUF_LEN]);

        let fm25cl = static_buffer
            .1
            .write(FM25CL::new(spi_device, txbuffer, rxbuffer));
        spi_device.set_client(fm25cl);

        fm25cl
    }
}
