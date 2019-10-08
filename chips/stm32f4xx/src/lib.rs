//! Peripheral implementations for the STM32F4xx MCU.
//!
//! STM32F446RE: <https://www.st.com/en/microcontrollers/stm32f4.html>

#![crate_name = "stm32f4xx"]
#![crate_type = "rlib"]
#![feature(asm, const_fn, in_band_lifetimes)]
#![no_std]
#![allow(unused_doc_comments)]

mod deferred_call_tasks;

pub mod chip;
pub mod nvic;

// Peripherals
pub mod dbg;
pub mod dma1;
pub mod exti;
pub mod gpio;
pub mod rcc;
pub mod spi;
pub mod syscfg;
pub mod tim2;
pub mod usart;

use cortexm4::{generic_isr, hard_fault_handler, svc_handler, systick_handler};

#[cfg(not(target_os = "none"))]
unsafe extern "C" fn unhandled_interrupt() {}

#[cfg(target_os = "none")]
unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    asm!(
    "mrs    r0, ipsr                    "
    : "={r0}"(interrupt_number)
    :
    : "r0"
    :
    );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined by platform
    fn reset_handler();
}

#[link_section = ".vectors"]
// used Ensures that the symbol is kept until the final binary
#[used]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MemManage
    unhandled_interrupt, // BusFault
    unhandled_interrupt, // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    svc_handler,         // SVC
    unhandled_interrupt, // DebugMon
    unhandled_interrupt,
    unhandled_interrupt, // PendSV
    systick_handler,     // SysTick
];

// STM32F446xx has total of 97 interrupts
// Extracted from `CMSIS/Device/ST/STM32F4xx/Include/stm32f446xx.h`
// NOTE: There are missing IRQn between 0 and 96
#[cfg(feature = "stm32f446re")]
#[link_section = ".irqs"]
#[used] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 97] = [
    generic_isr,         // WWDG (0)
    generic_isr,         // PVD (1)
    generic_isr,         // TAMP_STAMP (2)
    generic_isr,         // RTC_WKUP (3)
    generic_isr,         // FLASH (4)
    generic_isr,         // RCC (5)
    generic_isr,         // EXTI0 (6)
    generic_isr,         // EXTI1 (7)
    generic_isr,         // EXTI2 (8)
    generic_isr,         // EXTI3 (9)
    generic_isr,         // EXTI4 (10)
    generic_isr,         // DMA1_Stream0 (11)
    generic_isr,         // DMA1_Stream1 (12)
    generic_isr,         // DMA1_Stream2 (13)
    generic_isr,         // DMA1_Stream3 (14)
    generic_isr,         // DMA1_Stream4 (15)
    generic_isr,         // DMA1_Stream5 (16)
    generic_isr,         // DMA1_Stream6 (17)
    generic_isr,         // ADC (18)
    generic_isr,         // CAN1_TX (19)
    generic_isr,         // CAN1_RX0 (20)
    generic_isr,         // CAN1_RX1 (21)
    generic_isr,         // CAN1_SCE (22)
    generic_isr,         // EXTI9_5 (23)
    generic_isr,         // TIM1_BRK_TIM9 (24)
    generic_isr,         // TIM1_UP_TIM10 (25)
    generic_isr,         // TIM1_TRG_COM_TIM11 (26)
    generic_isr,         // TIM1_CC (27)
    generic_isr,         // TIM2 (28)
    generic_isr,         // TIM3 (29)
    generic_isr,         // TIM4 (30)
    generic_isr,         // I2C1_EV (31)
    generic_isr,         // I2C1_ER (32)
    generic_isr,         // I2C2_EV (33)
    generic_isr,         // I2C2_ER (34)
    generic_isr,         // SPI1 (35)
    generic_isr,         // SPI2 (36)
    generic_isr,         // USART1 (37)
    generic_isr,         // USART2 (38)
    generic_isr,         // USART3 (39)
    generic_isr,         // EXTI15_10 (40)
    generic_isr,         // RTC_Alarm (41)
    generic_isr,         // OTG_FS_WKUP (42)
    generic_isr,         // TIM8_BRK_TIM12 (43)
    generic_isr,         // TIM8_UP_TIM13 (44)
    generic_isr,         // TIM8_TRG_COM_TIM14 (45)
    generic_isr,         // TIM8_CC (46)
    generic_isr,         // DMA1_Stream7 (47)
    generic_isr,         // FMC (48)
    generic_isr,         // SDIO (49)
    generic_isr,         // TIM5 (50)
    generic_isr,         // SPI3 (51)
    generic_isr,         // UART4 (52)
    generic_isr,         // UART5 (53)
    generic_isr,         // TIM6_DAC (54)
    generic_isr,         // TIM7 (55)
    generic_isr,         // DMA2_Stream0 (56)
    generic_isr,         // DMA2_Stream1 (57)
    generic_isr,         // DMA2_Stream2 (58)
    generic_isr,         // DMA2_Stream3 (59)
    generic_isr,         // DMA2_Stream4 (60)
    unhandled_interrupt, // (61)
    unhandled_interrupt, // (62)
    generic_isr,         // CAN2_TX (63)
    generic_isr,         // CAN2_RX0 (64)
    generic_isr,         // CAN2_RX1 (65)
    generic_isr,         // CAN2_SCE (66)
    generic_isr,         // OTG_FS (67)
    generic_isr,         // DMA2_Stream5 (68)
    generic_isr,         // DMA2_Stream6 (69)
    generic_isr,         // DMA2_Stream7 (70)
    generic_isr,         // USART6 (71)
    generic_isr,         // I2C3_EV (72)
    generic_isr,         // I2C3_ER (73)
    generic_isr,         // OTG_HS_EP1_OUT (74)
    generic_isr,         // OTG_HS_EP1_IN (75)
    generic_isr,         // OTG_HS_WKUP (76)
    generic_isr,         // OTG_HS (77)
    generic_isr,         // DCMI (78)
    unhandled_interrupt, // (79)
    unhandled_interrupt, // (80)
    generic_isr,         // FPU (81)
    unhandled_interrupt, // (82)
    unhandled_interrupt, // (83)
    generic_isr,         // SPI4 (84)
    unhandled_interrupt, // (85)
    unhandled_interrupt, // (86)
    generic_isr,         // SAI1 (87)
    unhandled_interrupt, // (88)
    unhandled_interrupt, // (89)
    unhandled_interrupt, // (90)
    generic_isr,         // SAI2 (91)
    generic_isr,         // QUADSPI (92)
    generic_isr,         // CEC (93)
    generic_isr,         // SPDIF_RX (94)
    generic_isr,         // FMPI2C1_EV (95)
    generic_isr,         // FMPI2C1_ER (96)
];

// STM32F42xxx and STM32F43xxx has total of 91 interrupts
#[cfg(feature = "stm32f429zi")]
#[link_section = ".irqs"]
#[used] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 91] = [
    generic_isr, // WWDG (0)
    generic_isr, // PVD (1)
    generic_isr, // TAMP_STAMP (2)
    generic_isr, // RTC_WKUP (3)
    generic_isr, // FLASH (4)
    generic_isr, // RCC (5)
    generic_isr, // EXTI0 (6)
    generic_isr, // EXTI1 (7)
    generic_isr, // EXTI2 (8)
    generic_isr, // EXTI3 (9)
    generic_isr, // EXTI4 (10)
    generic_isr, // DMA1_Stream0 (11)
    generic_isr, // DMA1_Stream1 (12)
    generic_isr, // DMA1_Stream2 (13)
    generic_isr, // DMA1_Stream3 (14)
    generic_isr, // DMA1_Stream4 (15)
    generic_isr, // DMA1_Stream5 (16)
    generic_isr, // DMA1_Stream6 (17)
    generic_isr, // ADC (18)
    generic_isr, // CAN1_TX (19)
    generic_isr, // CAN1_RX0 (20)
    generic_isr, // CAN1_RX1 (21)
    generic_isr, // CAN1_SCE (22)
    generic_isr, // EXTI9_5 (23)
    generic_isr, // TIM1_BRK_TIM9 (24)
    generic_isr, // TIM1_UP_TIM10 (25)
    generic_isr, // TIM1_TRG_COM_TIM11 (26)
    generic_isr, // TIM1_CC (27)
    generic_isr, // TIM2 (28)
    generic_isr, // TIM3 (29)
    generic_isr, // TIM4 (30)
    generic_isr, // I2C1_EV (31)
    generic_isr, // I2C1_ER (32)
    generic_isr, // I2C2_EV (33)
    generic_isr, // I2C2_ER (34)
    generic_isr, // SPI1 (35)
    generic_isr, // SPI2 (36)
    generic_isr, // USART1 (37)
    generic_isr, // USART2 (38)
    generic_isr, // USART3 (39)
    generic_isr, // EXTI15_10 (40)
    generic_isr, // RTC_Alarm (41)
    generic_isr, // OTG_FS_WKUP (42)
    generic_isr, // TIM8_BRK_TIM12 (43)
    generic_isr, // TIM8_UP_TIM13 (44)
    generic_isr, // TIM8_TRG_COM_TIM14 (45)
    generic_isr, // TIM8_CC (46)
    generic_isr, // DMA1_Stream7 (47)
    generic_isr, // FMC (48)
    generic_isr, // SDIO (49)
    generic_isr, // TIM5 (50)
    generic_isr, // SPI3 (51)
    generic_isr, // UART4 (52)
    generic_isr, // UART5 (53)
    generic_isr, // TIM6_DAC (54)
    generic_isr, // TIM7 (55)
    generic_isr, // DMA2_Stream0 (56)
    generic_isr, // DMA2_Stream1 (57)
    generic_isr, // DMA2_Stream2 (58)
    generic_isr, // DMA2_Stream3 (59)
    generic_isr, // DMA2_Stream4 (60)
    generic_isr, // ETH (61)
    generic_isr, // ETH_WKUP (62)
    generic_isr, // CAN2_TX (63)
    generic_isr, // CAN2_RX0 (64)
    generic_isr, // CAN2_RX1 (65)
    generic_isr, // CAN2_SCE (66)
    generic_isr, // OTG_FS (67)
    generic_isr, // DMA2_Stream5 (68)
    generic_isr, // DMA2_Stream6 (69)
    generic_isr, // DMA2_Stream7 (70)
    generic_isr, // USART6 (71)
    generic_isr, // I2C3_EV (72)
    generic_isr, // I2C3_ER (73)
    generic_isr, // OTG_HS_EP1_OUT (74)
    generic_isr, // OTG_HS_EP1_IN (75)
    generic_isr, // OTG_HS_WKUP (76)
    generic_isr, // OTG_HS (77)
    generic_isr, // DCMI (78)
    generic_isr, // CRYP (79)
    generic_isr, // HASH_RNG (80)
    generic_isr, // FPU (81)
    generic_isr, // USART7 (82)
    generic_isr, // USART8 (83)
    generic_isr, // SPI4 (84)
    generic_isr, // SPI5 (85)
    generic_isr, // SPI6 (86)
    generic_isr, // SAI1 (87)
    generic_isr, // LCD-TFT (88)
    generic_isr, // LCD-TFT (89)
    generic_isr, // DMA2D(90)
];

extern "C" {
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

pub unsafe fn init() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);

    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
}
