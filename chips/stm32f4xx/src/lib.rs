//! Peripheral implementations for the STM32F4xx MCU.
//!
//! STM32F446RE: <https://www.st.com/en/microcontrollers/stm32f4.html>

#![crate_name = "stm32f4xx"]
#![crate_type = "rlib"]
#![feature(const_fn)]
#![feature(llvm_asm)]
#![no_std]

pub mod chip;
pub mod nvic;

// Peripherals
pub mod adc;
pub mod dbg;
pub mod deferred_calls;
pub mod dma1;
pub mod exti;
pub mod fsmc;
pub mod gpio;
pub mod i2c;
pub mod rcc;
pub mod spi;
pub mod syscfg;
pub mod tim2;
pub mod usart;

use cortexm4::{hard_fault_handler, svc_handler, systick_handler, unhandled_interrupt};

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
