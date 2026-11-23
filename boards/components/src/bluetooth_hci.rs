// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for a userspace Bluetooth HCI driver.
//!
//! Usage
//! -----
//! ```rust
//! let hci = components::bluetooth_hci::BluetoothHciComponent::new(
//!     board_kernel,
//!     capsules_extra::bluetooth_hci::DRIVER_NUM,
//!     transport,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! )
//! .finalize(components::bluetooth_hci_component_static!(TransportType));
//! ```

use capsules_extra::bluetooth_hci::BluetoothHci;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil::bluetooth_hci::{HciTransport, MAX_PACKET_LEN};

#[macro_export]
macro_rules! bluetooth_hci_component_static {
    ($T:ty $(,)?) => {{
        let driver = kernel::static_buf!(capsules_extra::bluetooth_hci::BluetoothHci<'static, $T>);
        let tx_buffer = kernel::static_buf!([u8; kernel::hil::bluetooth_hci::MAX_PACKET_LEN]);
        (driver, tx_buffer)
    }};
}

pub type BluetoothHciComponentType<T> = BluetoothHci<'static, T>;

pub struct BluetoothHciComponent<
    T: HciTransport<'static> + 'static,
    CAP: MemoryAllocationCapability,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    transport: &'static T,
    mem_cap: CAP,
}

impl<T: HciTransport<'static>, CAP: MemoryAllocationCapability> BluetoothHciComponent<T, CAP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        transport: &'static T,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            transport,
            mem_cap,
        }
    }
}

impl<T: HciTransport<'static>, CAP: MemoryAllocationCapability> Component
    for BluetoothHciComponent<T, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<BluetoothHci<'static, T>>,
        &'static mut MaybeUninit<[u8; MAX_PACKET_LEN]>,
    );
    type Output = &'static BluetoothHci<'static, T>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let tx_buffer = static_buffer.1.write([0; MAX_PACKET_LEN]);
        let driver = static_buffer.0.write(BluetoothHci::new(
            self.transport,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            tx_buffer,
        ));
        self.transport.set_client(driver);
        driver
    }
}
