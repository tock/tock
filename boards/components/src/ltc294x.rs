// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for LPS25HB pressure sensor.
//!
//! Usage
//! -----
//!
//! ```rust
//! let ltc294x = components::Ltc294xComponent::new(i2c_mux, 0x64, None)
//!     .finalize(components::ltc294x_component_static!());
//! let ltc294x_driver = components::Ltc294xDriverComponent::new(ltc294x, board_kernel, DRIVER_NUM)
//!     .finalize(components::ltc294x_driver_component_static!());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::ltc294x::LTC294XDriver;
use capsules_extra::ltc294x::LTC294X;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::i2c;

#[macro_export]
macro_rules! ltc294x_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let ltc294x = kernel::static_buf!(
            capsules_extra::ltc294x::LTC294X<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );
        let buffer = kernel::static_buf!([u8; capsules_extra::ltc294x::BUF_LEN]);

        (i2c_device, ltc294x, buffer)
    };};
}

#[macro_export]
macro_rules! ltc294x_driver_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::ltc294x::LTC294XDriver<'static>)
    };};
}

pub struct Ltc294xComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    interrupt_pin: Option<&'static dyn gpio::InterruptPin<'static>>,
}

impl<I: 'static + i2c::I2CMaster<'static>> Ltc294xComponent<I> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        interrupt_pin: Option<&'static dyn gpio::InterruptPin<'static>>,
    ) -> Self {
        Ltc294xComponent {
            i2c_mux,
            i2c_address,
            interrupt_pin,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Ltc294xComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<LTC294X<'static, I2CDevice<'static, I>>>,
        &'static mut MaybeUninit<[u8; capsules_extra::ltc294x::BUF_LEN]>,
    );
    type Output = &'static LTC294X<'static, I2CDevice<'static, I>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let ltc294x_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = s.2.write([0; capsules_extra::ltc294x::BUF_LEN]);

        let ltc294x =
            s.1.write(LTC294X::new(ltc294x_i2c, self.interrupt_pin, buffer));
        ltc294x_i2c.set_client(ltc294x);
        self.interrupt_pin.map(|pin| {
            pin.set_client(ltc294x);
        });

        ltc294x
    }
}

pub struct Ltc294xDriverComponent<I: 'static + i2c::I2CMaster<'static>> {
    ltc294x: &'static LTC294X<'static, I2CDevice<'static, I>>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<I: 'static + i2c::I2CMaster<'static>> Ltc294xDriverComponent<I> {
    pub fn new(
        ltc294x: &'static LTC294X<'static, I2CDevice<'static, I>>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Ltc294xDriverComponent {
            ltc294x,
            board_kernel,
            driver_num,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Ltc294xDriverComponent<I> {
    type StaticInput = &'static mut MaybeUninit<LTC294XDriver<'static, I2CDevice<'static, I>>>;
    type Output = &'static LTC294XDriver<'static, I2CDevice<'static, I>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let ltc294x_driver = s.write(LTC294XDriver::new(self.ltc294x, grant));
        self.ltc294x.set_client(ltc294x_driver);

        ltc294x_driver
    }
}
