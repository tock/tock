//! Named interrupts for the qemu-system-riscv32 virt machine.

#![allow(dead_code)]

pub const VIRTIO_MMIO_0: u32 = 1;
pub const VIRTIO_MMIO_1: u32 = 2;
pub const VIRTIO_MMIO_2: u32 = 3;
pub const VIRTIO_MMIO_3: u32 = 4;
pub const VIRTIO_MMIO_4: u32 = 5;
pub const VIRTIO_MMIO_5: u32 = 6;
pub const VIRTIO_MMIO_6: u32 = 7;
pub const VIRTIO_MMIO_7: u32 = 8;

pub const UART0: u32 = 10;
pub const RTC: u32 = 11;
