//! Components for the FXOS8700  on the imix board.
//!
//! This provides two Components. Fxos8700Component provides a kernel
//! implementation of the Fxos8700 over I2C, while NineDofComponent
//! provides a system call interface to the sensor. Note that only one
//! of these components should be allocated, as they use the same
//! static buffer: NineDofComponent instantiations a
//! Fxos8700Component, so if your code creates both components, then
//! there will be two Fxos8700Component instances conflicting on the
//! buffer.
//!
//! Usage
//! -----
//! ```rust
//! let ninedof = NineDofComponent::new(mux_i2c, &sam4l::gpio::PC[13]).finalize();
//! let fxos8700 = Fxos8700Component::new(mux_i2c, &sam4l::gpio::PC[13]).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included
#![allow(unused_imports)] // I2CDevice

use capsules::fxos8700cq;
use capsules::ninedof::NineDof;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use hil;
use kernel::component::Component;
use kernel::Grant;
use sam4l;

pub struct Fxos8700Component {
    i2c_mux: &'static MuxI2C<'static>,
    gpio: &'static sam4l::gpio::GPIOPin,
}

impl Fxos8700Component {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        gpio: &'static sam4l::gpio::GPIOPin,
    ) -> Fxos8700Component {
        Fxos8700Component {
            i2c_mux: i2c,
            gpio: gpio,
        }
    }
}

impl Component for Fxos8700Component {
    type Output = &'static fxos8700cq::Fxos8700cq<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x1e));
        let fxos8700 = static_init!(
            fxos8700cq::Fxos8700cq<'static>,
            fxos8700cq::Fxos8700cq::new(fxos8700_i2c, self.gpio, &mut fxos8700cq::BUF)
        );
        fxos8700_i2c.set_client(fxos8700);
        self.gpio.set_client(fxos8700);
        fxos8700
    }
}

pub struct NineDofComponent {
    i2c_mux: &'static MuxI2C<'static>,
    gpio: &'static sam4l::gpio::GPIOPin,
}

impl NineDofComponent {
    pub fn new(
        i2c: &'static MuxI2C<'static>,
        gpio: &'static sam4l::gpio::GPIOPin,
    ) -> NineDofComponent {
        NineDofComponent {
            i2c_mux: i2c,
            gpio: gpio,
        }
    }
}

impl Component for NineDofComponent {
    type Output = &'static NineDof<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(self.i2c_mux, 0x1e));
        let fxos8700 = static_init!(
            fxos8700cq::Fxos8700cq<'static>,
            fxos8700cq::Fxos8700cq::new(fxos8700_i2c, self.gpio, &mut fxos8700cq::BUF)
        );
        fxos8700_i2c.set_client(fxos8700);
        self.gpio.set_client(fxos8700);

        let ninedof = static_init!(NineDof<'static>, NineDof::new(fxos8700, Grant::create()));
        hil::sensors::NineDof::set_client(fxos8700, ninedof);

        ninedof
    }
}
