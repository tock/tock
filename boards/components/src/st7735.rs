//! Components for the ST7735 screen.
//!
//! SPI Interface
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::st7735::ST7735Component::new(board_kernel).finalize(
//!     components::st7735_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::Spi,
//!         // chip select
//!         stm32f4xx::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux
//!     )
//! );
//! ```
use capsules::st7735::ST7735;
use capsules::virtual_alarm::VirtualMuxAlarm;
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
    ($S:ty, $select: expr, $spi_mux: expr, $A:ty, $alarm_mux: expr) => {{
        use capsules::st7735::ST7735;
        use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        let st7735_spi: &'static capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));
        static st7735_alarm: let st7735_alarm = static_init!(
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new($alarm_mux)
        );
        static mut st7735: MaybeUninit<ST7735<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&st7735_spi, &st7735_alarm, &mut st7735)
    };};
}

pub struct ST7735Component<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> {
    _select: PhantomData<S>,
    _alarm: PhantomData<A>,
}

impl<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> ST7735Component<S, A> {
    pub fn new() -> ST7735Component<S, A> {
        ST7735Component {
            _select: PhantomData,
            _alarm: PhantomData,
        }
    }
}

impl<S: 'static + spi::SpiMaster, A: 'static + time::Alarm<'static>> Component
    for ST7735Component<S, A>
{
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static VirtualMuxAlarm<'static, A>,
        &'static mut MaybeUninit<ST7735<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static ST7735<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let st7735 = static_init_half!(
            static_buffer.2,
            ST7735<'static, VirtualMuxAlarm<'static, A>>,
            ST7735::new(
                static_buffer.0,
                static_buffer.1,
                &mut capsules::st7735::BUFFER
            )
        );
        static_buffer.0.set_client(st7735);
        static_buffer.1.set_client(st7735);

        st7735
    }
}
