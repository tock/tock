//! Peripheral implementations for the STM32F3xx MCU.
//!
//! STM32F303: <https://www.st.com/en/microcontrollers-microprocessors/stm32f303.html>

#![crate_name = "stm32f3xx"]
#![crate_type = "rlib"]
#![feature(llvm_asm, const_fn, in_band_lifetimes)]
#![no_std]
#![allow(unused_doc_comments)]

pub mod chip;
pub mod nvic;

// Peripherals
pub mod exti;
pub mod gpio;
pub mod rcc;
pub mod spi;
pub mod syscfg;
pub mod tim2;
pub mod usart;

use cortexm4::{generic_isr, hard_fault_handler, svc_handler, systick_handler};

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    llvm_asm!(
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

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
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

// STM32F303VCT6 has total of 82 interrupts
// Extracted from `CMSIS/Device/ST/STM32F3xx/Include/stm32f303xc.h`
// NOTE: There are missing IRQn between 0 and 81
#[cfg(feature = "stm32f303vct6")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 82] = [
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
    generic_isr,         // ADC1_2 (18)
    generic_isr,         // HP_USB or CAN1_TX (19)
    generic_isr,         // LP_USB or CAN1_RX0 (20)
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
    generic_isr,         // USB_WKUP (42)
    generic_isr,         // TIM8_BRK_TIM12 (43)
    generic_isr,         // TIM8_UP_TIM13 (44)
    generic_isr,         // TIM8_TRG_COM_TIM14 (45)
    generic_isr,         // TIM8_CC (46)
    generic_isr,         // ADC3 (47)
    unhandled_interrupt, // (48)
    unhandled_interrupt, // (49)
    unhandled_interrupt, // (50)
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
    generic_isr,         // ADC4 (61)
    unhandled_interrupt, // (62)
    unhandled_interrupt, // (63)
    generic_isr,         // COMP1_2_3 (64)
    generic_isr,         // COMP4_5_6 (65)
    generic_isr,         // COMP7 (66)
    unhandled_interrupt, //(67)
    unhandled_interrupt, //(68)
    unhandled_interrupt, //(69)
    unhandled_interrupt, //(70)
    unhandled_interrupt, //(71)
    unhandled_interrupt, //(72)
    unhandled_interrupt, //(73)
    generic_isr,         // USB_HP (74)
    generic_isr,         // USB_LP (75)
    generic_isr,         // USB_RMP_WKUP (76)
    unhandled_interrupt, // (77)
    unhandled_interrupt, // (78)
    unhandled_interrupt, // (79)
    unhandled_interrupt, // (80)
    generic_isr,         // FPU (81)
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
