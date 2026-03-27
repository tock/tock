// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Register base addresses extracted from `boards/nucleo_u545re_q/STM32U545.svd`.
//!
//! These constants are the first step towards replacing the current stub
//! implementations with real register-level drivers.

// Non-secure peripheral base addresses
pub const RCC_BASE: usize = 0x4602_0C00;

pub const GPIOA_BASE: usize = 0x4202_0000;
pub const GPIOB_BASE: usize = 0x4202_0400;
pub const GPIOC_BASE: usize = 0x4202_0800;
pub const GPIOD_BASE: usize = 0x4202_0C00;
pub const GPIOE_BASE: usize = 0x4202_1000;
pub const GPIOG_BASE: usize = 0x4202_1800;
pub const GPIOH_BASE: usize = 0x4202_1C00;

pub const USART1_BASE: usize = 0x4001_3800;
pub const USART3_BASE: usize = 0x4000_4800;
pub const LPUART1_BASE: usize = 0x4600_2400;

// Useful interrupt numbers from SVD
pub const RCC_IRQ: u32 = 9;
pub const USART1_IRQ: u32 = 61;
pub const USART3_IRQ: u32 = 63;
pub const LPUART1_IRQ: u32 = 66;
