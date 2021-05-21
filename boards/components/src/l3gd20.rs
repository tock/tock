//! Components for the L3GD20 sensor.
//!
//! SPI Interface
//!
//! Usage
//! -----
//! ```rust
//! let l3gd20 = components::l3gd20::L3gd20SpiComponent::new().finalize(
//!     components::l3gd20_spi_component_helper!(
//!         // spi type
//!         stm32f429zi::spi::Spi,
//!         // chip select
//!         stm32f429zi::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux
//!     )
//! );
//! ```
use capsules::l3gd20::L3gd20Spi;
use capsules::virtual_spi::VirtualSpiMasterDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::{create_capability, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! l3gd20_spi_component_helper {
    ($A:ty, $select:expr, $spi_mux:expr $(,)?) => {{
        use capsules::l3gd20::L3gd20Spi;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        let mut l3gd20_spi: &'static capsules::virtual_spi::VirtualSpiMasterDevice<'static, $A> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($A));
        static mut l3gd20spi: MaybeUninit<L3gd20Spi<'static>> = MaybeUninit::uninit();
        (&mut l3gd20_spi, &mut l3gd20spi)
    };};
}

pub struct L3gd20SpiComponent<S: 'static + spi::SpiMaster> {
    _select: PhantomData<S>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<S: 'static + spi::SpiMaster> L3gd20SpiComponent<S> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> L3gd20SpiComponent<S> {
        L3gd20SpiComponent {
            _select: PhantomData,
            board_kernel: board_kernel,
            driver_num,
        }
    }
}

impl<S: 'static + spi::SpiMaster> Component for L3gd20SpiComponent<S> {
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static mut MaybeUninit<L3gd20Spi<'static>>,
    );
    type Output = &'static L3gd20Spi<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let l3gd20 = static_init_half!(
            static_buffer.1,
            L3gd20Spi<'static>,
            L3gd20Spi::new(
                static_buffer.0,
                &mut capsules::l3gd20::TXBUFFER,
                &mut capsules::l3gd20::RXBUFFER,
                grant
            )
        );
        static_buffer.0.set_client(l3gd20);
        l3gd20.configure();

        l3gd20
    }
}
