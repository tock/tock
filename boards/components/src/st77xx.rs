//! Components for the ST77XX screen.
//!
//! Usage
//! -----
//! ```rust
//!
//! let bus = components::bus::SpiMasterBusComponent::new().finalize(
//!         components::spi_bus_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::SPI1,
//!         // chip select
//!         &stm32f303xc::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux
//!     ),
//! );
//!
//! let tft = components::st77xx::ST77XXComponent::new(alarm_mux).finalize(
//!     components::st77xx_component_helper!(
//!         // screen
//!         &capsules::st77xx::ST7789H2,
//!         // bus type
//!         capsules::bus::SpiMasterBus<
//!             'static,
//!             VirtualSpiMasterDevice<'static, stm32f4xx::spi::SPI1>,
//!         >,
//!         // bus
//!         &bus
//!         // timer type
//!         stm32f4xx::tim2::Tim2,
//!         // dc pin optional
//!         Some(stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()),
//!         // reset pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::st77xx::{ST77XXScreen, ScreenBus, ST77XX};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.

#[macro_export]
macro_rules! st77xx_spi_component_helper {
    ($screen:expr, $S:ty, $select:expr, $spi_mux: expr, $A:ty, $P:ty, $dc:expr, $reset:expr) => {{
        use capsules::st77xx::{SpiScreenBus, ST77XX};
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::spi::{self, SpiMasterDevice};

        static mut COMMAND_BUFFER: [u8; 1] = [0];

        let bus_spi: &'static VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));

        let st77xx_bus: &'static SpiScreenBus<'static, VirtualSpiMasterDevice<'static, $S>> = static_init!(
            SpiScreenBus<'static, VirtualSpiMasterDevice<'static, $S>>,
            SpiScreenBus::new(bus_spi, &mut COMMAND_BUFFER)
        );

        bus_spi.set_client(st77xx_bus);

        static mut st77xx_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut st77xx: MaybeUninit<
            ST77XX<'static, VirtualMuxAlarm<'static, $A>, SpiScreenBus<'static, VirtualSpiMasterDevice<'static, $S>>, $P>,
        > = MaybeUninit::uninit();
        (
            st77xx_bus,
            &mut st77xx_alarm,
            Some($dc),
            $reset,
            &mut st77xx,
            $screen,
        )
    };};
}

#[macro_export]
macro_rules! st77xx_bus_8080_component_helper {
    ($screen:expr, $B: ty, $bus: expr, $width: expr,  $A:ty, $P:ty, $reset:expr) => {{
        use capsules::st77xx::{Bus8080ScreenBus, ST77XX};
        use core::mem::MaybeUninit;
        use kernel::hil::bus8080::Bus8080;
        use kernel::hil::spi::{self, SpiMasterDevice};

        static mut COMMAND_BUFFER: [u8; 1] = [0];

        let st77xx_bus: &'static Bus8080ScreenBus<'static, $B> = static_init!(
            Bus8080ScreenBus<'static, $B>,
            Bus8080ScreenBus::new($bus, $width)
        );

        $bus.set_client(st77xx_bus);

        static mut st77xx_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut st77xx: MaybeUninit<
            ST77XX<'static, VirtualMuxAlarm<'static, $A>, Bus8080ScreenBus<'static, $B>, $P>,
        > = MaybeUninit::uninit();
        (
            st77xx_bus,
            &mut st77xx_alarm,
            None,
            $reset,
            &mut st77xx,
            $screen,
        )
    };};
}

pub struct ST77XXComponent<
    A: 'static + time::Alarm<'static>,
    B: 'static + ScreenBus<'static>,
    P: 'static + gpio::Pin,
> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    _screen_bus: PhantomData<B>,
    _pin: PhantomData<P>,
}

impl<
        A: 'static + time::Alarm<'static>,
        B: 'static + ScreenBus<'static>,
        P: 'static + gpio::Pin,
    > ST77XXComponent<A, B, P>
{
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> ST77XXComponent<A, B, P> {
        ST77XXComponent {
            alarm_mux: alarm_mux,
            _screen_bus: PhantomData,
            _pin: PhantomData,
        }
    }
}

impl<
        A: 'static + time::Alarm<'static>,
        B: 'static + ScreenBus<'static>,
        P: 'static + gpio::Pin,
    > Component for ST77XXComponent<A, B, P>
{
    type StaticInput = (
        &'static B,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        Option<&'static P>,
        &'static P,
        &'static mut MaybeUninit<ST77XX<'static, VirtualMuxAlarm<'static, A>, B, P>>,
        &'static ST77XXScreen,
    );
    type Output = &'static ST77XX<'static, VirtualMuxAlarm<'static, A>, B, P>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let st77xx_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let st77xx = static_init_half!(
            static_buffer.4,
            ST77XX<'static, VirtualMuxAlarm<'static, A>, B, P>,
            ST77XX::new(
                static_buffer.0,
                st77xx_alarm,
                static_buffer.2,
                static_buffer.3,
                &mut capsules::st77xx::BUFFER,
                &mut capsules::st77xx::SEQUENCE_BUFFER,
                static_buffer.5
            )
        );
        static_buffer.0.set_client(st77xx);
        st77xx_alarm.set_client(st77xx);

        st77xx
    }
}
