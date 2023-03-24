#![no_std]

pub mod adc;
pub mod chip;
pub mod clocks;
pub mod gpio;
pub mod i2c;
pub mod interrupts;
pub mod pwm;
pub mod resets;
pub mod spi;
pub mod sysinfo;
pub mod test;
pub mod timer;
pub mod uart;
pub mod usb;
pub mod watchdog;
pub mod xosc;

use cortexm0p::{
    self, initialize_ram_jump_to_main, unhandled_interrupt, CortexM0P, CortexMVariant,
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
    unhandled_interrupt,           // NMI
    CortexM0P::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,           // MemManage
    unhandled_interrupt,           // BusFault
    unhandled_interrupt,           // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM0P::SVC_HANDLER, // SVC
    unhandled_interrupt,    // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,        // PendSV
    CortexM0P::SYSTICK_HANDLER, // SysTick
];

// RP2040 has total of 26 interrupts, but the SDK declares 32 as 26 - 32 might be manually handled
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 32] = [
    CortexM0P::GENERIC_ISR, // TIMER0 (0)
    CortexM0P::GENERIC_ISR, // TIMER1 (1)
    CortexM0P::GENERIC_ISR, // TIMER2 (2)
    CortexM0P::GENERIC_ISR, // TIMER3 (3)
    CortexM0P::GENERIC_ISR, // PWM WRAP (4)
    CortexM0P::GENERIC_ISR, // USB (5)
    CortexM0P::GENERIC_ISR, // XIP (6)
    CortexM0P::GENERIC_ISR, // PIO0 INT0  (7)
    CortexM0P::GENERIC_ISR, // PIO0 INT1 (8)
    CortexM0P::GENERIC_ISR, // PIO1 INT0 (9)
    CortexM0P::GENERIC_ISR, // PIO1 INT1 (10)
    CortexM0P::GENERIC_ISR, // DMA0 (11)
    CortexM0P::GENERIC_ISR, // DMA1 (12)
    CortexM0P::GENERIC_ISR, // IO BANK 0 (13)
    CortexM0P::GENERIC_ISR, // IO QSPI (14)
    CortexM0P::GENERIC_ISR, // SIO PROC 0 (15)
    CortexM0P::GENERIC_ISR, // SIO PROC 1 (16)
    CortexM0P::GENERIC_ISR, // CLOCKS (17)
    CortexM0P::GENERIC_ISR, // SPI 0 (18)
    CortexM0P::GENERIC_ISR, // SPI 1 (19)
    CortexM0P::GENERIC_ISR, // UART 0 (20)
    CortexM0P::GENERIC_ISR, // UART 1 (21)
    CortexM0P::GENERIC_ISR, // ADC FIFO (22)
    CortexM0P::GENERIC_ISR, // I2C 0 (23)
    CortexM0P::GENERIC_ISR, // I2C 1 (24)
    CortexM0P::GENERIC_ISR, // RTC (25)
    unhandled_interrupt,    // (26)
    unhandled_interrupt,    // (27)
    unhandled_interrupt,    // (28)
    unhandled_interrupt,    // (29)
    unhandled_interrupt,    // (30)
    unhandled_interrupt,    // (31)
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
