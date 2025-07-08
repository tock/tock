// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for the SSD1306 OLED screen.
//!
//! Usage
//! -----
//! ```rust
//!
//! let ssd1306_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
//!     .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let ssd1306 = components::ssd1306::Ssd1306Component::new(ssd1306_i2c, true)
//!     .finalize(components::ssd1306_component_static!(nrf52840::i2c::TWI));
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;

// Setup static space for the objects.
#[macro_export]
macro_rules! ssd1306_component_static {
    ($I: ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::ssd1306::BUFFER_SIZE]);
        let ssd1306 = kernel::static_buf!(
            capsules_extra::ssd1306::Ssd1306<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );

        (buffer, ssd1306)
    };};
}

pub type Ssd1306ComponentType<I> = capsules_extra::ssd1306::Ssd1306<
    'static,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
>;

pub struct Ssd1306Component<I: hil::i2c::I2CMaster<'static> + 'static> {
    i2c_device: &'static capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
    use_charge_pump: bool,
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Ssd1306Component<I> {
    pub fn new(
        i2c_device: &'static capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
        use_charge_pump: bool,
    ) -> Ssd1306Component<I> {
        Ssd1306Component {
            i2c_device,
            use_charge_pump,
        }
    }
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Component for Ssd1306Component<I> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::ssd1306::BUFFER_SIZE]>,
        &'static mut MaybeUninit<
            capsules_extra::ssd1306::Ssd1306<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
            >,
        >,
    );
    type Output = &'static capsules_extra::ssd1306::Ssd1306<
        'static,
        capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, I>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .0
            .write([0; capsules_extra::ssd1306::BUFFER_SIZE]);

        let ssd1306 = static_buffer.1.write(capsules_extra::ssd1306::Ssd1306::new(
            self.i2c_device,
            buffer,
            self.use_charge_pump,
        ));
        self.i2c_device.set_client(ssd1306);

        ssd1306
    }
}
