// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Peripheral base addresses for the ARM MPS2 AN385/AN386 FPGA images,
//! from QEMU's `hw/arm/mps2.c` (`mps2_common_init`).
//!
//! NOTE: This file is unique to the ARM MPS2 family, it is not a precedent
//! other chips should follow. The MPS2 chips are synthetic, simulation-
//! only designs that have the unique property of **by specification**
//! having identical peripheral maps and implementations---literally the
//! same HDL code. Generally, it is up to a specific instantiation of an
//! unsafe chip to assert the memory map and underlying peripherals. Only
//! in this unique case does it make sense to de-duplicate these definitions
//! to a shared crate, and only because the chips that rely on these shared
//! definitions **by specification and implementation** have literally
//! identical peripherals.

use cortexm::mpu::MpuRegisters;
use kernel::utilities::StaticRef;
use qemu_arm_mps2::led::FpgaioRegisters;
use qemu_arm_mps2::spi::SpiRegisters;
use qemu_arm_mps2::timer::TimerRegisters;
use qemu_arm_mps2::uart::UartRegisters;
use qemu_arm_mps2::watchdog::WatchdogRegisters;

pub const TIMER0_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_0000 as *const TimerRegisters) };
pub const TIMER1_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_1000 as *const TimerRegisters) };

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_4000 as *const UartRegisters) };
pub const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_5000 as *const UartRegisters) };
pub const UART2_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_6000 as *const UartRegisters) };
pub const UART3_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_7000 as *const UartRegisters) };
pub const UART4_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000_9000 as *const UartRegisters) };

pub const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x4000_8000 as *const WatchdogRegisters) };

pub const SPI_SHIELD0_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x4002_6000 as *const SpiRegisters) };

pub const FPGAIO_BASE: StaticRef<FpgaioRegisters> =
    unsafe { StaticRef::new(0x4002_8000 as *const FpgaioRegisters) };

/// The ARMv7-M Memory Protection Unit, in the Private Peripheral Bus.
///
/// This address is fixed by the architecture.
pub const MPU_BASE: StaticRef<MpuRegisters> =
    unsafe { StaticRef::new(0xE000_ED90 as *const MpuRegisters) };
