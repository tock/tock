// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

//! Named constants for NVIC ids shared across the stm32u5xx family of chips

#![allow(non_upper_case_globals)]

pub const EXTI13_IRQ: u32 = 24;
pub const GPDMA1_CH0_IRQ: u32 = 29;
pub const GPDMA1_CH1_IRQ: u32 = 30;
pub const TIM2_IRQ: u32 = 45;
pub const USART1_IRQ: u32 = 61;
