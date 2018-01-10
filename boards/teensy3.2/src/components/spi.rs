use mk20;
use capsules::virtual_spi::{VirtualSpiMasterDevice, MuxSpiMaster};
use kernel::hil::spi::SpiMaster;
use spi::Spi;
use components::Component;

pub struct VirtualSpiComponent;

impl VirtualSpiComponent {
    pub fn new() -> Self {
        VirtualSpiComponent {}
    }
}

impl Component for VirtualSpiComponent {
    type Output = &'static Spi<'static, VirtualSpiMasterDevice<'static, mk20::spi::Spi<'static>>>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        mk20::spi::SPI0.init();
        mk20::spi::SPI1.init();
        mk20::spi::SPI2.init();

        let mux_spi0 = static_init!(
                MuxSpiMaster<'static, mk20::spi::Spi<'static>>,
                MuxSpiMaster::new(&mk20::spi::SPI0)
            );
        let mux_spi1 = static_init!(
                MuxSpiMaster<'static, mk20::spi::Spi<'static>>,
                MuxSpiMaster::new(&mk20::spi::SPI1)
            );
        let mux_spi2 = static_init!(
                MuxSpiMaster<'static, mk20::spi::Spi<'static>>,
                MuxSpiMaster::new(&mk20::spi::SPI2)
            );

        mk20::spi::SPI0.set_client(mux_spi0);
        mk20::spi::SPI1.set_client(mux_spi1);
        mk20::spi::SPI2.set_client(mux_spi2);

        let virtual_spi = static_init!(
                [VirtualSpiMasterDevice<'static, mk20::spi::Spi<'static>>; 3],
                [VirtualSpiMasterDevice::new(mux_spi0, 0),
                 VirtualSpiMasterDevice::new(mux_spi1, 0),
                 VirtualSpiMasterDevice::new(mux_spi2, 0)]
            );

        let spi = static_init!(
                Spi<'static, VirtualSpiMasterDevice<'static, mk20::spi::Spi<'static>>>,
                Spi::new(virtual_spi)
            );

        static mut SPI_READ_BUF: [u8; 1024] = [0; 1024];
        static mut SPI_WRITE_BUF: [u8; 1024] = [0; 1024];

        spi.config_buffers(&mut SPI_READ_BUF, &mut SPI_WRITE_BUF);

        virtual_spi[0].set_client(spi);
        virtual_spi[1].set_client(spi);
        virtual_spi[2].set_client(spi);

        Some(spi)
    }
}
