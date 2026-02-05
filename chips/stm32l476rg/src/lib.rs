// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

#![no_std]

pub use stm32l4xx::{chip, clocks, exti, flash, gpio, nvic, rcc, syscfg, usart};

pub mod chip_specs;
pub mod interrupt_service;
//pub mod stm32l476rg_nvic;

use cortexm4f::{unhandled_interrupt, CortexM4F, CortexMVariant};

// STM32L476rg has total of 82 interrupts
// Extracted from `CMSIS/Device/ST/STM32L4xx/Include/stm32l476xx.h`
// NOTE: There are missing IRQn between 0 and 81
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// `used` ensures that the symbol is kept until the final binary. However, as of
// May 2020, due to the compilation process, there must be some other compiled
// code here to make sure the object file is kept around. That means at minimum
// there must be an `init()` function here so that compiler does not just ignore
// the `IRQS` object. See https://github.com/rust-lang/rust/issues/56639 for a
// related discussion.
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 82] = [
    CortexM4F::GENERIC_ISR, // Window WatchDog Interrupt (0)
    CortexM4F::GENERIC_ISR, // PVD/PVM1/PVM2/PVM3/PVM4 through EXTI Line detection Interrupts (1)
    CortexM4F::GENERIC_ISR, // Tamper and TimeStamp interrupts through the EXTI line (2)
    CortexM4F::GENERIC_ISR, // RTC Wakeup interrupt through the EXTI line (3)
    CortexM4F::GENERIC_ISR, // FLASH global Interrupt (4)
    CortexM4F::GENERIC_ISR, // RCC global Interrupt (5)
    CortexM4F::GENERIC_ISR, // EXTI Line0 Interrupt (6)
    CortexM4F::GENERIC_ISR, // EXTI Line1 Interrupt (7)
    CortexM4F::GENERIC_ISR, // EXTI Line2 Interrupt (8)
    CortexM4F::GENERIC_ISR, // EXTI Line3 Interrupt (9)
    CortexM4F::GENERIC_ISR, // EXTI Line4 Interrupt (10)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 1 global Interrupt (11)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 2 global Interrupt (12)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 3 global Interrupt (13)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 4 global Interrupt (14)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 5 global Interrupt (15)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 6 global Interrupt (16)
    CortexM4F::GENERIC_ISR, // DMA1 Channel 7 global Interrupt (17)
    CortexM4F::GENERIC_ISR, // ADC1, ADC2 SAR global Interrupts (18)
    CortexM4F::GENERIC_ISR, // CAN1 TX Interrupt (19)
    CortexM4F::GENERIC_ISR, // CAN1 RX0 Interrupt (20)
    CortexM4F::GENERIC_ISR, // CAN1 RX1 Interrupt (21)
    CortexM4F::GENERIC_ISR, // CAN1 SCE Interrupt (22)
    CortexM4F::GENERIC_ISR, // External Line[9:5] Interrupts (23)
    CortexM4F::GENERIC_ISR, // TIM1 Break interrupt and TIM15 global interrupt (24)
    CortexM4F::GENERIC_ISR, // TIM1 Update Interrupt and TIM16 global interrupt (25)
    CortexM4F::GENERIC_ISR, // TIM1 Trigger and Commutation Interrupt and TIM17 global interrupt (26)
    CortexM4F::GENERIC_ISR, // TIM1 Capture Compare Interrupt (27)
    CortexM4F::GENERIC_ISR, // TIM2 global Interrupt (28)
    CortexM4F::GENERIC_ISR, // TIM3 global Interrupt (29)
    CortexM4F::GENERIC_ISR, // TIM4 global Interrupt (30)
    CortexM4F::GENERIC_ISR, // I2C1 Event Interrupt (31)
    CortexM4F::GENERIC_ISR, // I2C1 Error Interrupt (32)
    CortexM4F::GENERIC_ISR, // I2C2 Event Interrupt (33)
    CortexM4F::GENERIC_ISR, // I2C2 Error Interrupt (34)
    CortexM4F::GENERIC_ISR, // SPI1 global Interrupt (35)
    CortexM4F::GENERIC_ISR, // SPI2 global Interrupt (36)
    CortexM4F::GENERIC_ISR, // USART1 global Interrupt (37)
    CortexM4F::GENERIC_ISR, // USART2 global Interrupt (38)
    CortexM4F::GENERIC_ISR, // USART3 global Interrupt (39)
    CortexM4F::GENERIC_ISR, // External Line[15:10] Interrupts (40)
    CortexM4F::GENERIC_ISR, // RTC Alarm (A and B) through EXTI Line Interrupt (41)
    CortexM4F::GENERIC_ISR, // DFSDM1 Filter 3 global Interrupt (42)
    CortexM4F::GENERIC_ISR, // TIM8 Break Interrupt (43)
    CortexM4F::GENERIC_ISR, // TIM8 Update Interrupt (44)
    CortexM4F::GENERIC_ISR, // TIM8 Trigger and Commutation Interrupt (45)
    CortexM4F::GENERIC_ISR, // TIM8 Capture Compare Interrupt (46)
    CortexM4F::GENERIC_ISR, // ADC3 global  Interrupt (47)
    CortexM4F::GENERIC_ISR, // FMC global Interrupt (48)
    CortexM4F::GENERIC_ISR, // SDMMC1 global Interrupt (49)
    CortexM4F::GENERIC_ISR, // TIM5 global Interrupt (50)
    CortexM4F::GENERIC_ISR, // SPI3 global Interrupt (51)
    CortexM4F::GENERIC_ISR, // UART4 global Interrupt (52)
    CortexM4F::GENERIC_ISR, // UART5 global Interrupt (53)
    CortexM4F::GENERIC_ISR, // TIM6 global and DAC1&2 underrun error interrupts (54)
    CortexM4F::GENERIC_ISR, // TIM7 global interrupt (55)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 1 global Interrupt (56)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 2 global Interrupt (57)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 3 global Interrupt (58)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 4 global Interrupt (59)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 5 global Interrupt (60)
    CortexM4F::GENERIC_ISR, // DFSDM1 Filter 0 global Interrupt (61)
    CortexM4F::GENERIC_ISR, // DFSDM1 Filter 1 global Interrupt (62)
    CortexM4F::GENERIC_ISR, // DFSDM1 Filter 2 global Interrupt (63)
    CortexM4F::GENERIC_ISR, // COMP1 and COMP2 Interrupts (64)
    CortexM4F::GENERIC_ISR, // LP TIM1 interrupt (65)
    CortexM4F::GENERIC_ISR, // LP TIM2 interrupt (66)
    CortexM4F::GENERIC_ISR, // USB OTG FS global Interrupt (67)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 6 global interrupt (68)
    CortexM4F::GENERIC_ISR, // DMA2 Channel 7 global interrupt (69)
    CortexM4F::GENERIC_ISR, // LP UART1 interrupt (70)
    CortexM4F::GENERIC_ISR, // Quad SPI global interrupt (71)
    CortexM4F::GENERIC_ISR, // I2C3 event interrupt (72)
    CortexM4F::GENERIC_ISR, // I2C3 error interrupt (73)
    CortexM4F::GENERIC_ISR, // Serial Audio Interface 1 global interrupt (74)
    CortexM4F::GENERIC_ISR, // Serial Audio Interface 2 global interrupt (75)
    CortexM4F::GENERIC_ISR, // Serial Wire Interface 1 global interrupt (76)
    CortexM4F::GENERIC_ISR, // Touch Sense Controller global interrupt (77)
    CortexM4F::GENERIC_ISR, // LCD global interrupt (78)
    unhandled_interrupt,    // (79)
    CortexM4F::GENERIC_ISR, // RNG global interrupt (80)
    CortexM4F::GENERIC_ISR, // FPU global interrupt (81)
];

pub unsafe fn init() {
    stm32l4xx::init();
}
