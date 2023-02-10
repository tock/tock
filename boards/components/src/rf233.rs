//! Component for communicating with an RF233 chip (802.15.4) connected via SPI.
//!
//! This provides one Component, RF233Component, which provides basic
//! packet-level interfaces for communicating with 802.15.4.
//!
//! Usage
//! -----
//! ```rust
//! let rf233 = components::rf233::RF233Component::new(
//!     rf233_spi,
//!     &peripherals.pa[09], // reset
//!     &peripherals.pa[10], // sleep
//!     &peripherals.pa[08], // irq
//!     &peripherals.pa[08],
//!     RADIO_CHANNEL,
//! )
//! .finalize(components::rf233_component_static!(sam4l::spi::SpiHw));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_spi::VirtualSpiMasterDevice;
use extra_capsules::rf233::RF233;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::spi::{SpiMaster, SpiMasterDevice};

// Setup static space for the objects.
#[macro_export]
macro_rules! rf233_component_static {
    ($S:ty $(,)?) => {{
        kernel::static_buf!(
            extra_capsules::rf233::RF233<
                'static,
                core_capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
            >
        )
    };};
}

pub struct RF233Component<S: SpiMaster + 'static> {
    spi: &'static VirtualSpiMasterDevice<'static, S>,
    reset: &'static dyn hil::gpio::Pin,
    sleep: &'static dyn hil::gpio::Pin,
    irq: &'static dyn hil::gpio::InterruptPin<'static>,
    ctl: &'static dyn hil::gpio::InterruptPin<'static>,
    channel: u8,
}

impl<S: SpiMaster + 'static> RF233Component<S> {
    pub fn new(
        spi: &'static VirtualSpiMasterDevice<'static, S>,
        reset: &'static dyn hil::gpio::Pin,
        sleep: &'static dyn hil::gpio::Pin,
        irq: &'static dyn hil::gpio::InterruptPin<'static>,
        ctl: &'static dyn hil::gpio::InterruptPin<'static>,
        channel: u8,
    ) -> Self {
        Self {
            spi,
            reset,
            sleep,
            irq,
            ctl,
            channel,
        }
    }
}

impl<S: SpiMaster + 'static> Component for RF233Component<S> {
    type StaticInput = &'static mut MaybeUninit<RF233<'static, VirtualSpiMasterDevice<'static, S>>>;
    type Output = &'static RF233<'static, VirtualSpiMasterDevice<'static, S>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let rf233 = s.write(RF233::new(
            self.spi,
            self.reset,
            self.sleep,
            self.irq,
            self.channel,
        ));
        self.ctl.set_client(rf233);
        self.spi.set_client(rf233);
        rf233
    }
}
