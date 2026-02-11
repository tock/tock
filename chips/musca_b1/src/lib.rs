// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

#![no_std]
// GPIO has many register definitions in `register_structs()!`
// and requires a deeper recursion limit than the default to fully expand.
#![recursion_limit = "256"]

pub mod chip;
pub mod gpio;
pub mod interrupts;
pub mod timer;
pub mod uart;

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
pub static IRQS: [unsafe extern "C" fn(); 97] = [
    CortexM33::GENERIC_ISR, // NON_SECURE_WATCHDOG_RESET (0)
    CortexM33::GENERIC_ISR, // NON_SECURE_WATCHDOG_INT (1)
    CortexM33::GENERIC_ISR, // S32K_TIMER (2)
    CortexM33::GENERIC_ISR, // TIMER_0 (3)
    CortexM33::GENERIC_ISR, // TIMER_1 (4)
    CortexM33::GENERIC_ISR, // DUAL_TIMER (5)
    CortexM33::GENERIC_ISR, // MHU0_CPU_INT (6)
    CortexM33::GENERIC_ISR, // MHU1_CPU_INT (7)
    unhandled_interrupt,    // Reserved (8)
    CortexM33::GENERIC_ISR, // MPC_COMBINED (9)
    CortexM33::GENERIC_ISR, // PPC_COMBINED (10)
    CortexM33::GENERIC_ISR, // MSC_COMBINED (11)
    CortexM33::GENERIC_ISR, // BRIDGE_ERROR (12)
    CortexM33::GENERIC_ISR, // CPU0_ICACHE_INVALIDATION (13)
    unhandled_interrupt,    // Reserved (14)
    CortexM33::GENERIC_ISR, // SYS_PPU (15)
    CortexM33::GENERIC_ISR, // CPU0_PPU (16)
    CortexM33::GENERIC_ISR, // CPU1_PPU (17)
    CortexM33::GENERIC_ISR, // CPU0_DBG_PPU (18)
    CortexM33::GENERIC_ISR, // CPU1_DBG_PPU (19)
    unhandled_interrupt,    // Reserved (20)
    unhandled_interrupt,    // Reserved (21)
    CortexM33::GENERIC_ISR, // RAM0_PPU (22)
    CortexM33::GENERIC_ISR, // RAM1_PPU (23)
    CortexM33::GENERIC_ISR, // RAM2_PPU (24)
    CortexM33::GENERIC_ISR, // RAM3_PPU (25)
    CortexM33::GENERIC_ISR, // DBG_PPU (26)
    unhandled_interrupt,    // Reserved (27)
    CortexM33::GENERIC_ISR, // CPU_CTI_IRQ0 (28)
    CortexM33::GENERIC_ISR, // CPU_CTI_IRQ1 (29)
    unhandled_interrupt,    // Reserved (30)
    unhandled_interrupt,    // Reserved (31)
    unhandled_interrupt,    // Reserved (32)
    CortexM33::GENERIC_ISR, // GP_TIMER_COMBINED (33)
    CortexM33::GENERIC_ISR, // I2C0 (34)
    CortexM33::GENERIC_ISR, // I2C1 (35)
    CortexM33::GENERIC_ISR, // I2S (36)
    CortexM33::GENERIC_ISR, // SPI (37)
    CortexM33::GENERIC_ISR, // QSPI (38)
    CortexM33::GENERIC_ISR, // UART0_RX (39)
    CortexM33::GENERIC_ISR, // UART0_TX (40)
    CortexM33::GENERIC_ISR, // UART0_RT (41)
    CortexM33::GENERIC_ISR, // UART0_MS (42)
    CortexM33::GENERIC_ISR, // UART0_E (43)
    CortexM33::GENERIC_ISR, // UART0_COMBINED (44)
    CortexM33::GENERIC_ISR, // UART1_RX (45)
    CortexM33::GENERIC_ISR, // UART1_TX (46)
    CortexM33::GENERIC_ISR, // UART1_RT (47)
    CortexM33::GENERIC_ISR, // UART1_MS (48)
    CortexM33::GENERIC_ISR, // UART1_E (49)
    CortexM33::GENERIC_ISR, // UART1_COMBINED (50)
    CortexM33::GENERIC_ISR, // GPIO_0 (51)
    CortexM33::GENERIC_ISR, // GPIO_1 (52)
    CortexM33::GENERIC_ISR, // GPIO_2 (53)
    CortexM33::GENERIC_ISR, // GPIO_3 (54)
    CortexM33::GENERIC_ISR, // GPIO_4 (55)
    CortexM33::GENERIC_ISR, // GPIO_5 (56)
    CortexM33::GENERIC_ISR, // GPIO_6 (57)
    CortexM33::GENERIC_ISR, // GPIO_7 (58)
    CortexM33::GENERIC_ISR, // GPIO_8 (59)
    CortexM33::GENERIC_ISR, // GPIO_9 (60)
    CortexM33::GENERIC_ISR, // GPIO_10 (61)
    CortexM33::GENERIC_ISR, // GPIO_11 (62)
    CortexM33::GENERIC_ISR, // GPIO_12 (63)
    CortexM33::GENERIC_ISR, // GPIO_13 (64)
    CortexM33::GENERIC_ISR, // GPIO_14 (65)
    CortexM33::GENERIC_ISR, // GPIO_15 (66)
    CortexM33::GENERIC_ISR, // GPIO_COMBINED (67)
    CortexM33::GENERIC_ISR, // PVT_SENSOR (68)
    unhandled_interrupt,    // Reserved (69)
    CortexM33::GENERIC_ISR, // PWM0 (70)
    CortexM33::GENERIC_ISR, // RTC (71)
    CortexM33::GENERIC_ISR, // GP_TIMER_INT1 (72)
    CortexM33::GENERIC_ISR, // GP_TIMER_INT0 (73)
    CortexM33::GENERIC_ISR, // PWM1 (74)
    CortexM33::GENERIC_ISR, // PWM2 (75)
    CortexM33::GENERIC_ISR, // GPIO_COMBINED_NONSEC (76)
    CortexM33::GENERIC_ISR, // SDIO (77)
    unhandled_interrupt,    // Reserved (78)
    unhandled_interrupt,    // Reserved (79)
    unhandled_interrupt,    // Reserved (80)
    unhandled_interrupt,    // Reserved (81)
    unhandled_interrupt,    // Reserved (82)
    unhandled_interrupt,    // Reserved (83)
    CortexM33::GENERIC_ISR, // CRYPTO_RESET_STATUS (84)
    CortexM33::GENERIC_ISR, // HOSTMHUS0_ACCESS_NR2R (85)
    CortexM33::GENERIC_ISR, // HOSTMHUS0_ACCESS_R2NR (86)
    CortexM33::GENERIC_ISR, // HOSTMHUR0 (87)
    CortexM33::GENERIC_ISR, // HOSTMHUR0 (88)
    CortexM33::GENERIC_ISR, // HOSTMHUR0_COMBINED (89)
    CortexM33::GENERIC_ISR, // HOSTMHUS1_ACCESS_NR2R (90)
    CortexM33::GENERIC_ISR, // HOSTMHUS1_ACCESS_R2NR (91)
    CortexM33::GENERIC_ISR, // HOSTMHUR1 (92)
    CortexM33::GENERIC_ISR, // HOSTMHUR1 (93)
    CortexM33::GENERIC_ISR, // HOSTMHUR1_COMBINED (94)
    CortexM33::GENERIC_ISR, // FLASH0 (95)
    CortexM33::GENERIC_ISR, // FLASH1 (96)
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
}
