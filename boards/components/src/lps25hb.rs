// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for LPS25HB pressure sensor.

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::lps25hb::LPS25HB;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::i2c;

#[macro_export]
macro_rules! lps25hb_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static>);
        let lps25hb = kernel::static_buf!(
            capsules_extra::lps25hb::LPS25HB<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );
        let buffer = kernel::static_buf!([u8; capsules_extra::lps25hb::BUF_LEN]);

        (i2c_device, lps25hb, buffer)
    };};
}

pub struct Lps25hbComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<I: 'static + i2c::I2CMaster<'static>> Lps25hbComponent<I> {
    pub fn new(
        i2c_mux: &'static MuxI2C<'static, I>,
        i2c_address: u8,
        interrupt_pin: &'static dyn gpio::InterruptPin<'static>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Lps25hbComponent {
            i2c_mux,
            i2c_address,
            interrupt_pin,
            board_kernel,
            driver_num,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Lps25hbComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<LPS25HB<'static, I2CDevice<'static, I>>>,
        &'static mut MaybeUninit<[u8; capsules_extra::lps25hb::BUF_LEN]>,
    );
    type Output = &'static LPS25HB<'static, I2CDevice<'static, I>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let lps25hb_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let buffer = s.2.write([0; capsules_extra::lps25hb::BUF_LEN]);

        let lps25hb =
            s.1.write(LPS25HB::new(lps25hb_i2c, self.interrupt_pin, buffer, grant));
        lps25hb_i2c.set_client(lps25hb);
        self.interrupt_pin.set_client(lps25hb);

        lps25hb
    }
}
