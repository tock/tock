//! Components for the FXOS8700cq
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let fxos8700 = components::fxos8700::Fxos8700Component::new(mux_i2c, PinId::AdB1_00.get_pin().as_ref().unwrap())
//!    .finalize(());
//!
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!    .finalize(components::ninedof_component_helper!(fxos8700));
//! ```

// Based on the component written for sam4l by:
// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/03/2020

use capsules::fxos8700cq;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};

use kernel::component::Component;

use kernel::hil;
use kernel::hil::gpio;
use kernel::static_init;

pub struct Fxos8700Component {
    i2c_mux: &'static MuxI2C<'static>,
    gpio: &'static dyn gpio::InterruptPin<'static>,
}

impl Fxos8700Component {
    pub fn new<'a>(
        i2c: &'static MuxI2C<'static>,
        gpio: &'static dyn hil::gpio::InterruptPin<'static>,
    ) -> Fxos8700Component {
        Fxos8700Component {
            i2c_mux: i2c,
            gpio: gpio,
        }
    }
}

impl Component for Fxos8700Component {
    type StaticInput = ();
    type Output = &'static fxos8700cq::Fxos8700cq<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x1f));
        let fxos8700 = static_init!(
            fxos8700cq::Fxos8700cq<'static>,
            fxos8700cq::Fxos8700cq::new(fxos8700_i2c, self.gpio, &mut fxos8700cq::BUF)
        );
        fxos8700_i2c.set_client(fxos8700);
        self.gpio.set_client(fxos8700);
        fxos8700
    }
}
