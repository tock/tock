// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for the ATECC508A CryptoAuthentication Device.
//!
//! Usage
//! -----
//! ```rust
//!     let atecc508a =
//!         Atecc508aComponent::new(mux_i2c, 0x60).finalize(components::atecc508a_component_static!());
//! ```

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use capsules_extra::atecc508a::Atecc508a;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! atecc508a_component_static {
    ($I:ty $(,)?) => {{
        let i2c_device =
            kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>);
        let i2c_buffer = kernel::static_buf!([u8; 140]);
        let entropy_buffer = kernel::static_buf!([u8; 32]);
        let digest_buffer = kernel::static_buf!([u8; 64]);
        let verify_key_buffer = kernel::static_buf!([u8; 64]);
        let atecc508a = kernel::static_buf!(capsules_extra::atecc508a::Atecc508a<'static>);

        (
            i2c_device,
            i2c_buffer,
            entropy_buffer,
            digest_buffer,
            verify_key_buffer,
            atecc508a,
        )
    };};
}

pub struct Atecc508aComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    i2c_address: u8,
    wakeup_device: fn(),
}

impl<I: 'static + i2c::I2CMaster<'static>> Atecc508aComponent<I> {
    pub fn new(i2c: &'static MuxI2C<'static, I>, i2c_address: u8, wakeup_device: fn()) -> Self {
        Atecc508aComponent {
            i2c_mux: i2c,
            i2c_address,
            wakeup_device,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for Atecc508aComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; 140]>,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<Atecc508a<'static>>,
    );
    type Output = &'static Atecc508a<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let atecc508a_i2c = s.0.write(I2CDevice::new(self.i2c_mux, self.i2c_address));

        let i2c_buffer = s.1.write([0; 140]);
        let entropy_buffer = s.2.write([0; 32]);
        let digest_buffer = s.3.write([0; 64]);
        let verify_key_buffer = s.4.write([0; 64]);

        let atecc508a = s.5.write(Atecc508a::new(
            atecc508a_i2c,
            i2c_buffer,
            entropy_buffer,
            digest_buffer,
            self.wakeup_device,
        ));
        atecc508a.set_public_key(Some(verify_key_buffer));

        atecc508a_i2c.set_client(atecc508a);
        atecc508a
    }
}
