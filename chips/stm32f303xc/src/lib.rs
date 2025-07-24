// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Peripheral implementations for the STM32F3xx MCU.
//!
//! STM32F303: <https://www.st.com/en/microcontrollers-microprocessors/stm32f303.html>

#![no_std]

pub mod chip;
pub mod nvic;

// Peripherals
pub mod adc;
pub mod dma;
pub mod exti;
pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod rcc;
pub mod spi;
pub mod syscfg;
pub mod tim2;
pub mod usart;
pub mod wdt;

use cortexm4f::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM4F, CortexMVariant};

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    initialize_ram_jump_to_main,
    unhandled_interrupt,           // NMI
    CortexM4F::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,           // MemManage
    unhandled_interrupt,           // BusFault
    unhandled_interrupt,           // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM4F::SVC_HANDLER, // SVC
    unhandled_interrupt,    // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,        // PendSV
    CortexM4F::SYSTICK_HANDLER, // SysTick
];

// STM32F303VCT6 has total of 82 interrupts
// Extracted from `CMSIS/Device/ST/STM32F3xx/Include/stm32f303xc.h`
// NOTE: There are missing IRQn between 0 and 81
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 82] = [
    CortexM4F::GENERIC_ISR, // WWDG (0)
    CortexM4F::GENERIC_ISR, // PVD (1)
    CortexM4F::GENERIC_ISR, // TAMP_STAMP (2)
    CortexM4F::GENERIC_ISR, // RTC_WKUP (3)
    CortexM4F::GENERIC_ISR, // FLASH (4)
    CortexM4F::GENERIC_ISR, // RCC (5)
    CortexM4F::GENERIC_ISR, // EXTI0 (6)
    CortexM4F::GENERIC_ISR, // EXTI1 (7)
    CortexM4F::GENERIC_ISR, // EXTI2 (8)
    CortexM4F::GENERIC_ISR, // EXTI3 (9)
    CortexM4F::GENERIC_ISR, // EXTI4 (10)
    CortexM4F::GENERIC_ISR, // DMA1_Stream0 (11)
    CortexM4F::GENERIC_ISR, // DMA1_Stream1 (12)
    CortexM4F::GENERIC_ISR, // DMA1_Stream2 (13)
    CortexM4F::GENERIC_ISR, // DMA1_Stream3 (14)
    CortexM4F::GENERIC_ISR, // DMA1_Stream4 (15)
    CortexM4F::GENERIC_ISR, // DMA1_Stream5 (16)
    CortexM4F::GENERIC_ISR, // DMA1_Stream6 (17)
    CortexM4F::GENERIC_ISR, // ADC1_2 (18)
    CortexM4F::GENERIC_ISR, // HP_USB or CAN1_TX (19)
    CortexM4F::GENERIC_ISR, // LP_USB or CAN1_RX0 (20)
    CortexM4F::GENERIC_ISR, // CAN1_RX1 (21)
    CortexM4F::GENERIC_ISR, // CAN1_SCE (22)
    CortexM4F::GENERIC_ISR, // EXTI9_5 (23)
    CortexM4F::GENERIC_ISR, // TIM1_BRK_TIM9 (24)
    CortexM4F::GENERIC_ISR, // TIM1_UP_TIM10 (25)
    CortexM4F::GENERIC_ISR, // TIM1_TRG_COM_TIM11 (26)
    CortexM4F::GENERIC_ISR, // TIM1_CC (27)
    CortexM4F::GENERIC_ISR, // TIM2 (28)
    CortexM4F::GENERIC_ISR, // TIM3 (29)
    CortexM4F::GENERIC_ISR, // TIM4 (30)
    CortexM4F::GENERIC_ISR, // I2C1_EV (31)
    CortexM4F::GENERIC_ISR, // I2C1_ER (32)
    CortexM4F::GENERIC_ISR, // I2C2_EV (33)
    CortexM4F::GENERIC_ISR, // I2C2_ER (34)
    CortexM4F::GENERIC_ISR, // SPI1 (35)
    CortexM4F::GENERIC_ISR, // SPI2 (36)
    CortexM4F::GENERIC_ISR, // USART1 (37)
    CortexM4F::GENERIC_ISR, // USART2 (38)
    CortexM4F::GENERIC_ISR, // USART3 (39)
    CortexM4F::GENERIC_ISR, // EXTI15_10 (40)
    CortexM4F::GENERIC_ISR, // RTC_Alarm (41)
    CortexM4F::GENERIC_ISR, // USB_WKUP (42)
    CortexM4F::GENERIC_ISR, // TIM8_BRK_TIM12 (43)
    CortexM4F::GENERIC_ISR, // TIM8_UP_TIM13 (44)
    CortexM4F::GENERIC_ISR, // TIM8_TRG_COM_TIM14 (45)
    CortexM4F::GENERIC_ISR, // TIM8_CC (46)
    CortexM4F::GENERIC_ISR, // ADC3 (47)
    unhandled_interrupt,    // (48)
    unhandled_interrupt,    // (49)
    unhandled_interrupt,    // (50)
    CortexM4F::GENERIC_ISR, // SPI3 (51)
    CortexM4F::GENERIC_ISR, // UART4 (52)
    CortexM4F::GENERIC_ISR, // UART5 (53)
    CortexM4F::GENERIC_ISR, // TIM6_DAC (54)
    CortexM4F::GENERIC_ISR, // TIM7 (55)
    CortexM4F::GENERIC_ISR, // DMA2_Stream0 (56)
    CortexM4F::GENERIC_ISR, // DMA2_Stream1 (57)
    CortexM4F::GENERIC_ISR, // DMA2_Stream2 (58)
    CortexM4F::GENERIC_ISR, // DMA2_Stream3 (59)
    CortexM4F::GENERIC_ISR, // DMA2_Stream4 (60)
    CortexM4F::GENERIC_ISR, // ADC4 (61)
    unhandled_interrupt,    // (62)
    unhandled_interrupt,    // (63)
    CortexM4F::GENERIC_ISR, // COMP1_2_3 (64)
    CortexM4F::GENERIC_ISR, // COMP4_5_6 (65)
    CortexM4F::GENERIC_ISR, // COMP7 (66)
    unhandled_interrupt,    //(67)
    unhandled_interrupt,    //(68)
    unhandled_interrupt,    //(69)
    unhandled_interrupt,    //(70)
    unhandled_interrupt,    //(71)
    unhandled_interrupt,    //(72)
    unhandled_interrupt,    //(73)
    CortexM4F::GENERIC_ISR, // USB_HP (74)
    CortexM4F::GENERIC_ISR, // USB_LP (75)
    CortexM4F::GENERIC_ISR, // USB_RMP_WKUP (76)
    unhandled_interrupt,    // (77)
    unhandled_interrupt,    // (78)
    unhandled_interrupt,    // (79)
    unhandled_interrupt,    // (80)
    CortexM4F::GENERIC_ISR, // FPU (81)
];

pub unsafe fn init() {
    cortexm4f::nvic::disable_all();
    cortexm4f::nvic::clear_all_pending();
    cortexm4f::nvic::enable_all();
}
