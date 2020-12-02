//! Bus Components for Intel8080 Parallel Interface, I2C, SPI
//!
//! Example
//!
//! Intel 8080 Parallel Interface
//!
//! let bus = components::bus::Bus8080BusComponent::new().finalize(
//!     components::bus8080_bus_component_helper!(
//!         // bus type
//!         stm32f412g::fsmc::Fsmc,
//!         // bus
//!         &stm32f412g::fsmc::FSMC
//!     ),
//! );
//!
//! SPI
//!
//! ```rust
//! let bus = components::bus::SpiMasterBusComponent::new().finalize(
//!     components::spi_bus_component_helper!(
//!         // spi type
//!         nrf52840::spi::SPIM,
//!         // chip select
//!         &nrf52840::gpio::PORT[GPIO_D4],
//!         // spi mux
//!         spi_mux
//!     ),
//! );
//! ```
use capsules::bus::{Bus8080Bus, I2CMasterBus, SpiMasterBus};
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use capsules::virtual_spi::VirtualSpiMasterDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::bus8080;
use kernel::hil::spi;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! bus8080_bus_component_helper {
    ($B:ty, $bus8080:expr $(,)?) => {{
        use capsules::bus::Bus8080Bus;
        use core::mem::{size_of, MaybeUninit};
        static mut bus: MaybeUninit<Bus8080Bus<'static, $B>> = MaybeUninit::uninit();
        ($bus8080, &mut bus)
    };};
}

#[macro_export]
macro_rules! spi_bus_component_helper {
    ($S:ty, $select:expr, $spi_mux:expr $(,)?) => {{
        use capsules::bus::SpiMasterBus;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::{size_of, MaybeUninit};
        let bus_spi: &'static VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));
        static mut ADDRESS_BUFFER: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
        static mut bus: MaybeUninit<SpiMasterBus<'static, VirtualSpiMasterDevice<'static, $S>>> =
            MaybeUninit::uninit();
        (&bus_spi, &mut bus, &mut ADDRESS_BUFFER)
    };};
}

#[macro_export]
macro_rules! i2c_master_bus_component_helper {
    () => {{
        use capsules::bus::I2CMasterBus;
        use core::mem::{size_of, MaybeUninit};
        static mut ADDRESS_BUFFER: [u8; 1] = [0; 1];
        static mut bus: MaybeUninit<I2CMasterBus<'static>> = MaybeUninit::uninit();
        (&mut bus, &mut ADDRESS_BUFFER)
    };};
}

pub struct Bus8080BusComponent<B: 'static + bus8080::Bus8080<'static>> {
    _bus: PhantomData<B>,
}

impl<B: 'static + bus8080::Bus8080<'static>> Bus8080BusComponent<B> {
    pub fn new() -> Bus8080BusComponent<B> {
        Bus8080BusComponent { _bus: PhantomData }
    }
}

impl<B: 'static + bus8080::Bus8080<'static>> Component for Bus8080BusComponent<B> {
    type StaticInput = (&'static B, &'static mut MaybeUninit<Bus8080Bus<'static, B>>);
    type Output = &'static Bus8080Bus<'static, B>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bus = static_init_half!(
            static_buffer.1,
            Bus8080Bus<'static, B>,
            Bus8080Bus::new(static_buffer.0)
        );
        static_buffer.0.set_client(bus);

        bus
    }
}

pub struct SpiMasterBusComponent<S: 'static + spi::SpiMaster> {
    _select: PhantomData<S>,
}

impl<S: 'static + spi::SpiMaster> SpiMasterBusComponent<S> {
    pub fn new() -> SpiMasterBusComponent<S> {
        SpiMasterBusComponent {
            _select: PhantomData,
        }
    }
}

impl<S: 'static + spi::SpiMaster> Component for SpiMasterBusComponent<S> {
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static mut MaybeUninit<SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut [u8],
    );
    type Output = &'static SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bus = static_init_half!(
            static_buffer.1,
            SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>,
            SpiMasterBus::new(static_buffer.0, static_buffer.2)
        );
        static_buffer.0.set_client(bus);

        bus
    }
}

pub struct I2CMasterBusComponent {
    i2c_mux: &'static MuxI2C<'static>,
    address: u8,
}

impl I2CMasterBusComponent {
    pub fn new(i2c_mux: &'static MuxI2C<'static>, address: u8) -> I2CMasterBusComponent {
        I2CMasterBusComponent {
            i2c_mux: i2c_mux,
            address: address,
        }
    }
}

impl Component for I2CMasterBusComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CMasterBus<'static, I2CDevice<'static>>>,
        &'static mut [u8],
    );
    type Output = &'static I2CMasterBus<'static, I2CDevice<'static>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bus_i2c: &'static I2CDevice<'static> =
            crate::i2c::I2CComponent::new(self.i2c_mux, self.address)
                .finalize(crate::i2c_component_helper!());

        let bus = static_init_half!(
            static_buffer.0,
            I2CMasterBus<'static, I2CDevice<'static>>,
            I2CMasterBus::new(bus_i2c, static_buffer.1)
        );
        bus_i2c.set_client(bus);

        bus
    }
}
