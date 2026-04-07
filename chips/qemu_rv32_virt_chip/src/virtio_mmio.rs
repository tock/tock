// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! QEMU VirtIO MMIO instantiation

use tock_registers::Mmio32;

type Registers = virtio::transports::mmio::virtio_mmio_device_registers::Real<Mmio32>;

pub const VIRTIO_MMIO_0_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_1000)) };
pub const VIRTIO_MMIO_1_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_2000)) };
pub const VIRTIO_MMIO_2_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_3000)) };
pub const VIRTIO_MMIO_3_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_4000)) };
pub const VIRTIO_MMIO_4_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_5000)) };
pub const VIRTIO_MMIO_5_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_6000)) };
pub const VIRTIO_MMIO_6_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_7000)) };
pub const VIRTIO_MMIO_7_BASE: Registers = unsafe { Registers::new(Mmio32::with_addr(0x1000_8000)) };
