// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

#![no_std]
// GPIO has many register definitions in `register_structs()!`
// and requires a deeper recursion limit than the default to fully expand.
#![recursion_limit = "256"]

pub mod chip;
pub mod clocks;
pub mod gpio;
pub mod interrupts;
pub mod resets;
pub mod ticks;
pub mod timer;
pub mod uart;
pub mod xosc;

use cortexm33::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM33, CortexMVariant};

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
    CortexM33::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,           // MemManage
    unhandled_interrupt,           // BusFault
    unhandled_interrupt,           // UsageFault
    unhandled_interrupt,           // SecureFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM33::SVC_HANDLER, // SVC
    unhandled_interrupt,    // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,        // PendSV
    CortexM33::SYSTICK_HANDLER, // SysTick
];

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 52] = [
    CortexM33::GENERIC_ISR, // TIMER0 0 (0)
    CortexM33::GENERIC_ISR, // TIMER0 1 (1)
    CortexM33::GENERIC_ISR, // TIMER0 2 (2)
    CortexM33::GENERIC_ISR, // TIMER0 3 (3)
    CortexM33::GENERIC_ISR, // TIMER1 0 (4)
    CortexM33::GENERIC_ISR, // TIMER1 1 (5)
    CortexM33::GENERIC_ISR, // TIMER1 2 (6)
    CortexM33::GENERIC_ISR, // TIMER1 3 (7)
    CortexM33::GENERIC_ISR, // PWM_IRQ_WRAP 0 (8)
    CortexM33::GENERIC_ISR, // PWM_IRQ_WRAP 1 (9)
    CortexM33::GENERIC_ISR, // DMA 0 (10)
    CortexM33::GENERIC_ISR, // DMA 1 (11)
    CortexM33::GENERIC_ISR, // DMA 2 (12)
    CortexM33::GENERIC_ISR, // DMA 3 (13)
    CortexM33::GENERIC_ISR, // USB (14)
    CortexM33::GENERIC_ISR, // PIO0 0 (15)
    CortexM33::GENERIC_ISR, // PIO0 1 (16)
    CortexM33::GENERIC_ISR, // PIO1 0 (17)
    CortexM33::GENERIC_ISR, // PIO1 1 (18)
    CortexM33::GENERIC_ISR, // PIO2 0 (19)
    CortexM33::GENERIC_ISR, // PIO2 1 (20)
    CortexM33::GENERIC_ISR, // IO_IRQ_BANK 0 (21)
    CortexM33::GENERIC_ISR, // IO_IRQ_BANK 0 NS (22)
    CortexM33::GENERIC_ISR, // IO_IRQ_QSPI (23)
    CortexM33::GENERIC_ISR, // IO_IRQ_QSPI_NS (24)
    CortexM33::GENERIC_ISR, // SIO_IRQ_FIFO (25)
    CortexM33::GENERIC_ISR, // SIO_IRQ_BELL (26)
    CortexM33::GENERIC_ISR, // SIO_IRQ_FIFO_NS (27)
    CortexM33::GENERIC_ISR, // SIO_IRQ_BELL_NS (28)
    CortexM33::GENERIC_ISR, // SIO_IRQ_MTIMECMP (29)
    CortexM33::GENERIC_ISR, // CLOCKS (30)
    CortexM33::GENERIC_ISR, // SPI 0 (31)
    CortexM33::GENERIC_ISR, // SPI 1 (32)
    CortexM33::GENERIC_ISR, // UART 0 (33)
    CortexM33::GENERIC_ISR, // UART 1 (34)
    CortexM33::GENERIC_ISR, // ADC_IRQ_FIFO (35)
    CortexM33::GENERIC_ISR, // I2C 0 (36)
    CortexM33::GENERIC_ISR, // I2C 1 (37)
    CortexM33::GENERIC_ISR, // OTP (38)
    CortexM33::GENERIC_ISR, // TRNG (39)
    CortexM33::GENERIC_ISR, // PROC 0 (40)
    CortexM33::GENERIC_ISR, // PROC 1 (41)
    CortexM33::GENERIC_ISR, // PLL_SYS (42)
    CortexM33::GENERIC_ISR, // PLL_USB (43)
    CortexM33::GENERIC_ISR, // POWMAN_IRQ_POW (44)
    CortexM33::GENERIC_ISR, // POWMAN_IRQ_TIMER (45)
    unhandled_interrupt,    // (46)
    unhandled_interrupt,    // (47)
    unhandled_interrupt,    // (48)
    unhandled_interrupt,    // (49)
    unhandled_interrupt,    // (50)
    unhandled_interrupt,    // (51)
];

extern "C" {
    static mut _szero: usize;
    static mut _ezero: usize;
    static mut _etext: usize;
    static mut _srelocate: usize;
    static mut _erelocate: usize;
}

pub unsafe fn init() {
    cortexm33::nvic::disable_all();
    cortexm33::nvic::clear_all_pending();
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
