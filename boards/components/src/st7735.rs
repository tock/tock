//! Components for the ST7735 SPI screen.
//!
//! SPI Interface
//!
//! Usage
//! -----
//! ```rust
//! let tft = components::st7735::ST7735Component::new(alarm_mux).finalize(
//!     components::st7735_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::Spi,
//!         // chip select
//!         stm32f4xx::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux,
//!         // timer type
//!         stm32f4xx::tim2::Tim2,
//!         // dc pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
//!         // reset pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::st7735::ST7735;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_spi::VirtualSpiMasterDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! st7735_component_helper {
    ($S:ty, $select:expr, $spi_mux: expr, $A:ty, $dc:expr, $reset:expr) => {{
        use capsules::st7735::ST7735;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        let st7735_spi: &'static capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));
        static mut st7735_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut st7735: MaybeUninit<ST7735<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&st7735_spi, &mut st7735_alarm, $dc, $reset, &mut st7735)
    };};
}

pub struct ST7735Component<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> {
    _select: PhantomData<S>,
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> ST7735Component<S, A> {
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> ST7735Component<S, A> {
        ST7735Component {
            _select: PhantomData,
            alarm_mux: alarm_mux,
        }
    }
}

impl<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> Component
    for ST7735Component<S, A>
{
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static mut MaybeUninit<ST7735<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static ST7735<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let st7735_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let st7735 = static_init_half!(
            static_buffer.4,
            ST7735<'static, VirtualMuxAlarm<'static, A>>,
            ST7735::new(
                static_buffer.0,
                st7735_alarm,
                static_buffer.2,
                static_buffer.3,
                &mut capsules::st7735::BUFFER,
                &mut capsules::st7735::SEQUENCE_BUFFER
            )
        );
        static_buffer.0.set_client(st7735);
        st7735_alarm.set_client(st7735);

        st7735
    }
}
