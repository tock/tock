// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! The `unsafe` code for the ARM MPS2 AN385/AN386 FPGA images under QEMU;
//! the peripheral drivers are in `qemu_arm_mps2`.

#![no_std]

pub mod addresses;
pub mod chip;

use kernel::collections::ring_buffer::RingBuffer;
use kernel::utilities::slice_uninit::mut_slice_as_maybeuninit;
use qemu_arm_mps2::Mps2DefaultPeripherals;
use qemu_arm_mps2::uart::RING_BUFFER_SIZE_BYTE;

/// Binds each driver to its register block on this machine.
///
/// # Safety
///
/// Must only be called once, as each driver takes ownership of its
/// peripheral's registers.
pub unsafe fn default_peripherals() -> Mps2DefaultPeripherals<'static> {
    let uart0_ring_buffer_bytes =
        kernel::static_init!([u8; RING_BUFFER_SIZE_BYTE], [0; RING_BUFFER_SIZE_BYTE]);
    let uart0_ring_buffer = kernel::static_init!(
        RingBuffer<'static, u8>,
        RingBuffer::new(mut_slice_as_maybeuninit(uart0_ring_buffer_bytes))
    );

    Mps2DefaultPeripherals::new(
        addresses::UART0_BASE,
        addresses::TIMER0_BASE,
        addresses::FPGAIO_BASE,
        addresses::SPI_SHIELD0_BASE,
        addresses::WATCHDOG_BASE,
        uart0_ring_buffer,
    )
}
