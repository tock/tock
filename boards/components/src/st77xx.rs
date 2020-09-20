//! Components for the ST77XX screen.
//!
//! Usage
//! -----
//! ```rust
//!
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
//!
//! let tft = components::st77xx::ST77XXComponent::new(mux_alarm).finalize(
//!     components::st77xx_component_helper!(
//!         // screen
//!         &capsules::st77xx::ST7735,
//!         // bus type
//!         capsules::bus::SpiMasterBus<
//!             'static,
//!             VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
//!         >,
//!         // bus
//!         &bus,
//!         // timer type
//!         nrf52840::rtc::Rtc,
//!         // pin type
//!         nrf52::gpio::GPIOPin<'static>,
//!         // dc
//!         Some(&nrf52840::gpio::PORT[GPIO_D3]),
//!         // reset
//!         &nrf52840::gpio::PORT[GPIO_D2]
//!     ),
//! );
//! ```
use capsules::bus;
use capsules::st77xx::{ST77XXScreen, ST77XX};
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
macro_rules! st77xx_component_helper {
    ($screen:expr, $B: ty, $bus:expr, $A:ty, $P:ty, $dc:expr, $reset:expr) => {{
        use capsules::bus::Bus;
        use capsules::st77xx::ST77XX;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::spi::{self, SpiMasterDevice};
        let st77xx_bus: &$B = $bus;
        static mut st77xx_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut st77xx: MaybeUninit<ST77XX<'static, VirtualMuxAlarm<'static, $A>, $B, $P>> =
            MaybeUninit::uninit();
        (
            st77xx_bus,
            &mut st77xx_alarm,
            $dc,
            $reset,
            &mut st77xx,
            $screen,
        )
    };};
}

pub struct ST77XXComponent<
    A: 'static + time::Alarm<'static>,
    B: 'static + bus::Bus<'static>,
    P: 'static + gpio::Pin,
> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    _bus: PhantomData<B>,
    _pin: PhantomData<P>,
}

impl<A: 'static + time::Alarm<'static>, B: 'static + bus::Bus<'static>, P: 'static + gpio::Pin>
    ST77XXComponent<A, B, P>
{
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> ST77XXComponent<A, B, P> {
        ST77XXComponent {
            alarm_mux: alarm_mux,
            _bus: PhantomData,
            _pin: PhantomData,
        }
    }
}

impl<A: 'static + time::Alarm<'static>, B: 'static + bus::Bus<'static>, P: 'static + gpio::Pin>
    Component for ST77XXComponent<A, B, P>
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
