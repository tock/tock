// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Peripheral implementations for the IMXRT1050 and IMXRT1060 MCUs.
//!
//! imxrt1050 chip: <https://www.nxp.com/design/development-boards/i-mx-evaluation-and-development-boards/i-mx-rt1050-evaluation-kit:MIMXRT1050-EVK>

#![no_std]

pub mod chip;
pub mod nvic;

// Peripherals
pub mod ccm;
pub mod ccm_analog;
pub mod dcdc;
pub mod dma;
pub mod gpio;
pub mod gpt;
pub mod iomuxc;
pub mod iomuxc_snvs;
pub mod lpi2c;
pub mod lpuart;

use cortexm7::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM7, CortexMVariant};

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
    CortexM7::HARD_FAULT_HANDLER, // Hard Fault
    unhandled_interrupt,          // MemManage
    unhandled_interrupt,          // BusFault
    unhandled_interrupt,          // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM7::SVC_HANDLER, // SVC
    unhandled_interrupt,   // DebugMon
    unhandled_interrupt,
    unhandled_interrupt,       // PendSV
    CortexM7::SYSTICK_HANDLER, // SysTick
];

// imxrt 1050 has total of 160 interrupts
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 160] = [
    CortexM7::GENERIC_ISR, // eDMA (0)
    CortexM7::GENERIC_ISR, // eDMA (1)
    CortexM7::GENERIC_ISR, // eDMA (2)
    CortexM7::GENERIC_ISR, // eDMA (3)
    CortexM7::GENERIC_ISR, // eDMA (4)
    CortexM7::GENERIC_ISR, // eDMA (5)
    CortexM7::GENERIC_ISR, // eDMA (6)
    CortexM7::GENERIC_ISR, // eDMA (7)
    CortexM7::GENERIC_ISR, // eDMA (8)
    CortexM7::GENERIC_ISR, // eDMA (9)
    CortexM7::GENERIC_ISR, // eDMA (10)
    CortexM7::GENERIC_ISR, // eDMA (11)
    CortexM7::GENERIC_ISR, // eDMA (12)
    CortexM7::GENERIC_ISR, // eDMA (13)
    CortexM7::GENERIC_ISR, // eDMA (14)
    CortexM7::GENERIC_ISR, // eDMA (15)
    CortexM7::GENERIC_ISR, // Error_interrupt (16)
    CortexM7::GENERIC_ISR, // CM7 (17)
    CortexM7::GENERIC_ISR, // CM7 (18)
    CortexM7::GENERIC_ISR, // CM7 (19)
    CortexM7::GENERIC_ISR, // LPUART1 (20)
    CortexM7::GENERIC_ISR, // LPUART2 (21)
    CortexM7::GENERIC_ISR, // LPUART3 (22)
    CortexM7::GENERIC_ISR, // LPUART4 (23)
    CortexM7::GENERIC_ISR, // LPUART5 (24)
    CortexM7::GENERIC_ISR, // LPUART6 (25)
    CortexM7::GENERIC_ISR, // LPUART7 (26)
    CortexM7::GENERIC_ISR, // LPUART8 (27)
    CortexM7::GENERIC_ISR, // LPI2C1 (28)
    CortexM7::GENERIC_ISR, // LPI2C2 (29)
    CortexM7::GENERIC_ISR, // LPI2C3 (30)
    CortexM7::GENERIC_ISR, // LPI2C4 (31)
    CortexM7::GENERIC_ISR, // LPSPI1 (32)
    CortexM7::GENERIC_ISR, // LPSPI2 (33)
    CortexM7::GENERIC_ISR, // LPSPI3 (34)
    CortexM7::GENERIC_ISR, // LPSPI4 (35)
    CortexM7::GENERIC_ISR, // FLEXCAN1 (36)
    CortexM7::GENERIC_ISR, // FLEXCAN2 (37)
    CortexM7::GENERIC_ISR, // CM7 (38)
    CortexM7::GENERIC_ISR, // KPP (39)
    CortexM7::GENERIC_ISR, // TSC_DIG (40)
    CortexM7::GENERIC_ISR, // GPR_IRQ (41)
    CortexM7::GENERIC_ISR, // LCDIF (42)
    CortexM7::GENERIC_ISR, // CSI (43)
    CortexM7::GENERIC_ISR, // PXP (44)
    CortexM7::GENERIC_ISR, // WDOG2 (45)
    CortexM7::GENERIC_ISR, // SNVS_HP_WRAPPER (46)
    CortexM7::GENERIC_ISR, // SNVS_HP_WRAPPER (47)
    CortexM7::GENERIC_ISR, // SNVS_HP_WRAPPER / SNVS_LP_WRAPPER (48)
    CortexM7::GENERIC_ISR, // CSU (49)
    CortexM7::GENERIC_ISR, // DCP (50)
    CortexM7::GENERIC_ISR, // DCP (51)
    CortexM7::GENERIC_ISR, // DCP (52)
    CortexM7::GENERIC_ISR, // TRNG (53)
    CortexM7::GENERIC_ISR, // Reserved (54)
    CortexM7::GENERIC_ISR, // BEE (55)
    CortexM7::GENERIC_ISR, // SAI1 (56)
    CortexM7::GENERIC_ISR, // SAI2 (57)
    CortexM7::GENERIC_ISR, // SAI3 (58)
    CortexM7::GENERIC_ISR, // SAI3 (59)
    CortexM7::GENERIC_ISR, // SPDIF (60)
    CortexM7::GENERIC_ISR, // PMU (61)
    CortexM7::GENERIC_ISR, // Reserved (62)
    CortexM7::GENERIC_ISR, // Temperature Monitor (63)
    CortexM7::GENERIC_ISR, // Temperature Monitor (64)
    CortexM7::GENERIC_ISR, // USB PHY (65)
    CortexM7::GENERIC_ISR, // USB PHY (66)
    CortexM7::GENERIC_ISR, // ADC1 (67)
    CortexM7::GENERIC_ISR, // ADC2 (68)
    CortexM7::GENERIC_ISR, // DCDC (69)
    CortexM7::GENERIC_ISR, // Reserved (70)
    CortexM7::GENERIC_ISR, // Reserved (71)
    CortexM7::GENERIC_ISR, // GPIO1 (72)
    CortexM7::GENERIC_ISR, // GPIO1 (73)
    CortexM7::GENERIC_ISR, // GPIO1 (74)
    CortexM7::GENERIC_ISR, // GPIO1 (75)
    CortexM7::GENERIC_ISR, // GPIO1 (76)
    CortexM7::GENERIC_ISR, // GPIO1 (77)
    CortexM7::GENERIC_ISR, // GPIO1 (78)
    CortexM7::GENERIC_ISR, // GPIO1 (79)
    CortexM7::GENERIC_ISR, // GPIO1_1 (80)
    CortexM7::GENERIC_ISR, // GPIO1_2 (81)
    CortexM7::GENERIC_ISR, // GPIO2_1 (82)
    CortexM7::GENERIC_ISR, // GPIO2_2 (83)
    CortexM7::GENERIC_ISR, // GPIO3_1 (84)
    CortexM7::GENERIC_ISR, // GPIO3_2 (85)
    CortexM7::GENERIC_ISR, // GPIO4_1 (86)
    CortexM7::GENERIC_ISR, // GPIO4_2 (87)
    CortexM7::GENERIC_ISR, // GPIO5_1 (88)
    CortexM7::GENERIC_ISR, // GPIO5_2 (89)
    CortexM7::GENERIC_ISR, // FLEXIO1 (90)
    CortexM7::GENERIC_ISR, // FLEXIO2 (91)
    CortexM7::GENERIC_ISR, // WDOG1 (92)
    CortexM7::GENERIC_ISR, // RTWDOG (93)
    CortexM7::GENERIC_ISR, // EWM (94)
    CortexM7::GENERIC_ISR, // CCM (95)
    CortexM7::GENERIC_ISR, // CCM (96)
    CortexM7::GENERIC_ISR, // GPC (97)
    CortexM7::GENERIC_ISR, // SRC (98)
    CortexM7::GENERIC_ISR, // Reserved (99)
    CortexM7::GENERIC_ISR, // GPT1 (100)
    CortexM7::GENERIC_ISR, // GPT2 (101)
    CortexM7::GENERIC_ISR, // FLEXPWM1 (102)
    CortexM7::GENERIC_ISR, // FLEXPWM1 (103)
    CortexM7::GENERIC_ISR, // FLEXPWM1 (104)
    CortexM7::GENERIC_ISR, // FLEXPWM1 (105)
    CortexM7::GENERIC_ISR, // FLEXPWM1 (106)
    CortexM7::GENERIC_ISR, // Reserved (107)
    CortexM7::GENERIC_ISR, // FLEXSPI (108)
    CortexM7::GENERIC_ISR, // SEMC (109)
    CortexM7::GENERIC_ISR, // USDHC1 (110)
    CortexM7::GENERIC_ISR, // USDHC2 (111)
    CortexM7::GENERIC_ISR, // USB (112)
    CortexM7::GENERIC_ISR, // USB (113)
    CortexM7::GENERIC_ISR, // ENET (114)
    CortexM7::GENERIC_ISR, // ENET (115)
    CortexM7::GENERIC_ISR, // XBAR1 (116)
    CortexM7::GENERIC_ISR, // XBAR1 (117)
    CortexM7::GENERIC_ISR, // ADC_ETC (118)
    CortexM7::GENERIC_ISR, // ADC_ETC (119)
    CortexM7::GENERIC_ISR, // ADC_ETC (120)
    CortexM7::GENERIC_ISR, // ADC_ETC (121)
    CortexM7::GENERIC_ISR, // PIT (122)
    CortexM7::GENERIC_ISR, // ACMP (123)
    CortexM7::GENERIC_ISR, // ACMP (124)
    CortexM7::GENERIC_ISR, // ACMP (125)
    CortexM7::GENERIC_ISR, // ACMP (126)
    CortexM7::GENERIC_ISR, // Reserved (127)
    CortexM7::GENERIC_ISR, // Reserved (128)
    CortexM7::GENERIC_ISR, // ENC1 (129)
    CortexM7::GENERIC_ISR, // ENC2 (130)
    CortexM7::GENERIC_ISR, // ENC3 (131)
    CortexM7::GENERIC_ISR, // ENC4 (132)
    CortexM7::GENERIC_ISR, // QTIMER1 (133)
    CortexM7::GENERIC_ISR, // QTIMER2 (134)
    CortexM7::GENERIC_ISR, // QTIMER3 (135)
    CortexM7::GENERIC_ISR, // QTIMER4 (136)
    CortexM7::GENERIC_ISR, // FLEXPWM2 (137)
    CortexM7::GENERIC_ISR, // FLEXPWM2 (138)
    CortexM7::GENERIC_ISR, // FLEXPWM2 (139)
    CortexM7::GENERIC_ISR, // FLEXPWM2 (140)
    CortexM7::GENERIC_ISR, // FLEXPWM2 (141)
    CortexM7::GENERIC_ISR, // FLEXPWM3 (142)
    CortexM7::GENERIC_ISR, // FLEXPWM3 (143)
    CortexM7::GENERIC_ISR, // FLEXPWM3 (144)
    CortexM7::GENERIC_ISR, // FLEXPWM3 (145)
    CortexM7::GENERIC_ISR, // FLEXPWM3 (146)
    CortexM7::GENERIC_ISR, // FLEXPWM4 (147)
    CortexM7::GENERIC_ISR, // FLEXPWM4 (148)
    CortexM7::GENERIC_ISR, // FLEXPWM4 (149)
    CortexM7::GENERIC_ISR, // FLEXPWM4 (150)
    CortexM7::GENERIC_ISR, // FLEXPWM4 (151)
    CortexM7::GENERIC_ISR, // Reserved (152)
    CortexM7::GENERIC_ISR, // Reserved (153)
    CortexM7::GENERIC_ISR, // Reserved (154)
    CortexM7::GENERIC_ISR, // Reserved (155)
    CortexM7::GENERIC_ISR, // Reserved (156)
    CortexM7::GENERIC_ISR, // Reserved (157)
    CortexM7::GENERIC_ISR, // Reserved (158)
    CortexM7::GENERIC_ISR, // Reserved (159)
];

pub unsafe fn init() {
    cortexm7::nvic::disable_all();
    cortexm7::nvic::clear_all_pending();

    cortexm7::scb::set_vector_table_offset(core::ptr::addr_of!(BASE_VECTORS) as *const ());

    cortexm7::nvic::enable_all();
}
