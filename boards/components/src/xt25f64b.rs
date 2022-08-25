//! Component for the XT25F64B block storage chip.
//!
//! Usage
//! -----
//! ```rust
//! let xt25f64b = components::xt25f64b::Xt25f64bComponent::new(
//!     &gpio_port[driver.write_protect_pin],
//!     &gpio_port[driver.hold_pin],
//!     &gpio_port[driver.chip_select] as &dyn kernel::hil::gpio::Pin,
//!     mux_alarm,
//!     mux_spi,
//! )
//! .finalize(components::xt25f64b_component_helper!(
//!     nrf52::spi::SPIM,
//!     nrf52::gpio::GPIOPin,
//!     nrf52::rtc::Rtc,
//! ));
//! ```
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use capsules::xt25f64b::XT25F64B;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::spi::SpiMasterDevice;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! xt25f64b_component_helper {
    ($S:ty, $P:ty, $A:ty $(,)?) => {{
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use capsules::xt25f64b::XT25F64B;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualSpiMasterDevice<'static, $S>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF3: MaybeUninit<
            XT25F64B<
                'static,
                VirtualSpiMasterDevice<'static, $S>,
                $P,
                VirtualMuxAlarm<'static, $A>,
            >,
        > = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub struct Xt25f64bComponent<
    S: 'static + hil::spi::SpiMaster,
    P: 'static + hil::gpio::Pin,
    A: 'static + hil::time::Alarm<'static>,
> {
    write_protect_pin: Option<&'static P>,
    hold_pin: Option<&'static P>,
    chip_select: S::ChipSelect,
    mux_alarm: &'static MuxAlarm<'static, A>,
    mux_spi: &'static MuxSpiMaster<'static, S>,
}

impl<
        S: 'static + hil::spi::SpiMaster,
        P: 'static + hil::gpio::Pin,
        A: 'static + hil::time::Alarm<'static>,
    > Xt25f64bComponent<S, P, A>
{
    pub fn new(
        write_protect_pin: Option<&'static P>,
        hold_pin: Option<&'static P>,
        chip_select: S::ChipSelect,
        mux_alarm: &'static MuxAlarm<'static, A>,
        mux_spi: &'static MuxSpiMaster<'static, S>,
    ) -> Xt25f64bComponent<S, P, A> {
        Xt25f64bComponent {
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
    > Component for Xt25f64bComponent<S, P, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            XT25F64B<'static, VirtualSpiMasterDevice<'static, S>, P, VirtualMuxAlarm<'static, A>>,
        >,
    );
    type Output = &'static XT25F64B<
        'static,
        VirtualSpiMasterDevice<'static, S>,
        P,
        VirtualMuxAlarm<'static, A>,
    >;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let xt25f64b_spi = static_init_half!(
            static_buffer.0,
            VirtualSpiMasterDevice<'static, S>,
            VirtualSpiMasterDevice::new(self.mux_spi, self.chip_select)
        );
        // Create an alarm for this chip.
        let xt25f64b_virtual_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.mux_alarm)
        );
        xt25f64b_virtual_alarm.setup();

        let xt25f64b = static_init_half!(
            static_buffer.2,
            capsules::xt25f64b::XT25F64B<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, S>,
                P,
                VirtualMuxAlarm<'static, A>,
            >,
            capsules::xt25f64b::XT25F64B::new(
                xt25f64b_spi,
                xt25f64b_virtual_alarm,
                &mut capsules::xt25f64b::TXBUFFER,
                &mut capsules::xt25f64b::RXBUFFER,
                self.write_protect_pin,
                self.hold_pin,
            )
        );
        xt25f64b_spi.setup();
        xt25f64b_spi.set_client(xt25f64b);
        xt25f64b_virtual_alarm.set_alarm_client(xt25f64b);
        xt25f64b
    }
}
