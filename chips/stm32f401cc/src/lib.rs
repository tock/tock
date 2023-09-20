// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

#![no_std]

use cortexm4::{unhandled_interrupt, CortexM4, CortexMVariant};

pub use stm32f4xx::{
    adc, chip, clocks, dbg, dma, exti, flash, gpio, nvic, rcc, spi, syscfg, tim2, usart,
};

pub mod chip_specs;
pub mod interrupt_service;

// Extracted from RM0368 Reference manual, Table 38
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// "used" ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 85] = [
    CortexM4::GENERIC_ISR, // WWDG (0)
    CortexM4::GENERIC_ISR, // PVD (1)
    CortexM4::GENERIC_ISR, // TAMP_STAMP (2)
    CortexM4::GENERIC_ISR, // RTC_WKUP (3)
    CortexM4::GENERIC_ISR, // FLASH (4)
    CortexM4::GENERIC_ISR, // RCC (5)
    CortexM4::GENERIC_ISR, // EXTI0 (6)
    CortexM4::GENERIC_ISR, // EXTI1 (7)
    CortexM4::GENERIC_ISR, // EXTI2 (8)
    CortexM4::GENERIC_ISR, // EXTI3 (9)
    CortexM4::GENERIC_ISR, // EXTI4 (10)
    CortexM4::GENERIC_ISR, // DMA1_Stream0 (11)
    CortexM4::GENERIC_ISR, // DMA1_Stream1 (12)
    CortexM4::GENERIC_ISR, // DMA1_Stream2 (13)
    CortexM4::GENERIC_ISR, // DMA1_Stream3 (14)
    CortexM4::GENERIC_ISR, // DMA1_Stream4 (15)
    CortexM4::GENERIC_ISR, // DMA1_Stream5 (16)
    CortexM4::GENERIC_ISR, // DMA1_Stream6 (17)
    CortexM4::GENERIC_ISR, // ADC (18)
    unhandled_interrupt,   // (19)
    unhandled_interrupt,   // (20)
    unhandled_interrupt,   // (21)
    unhandled_interrupt,   // (22)
    CortexM4::GENERIC_ISR, // EXTI9_5 (23)
    CortexM4::GENERIC_ISR, // TIM1_BRK_TIM9 (24)
    CortexM4::GENERIC_ISR, // TIM1_UP_TIM10 (25)
    CortexM4::GENERIC_ISR, // TIM1_TRG_COM_TIM11 (26)
    CortexM4::GENERIC_ISR, // TIM1_CC (27)
    CortexM4::GENERIC_ISR, // TIM2 (28)
    CortexM4::GENERIC_ISR, // TIM3 (29)
    CortexM4::GENERIC_ISR, // TIM4 (30)
    CortexM4::GENERIC_ISR, // I2C1_EV (31)
    CortexM4::GENERIC_ISR, // I2C1_ER (32)
    CortexM4::GENERIC_ISR, // I2C2_EV (33)
    CortexM4::GENERIC_ISR, // I2C2_ER (34)
    CortexM4::GENERIC_ISR, // SPI1 (35)
    CortexM4::GENERIC_ISR, // SPI2 (36)
    CortexM4::GENERIC_ISR, // USART1 (37)
    CortexM4::GENERIC_ISR, // USART2 (38)
    CortexM4::GENERIC_ISR, // USART3 (39)
    CortexM4::GENERIC_ISR, // EXTI15_10 (40)
    CortexM4::GENERIC_ISR, // RTC_Alarm (41)
    CortexM4::GENERIC_ISR, // OTG_FS_WKUP (42)
    unhandled_interrupt,   // (43)
    unhandled_interrupt,   // (44)
    unhandled_interrupt,   // (45)
    unhandled_interrupt,   // (45)
    CortexM4::GENERIC_ISR, // DMA1_Stream7 (47)
    unhandled_interrupt,   // (48)
    CortexM4::GENERIC_ISR, // SDIO (49)
    CortexM4::GENERIC_ISR, // TIM5 (50)
    CortexM4::GENERIC_ISR, // SPI3 (51)
    unhandled_interrupt,   // (52)
    unhandled_interrupt,   // (53)
    unhandled_interrupt,   // (54)
    unhandled_interrupt,   // (55)
    CortexM4::GENERIC_ISR, // DMA2_Stream0 (56)
    CortexM4::GENERIC_ISR, // DMA2_Stream1 (57)
    CortexM4::GENERIC_ISR, // DMA2_Stream2 (58)
    CortexM4::GENERIC_ISR, // DMA2_Stream3 (59)
    CortexM4::GENERIC_ISR, // DMA2_Stream4 (60)
    unhandled_interrupt,   // (61)
    unhandled_interrupt,   // (62)
    unhandled_interrupt,   // (63)
    unhandled_interrupt,   // (64)
    unhandled_interrupt,   // (65)
    unhandled_interrupt,   // (66)
    CortexM4::GENERIC_ISR, // OTG_FS (67)
    CortexM4::GENERIC_ISR, // DMA2_Stream5 (68)
    CortexM4::GENERIC_ISR, // DMA2_Stream6 (69)
    CortexM4::GENERIC_ISR, // DMA2_Stream7 (70)
    CortexM4::GENERIC_ISR, // USART6 (71)
    CortexM4::GENERIC_ISR, // I2C3_EV (72)
    CortexM4::GENERIC_ISR, // I2C3_ER (73)
    unhandled_interrupt,   // (74)
    unhandled_interrupt,   // (75)
    unhandled_interrupt,   // (76)
    unhandled_interrupt,   // (77)
    unhandled_interrupt,   // (78)
    unhandled_interrupt,   // (79)
    unhandled_interrupt,   // (80)
    CortexM4::GENERIC_ISR, // FPU (81)
    unhandled_interrupt,   // (82)
    unhandled_interrupt,   // (83)
    CortexM4::GENERIC_ISR, // SPI4 (84)
];

pub unsafe fn init() {
    stm32f4xx::init();
}
