//! Component for communicating with the RF233 (802.15.4) on imix boards.
//!
//! This provides one Component, RF233Component, which provides basic
//! packet-level interfaces for communicating with 802.15.4.
//!
//! Usage
//! -----
//! ```rust
//! let rf233 = RF233Component::new(rf233_spi,
//!                                 &sam4l::gpio::PA[09], // reset
//!                                 &sam4l::gpio::PA[10], // sleep
//!                                 &sam4l::gpio::PA[08], // irq
//!                                 &sam4l::gpio::PA[08]).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>

use capsules::rf233::RF233;
use capsules::virtual_spi::VirtualSpiMasterDevice;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init;

pub struct RF233Component {
    spi: &'static VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>,
    reset: &'static dyn hil::gpio::Pin,
    sleep: &'static dyn hil::gpio::Pin,
    irq: &'static dyn hil::gpio::InterruptPin,
    ctl: &'static sam4l::gpio::GPIOPin,
    channel: u8,
}

impl RF233Component {
    pub fn new(
        spi: &'static VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>,
        reset: &'static dyn hil::gpio::Pin,
        sleep: &'static dyn hil::gpio::Pin,
        irq: &'static dyn hil::gpio::InterruptPin,
        ctl: &'static sam4l::gpio::GPIOPin,
        channel: u8,
    ) -> RF233Component {
        RF233Component {
            spi: spi,
            reset: reset,
            sleep: sleep,
            irq: irq,
            ctl: ctl,
            channel: channel,
        }
    }
}

impl Component for RF233Component {
    type Output = &'static RF233<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let rf233: &RF233<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>> = static_init!(
            RF233<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>,
            RF233::new(self.spi, self.reset, self.sleep, self.irq, self.channel)
        );
        self.ctl.set_client(rf233);
        self.spi.set_client(rf233);
        rf233
    }
}
