// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Components for SPI.
//!
//! This provides four components.
//!
//! 1. `SpiMuxComponent` provides a virtualization layer for a SPI controller.
//! 2. `SpiSyscallComponent` provides a controller system call interface to SPI.
//! 3. `SpiPSyscallComponent` provides a peripheral system call interface to SPI.
//! 4. `SpiComponent` provides a virtualized client to the SPI bus.
//!
//! `SpiSyscallComponent` is used for processes, while `SpiComponent` is used
//! for kernel capsules that need access to the SPI bus.
//!
//! Usage
//! -----
//! ```rust
//! let mux_spi = components::spi::SpiMuxComponent::new(&sam4l::spi::SPI).finalize(
//!     components::spi_mux_component_static!(sam4l::spi::SpiHw));
//! let spi_syscalls = SpiSyscallComponent::new(mux_spi, 3).finalize(
//!     components::spi_syscalls_component_static!(sam4l::spi::SpiHw));
//! let rf233_spi = SpiComponent::new(mux_spi, 3).finalize(
//!     components::spi_component_static!(sam4l::spi::SpiHw));
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

use core::mem::MaybeUninit;

use capsules_core::spi_controller::{Spi, DEFAULT_READ_BUF_LENGTH, DEFAULT_WRITE_BUF_LENGTH};
use capsules_core::spi_peripheral::SpiPeripheral;
use capsules_core::virtualizers::virtual_spi;
use capsules_core::virtualizers::virtual_spi::{MuxSpiMaster, VirtualSpiMasterDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::spi;
use kernel::hil::spi::{SpiMasterDevice, SpiSlaveDevice};

// Setup static space for the objects.
#[macro_export]
macro_rules! spi_mux_component_static {
    ($S:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_spi::MuxSpiMaster<'static, $S>)
    };};
}

#[macro_export]
macro_rules! spi_syscall_component_static {
    ($S:ty $(,)?) => {{
        let virtual_spi = kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>
        );
        let spi = kernel::static_buf!(
            capsules_core::spi_controller::Spi<
                'static,
                capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>,
            >
        );

        let spi_read_buf =
            kernel::static_buf!([u8; capsules_core::spi_controller::DEFAULT_READ_BUF_LENGTH]);
        let spi_write_buf =
            kernel::static_buf!([u8; capsules_core::spi_controller::DEFAULT_WRITE_BUF_LENGTH]);

        (virtual_spi, spi, spi_read_buf, spi_write_buf)
    };};
}

#[macro_export]
macro_rules! spi_syscallp_component_static {
    ($S:ty $(,)?) => {{
        let spi_slave = kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::SpiSlaveDevice<'static, $S>
        );
        let spi_peripheral = kernel::static_buf!(
            capsules_core::spi_peripheral::SpiPeripheral<
                'static,
                capsules_core::virtualizers::virtual_spi::SpiSlaveDevice<'static, $S>,
            >
        );

        let spi_read_buf =
            kernel::static_buf!([u8; capsules_core::spi_controller::DEFAULT_READ_BUF_LENGTH]);
        let spi_write_buf =
            kernel::static_buf!([u8; capsules_core::spi_controller::DEFAULT_WRITE_BUF_LENGTH]);

        (spi_slave, spi_peripheral, spi_read_buf, spi_write_buf)
    };};
}

#[macro_export]
macro_rules! spi_component_static {
    ($S:ty $(,)?) => {{
        kernel::static_buf!(
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<'static, $S>
        )
    };};
}

#[macro_export]
macro_rules! spi_peripheral_component_static {
    ($S:ty $(,)?) => {{
        kernel::static_buf!(capsules_core::spi_peripheral::SpiPeripheral<'static, $S>)
    };};
}

pub struct SpiMuxComponent<S: 'static + spi::SpiMaster<'static>> {
    spi: &'static S,
}

pub struct SpiSyscallComponent<S: 'static + spi::SpiMaster<'static>> {
    board_kernel: &'static kernel::Kernel,
    spi_mux: &'static MuxSpiMaster<'static, S>,
    chip_select: S::ChipSelect,
    driver_num: usize,
}

pub struct SpiSyscallPComponent<S: 'static + spi::SpiSlave<'static>> {
    board_kernel: &'static kernel::Kernel,
    spi_slave: &'static S,
    driver_num: usize,
}

pub struct SpiComponent<S: 'static + spi::SpiMaster<'static>> {
    spi_mux: &'static MuxSpiMaster<'static, S>,
    chip_select: S::ChipSelect,
}

impl<S: 'static + spi::SpiMaster<'static>> SpiMuxComponent<S> {
    pub fn new(spi: &'static S) -> Self {
        Self { spi }
    }
}

impl<S: 'static + spi::SpiMaster<'static>> Component for SpiMuxComponent<S> {
    type StaticInput = &'static mut MaybeUninit<MuxSpiMaster<'static, S>>;
    type Output = &'static MuxSpiMaster<'static, S>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mux_spi = static_buffer.write(MuxSpiMaster::new(self.spi));
        kernel::deferred_call::DeferredCallClient::register(mux_spi);

        self.spi.set_client(mux_spi);

        if let Err(error) = self.spi.init() {
            panic!("SPI init failed ({:?})", error);
        }

        mux_spi
    }
}

impl<S: 'static + spi::SpiMaster<'static>> SpiSyscallComponent<S> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux: &'static MuxSpiMaster<'static, S>,
        chip_select: S::ChipSelect,
        driver_num: usize,
    ) -> Self {
        SpiSyscallComponent {
            board_kernel: board_kernel,
            spi_mux: mux,
            chip_select: chip_select,
            driver_num,
        }
    }
}

impl<S: 'static + spi::SpiMaster<'static>> Component for SpiSyscallComponent<S> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>,
        &'static mut MaybeUninit<Spi<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut MaybeUninit<[u8; DEFAULT_READ_BUF_LENGTH]>,
        &'static mut MaybeUninit<[u8; DEFAULT_WRITE_BUF_LENGTH]>,
    );
    type Output = &'static Spi<'static, VirtualSpiMasterDevice<'static, S>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let syscall_spi_device = static_buffer
            .0
            .write(VirtualSpiMasterDevice::new(self.spi_mux, self.chip_select));

        let spi_syscalls = static_buffer.1.write(Spi::new(
            syscall_spi_device,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        let spi_read_buf = static_buffer.2.write([0; DEFAULT_READ_BUF_LENGTH]);
        let spi_write_buf = static_buffer.3.write([0; DEFAULT_WRITE_BUF_LENGTH]);

        spi_syscalls.config_buffers(spi_read_buf, spi_write_buf);
        syscall_spi_device.setup();
        syscall_spi_device.set_client(spi_syscalls);
        spi_syscalls
    }
}

impl<S: 'static + spi::SpiSlave<'static>> SpiSyscallPComponent<S> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        slave: &'static S,
        driver_num: usize,
    ) -> Self {
        SpiSyscallPComponent {
            board_kernel,
            spi_slave: slave,
            driver_num,
        }
    }
}

impl<S: 'static + spi::SpiSlave<'static>> Component for SpiSyscallPComponent<S> {
    type StaticInput = (
        &'static mut MaybeUninit<virtual_spi::SpiSlaveDevice<'static, S>>,
        &'static mut MaybeUninit<SpiPeripheral<'static, virtual_spi::SpiSlaveDevice<'static, S>>>,
        &'static mut MaybeUninit<[u8; DEFAULT_READ_BUF_LENGTH]>,
        &'static mut MaybeUninit<[u8; DEFAULT_WRITE_BUF_LENGTH]>,
    );
    type Output = &'static SpiPeripheral<'static, virtual_spi::SpiSlaveDevice<'static, S>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let syscallp_spi_device = static_buffer
            .0
            .write(virtual_spi::SpiSlaveDevice::new(self.spi_slave));

        let spi_syscallsp = static_buffer.1.write(SpiPeripheral::new(
            syscallp_spi_device,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        let spi_read_buf = static_buffer.2.write([0; DEFAULT_READ_BUF_LENGTH]);
        let spi_write_buf = static_buffer.3.write([0; DEFAULT_WRITE_BUF_LENGTH]);

        spi_syscallsp.config_buffers(spi_read_buf, spi_write_buf);
        syscallp_spi_device.set_client(spi_syscallsp);

        spi_syscallsp
    }
}

impl<S: 'static + spi::SpiMaster<'static>> SpiComponent<S> {
    pub fn new(mux: &'static MuxSpiMaster<'static, S>, chip_select: S::ChipSelect) -> Self {
        SpiComponent {
            spi_mux: mux,
            chip_select: chip_select,
        }
    }
}

impl<S: 'static + spi::SpiMaster<'static>> Component for SpiComponent<S> {
    type StaticInput = &'static mut MaybeUninit<VirtualSpiMasterDevice<'static, S>>;
    type Output = &'static VirtualSpiMasterDevice<'static, S>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let spi_device =
            static_buffer.write(VirtualSpiMasterDevice::new(self.spi_mux, self.chip_select));
        spi_device.setup();
        spi_device
    }
}

pub struct SpiPeripheralComponent<S: 'static + spi::SpiSlave<'static>> {
    board_kernel: &'static kernel::Kernel,
    device: &'static S,
    driver_num: usize,
}

impl<S: 'static + spi::SpiSlave<'static>> SpiPeripheralComponent<S> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        device: &'static S,
        driver_num: usize,
    ) -> Self {
        SpiPeripheralComponent {
            board_kernel,
            device,
            driver_num,
        }
    }
}

impl<S: 'static + spi::SpiSlave<'static> + kernel::hil::spi::SpiSlaveDevice<'static>> Component
    for SpiPeripheralComponent<S>
{
    type StaticInput = &'static mut MaybeUninit<SpiPeripheral<'static, S>>;
    type Output = &'static SpiPeripheral<'static, S>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let spi_device = static_buffer.write(SpiPeripheral::new(
            self.device,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        spi_device
    }
}
