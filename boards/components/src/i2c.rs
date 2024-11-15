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

use capsules_core::virtualizers::virtual_i2c::{I2CDevice, I2CMultiDevice, MuxI2C};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
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

#[macro_export]
macro_rules! i2c_master_slave_component_static {
    ($I:ty $(,)?) => {{
        let i2c_master_buffer = kernel::static_buf!([u8; 32]);
        let i2c_slave_buffer1 = kernel::static_buf!([u8; 32]);
        let i2c_slave_buffer2 = kernel::static_buf!([u8; 32]);

        let driver = kernel::static_buf!(
            capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver<'static, $I>
        );

        (
            driver,
            i2c_master_buffer,
            i2c_slave_buffer1,
            i2c_slave_buffer2,
        )
    };};
}

#[macro_export]
macro_rules! i2c_master_component_static {
    ($I:ty $(,)?) => {{
        let i2c_master_buffer = kernel::static_buf!([u8; capsules_core::i2c_master::BUFFER_LENGTH]);
        let i2c_device = kernel::static_buf!(
            capsules_core::virtualizers::virtual_i2c::I2CMultiDevice<'static, $I>
        );

        let driver = kernel::static_buf!(
            capsules_core::i2c_master::I2CMasterDriver<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CMultiDevice<'static, $I>,
            >
        );

        (i2c_device, i2c_master_buffer, driver)
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
            address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for I2CComponent<I> {
    type StaticInput = &'static mut MaybeUninit<I2CDevice<'static, I>>;
    type Output = &'static I2CDevice<'static, I>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let i2c_device = static_buffer.write(I2CDevice::<I>::new(self.i2c_mux, self.address));

        i2c_device
    }
}

pub struct I2CMasterSlaveDriverComponent<I: 'static + i2c::I2CMasterSlave<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    i2c: &'static I,
}

impl<I: 'static + i2c::I2CMasterSlave<'static>> I2CMasterSlaveDriverComponent<I> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, i2c: &'static I) -> Self {
        I2CMasterSlaveDriverComponent {
            board_kernel,
            driver_num,
            i2c,
        }
    }
}

impl<I: 'static + i2c::I2CMasterSlave<'static>> Component for I2CMasterSlaveDriverComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<
            capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver<'static, I>,
        >,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 32]>,
    );
    type Output = &'static capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver<'static, I>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let i2c_master_buffer = static_buffer.1.write([0; 32]);
        let i2c_slave_buffer1 = static_buffer.2.write([0; 32]);
        let i2c_slave_buffer2 = static_buffer.3.write([0; 32]);

        let i2c_master_slave_driver = static_buffer.0.write(
            capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver::new(
                self.i2c,
                i2c_master_buffer,
                i2c_slave_buffer1,
                i2c_slave_buffer2,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            ),
        );

        self.i2c.set_master_client(i2c_master_slave_driver);
        self.i2c.set_slave_client(i2c_master_slave_driver);

        i2c_master_slave_driver
    }
}

pub type I2CMasterDriverComponentType<I> =
    capsules_core::i2c_master::I2CMasterDriver<'static, I2CDevice<'static, I>>;

pub struct I2CMasterDriverComponent<I: 'static + i2c::I2CMaster<'static>> {
    i2c_mux: &'static MuxI2C<'static, I>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<I: 'static + i2c::I2CMaster<'static>> I2CMasterDriverComponent<I> {
    pub fn new(
        i2c: &'static MuxI2C<'static, I>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        I2CMasterDriverComponent {
            i2c_mux: i2c,
            board_kernel,
            driver_num,
        }
    }
}

impl<I: 'static + i2c::I2CMaster<'static>> Component for I2CMasterDriverComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<I2CMultiDevice<'static, I>>,
        &'static mut MaybeUninit<[u8; capsules_core::i2c_master::BUFFER_LENGTH]>,
        &'static mut MaybeUninit<
            capsules_core::i2c_master::I2CMasterDriver<'static, I2CMultiDevice<'static, I>>,
        >,
    );
    type Output =
        &'static capsules_core::i2c_master::I2CMasterDriver<'static, I2CMultiDevice<'static, I>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let userspace_device = static_buffer.0.write(I2CMultiDevice::new(self.i2c_mux));
        let i2c_master_buffer = static_buffer
            .1
            .write([0; capsules_core::i2c_master::BUFFER_LENGTH]);

        let i2c_master = static_buffer
            .2
            .write(capsules_core::i2c_master::I2CMasterDriver::new(
                userspace_device,
                i2c_master_buffer,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            ));

        userspace_device.set_client(i2c_master);

        i2c_master
    }
}
