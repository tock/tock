//! Implementation of the nested vectored interrupt controller (NVIC).

use core;
use core::intrinsics;
use kernel::common::VolatileCell;

#[repr(C, packed)]
struct Nvic {
    iser: [VolatileCell<u32>; 3],
    _reserved0: [u32; 29],
    icer: [VolatileCell<u32>; 3],
    _reserved1: [u32; 29],
    ispr: [VolatileCell<u32>; 3],
    _reserved2: [u32; 29],
    icpr: [VolatileCell<u32>; 3],
    _reserved3: [u32; 29],
    iabr: [VolatileCell<u32>; 3],
    _reserved4: [u32; 61],
    ip: [VolatileCell<u8>; 84],
    _reserved5: [u32; 620],
    stir: VolatileCell<u32>,
}

#[repr(C)]
#[derive(Copy,Clone)]
#[allow(non_camel_case_types)]
pub enum NvicIdx {
    WWDG,
    PVD,
    TAMPER,
    RTC,
    FLASH,
    RCC,
    EXTI0,
    EXTI1,
    EXTI2,
    EXTI3,
    EXTI4,
    DMA1_Channel1,
    DMA1_Channel2,
    DMA1_Channel3,
    DMA1_Channel4,
    DMA1_Channel5,
    DMA1_Channel6,
    DMA1_Channel7,
    ADC1_2,
    USB_HP_CAN1_TX,
    USB_LP_CAN1_RX0,
    CAN1_RX1,
    CAN1_SCE,
    EXTI9_5,
    TIM1_BRK,
    TIM1_UP,
    TIM1_TRG_COM,
    TIM1_CC,
    TIM2,
    TIM3,
    TIM4,
    I2C1_EV,
    I2C1_ER,
    I2C2_EV,
    I2C2_ER,
    SPI1,
    SPI2,
    USART1,
    USART2,
    USART3,
    EXTI15_10,
    RTC_Alarm,
    USBWakeUp,
    TIM8_BRK,
    TIM8_UP,
    TIM8_TRG_COM,
    TIM8_CC,
    ADC3,
    FSMC,
    SDIO,
    TIM5,
    SPI3,
    UART4,
    UART5,
    TIM6,
    TIM7,
    DMA2_Channel1,
    DMA2_Channel2,
    DMA2_Channel3,
    DMA2_Channel4_5,
}

impl core::default::Default for NvicIdx {
    fn default() -> NvicIdx {
        NvicIdx::WWDG
    }
}

const BASE_ADDRESS: usize = 0xe000e100;

pub unsafe fn enable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.iser[interrupt / 32].set(1 << (interrupt % 32));
}

pub unsafe fn disable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icer[interrupt / 32].set(1 << (interrupt % 32));
}

pub unsafe fn clear_pending(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icpr[interrupt / 32].set(1 << (interrupt % 32));
}
