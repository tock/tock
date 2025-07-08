// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for the SH1106 OLED screen.
//!
//! Usage
//! -----
//! ```rust
//!
//! let oled_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
//!     .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let sh1106 = components::sh1106::Sh1106Component::new(oled_i2c, true)
//!     .finalize(components::sh1106_component_static!(nrf52840::i2c::TWI));
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! sh1106_component_static {
    ($I: ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::sh1106::BUFFER_SIZE]);
        let sh1106 = kernel::static_buf!(
            capsules_extra::sh1106::Sh1106<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );

        (buffer, sh1106)
    };};
}

pub type Sh1106ComponentType<I> = capsules_extra::sh1106::Sh1106<
    'static,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
>;

pub struct Sh1106Component<I: hil::i2c::I2CMaster<'static> + 'static> {
    i2c_device: &'static capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
    use_charge_pump: bool,
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Sh1106Component<I> {
    pub fn new(
        i2c_device: &'static capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
        use_charge_pump: bool,
    ) -> Sh1106Component<I> {
        Sh1106Component {
            i2c_device,
            use_charge_pump,
        }
    }
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Component for Sh1106Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::sh1106::BUFFER_SIZE]>,
        &'static mut MaybeUninit<
            capsules_extra::sh1106::Sh1106<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
            >,
        >,
    );
    type Output = &'static capsules_extra::sh1106::Sh1106<
        'static,
        capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .0
            .write([0; capsules_extra::sh1106::BUFFER_SIZE]);

        let sh1106 = static_buffer.1.write(capsules_extra::sh1106::Sh1106::new(
            self.i2c_device,
            buffer,
            self.use_charge_pump,
        ));
        self.i2c_device.set_client(sh1106);

        sh1106
    }
}
