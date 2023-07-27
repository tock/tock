// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the FXOS8700cq
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let fxos8700 = components::fxos8700::Fxos8700Component::new(mux_i2c, PinId::AdB1_00.get_pin().as_ref().unwrap())
//!    .finalize(components::fxos8700_component_static!());
//!
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!    .finalize(components::ninedof_component_static!(fxos8700));
//! ```

// Based on the component written for sam4l by:
// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/03/2020

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::fxos8700cq::Fxos8700cq;

use kernel::component::Component;

use core::mem::MaybeUninit;
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::i2c;

#[macro_export]
macro_rules! fxos8700_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<$I>);
        let buffer = kernel::static_buf!([u8; capsules_extra::fxos8700cq::BUF_LEN]);
        let fxo = kernel::static_buf!(capsules_extra::fxos8700cq::Fxos8700cq<'static>);

        (i2c_device, buffer, fxo)
    };};
}

pub struct Fxos8700Component<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    gpio: &'static dyn gpio::InterruptPin<'static>,
}

impl<I: 'static + i2c::I2CMaster<'static>> Fxos8700Component<I> {
    pub fn new<'a>(
        i2c: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        gpio: &'static dyn hil::gpio::InterruptPin<'static>,
    ) -> Fxos8700Component<I> {
        Fxos8700Component {
            i2c_mux: i2c,
            i2c_address,
            gpio: gpio,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Fxos8700Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; capsules_extra::fxos8700cq::BUF_LEN]>,
        &'static mut MaybeUninit<Fxos8700cq<'static>>,
    );
    type Output = &'static Fxos8700cq<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let fxos8700_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));
        let buffer = s.1.write([0; capsules_extra::fxos8700cq::BUF_LEN]);
        let fxos8700 = s.2.write(Fxos8700cq::new(fxos8700_i2c, self.gpio, buffer));

        fxos8700_i2c.set_client(fxos8700);
        self.gpio.set_client(fxos8700);

        fxos8700
    }
}
