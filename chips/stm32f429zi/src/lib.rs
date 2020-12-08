#![no_std]

use cortexm4::generic_isr;

pub use stm32f4xx::{adc, chip, dbg, dma1, exti, gpio, nvic, rcc, spi, syscfg, tim2, usart};

pub mod interrupt_service;
pub mod stm32f429zi_nvic;

// STM32F42xxx and STM32F43xxx has total of 91 interrupts
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// `used` ensures that the symbol is kept until the final binary. However, as of
// May 2020, due to the compilation process, there must be some other compiled
// code here to make sure the object file is kept around. That means at minimum
// there must be an `init()` function here so that compiler does not just ignore
// the `IRQS` object. See https://github.com/rust-lang/rust/issues/56639 for a
// related discussion.
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
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

pub unsafe fn init() {
    stm32f4xx::init();
}
