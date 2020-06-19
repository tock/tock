//! Component for the MX25R6435F flash chip.
//!
//! Usage
//! -----
//! ```rust
//! let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
//!     &gpio_port[driver.write_protect_pin],
//!     &gpio_port[driver.hold_pin],
//!     &gpio_port[driver.chip_select] as &dyn kernel::hil::gpio::Pin,
//!     mux_alarm,
//!     mux_spi,
//! )
//! .finalize(components::mx25r6435f_component_helper!(
//!     nrf52::spi::SPIM,
//!     nrf52::gpio::GPIOPin,
//!     nrf52::rtc::Rtc
//! ));
//! ```
use capsules::mx25r6435f::MX25R6435F;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! mx25r6435f_component_helper {
    ($S:ty, $P: ty, $A: ty) => {{
        use capsules::mx25r6435f::MX25R6435F;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualSpiMasterDevice<'static, $S>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF3: MaybeUninit<
            MX25R6435F<
                'static,
                VirtualSpiMasterDevice<'static, $S>,
                $P,
                VirtualMuxAlarm<'static, $A>,
            >,
        > = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub struct Mx25r6435fComponent<
    S: 'static + hil::spi::SpiMaster,
    P: 'static + hil::gpio::Pin,
    A: 'static + hil::time::Alarm<'static>,
> {
    write_protect_pin: &'static P,
    hold_pin: &'static P,
    chip_select: S::ChipSelect,
    mux_alarm: &'static MuxAlarm<'static, A>,
    mux_spi: &'static MuxSpiMaster<'static, S>,
}

impl<
        S: 'static + hil::spi::SpiMaster,
        P: 'static + hil::gpio::Pin,
        A: 'static + hil::time::Alarm<'static>,
    > Mx25r6435fComponent<S, P, A>
{
    pub fn new(
        write_protect_pin: &'static P,
        hold_pin: &'static P,
        chip_select: S::ChipSelect,
        mux_alarm: &'static MuxAlarm<'static, A>,
        mux_spi: &'static MuxSpiMaster<'static, S>,
    ) -> Mx25r6435fComponent<S, P, A> {
        Mx25r6435fComponent {
            write_protect_pin,
            hold_pin,
            chip_select,
            mux_alarm,
            mux_spi,
        }
    }
}

impl<
        S: 'static + hil::spi::SpiMaster,
        P: 'static + hil::gpio::Pin,
        A: 'static + hil::time::Alarm<'static>,
    > Component for Mx25r6435fComponent<S, P, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            MX25R6435F<'static, VirtualSpiMasterDevice<'static, S>, P, VirtualMuxAlarm<'static, A>>,
        >,
    );
    type Output = &'static MX25R6435F<
        'static,
        VirtualSpiMasterDevice<'static, S>,
        P,
        VirtualMuxAlarm<'static, A>,
    >;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mx25r6435f_spi = static_init_half!(
            static_buffer.0,
            VirtualSpiMasterDevice<'static, S>,
            VirtualSpiMasterDevice::new(self.mux_spi, self.chip_select)
        );
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux_alarm)
        );

        let mx25r6435f = static_init_half!(
            static_buffer.2,
            capsules::mx25r6435f::MX25R6435F<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, S>,
                P,
                VirtualMuxAlarm<'static, A>,
            >,
            capsules::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                &mut capsules::mx25r6435f::TXBUFFER,
                &mut capsules::mx25r6435f::RXBUFFER,
                Some(self.write_protect_pin),
                Some(self.hold_pin)
            )
        );
        mx25r6435f_spi.set_client(mx25r6435f);
        mx25r6435f_virtual_alarm.set_alarm_client(mx25r6435f);
        mx25r6435f
    }
}
