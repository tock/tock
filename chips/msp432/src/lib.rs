// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

#![no_std]

use cortexm4::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM4, CortexMVariant};

pub mod adc;
pub mod chip;
pub mod cs;
pub mod dma;
pub mod flctl;
pub mod gpio;
pub mod i2c;
pub mod nvic;
pub mod pcm;
pub mod ref_module;
pub mod sysctl;
pub mod timer;
pub mod uart;
pub mod usci;
pub mod wdt;

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
    unhandled_interrupt,          // NMI
    CortexM4::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,          // MemManage
    unhandled_interrupt,          // BusFault
    unhandled_interrupt,          // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM4::SVC_HANDLER, // SVC
    unhandled_interrupt,   // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,       // PendSV
    CortexM4::SYSTICK_HANDLER, // SysTick
];

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 64] = [
    CortexM4::GENERIC_ISR, // Power Supply System (PSS) (0)
    CortexM4::GENERIC_ISR, // Clock System (CS) (1)
    CortexM4::GENERIC_ISR, // Power Control Manager (PCM) (2)
    CortexM4::GENERIC_ISR, // Watchdog Timer A (WDT_A) (3)
    CortexM4::GENERIC_ISR, // FPU_INT, Combined interrupt from flags in FPSCR (4)
    CortexM4::GENERIC_ISR, // FLash Controller (FLCTL) (5)
    CortexM4::GENERIC_ISR, // Comparator E0 (6)
    CortexM4::GENERIC_ISR, // Comparator E1 (7)
    CortexM4::GENERIC_ISR, // Timer A0 TA0CCTL0.CCIFG (8)
    CortexM4::GENERIC_ISR, // Timer A0 TA0CCTLx.CCIFG (x = 1 to 4), TA0CTL.TAIFG (9)
    CortexM4::GENERIC_ISR, // Timer A1 TA1CCTL0.CCIFG (10)
    CortexM4::GENERIC_ISR, // Timer A1 TA1CCTLx.CCIFG (x = 1 to 4), TA1CTL.TAIFG (11)
    CortexM4::GENERIC_ISR, // Timer A2 TA2CCTL0.CCIFG (12)
    CortexM4::GENERIC_ISR, // Timer A2 TA2CCTLx.CCIFG (x = 1 to 4), TA2CTL.TAIFG (13)
    CortexM4::GENERIC_ISR, // Timer A3 TA3CCTL0.CCIFG (13)
    CortexM4::GENERIC_ISR, // Timer A3 TA3CCTLx.CCIFG (x = 1 to 4), TA3CTL.TAIFG (15)
    CortexM4::GENERIC_ISR, // eUSCI A0 (16)
    CortexM4::GENERIC_ISR, // eUSCI A1 (17)
    CortexM4::GENERIC_ISR, // eUSCI A2 (18)
    CortexM4::GENERIC_ISR, // eUSCI A3 (19)
    CortexM4::GENERIC_ISR, // eUSCI B0 (20)
    CortexM4::GENERIC_ISR, // eUSCI B1 (21)
    CortexM4::GENERIC_ISR, // eUSCI B2 (22)
    CortexM4::GENERIC_ISR, // eUSCI B3 (23)
    CortexM4::GENERIC_ISR, // Precision ADC (24)
    CortexM4::GENERIC_ISR, // Timer32 INT1 (25)
    CortexM4::GENERIC_ISR, // Timer32 INT2 (26)
    CortexM4::GENERIC_ISR, // Timer32 combined interrupt (27)
    CortexM4::GENERIC_ISR, // AES256 (28)
    CortexM4::GENERIC_ISR, // RTC_C (29)
    CortexM4::GENERIC_ISR, // DMA error (30)
    CortexM4::GENERIC_ISR, // DMA INT3 (31)
    CortexM4::GENERIC_ISR, // DMA INT2 (32)
    CortexM4::GENERIC_ISR, // DMA INT1 (33)
    CortexM4::GENERIC_ISR, // DMA INT0 (34)
    CortexM4::GENERIC_ISR, // IO Port 1 (35)
    CortexM4::GENERIC_ISR, // IO Port 2 (36)
    CortexM4::GENERIC_ISR, // IO Port 3 (37)
    CortexM4::GENERIC_ISR, // IO Port 4 (38)
    CortexM4::GENERIC_ISR, // IO Port 5 (39)
    CortexM4::GENERIC_ISR, // IO Port 6 (40)
    unhandled_interrupt,   // Reserved (41)
    unhandled_interrupt,   // Reserved (42)
    unhandled_interrupt,   // Reserved (43)
    unhandled_interrupt,   // Reserved (44)
    unhandled_interrupt,   // Reserved (45)
    unhandled_interrupt,   // Reserved (46)
    unhandled_interrupt,   // Reserved (47)
    unhandled_interrupt,   // Reserved (48)
    unhandled_interrupt,   // Reserved (49)
    unhandled_interrupt,   // Reserved (50)
    unhandled_interrupt,   // Reserved (51)
    unhandled_interrupt,   // Reserved (52)
    unhandled_interrupt,   // Reserved (53)
    unhandled_interrupt,   // Reserved (54)
    unhandled_interrupt,   // Reserved (55)
    unhandled_interrupt,   // Reserved (56)
    unhandled_interrupt,   // Reserved (57)
    unhandled_interrupt,   // Reserved (58)
    unhandled_interrupt,   // Reserved (59)
    unhandled_interrupt,   // Reserved (60)
    unhandled_interrupt,   // Reserved (61)
    unhandled_interrupt,   // Reserved (62)
    unhandled_interrupt,   // Reserved (63)
];

pub unsafe fn init() {
    cortexm4::nvic::disable_all();
    cortexm4::nvic::clear_all_pending();
    cortexm4::nvic::enable_all();
}
