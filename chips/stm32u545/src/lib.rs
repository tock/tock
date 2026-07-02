// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

#![no_std]

pub use stm32u5xx::{adc, chip, dma, exti, gpio, pwr, rcc, rtc, tim, usart};

use cortexm33::{CortexM33, CortexMVariant};

#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// Link to the STM32U5 series reference manual (RM0456):
// Table 186 "STM32U5 series vector table"
// https://www.st.com/resource/en/reference_manual/rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf
// Link to the STM32U545RE datasheet confirming 141 maskable interrupt channels:
// https://www.st.com/resource/en/datasheet/stm32u545re.pdf (Section 3.19.1)
pub static IRQS: [unsafe extern "C" fn(); 141] = [<CortexM33 as CortexMVariant>::GENERIC_ISR; 141];
