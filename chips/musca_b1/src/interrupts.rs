// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

/* --- Watchdog & Timers --- */
pub const NON_SECURE_WATCHDOG_RESET: u32 = 0;
pub const NON_SECURE_WATCHDOG_INT: u32 = 1;
pub const S32K_TIMER: u32 = 2;
pub const TIMER_0: u32 = 3;
pub const TIMER_1: u32 = 4;
pub const DUAL_TIMER: u32 = 5;

/* --- Message Handling Units (MHU) --- */
pub const MHU0_CPU_INT: u32 = 6;
pub const MHU1_CPU_INT: u32 = 7;

/* --- System Security & Controllers --- */
pub const MPC_COMBINED: u32 = 9; // Secure
pub const PPC_COMBINED: u32 = 10; // Secure
pub const MSC_COMBINED: u32 = 11; // Secure
pub const BRIDGE_ERROR: u32 = 12; // Secure
pub const CPU0_ICACHE_INVALIDATION: u32 = 13;

/* --- Power Policy Units (PPU) --- */
pub const SYS_PPU: u32 = 15;
pub const CPU0_PPU: u32 = 16;
pub const CPU1_PPU: u32 = 17;
pub const CPU0_DBG_PPU: u32 = 18;
pub const CPU1_DBG_PPU: u32 = 19;
pub const RAM0_PPU: u32 = 22;
pub const RAM1_PPU: u32 = 23;
pub const RAM2_PPU: u32 = 24;
pub const RAM3_PPU: u32 = 25;
pub const DBG_PPU: u32 = 26;

/* --- Debug & Control --- */
pub const CPU_CTI_IRQ0: u32 = 28;
pub const CPU_CTI_IRQ1: u32 = 29;

/* --- Peripherals --- */
pub const GP_TIMER_COMBINED: u32 = 33;
pub const I2C0: u32 = 34;
pub const I2C1: u32 = 35;
pub const I2S: u32 = 36;
pub const SPI: u32 = 37;
pub const QSPI: u32 = 38;

/* --- UART 0 --- */
pub const UART0_RX: u32 = 39;
pub const UART0_TX: u32 = 40;
pub const UART0_RT: u32 = 41;
pub const UART0_MS: u32 = 42;
pub const UART0_E: u32 = 43;
pub const UART0_COMBINED: u32 = 44;

/* --- UART 1 --- */
pub const UART1_RX: u32 = 45;
pub const UART1_TX: u32 = 46;
pub const UART1_RT: u32 = 47;
pub const UART1_MS: u32 = 48;
pub const UART1_E: u32 = 49;
pub const UART1_COMBINED: u32 = 50;

/* --- GPIO --- */
pub const GPIO_0: u32 = 51;
pub const GPIO_1: u32 = 52;
pub const GPIO_2: u32 = 53;
pub const GPIO_3: u32 = 54;
pub const GPIO_4: u32 = 55;
pub const GPIO_5: u32 = 56;
pub const GPIO_6: u32 = 57;
pub const GPIO_7: u32 = 58;
pub const GPIO_8: u32 = 59;
pub const GPIO_9: u32 = 60;
pub const GPIO_10: u32 = 61;
pub const GPIO_11: u32 = 62;
pub const GPIO_12: u32 = 63;
pub const GPIO_13: u32 = 64;
pub const GPIO_14: u32 = 65;
pub const GPIO_15: u32 = 66;
pub const GPIO_COMBINED: u32 = 67;
pub const GPIO_COMBINED_NONSEC: u32 = 76;

/* --- Additional Peripherals --- */
pub const PVT_SENSOR: u32 = 68;
pub const PWM0: u32 = 70;
pub const RTC: u32 = 71;
pub const GP_TIMER_INT1: u32 = 72; // Comparator 1
pub const GP_TIMER_INT0: u32 = 73; // Comparator 0
pub const PWM1: u32 = 74;
pub const PWM2: u32 = 75;
pub const SDIO: u32 = 77;

/* --- Crypto --- */
pub const CRYPTO_RESET_STATUS: u32 = 84;
pub const HOSTMHUS0_ACCESS_NR2R: u32 = 85;
pub const HOSTMHUS0_ACCESS_R2NR: u32 = 86;
pub const HOSTMHUR0: u32 = 87; // IRQ 87 & 88
pub const HOSTMHUR0_COMBINED: u32 = 89;
pub const HOSTMHUS1_ACCESS_NR2R: u32 = 90;
pub const HOSTMHUS1_ACCESS_R2NR: u32 = 91;
pub const HOSTMHUR1: u32 = 92; // IRQ 92 & 93
pub const HOSTMHUR1_COMBINED: u32 = 94;
/* --- Flash --- */
pub const FLASH0: u32 = 95;
pub const FLASH1: u32 = 96;
