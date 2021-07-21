#![feature(const_fn_trait_bound, asm)]
#![no_std]

pub mod adc;
pub mod chip;
pub mod clocks;
pub mod gpio;
pub mod interrupts;
pub mod resets;
pub mod spi;
pub mod sysinfo;
pub mod timer;
pub mod uart;
pub mod watchdog;
pub mod xosc;

use cortexm0p::{
    self, generic_isr, hard_fault_handler, initialize_ram_jump_to_main, svc_handler,
    systick_handler, unhandled_interrupt,
};

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

// RP2040 has total of 26 interrupts, but the SDK declares 32 as 26 - 32 might be manually handled
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 32] = [
    generic_isr,         // TIMER0 (0)
    generic_isr,         // TIMER1 (1)
    generic_isr,         // TIMER2 (2)
    generic_isr,         // TIMER3 (3)
    generic_isr,         // PWM WRAP (4)
    generic_isr,         // USB (5)
    generic_isr,         // XIP (6)
    generic_isr,         // PIO0 INT0  (7)
    generic_isr,         // PIO0 INT1 (8)
    generic_isr,         // PIO1 INT0 (9)
    generic_isr,         // PIO1 INT1 (10)
    generic_isr,         // DMA0 (11)
    generic_isr,         // DMA1 (12)
    generic_isr,         // IO BANK 0 (13)
    generic_isr,         // IO QSPI (14)
    generic_isr,         // SIO PROC 0 (15)
    generic_isr,         // SIO PROC 1 (16)
    generic_isr,         // CLOCKS (17)
    generic_isr,         // SPI 0 (18)
    generic_isr,         // SPI 1 (19)
    generic_isr,         // UART 0 (20)
    generic_isr,         // UART 1 (21)
    generic_isr,         // ADC FIFO (22)
    generic_isr,         // I2C 0 (23)
    generic_isr,         // I2C 1 (24)
    generic_isr,         // RTC (25)
    unhandled_interrupt, // (26)
    unhandled_interrupt, // (27)
    unhandled_interrupt, // (28)
    unhandled_interrupt, // (29)
    unhandled_interrupt, // (30)
    unhandled_interrupt, // (31)
];

extern "C" {
    static mut _szero: usize;
    static mut _ezero: usize;
    static mut _etext: usize;
    static mut _srelocate: usize;
    static mut _erelocate: usize;
}

pub unsafe fn init() {
    cortexm0p::nvic::disable_all();
    cortexm0p::nvic::clear_all_pending();
    let sio = gpio::SIO::new();
    let processor = sio.get_processor();
    match processor {
        chip::Processor::Processor0 => {}
        _ => panic!(
            "Kernel should run only using processor 0 (now processor {})",
            processor as u8
        ),
    }
}
