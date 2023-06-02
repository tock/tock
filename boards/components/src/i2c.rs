// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for I2C.
//!
//! This provides two components.
//!
//! 1. `I2CMuxComponent` provides a virtualization layer for a I2C bus.
//!
//! 2. `I2CComponent` provides a virtualized client to the I2C bus.
//!
//! Usage
//! -----
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f3xx::i2c::I2C1, None, dynamic_deferred_caller)
//!     .finalize(components::i2c_mux_component_static!());
//! let client_i2c = components::i2c::I2CComponent::new(mux_i2c, 0x19)
//!     .finalize(components::i2c_component_static!());
//! ```

// Author: Alexandru Radovici <msg4alex@gmail.com>

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, MuxI2C};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::i2c::{self, NoSMBus};

// Setup static space for the objects.
#[macro_export]
macro_rules! i2c_mux_component_static {
    ($I:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::MuxI2C<'static, $I>)
    };};
    ($I:ty, $S:ty $(,)?) => {{
        kernel::static_buf!(capsules::virtual_i2c::MuxI2C<'static, $I, $S>)
    };};
}

#[macro_export]
macro_rules! i2c_component_static {
    ($I:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>)
    };};
}

pub struct I2CMuxComponent<
    I: 'static + i2c::I2CMaster<'static>,
    S: 'static + i2c::SMBusMaster<'static> = NoSMBus,
> {
    i2c: &'static I,
    smbus: Option<&'static S>,
}

impl<I: 'static + i2c::I2CMaster<'static>, S: 'static + i2c::SMBusMaster<'static>>
    I2CMuxComponent<I, S>
{
    pub fn new(i2c: &'static I, smbus: Option<&'static S>) -> Self {
        I2CMuxComponent { i2c, smbus }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>, S: 'static + i2c::SMBusMaster<'static>> Component
    for I2CMuxComponent<I, S>
{
    type StaticInput = &'static mut MaybeUninit<MuxI2C<'static, I, S>>;
    type Output = &'static MuxI2C<'static, I, S>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mux_i2c = static_buffer.write(MuxI2C::new(self.i2c, self.smbus));
        kernel::deferred_call::DeferredCallClient::register(mux_i2c);

        self.i2c.set_master_client(mux_i2c);

        mux_i2c
    }
}

pub struct I2CComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    address: u8,
}

impl<I: 'static + i2c::I2CMaster<'static>> I2CComponent<I> {
    pub fn new(mux: &'static MuxI2C<'static, I>, address: u8) -> Self {
        I2CComponent {
            i2c_mux: mux,
            address: address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for I2CComponent<I> {
    type StaticInput = &'static mut MaybeUninit<I2CDevice<'static, I>>;
    type Output = &'static I2CDevice<'static, I>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let i2c_device = static_buffer.write(I2CDevice::new(self.i2c_mux, self.address));

        i2c_device
    }
}
