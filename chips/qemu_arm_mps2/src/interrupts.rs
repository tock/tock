// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! NVIC external interrupt numbers for the ARM MPS2 AN385/AN386 FPGA images.
//!
//! Taken from QEMU's `hw/arm/mps2.c` (`mps2_common_init`), which is shared
//! by both FPGA images.
//!
//! The current `Mps2DefaultPeripherals` implementation drives only UART0,
//! TIMER0, and the Shield0 PL022.

pub const UART0_RX: u32 = 0;
pub const UART0_TX: u32 = 1;
pub const UART1_RX: u32 = 2;
pub const UART1_TX: u32 = 3;
pub const UART2_RX: u32 = 4;
pub const UART2_TX: u32 = 5;
pub const TIMER0: u32 = 8;
pub const TIMER1: u32 = 9;
pub const DUALTIMER: u32 = 10;
pub const UART3_RX: u32 = 18;
pub const UART3_TX: u32 = 19;
pub const UART4_RX: u32 = 20;
pub const UART4_TX: u32 = 21;
/// Shared by the Shield0 and Shield1 PL022 instances via an OR-gate; only
/// Shield0 is driven by this chip crate.
pub const SPI_SHIELD: u32 = 24;
