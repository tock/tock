// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! QEMU VirtIO MMIO instantiation

use kernel::utilities::StaticRef;
use virtio::transports::mmio::VirtIOMMIODeviceRegisters;

pub const VIRTIO_MMIO_0_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_1000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_1_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_2000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_2_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_3000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_3_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_4000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_4_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_5000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_5_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_6000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_6_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_7000 as *const VirtIOMMIODeviceRegisters) };
pub const VIRTIO_MMIO_7_BASE: StaticRef<VirtIOMMIODeviceRegisters> =
    unsafe { StaticRef::new(0x1000_8000 as *const VirtIOMMIODeviceRegisters) };
