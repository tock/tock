// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

register_structs! {
    /// SYSCON
    pub SysconRegisters {
        /// Memory Remap control register
        (0x000 => memoryremap: ReadWrite<u32, MEMORYREMAP::Register>),
        (0x004 => _reserved0),
        /// AHB Matrix priority control register Priority values are 3 = highest, 0 = lowest
        (0x010 => ahbmatprio: ReadWrite<u32, AHBMATPRIO::Register>),
        (0x014 => _reserved1),
        /// System tick calibration for secure part of CPU0
        (0x038 => cpu0stckcal: ReadWrite<u32, CPU0STCKCAL::Register>),
        /// System tick calibration for non-secure part of CPU0
        (0x03C => cpu0nstckcal: ReadWrite<u32, CPU0NSTCKCAL::Register>),
        /// System tick calibration for CPU1
        (0x040 => cpu1stckcal: ReadWrite<u32, CPU1STCKCAL::Register>),
        (0x044 => _reserved2),
        /// NMI Source Select
        (0x048 => nmisrc: ReadWrite<u32, NMISRC::Register>),
        (0x04C => _reserved3),
        /// Peripheral reset control 0
        (0x100 => pub presetctrl0: ReadWrite<u32, PRESETCTRL0::Register>),
        /// Peripheral reset control 1
        (0x104 => presetctrl1: ReadWrite<u32, PRESETCTRL1::Register>),
        /// Peripheral reset control 2
        (0x108 => presetctrl2: ReadWrite<u32, PRESETCTRL2::Register>),
        (0x10C => _reserved4),
        /// Peripheral reset control set register
        (0x120 => presetctrlset_0: ReadWrite<u32, PRESETCTRLSET0::Register>),
        /// Peripheral reset control set register
        (0x124 => presetctrlset_1: ReadWrite<u32, PRESETCTRLSET1::Register>),
        /// Peripheral reset control set register
        (0x128 => presetctrlset_2: ReadWrite<u32, PRESETCTRLSET2::Register>),
        (0x12C => _reserved5),
        /// Peripheral reset control clear register
        (0x140 => presetctrlclr_0: ReadWrite<u32>),
        /// Peripheral reset control clear register
        (0x144 => presetctrlclr_1: ReadWrite<u32>),
        /// Peripheral reset control clear register
        (0x148 => presetctrlclr_2: ReadWrite<u32>),
        (0x14C => _reserved6),
        /// generate a software_reset
        (0x160 => swr_reset: WriteOnly<u32>),
        (0x164 => _reserved7),
        /// AHB Clock control 0
        (0x200 => pub ahbclkctrl0: ReadWrite<u32, AHBCLKCTRL0::Register>),
        /// AHB Clock control 1
        (0x204 => pub ahbclkctrl1: ReadWrite<u32, AHBCLKCTRL1::Register>),
        /// AHB Clock control 2
        (0x208 => ahbclkctrl2: ReadWrite<u32, AHBCLKCTRL2::Register>),
        (0x20C => _reserved8),
        /// Peripheral reset control register
        (0x220 => pub ahbclkctrlset_0: ReadWrite<u32, AHBCLKCTRLSET0::Register>),
        /// Peripheral reset control register
        (0x224 => ahbclkctrlset_1: ReadWrite<u32, AHBCLKCTRLSET1::Register>),
        /// Peripheral reset control register
        (0x228 => ahbclkctrlset_2: ReadWrite<u32, AHBCLKCTRLSET2::Register>),
        (0x22C => _reserved9),
        /// Peripheral reset control register
        (0x240 => ahbclkctrlclr_0: ReadWrite<u32, AHBCLKCTRLCLR0::Register>),
        /// Peripheral reset control register
        (0x244 => ahbclkctrlclr_1: ReadWrite<u32, AHBCLKCTRLCLR1::Register>),
        /// Peripheral reset control register
        (0x248 => ahbclkctrlclr_2: ReadWrite<u32, AHBCLKCTRLCLR2::Register>),
        (0x24C => _reserved10),
        /// System Tick Timer for CPU0 source select
        (0x260 => systickclksel0: ReadWrite<u32>),
        /// System Tick Timer for CPU1 source select
        (0x264 => systickclksel1: ReadWrite<u32>),
        /// Trace clock source select
        (0x268 => traceclksel: ReadWrite<u32>),
        /// CTimer 0 clock source select
        (0x26C => pub ctimerclksel0: ReadWrite<u32, CTIMERCLKSEL0::Register>),
        /// CTimer 1 clock source select
        (0x270 => ctimerclksel1: ReadWrite<u32>),
        /// CTimer 2 clock source select
        (0x274 => ctimerclksel2: ReadWrite<u32>),
        /// CTimer 3 clock source select
        (0x278 => ctimerclksel3: ReadWrite<u32>),
        /// CTimer 4 clock source select
        (0x27C => ctimerclksel4: ReadWrite<u32>),
        /// Main clock A source select
        (0x280 => mainclksela: ReadWrite<u32>),
        /// Main clock source select
        (0x284 => mainclkselb: ReadWrite<u32>),
        /// CLKOUT clock source select
        (0x288 => pub clkoutsel: ReadWrite<u32, CLKOUTSEL::Register>),
        (0x28C => _reserved11),
        /// PLL0 clock source select
        (0x290 => pll0clksel: ReadWrite<u32>),
        /// PLL1 clock source select
        (0x294 => pll1clksel: ReadWrite<u32>),
        (0x298 => _reserved12),
        /// ADC clock source select
        (0x2A4 => adcclksel: ReadWrite<u32>),
        /// FS USB clock source select
        (0x2A8 => usb0clksel: ReadWrite<u32>),
        (0x2AC => _reserved13),
        /// Flexcomm Interface 0 clock source select for Fractional Rate Divider
        (0x2B0 => pub fcclksel0: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 1 clock source select for Fractional Rate Divider
        (0x2B4 => pub fcclksel1: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 2 clock source select for Fractional Rate Divider
        (0x2B8 => pub fcclksel2: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 3 clock source select for Fractional Rate Divider
        (0x2BC => pub fcclksel3: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 4 clock source select for Fractional Rate Divider
        (0x2C0 => pub fcclksel4: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 5 clock source select for Fractional Rate Divider
        (0x2C4 => pub fcclksel5: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 6 clock source select for Fractional Rate Divider
        (0x2C8 => pub fcclksel6: ReadWrite<u32, FCCLKSEL::Register>),
        /// Flexcomm Interface 7 clock source select for Fractional Rate Divider
        (0x2CC => pub fcclksel7: ReadWrite<u32, FCCLKSEL::Register>),
        /// HS LSPI clock source select
        (0x2D0 => hslspiclksel: ReadWrite<u32>),
        (0x2D4 => _reserved14),
        /// MCLK clock source select
        (0x2E0 => mclkclksel: ReadWrite<u32>),
        (0x2E4 => _reserved15),
        /// SCTimer/PWM clock source select
        (0x2F0 => sctclksel: ReadWrite<u32>),
        (0x2F4 => _reserved16),
        /// SDIO clock source select
        (0x2F8 => sdioclksel: ReadWrite<u32>),
        (0x2FC => _reserved17),
        /// System Tick Timer divider for CPU0
        (0x300 => systickclkdiv0: ReadWrite<u32, SYSTICKCLKDIV0::Register>),
        /// System Tick Timer divider for CPU1
        (0x304 => systickclkdiv1: ReadWrite<u32, SYSTICKCLKDIV1::Register>),
        /// TRACE clock divider
        (0x308 => traceclkdiv: ReadWrite<u32, TRACECLKDIV::Register>),
        (0x30C => _reserved18),
        /// Fractional rate divider for flexcomm 0
        (0x320 => flexfrg0ctrl: ReadWrite<u32, FLEXFRG0CTRL::Register>),
        /// Fractional rate divider for flexcomm 1
        (0x324 => flexfrg1ctrl: ReadWrite<u32, FLEXFRG1CTRL::Register>),
        /// Fractional rate divider for flexcomm 2
        (0x328 => flexfrg2ctrl: ReadWrite<u32, FLEXFRG2CTRL::Register>),
        /// Fractional rate divider for flexcomm 3
        (0x32C => flexfrg3ctrl: ReadWrite<u32, FLEXFRG3CTRL::Register>),
        /// Fractional rate divider for flexcomm 4
        (0x330 => flexfrg4ctrl: ReadWrite<u32, FLEXFRG4CTRL::Register>),
        /// Fractional rate divider for flexcomm 5
        (0x334 => flexfrg5ctrl: ReadWrite<u32, FLEXFRG5CTRL::Register>),
        /// Fractional rate divider for flexcomm 6
        (0x338 => flexfrg6ctrl: ReadWrite<u32, FLEXFRG6CTRL::Register>),
        /// Fractional rate divider for flexcomm 7
        (0x33C => flexfrg7ctrl: ReadWrite<u32, FLEXFRG7CTRL::Register>),
        (0x340 => _reserved19),
        /// System clock divider
        (0x380 => ahbclkdiv: ReadWrite<u32, AHBCLKDIV::Register>),
        /// CLKOUT clock divider
        (0x384 => clkoutdiv: ReadWrite<u32, CLKOUTDIV::Register>),
        /// FRO_HF (96MHz) clock divider
        (0x388 => frohfdiv: ReadWrite<u32, FROHFDIV::Register>),
        /// WDT clock divider
        (0x38C => wdtclkdiv: ReadWrite<u32, WDTCLKDIV::Register>),
        (0x390 => _reserved20),
        /// ADC clock divider
        (0x394 => adcclkdiv: ReadWrite<u32, ADCCLKDIV::Register>),
        /// USB0 Clock divider
        (0x398 => usb0clkdiv: ReadWrite<u32, USB0CLKDIV::Register>),
        (0x39C => _reserved21),
        /// I2S MCLK clock divider
        (0x3AC => mclkdiv: ReadWrite<u32, MCLKDIV::Register>),
        (0x3B0 => _reserved22),
        /// SCT/PWM clock divider
        (0x3B4 => sctclkdiv: ReadWrite<u32, SCTCLKDIV::Register>),
        (0x3B8 => _reserved23),
        /// SDIO clock divider
        (0x3BC => sdioclkdiv: ReadWrite<u32, SDIOCLKDIV::Register>),
        (0x3C0 => _reserved24),
        /// PLL0 clock divider
        (0x3C4 => pll0clkdiv: ReadWrite<u32, PLL0CLKDIV::Register>),
        (0x3C8 => _reserved25),
        /// Control clock configuration registers access (like xxxDIV, xxxSEL)
        (0x3FC => clockgenupdatelockout: ReadWrite<u32>),
        /// FMC configuration register
        (0x400 => fmccr: ReadWrite<u32, FMCCR::Register>),
        (0x404 => _reserved26),
        /// USB0 need clock control
        (0x40C => usb0needclkctrl: ReadWrite<u32, USB0NEEDCLKCTRL::Register>),
        /// USB0 need clock status
        (0x410 => usb0needclkstat: ReadWrite<u32, USB0NEEDCLKSTAT::Register>),
        (0x414 => _reserved27),
        /// FMCflush control
        (0x41C => fmcflush: WriteOnly<u32>),
        /// MCLK control
        (0x420 => mclkio: ReadWrite<u32>),
        /// USB1 need clock control
        (0x424 => usb1needclkctrl: ReadWrite<u32, USB1NEEDCLKCTRL::Register>),
        /// USB1 need clock status
        (0x428 => usb1needclkstat: ReadWrite<u32, USB1NEEDCLKSTAT::Register>),
        (0x42C => _reserved28),
        /// SDIO CCLKIN phase and delay control
        (0x460 => sdioclkctrl: ReadWrite<u32, SDIOCLKCTRL::Register>),
        (0x464 => _reserved29),
        /// PLL1 550m control
        (0x560 => pll1ctrl: ReadWrite<u32, PLL1CTRL::Register>),
        /// PLL1 550m status
        (0x564 => pll1stat: ReadWrite<u32, PLL1STAT::Register>),
        /// PLL1 550m N divider
        (0x568 => pll1ndec: ReadWrite<u32, PLL1NDEC::Register>),
        /// PLL1 550m M divider
        (0x56C => pll1mdec: ReadWrite<u32, PLL1MDEC::Register>),
        /// PLL1 550m P divider
        (0x570 => pll1pdec: ReadWrite<u32, PLL1PDEC::Register>),
        (0x574 => _reserved30),
        /// PLL0 550m control
        (0x580 => pll0ctrl: ReadWrite<u32, PLL0CTRL::Register>),
        /// PLL0 550m status
        (0x584 => pll0stat: ReadWrite<u32, PLL0STAT::Register>),
        /// PLL0 550m N divider
        (0x588 => pll0ndec: ReadWrite<u32, PLL0NDEC::Register>),
        /// PLL0 550m P divider
        (0x58C => pll0pdec: ReadWrite<u32, PLL0PDEC::Register>),
        /// PLL0 Spread Spectrum Wrapper control register 0
        (0x590 => pll0sscg0: ReadWrite<u32>),
        /// PLL0 Spread Spectrum Wrapper control register 1
        (0x594 => pll0sscg1: ReadWrite<u32, PLL0SSCG1::Register>),
        (0x598 => _reserved31),
        /// Functional retention control register
        (0x704 => funcretentionctrl: ReadWrite<u32, FUNCRETENTIONCTRL::Register>),
        (0x708 => _reserved32),
        /// CPU Control for multiple processors
        (0x800 => cpuctrl: ReadWrite<u32, CPUCTRL::Register>),
        /// Coprocessor Boot Address
        (0x804 => cpboot: ReadWrite<u32>),
        (0x808 => _reserved33),
        /// CPU Status
        (0x80C => cpstat: ReadWrite<u32, CPSTAT::Register>),
        (0x810 => _reserved34),
        /// Various system clock controls : Flash clock (48 MHz) control, clocks to Frequenc
        (0xA18 => clock_ctrl: ReadWrite<u32, CLOCK_CTRL::Register>),
        (0xA1C => _reserved35),
        /// Comparator Interrupt control
        (0xB10 => comp_int_ctrl: ReadWrite<u32, COMP_INT_CTRL::Register>),
        /// Comparator Interrupt status
        (0xB14 => comp_int_status: ReadWrite<u32, COMP_INT_STATUS::Register>),
        (0xB18 => _reserved36),
        /// Control automatic clock gating
        (0xE04 => autoclkgateoverride: ReadWrite<u32, AUTOCLKGATEOVERRIDE::Register>),
        /// Enable bypass of the first stage of synchonization inside GPIO_INT module
        (0xE08 => gpiopsync: ReadWrite<u32>),
        (0xE0C => _reserved37),
        /// Control write access to security registers.
        (0xFA0 => debug_lock_en: ReadWrite<u32>),
        /// Cortex M33 (CPU0) and micro Cortex M33 (CPU1) debug features control.
        (0xFA4 => debug_features: ReadWrite<u32, DEBUG_FEATURES::Register>),
        /// Cortex M33 (CPU0) and micro Cortex M33 (CPU1) debug features control DUPLICATE r
        (0xFA8 => debug_features_dp: ReadWrite<u32, DEBUG_FEATURES_DP::Register>),
        (0xFAC => _reserved38),
        /// block quiddikey/PUF all index.
        (0xFBC => key_block: WriteOnly<u32>),
        /// Debug authentication BEACON register
        (0xFC0 => debug_auth_beacon: ReadWrite<u32>),
        (0xFC4 => _reserved39),
        /// CPUs configuration register
        (0xFD4 => cpucfg: ReadWrite<u32>),
        (0xFD8 => _reserved40),
        /// Device ID
        (0xFF8 => device_id0: ReadOnly<u32>),
        /// Chip revision ID and Number
        (0xFFC => dieid: ReadOnly<u32, DIEID::Register>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
MEMORYREMAP [
    /// Select the location of the vector table :.
    MAP OFFSET(0) NUMBITS(2) [
        /// Vector Table in ROM.
        VectorTableInROM = 0,
        /// Vector Table in RAM.
        VectorTableInRAM = 1,
        /// Vector Table in Flash.
        VectorTableInFlash = 2
    ]
],
AHBMATPRIO [
    /// CPU0 C-AHB bus.
    PRI_CPU0_CBUS OFFSET(0) NUMBITS(2) [],
    /// CPU0 S-AHB bus.
    PRI_CPU0_SBUS OFFSET(2) NUMBITS(2) [],
    /// CPU1 C-AHB bus.
    PRI_CPU1_CBUS OFFSET(4) NUMBITS(2) [],
    /// CPU1 S-AHB bus.
    PRI_CPU1_SBUS OFFSET(6) NUMBITS(2) [],
    /// USB-FS.(USB0)
    PRI_USB_FS OFFSET(8) NUMBITS(2) [],
    /// DMA0 controller priority.
    PRI_SDMA0 OFFSET(10) NUMBITS(2) [],
    /// SDIO.
    PRI_SDIO OFFSET(16) NUMBITS(2) [],
    /// PQ (HW Accelerator).
    PRI_PQ OFFSET(18) NUMBITS(2) [],
    /// HASH_AES.
    PRI_HASH_AES OFFSET(20) NUMBITS(2) [],
    /// USB-HS.(USB1)
    PRI_USB_HS OFFSET(22) NUMBITS(2) [],
    /// DMA1 controller priority.
    PRI_SDMA1 OFFSET(24) NUMBITS(2) []
],
CPU0STCKCAL [
    /// Reload value for 10ms (100Hz) timing, subject to system clock skew errors. If th
    TENMS OFFSET(0) NUMBITS(24) [],
    /// Initial value for the Systick timer.
    SKEW OFFSET(24) NUMBITS(1) [],
    /// Indicates whether the device provides a reference clock to the processor: 0 = re
    NOREF OFFSET(25) NUMBITS(1) []
],
CPU0NSTCKCAL [
    /// Reload value for 10 ms (100 Hz) timing, subject to system clock skew errors. If
    TENMS OFFSET(0) NUMBITS(24) [],
    /// Indicates whether the TENMS value is exact: 0 = TENMS value is exact; 1 = TENMS
    SKEW OFFSET(24) NUMBITS(1) [],
    /// Initial value for the Systick timer.
    NOREF OFFSET(25) NUMBITS(1) []
],
CPU1STCKCAL [
    /// Reload value for 10ms (100Hz) timing, subject to system clock skew errors. If th
    TENMS OFFSET(0) NUMBITS(24) [],
    /// Indicates whether the TENMS value is exact: 0 = TENMS value is exact; 1 = TENMS
    SKEW OFFSET(24) NUMBITS(1) [],
    /// Indicates whether the device provides a reference clock to the processor: 0 = re
    NOREF OFFSET(25) NUMBITS(1) []
],
NMISRC [
    /// The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) fo
    IRQCPU0 OFFSET(0) NUMBITS(6) [],
    /// The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) fo
    IRQCPU1 OFFSET(8) NUMBITS(6) [],
    /// Write a 1 to this bit to enable the Non-Maskable Interrupt (NMI) source selected
    NMIENCPU1 OFFSET(30) NUMBITS(1) [],
    /// Write a 1 to this bit to enable the Non-Maskable Interrupt (NMI) source selected
    NMIENCPU0 OFFSET(31) NUMBITS(1) []
],
PRESETCTRL0 [
    /// ROM reset control.
    ROM_RST OFFSET(1) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SRAM Controller 1 reset control.
    SRAM_CTRL1_RST OFFSET(3) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SRAM Controller 2 reset control.
    SRAM_CTRL2_RST OFFSET(4) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SRAM Controller 3 reset control.
    SRAM_CTRL3_RST OFFSET(5) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SRAM Controller 4 reset control.
    SRAM_CTRL4_RST OFFSET(6) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Flash controller reset control.
    FLASH_RST OFFSET(7) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FMC controller reset control.
    FMC_RST OFFSET(8) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Input Mux reset control.
    MUX_RST OFFSET(11) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// I/O controller reset control.
    IOCON_RST OFFSET(13) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO0 reset control.
    GPIO0_RST OFFSET(14) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO1 reset control.
    GPIO1_RST OFFSET(15) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO2 reset control.
    GPIO2_RST OFFSET(16) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO3 reset control.
    GPIO3_RST OFFSET(17) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Pin interrupt (PINT) reset control.
    PINT_RST OFFSET(18) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Group interrupt (GINT) reset control.
    GINT_RST OFFSET(19) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// DMA0 reset control.
    DMA0_RST OFFSET(20) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// CRCGEN reset control.
    CRCGEN_RST OFFSET(21) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Watchdog Timer reset control.
    WWDT_RST OFFSET(22) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Real Time Clock (RTC) reset control.
    RTC_RST OFFSET(23) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Inter CPU communication Mailbox reset control.
    MAILBOX_RST OFFSET(26) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// ADC reset control.
    ADC_RST OFFSET(27) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ]
],
PRESETCTRLX0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRL1 [
    /// MRT reset control.
    MRT_RST OFFSET(0) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// OS Event Timer reset control.
    OSTIMER_RST OFFSET(1) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SCT reset control.
    SCT_RST OFFSET(2) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SCTIPU reset control.
    SCTIPU_RST OFFSET(6) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// UTICK reset control.
    UTICK_RST OFFSET(10) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC0 reset control.
    FC0_RST OFFSET(11) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC1 reset control.
    FC1_RST OFFSET(12) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC2 reset control.
    FC2_RST OFFSET(13) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC3 reset control.
    FC3_RST OFFSET(14) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC4 reset control.
    FC4_RST OFFSET(15) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC5 reset control.
    FC5_RST OFFSET(16) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC6 reset control.
    FC6_RST OFFSET(17) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// FC7 reset control.
    FC7_RST OFFSET(18) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Timer 2 reset control.
    TIMER2_RST OFFSET(22) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB0 DEV reset control.
    USB0_DEV_RST OFFSET(25) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Timer 0 reset control.
    TIMER0_RST OFFSET(26) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Timer 1 reset control.
    TIMER1_RST OFFSET(27) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ]
],
PRESETCTRLX1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRL2 [
    /// DMA1 reset control.
    DMA1_RST OFFSET(1) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Comparator reset control.
    COMP_RST OFFSET(2) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SDIO reset control.
    SDIO_RST OFFSET(3) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB1 Host reset control.
    USB1_HOST_RST OFFSET(4) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB1 dev reset control.
    USB1_DEV_RST OFFSET(5) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB1 RAM reset control.
    USB1_RAM_RST OFFSET(6) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB1 PHY reset control.
    USB1_PHY_RST OFFSET(7) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Frequency meter reset control.
    FREQME_RST OFFSET(8) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// RNG reset control.
    RNG_RST OFFSET(13) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// SYSCTL Block reset.
    SYSCTL_RST OFFSET(15) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB0 Host Master reset control.
    USB0_HOSTM_RST OFFSET(16) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// USB0 Host Slave reset control.
    USB0_HOSTS_RST OFFSET(17) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// HASH_AES reset control.
    HASH_AES_RST OFFSET(18) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Power Quad reset control.
    PQ_RST OFFSET(19) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// PLU LUT reset control.
    PLULUT_RST OFFSET(20) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Timer 3 reset control.
    TIMER3_RST OFFSET(21) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Timer 4 reset control.
    TIMER4_RST OFFSET(22) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// PUF reset control reset control.
    PUF_RST OFFSET(23) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// Casper reset control.
    CASPER_RST OFFSET(24) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// analog control reset control.
    ANALOG_CTRL_RST OFFSET(27) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// HS LSPI reset control.
    HS_LSPI_RST OFFSET(28) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO secure reset control.
    GPIO_SEC_RST OFFSET(29) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ],
    /// GPIO secure int reset control.
    GPIO_SEC_INT_RST OFFSET(30) NUMBITS(1) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Bloc is reset.
        BlocIsReset = 1
    ]
],
PRESETCTRLX2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
SWR_RESET [
    /// Write 0x5A00_0001 to generate a software_reset.
    SWR_RESET OFFSET(0) NUMBITS(32) [
        /// Bloc is not reset.
        BlocIsNotReset = 0,
        /// Generate a software reset.
        GenerateASoftwareReset = 1509949441
    ]
],
pub AHBCLKCTRL0 [
    /// Enables the clock for the ROM.
    ROM OFFSET(1) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SRAM Controller 1.
    SRAM_CTRL1 OFFSET(3) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SRAM Controller 2.
    SRAM_CTRL2 OFFSET(4) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SRAM Controller 3.
    SRAM_CTRL3 OFFSET(5) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SRAM Controller 4.
    SRAM_CTRL4 OFFSET(6) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Flash controller.
    FLASH OFFSET(7) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FMC controller.
    FMC OFFSET(8) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Input Mux.
    MUX OFFSET(11) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the I/O controller.
    IOCON OFFSET(13) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO0.
    GPIO0 OFFSET(14) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO1.
    GPIO1 OFFSET(15) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO2.
    GPIO2 OFFSET(16) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO3.
    GPIO3 OFFSET(17) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Pin interrupt (PINT).
    PINT OFFSET(18) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Group interrupt (GINT).
    GINT OFFSET(19) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the DMA0.
    DMA0 OFFSET(20) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the CRCGEN.
    CRCGEN OFFSET(21) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Watchdog Timer.
    WWDT OFFSET(22) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Real Time Clock (RTC).
    RTC OFFSET(23) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Inter CPU communication Mailbox.
    MAILBOX OFFSET(26) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the ADC.
    ADC OFFSET(27) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ]
],
AHBCLKCTRLX0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub AHBCLKCTRL1 [
    /// Enables the clock for the MRT.
    MRT OFFSET(0) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the OS Event Timer.
    OSTIMER OFFSET(1) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SCT.
    SCT OFFSET(2) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the UTICK.
    UTICK OFFSET(10) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC0.
    FC0 OFFSET(11) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC1.
    FC1 OFFSET(12) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC2.
    FC2 OFFSET(13) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC3.
    FC3 OFFSET(14) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC4.
    FC4 OFFSET(15) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC5.
    FC5 OFFSET(16) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC6.
    FC6 OFFSET(17) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the FC7.
    FC7 OFFSET(18) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Timer 2.
    TIMER2 OFFSET(22) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB0 DEV.
    USB0_DEV OFFSET(25) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Timer 0.
    TIMER0 OFFSET(26) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Timer 1.
    TIMER1 OFFSET(27) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ]
],
AHBCLKCTRLX1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRL2 [
    /// Enables the clock for the DMA1.
    DMA1 OFFSET(1) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Comparator.
    COMP OFFSET(2) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the SDIO.
    SDIO OFFSET(3) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB1 Host.
    USB1_HOST OFFSET(4) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB1 dev.
    USB1_DEV OFFSET(5) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB1 RAM.
    USB1_RAM OFFSET(6) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB1 PHY.
    USB1_PHY OFFSET(7) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Frequency meter.
    FREQME OFFSET(8) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the RNG.
    RNG OFFSET(13) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// SYSCTL block clock.
    SYSCTL OFFSET(15) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB0 Host Master.
    USB0_HOSTM OFFSET(16) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the USB0 Host Slave.
    USB0_HOSTS OFFSET(17) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the HASH_AES.
    HASH_AES OFFSET(18) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Power Quad.
    PQ OFFSET(19) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the PLU LUT.
    PLULUT OFFSET(20) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Timer 3.
    TIMER3 OFFSET(21) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Timer 4.
    TIMER4 OFFSET(22) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the PUF reset control.
    PUF OFFSET(23) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the Casper.
    CASPER OFFSET(24) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the analog control.
    ANALOG_CTRL OFFSET(27) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the HS LSPI.
    HS_LSPI OFFSET(28) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO secure.
    GPIO_SEC OFFSET(29) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ],
    /// Enables the clock for the GPIO secure int.
    GPIO_SEC_INT OFFSET(30) NUMBITS(1) [
        /// Disable Clock.
        DisableClock = 0,
        /// Enable Clock.
        EnableClock = 1
    ]
],
AHBCLKCTRLX2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
SYSTICKCLKSEL0 [
    /// System Tick Timer for CPU0 source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// System Tick 0 divided clock.
        SystemTick0DividedClock = 0,
        /// FRO 1MHz clock.
        FRO1MHzClock = 1,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 2,
        /// No clock.
        NoClock = 3
    ]
],
SYSTICKCLKSELX0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
SYSTICKCLKSEL1 [
    /// System Tick Timer for CPU1 source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// System Tick 1 divided clock.
        SystemTick1DividedClock = 0,
        /// FRO 1MHz clock.
        FRO1MHzClock = 1,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 2,
        /// No clock.
        NoClock = 3
    ]
],
SYSTICKCLKSELX1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
TRACECLKSEL [
    /// Trace clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Trace divided clock.
        TraceDividedClock = 0,
        /// FRO 1MHz clock.
        FRO1MHzClock = 1,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 2,
        /// No clock.
        NoClock = 3
    ]
],
pub CTIMERCLKSEL0 [
    /// CTimer 0 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6
    ]
],
CTIMERCLKSELX0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
CTIMERCLKSEL1 [
    /// CTimer 1 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6
    ]
],
CTIMERCLKSELX1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
CTIMERCLKSEL2 [
    /// CTimer 2 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6
    ]
],
CTIMERCLKSELX2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
CTIMERCLKSEL3 [
    /// CTimer 3 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6
    ]
],
CTIMERCLKSELX3 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
CTIMERCLKSEL4 [
    /// CTimer 4 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6
    ]
],
CTIMERCLKSELX4 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
MAINCLKSELA [
    /// Main clock A source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// FRO 12 MHz clock.
        FRO12MHzClock = 0,
        /// CLKIN clock.
        CLKINClock = 1,
        /// FRO 1MHz clock.
        FRO1MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3
    ]
],
MAINCLKSELB [
    /// Main clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main Clock A.
        MainClockA = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// PLL1 clock.
        PLL1Clock = 2,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 3
    ]
],
pub CLKOUTSEL [
    /// CLKOUT clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// CLKIN clock.
        CLKINClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// PLL1 clock.
        PLL1Clock = 5,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
PLL0CLKSEL [
    /// PLL0 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// FRO 12 MHz clock.
        FRO12MHzClock = 0,
        /// CLKIN clock.
        CLKINClock = 1,
        /// FRO 1MHz clock.
        FRO1MHzClock = 2,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 3,
        /// No clock.
        NoClock = 4
    ]
],
PLL1CLKSEL [
    /// PLL1 clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// FRO 12 MHz clock.
        FRO12MHzClock = 0,
        /// CLKIN clock.
        CLKINClock = 1,
        /// FRO 1MHz clock.
        FRO1MHzClock = 2,
        /// Oscillator 32kHz clock.
        Oscillator32kHzClock = 3,
        /// No clock.
        NoClock = 4
    ]
],
ADCCLKSEL [
    /// ADC clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 2,
        /// No clock.
        NoClock = 4
    ]
],
USB0CLKSEL [
    /// FS USB clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// PLL1 clock.
        PLL1Clock = 5
    ]
],
pub FCCLKSEL [
    /// Flexcomm Interface 0 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL1 [
    /// Flexcomm Interface 1 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL2 [
    /// Flexcomm Interface 2 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL3 [
    /// Flexcomm Interface 3 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX3 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL4 [
    /// Flexcomm Interface 4 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX4 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL5 [
    /// Flexcomm Interface 5 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX5 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL6 [
    /// Flexcomm Interface 6 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX6 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub FCCLKSEL7 [
    /// Flexcomm Interface 7 clock source select for Fractional Rate Divider.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// MCLK clock.
        MCLKClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6,
        /// No clock.
        NoClock = 7
    ]
],
FCCLKSELX7 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
HSLSPICLKSEL [
    /// HS LSPI clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// system PLL divided clock.
        SystemPLLDividedClock = 1,
        /// FRO 12 MHz clock.
        FRO12MHzClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// FRO 1MHz clock.
        FRO1MHzClock = 4,
        /// No clock.
        NoClock = 5,
        /// Oscillator 32 kHz clock.
        Oscillator32KHzClock = 6
    ]
],
MCLKCLKSEL [
    /// MCLK clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// FRO 96 MHz clock.
        FRO96MHzClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 4
    ]
],
SCTCLKSEL [
    /// SCTimer/PWM clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// CLKIN clock.
        CLKINClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// No clock.
        NoClock = 4,
        /// MCLK clock.
        MCLKClock = 5
    ]
],
SDIOCLKSEL [
    /// SDIO clock source select.
    SEL OFFSET(0) NUMBITS(3) [
        /// Main clock.
        MainClock = 0,
        /// PLL0 clock.
        PLL0Clock = 1,
        /// No clock.
        NoClock = 2,
        /// FRO 96 MHz clock.
        FRO96MHzClock = 3,
        /// PLL1 clock.
        PLL1Clock = 5
    ]
],
SYSTICKCLKDIV0 [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
SYSTICKCLKDIV1 [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
TRACECLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
FLEXFRG0CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG1CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG2CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG3CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL3 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG4CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL4 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG5CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL5 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG6CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL6 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
FLEXFRG7CTRL [
    /// Denominator of the fractional rate divider.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Numerator of the fractional rate divider.
    MULT OFFSET(8) NUMBITS(8) []
],
FLEXFRGXCTRL7 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
CLKOUTDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
FROHFDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
WDTCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(6) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
ADCCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(3) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
USB0CLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
MCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
SCTCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
SDIOCLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
PLL0CLKDIV [
    /// Clock divider value.
    DIV OFFSET(0) NUMBITS(8) [],
    /// Resets the divider counter.
    RESET OFFSET(29) NUMBITS(1) [
        /// Divider is not reset.
        DividerIsNotReset = 0,
        /// Divider is reset.
        DividerIsReset = 1
    ],
    /// Halts the divider counter.
    HALT OFFSET(30) NUMBITS(1) [
        /// Divider clock is running.
        DividerClockIsRunning = 0,
        /// Divider clock is stoped.
        DividerClockIsStoped = 1
    ],
    /// Divider status flag.
    REQFLAG OFFSET(31) NUMBITS(1) [
        /// Divider clock is stable.
        DividerClockIsStable = 0,
        /// Clock frequency is not stable.
        ClockFrequencyIsNotStable = 1
    ]
],
CLOCKGENUPDATELOCKOUT [
    /// Control clock configuration registers access (like xxxDIV, xxxSEL).
    CLOCKGENUPDATELOCKOUT OFFSET(0) NUMBITS(32) [
        /// all hardware clock configruration are freeze.
        AllHardwareClockConfigrurationAreFreeze = 0,
        /// update all clock configuration.
        UpdateAllClockConfiguration = 1
    ]
],
FMCCR [
    /// Instruction fetch configuration.
    FETCHCFG OFFSET(0) NUMBITS(2) [
        /// Instruction fetches from flash are not buffered.
        InstructionFetchesFromFlashAreNotBuffered = 0,
        /// One buffer is used for all instruction fetches.
        OneBufferIsUsedForAllInstructionFetches = 1,
        /// All buffers may be used for instruction fetches.
        AllBuffersMayBeUsedForInstructionFetches = 2
    ],
    /// Data read configuration.
    DATACFG OFFSET(2) NUMBITS(2) [
        /// Data accesses from flash are not buffered.
        DataAccessesFromFlashAreNotBuffered = 0,
        /// One buffer is used for all data accesses.
        OneBufferIsUsedForAllDataAccesses = 1,
        /// All buffers can be used for data accesses.
        AllBuffersCanBeUsedForDataAccesses = 2
    ],
    /// Acceleration enable.
    ACCEL OFFSET(4) NUMBITS(1) [
        /// Flash acceleration is disabled.
        FlashAccelerationIsDisabled = 0,
        /// Flash acceleration is enabled.
        FlashAccelerationIsEnabled = 1
    ],
    /// Prefetch enable.
    PREFEN OFFSET(5) NUMBITS(1) [
        /// No instruction prefetch is performed.
        NoInstructionPrefetchIsPerformed = 0,
        /// Instruction prefetch is enabled.
        InstructionPrefetchIsEnabled = 1
    ],
    /// Prefetch override.
    PREFOVR OFFSET(6) NUMBITS(1) [
        /// Any previously initiated prefetch will be completed.
        AnyPreviouslyInitiatedPrefetchWillBeCompleted = 0,
        /// Any previously initiated prefetch will be aborted, and the next flash line follo
        OVERRIDE = 1
    ],
    /// Flash memory access time.
    FLASHTIM OFFSET(12) NUMBITS(4) [
        /// 1 system clock flash access time (for system clock rates up to 11 MHz).
        _1SystemClockFlashAccessTimeForSystemClockRatesUpTo11MHz = 0,
        /// 2 system clocks flash access time (for system clock rates up to 22 MHz).
        _2SystemClocksFlashAccessTimeForSystemClockRatesUpTo22MHz = 1,
        /// 3 system clocks flash access time (for system clock rates up to 33 MHz).
        _3SystemClocksFlashAccessTimeForSystemClockRatesUpTo33MHz = 2,
        /// 4 system clocks flash access time (for system clock rates up to 44 MHz).
        _4SystemClocksFlashAccessTimeForSystemClockRatesUpTo44MHz = 3,
        /// 5 system clocks flash access time (for system clock rates up to 55 MHz).
        _5SystemClocksFlashAccessTimeForSystemClockRatesUpTo55MHz = 4,
        /// 6 system clocks flash access time (for system clock rates up to 66 MHz).
        _6SystemClocksFlashAccessTimeForSystemClockRatesUpTo66MHz = 5,
        /// 7 system clocks flash access time (for system clock rates up to 77 MHz).
        _7SystemClocksFlashAccessTimeForSystemClockRatesUpTo77MHz = 6,
        /// 8 system clocks flash access time (for system clock rates up to 88 MHz).
        _8SystemClocksFlashAccessTimeForSystemClockRatesUpTo88MHz = 7,
        /// 9 system clocks flash access time (for system clock rates up to 100 MHz).
        _9SystemClocksFlashAccessTimeForSystemClockRatesUpTo100MHz = 8,
        /// 10 system clocks flash access time (for system clock rates up to 115 MHz).
        _10SystemClocksFlashAccessTimeForSystemClockRatesUpTo115MHz = 9,
        /// 11 system clocks flash access time (for system clock rates up to 130 MHz).
        _11SystemClocksFlashAccessTimeForSystemClockRatesUpTo130MHz = 10,
        /// 12 system clocks flash access time (for system clock rates up to 150 MHz).
        _12SystemClocksFlashAccessTimeForSystemClockRatesUpTo150MHz = 11
    ]
],
USB0NEEDCLKCTRL [
    /// USB0 Device USB0_NEEDCLK signal control:.
    AP_FS_DEV_NEEDCLK OFFSET(0) NUMBITS(1) [
        /// Under hardware control.
        UnderHardwareControl = 0,
        /// Forced high.
        ForcedHigh = 1
    ],
    /// USB0 Device USB0_NEEDCLK polarity for triggering the USB0 wake-up interrupt:.
    POL_FS_DEV_NEEDCLK OFFSET(1) NUMBITS(1) [
        /// Falling edge of device USB0_NEEDCLK triggers wake-up.
        FallingEdgeOfDeviceUSB0_NEEDCLKTriggersWakeUp = 0,
        /// Rising edge of device USB0_NEEDCLK triggers wake-up.
        RisingEdgeOfDeviceUSB0_NEEDCLKTriggersWakeUp = 1
    ],
    /// USB0 Host USB0_NEEDCLK signal control:.
    AP_FS_HOST_NEEDCLK OFFSET(2) NUMBITS(1) [
        /// Under hardware control.
        UnderHardwareControl = 0,
        /// Forced high.
        ForcedHigh = 1
    ],
    /// USB0 Host USB0_NEEDCLK polarity for triggering the USB0 wake-up interrupt:.
    POL_FS_HOST_NEEDCLK OFFSET(3) NUMBITS(1) [
        /// Falling edge of device USB0_NEEDCLK triggers wake-up.
        FallingEdgeOfDeviceUSB0_NEEDCLKTriggersWakeUp = 0,
        /// Rising edge of device USB0_NEEDCLK triggers wake-up.
        RisingEdgeOfDeviceUSB0_NEEDCLKTriggersWakeUp = 1
    ]
],
USB0NEEDCLKSTAT [
    /// USB0 Device USB0_NEEDCLK signal status:.
    DEV_NEEDCLK OFFSET(0) NUMBITS(1) [
        /// USB0 Device clock is low.
        USB0DeviceClockIsLow = 0,
        /// USB0 Device clock is high.
        USB0DeviceClockIsHigh = 1
    ],
    /// USB0 Host USB0_NEEDCLK signal status:.
    HOST_NEEDCLK OFFSET(1) NUMBITS(1) [
        /// USB0 Host clock is low.
        USB0HostClockIsLow = 0,
        /// USB0 Host clock is high.
        USB0HostClockIsHigh = 1
    ]
],
FMCFLUSH [
    /// Flush control
    FLUSH OFFSET(0) NUMBITS(1) [
        /// No action is performed.
        NoActionIsPerformed = 0,
        /// Flush the FMC buffer contents.
        FlushTheFMCBufferContents = 1
    ]
],
MCLKIO [
    /// MCLK control.
    MCLKIO OFFSET(0) NUMBITS(1) [
        /// input mode.
        InputMode = 0,
        /// output mode.
        OutputMode = 1
    ]
],
USB1NEEDCLKCTRL [
    /// USB1 Device need_clock signal control:
    AP_HS_DEV_NEEDCLK OFFSET(0) NUMBITS(1) [
        /// HOST_NEEDCLK is under hardware control.
        HOST_NEEDCLKIsUnderHardwareControl = 0,
        /// HOST_NEEDCLK is forced high.
        HOST_NEEDCLKIsForcedHigh = 1
    ],
    /// USB1 device need clock polarity for triggering the USB1_NEEDCLK wake-up interrup
    POL_HS_DEV_NEEDCLK OFFSET(1) NUMBITS(1) [
        /// Falling edge of DEV_NEEDCLK triggers wake-up.
        FallingEdgeOfDEV_NEEDCLKTriggersWakeUp = 0,
        /// Rising edge of DEV_NEEDCLK triggers wake-up.
        RisingEdgeOfDEV_NEEDCLKTriggersWakeUp = 1
    ],
    /// USB1 Host need clock signal control:
    AP_HS_HOST_NEEDCLK OFFSET(2) NUMBITS(1) [
        /// HOST_NEEDCLK is under hardware control.
        HOST_NEEDCLKIsUnderHardwareControl = 0,
        /// HOST_NEEDCLK is forced high.
        HOST_NEEDCLKIsForcedHigh = 1
    ],
    /// USB1 host need clock polarity for triggering the USB1_NEEDCLK wake-up interrupt.
    POL_HS_HOST_NEEDCLK OFFSET(3) NUMBITS(1) [
        /// Falling edge of HOST_NEEDCLK triggers wake-up.
        FallingEdgeOfHOST_NEEDCLKTriggersWakeUp = 0,
        /// Rising edge of HOST_NEEDCLK triggers wake-up.
        RisingEdgeOfHOST_NEEDCLKTriggersWakeUp = 1
    ],
    /// Software override of device controller PHY wake up logic.
    HS_DEV_WAKEUP_N OFFSET(4) NUMBITS(1) [
        /// Forces USB1_PHY to wake-up.
        ForcesUSB1_PHYToWakeUp = 0,
        /// Normal USB1_PHY behavior.
        NormalUSB1_PHYBehavior = 1
    ]
],
USB1NEEDCLKSTAT [
    /// USB1 Device need_clock signal status:.
    DEV_NEEDCLK OFFSET(0) NUMBITS(1) [
        /// DEV_NEEDCLK is low.
        DEV_NEEDCLKIsLow = 0,
        /// DEV_NEEDCLK is high.
        DEV_NEEDCLKIsHigh = 1
    ],
    /// USB1 Host need_clock signal status:.
    HOST_NEEDCLK OFFSET(1) NUMBITS(1) [
        /// HOST_NEEDCLK is low.
        HOST_NEEDCLKIsLow = 0,
        /// HOST_NEEDCLK is high.
        HOST_NEEDCLKIsHigh = 1
    ]
],
SDIOCLKCTRL [
    /// Programmable delay value by which cclk_in_drv is phase-shifted with regard to cc
    CCLK_DRV_PHASE OFFSET(0) NUMBITS(2) [
        /// 0 degree shift.
        _0DegreeShift = 0,
        /// 90 degree shift.
        _90DegreeShift = 1,
        /// 180 degree shift.
        _180DegreeShift = 2,
        /// 270 degree shift.
        _270DegreeShift = 3
    ],
    /// Programmable delay value by which cclk_in_sample is delayed with regard to cclk_
    CCLK_SAMPLE_PHASE OFFSET(2) NUMBITS(2) [
        /// 0 degree shift.
        _0DegreeShift = 0,
        /// 90 degree shift.
        _90DegreeShift = 1,
        /// 180 degree shift.
        _180DegreeShift = 2,
        /// 270 degree shift.
        _270DegreeShift = 3
    ],
    /// Enables the delays CCLK_DRV_PHASE and CCLK_SAMPLE_PHASE.
    PHASE_ACTIVE OFFSET(7) NUMBITS(1) [
        /// Bypassed.
        Bypassed = 0,
        /// Activates phase shift logic. When active, the clock divider is active and phase
        PH_SHIFT = 1
    ],
    /// Programmable delay value by which cclk_in_drv is delayed with regard to cclk_in.
    CCLK_DRV_DELAY OFFSET(16) NUMBITS(5) [],
    /// Enables drive delay, as controlled by the CCLK_DRV_DELAY field.
    CCLK_DRV_DELAY_ACTIVE OFFSET(23) NUMBITS(1) [
        /// Disable drive delay.
        DisableDriveDelay = 0,
        /// Enable drive delay.
        EnableDriveDelay = 1
    ],
    /// Programmable delay value by which cclk_in_sample is delayed with regard to cclk_
    CCLK_SAMPLE_DELAY OFFSET(24) NUMBITS(5) [],
    /// Enables sample delay, as controlled by the CCLK_SAMPLE_DELAY field.
    CCLK_SAMPLE_DELAY_ACTIVE OFFSET(31) NUMBITS(1) [
        /// Disables sample delay.
        DisablesSampleDelay = 0,
        /// Enables sample delay.
        EnablesSampleDelay = 1
    ]
],
PLL1CTRL [
    /// Bandwidth select R value.
    SELR OFFSET(0) NUMBITS(4) [],
    /// Bandwidth select I value.
    SELI OFFSET(4) NUMBITS(6) [],
    /// Bandwidth select P value.
    SELP OFFSET(10) NUMBITS(5) [],
    /// Bypass PLL input clock is sent directly to the PLL output (default).
    BYPASSPLL OFFSET(15) NUMBITS(1) [
        /// use PLL.
        UsePLL = 0,
        /// PLL input clock is sent directly to the PLL output.
        PLLInputClockIsSentDirectlyToThePLLOutput = 1
    ],
    /// bypass of the divide-by-2 divider in the post-divider.
    BYPASSPOSTDIV2 OFFSET(16) NUMBITS(1) [
        /// use the divide-by-2 divider in the post-divider.
        UseTheDivideBy2DividerInThePostDivider = 0,
        /// bypass of the divide-by-2 divider in the post-divider.
        BypassOfTheDivideBy2DividerInThePostDivider = 1
    ],
    /// limup_off = 1 in spread spectrum and fractional PLL applications.
    LIMUPOFF OFFSET(17) NUMBITS(1) [],
    /// control of the bandwidth of the PLL.
    BWDIRECT OFFSET(18) NUMBITS(1) [
        /// the bandwidth is changed synchronously with the feedback-divider.
        TheBandwidthIsChangedSynchronouslyWithTheFeedbackDivider = 0,
        /// modify the bandwidth of the PLL directly.
        ModifyTheBandwidthOfThePLLDirectly = 1
    ],
    /// bypass of the pre-divider.
    BYPASSPREDIV OFFSET(19) NUMBITS(1) [
        /// use the pre-divider.
        UseThePreDivider = 0,
        /// bypass of the pre-divider.
        BypassOfThePreDivider = 1
    ],
    /// bypass of the post-divider.
    BYPASSPOSTDIV OFFSET(20) NUMBITS(1) [
        /// use the post-divider.
        UseThePostDivider = 0,
        /// bypass of the post-divider.
        BypassOfThePostDivider = 1
    ],
    /// enable the output clock.
    CLKEN OFFSET(21) NUMBITS(1) [
        /// Disable the output clock.
        DisableTheOutputClock = 0,
        /// Enable the output clock.
        EnableTheOutputClock = 1
    ],
    /// 1: free running mode.
    FRMEN OFFSET(22) NUMBITS(1) [],
    /// free running mode clockstable: Warning: Only make frm_clockstable = 1 after the
    FRMCLKSTABLE OFFSET(23) NUMBITS(1) [],
    /// Skew mode.
    SKEWEN OFFSET(24) NUMBITS(1) [
        /// skewmode is disable.
        SkewmodeIsDisable = 0,
        /// skewmode is enable.
        SkewmodeIsEnable = 1
    ]
],
PLL1STAT [
    /// lock detector output (active high) Warning: The lock signal is only reliable bet
    LOCK OFFSET(0) NUMBITS(1) [],
    /// pre-divider ratio change acknowledge.
    PREDIVACK OFFSET(1) NUMBITS(1) [],
    /// feedback divider ratio change acknowledge.
    FEEDDIVACK OFFSET(2) NUMBITS(1) [],
    /// post-divider ratio change acknowledge.
    POSTDIVACK OFFSET(3) NUMBITS(1) [],
    /// free running detector output (active high).
    FRMDET OFFSET(4) NUMBITS(1) []
],
PLL1NDEC [
    /// pre-divider divider ratio (N-divider).
    NDIV OFFSET(0) NUMBITS(8) [],
    /// pre-divider ratio change request.
    NREQ OFFSET(8) NUMBITS(1) []
],
PLL1MDEC [
    /// feedback divider divider ratio (M-divider).
    MDIV OFFSET(0) NUMBITS(16) [],
    /// feedback ratio change request.
    MREQ OFFSET(16) NUMBITS(1) []
],
PLL1PDEC [
    /// post-divider divider ratio (P-divider)
    PDIV OFFSET(0) NUMBITS(5) [],
    /// feedback ratio change request.
    PREQ OFFSET(5) NUMBITS(1) []
],
PLL0CTRL [
    /// Bandwidth select R value.
    SELR OFFSET(0) NUMBITS(4) [],
    /// Bandwidth select I value.
    SELI OFFSET(4) NUMBITS(6) [],
    /// Bandwidth select P value.
    SELP OFFSET(10) NUMBITS(5) [],
    /// Bypass PLL input clock is sent directly to the PLL output (default).
    BYPASSPLL OFFSET(15) NUMBITS(1) [
        /// use PLL.
        UsePLL = 0,
        /// Bypass PLL input clock is sent directly to the PLL output.
        BypassPLLInputClockIsSentDirectlyToThePLLOutput = 1
    ],
    /// bypass of the divide-by-2 divider in the post-divider.
    BYPASSPOSTDIV2 OFFSET(16) NUMBITS(1) [
        /// use the divide-by-2 divider in the post-divider.
        UseTheDivideBy2DividerInThePostDivider = 0,
        /// bypass of the divide-by-2 divider in the post-divider.
        BypassOfTheDivideBy2DividerInThePostDivider = 1
    ],
    /// limup_off = 1 in spread spectrum and fractional PLL applications.
    LIMUPOFF OFFSET(17) NUMBITS(1) [],
    /// Control of the bandwidth of the PLL.
    BWDIRECT OFFSET(18) NUMBITS(1) [
        /// the bandwidth is changed synchronously with the feedback-divider.
        TheBandwidthIsChangedSynchronouslyWithTheFeedbackDivider = 0,
        /// modify the bandwidth of the PLL directly.
        ModifyTheBandwidthOfThePLLDirectly = 1
    ],
    /// bypass of the pre-divider.
    BYPASSPREDIV OFFSET(19) NUMBITS(1) [
        /// use the pre-divider.
        UseThePreDivider = 0,
        /// bypass of the pre-divider.
        BypassOfThePreDivider = 1
    ],
    /// bypass of the post-divider.
    BYPASSPOSTDIV OFFSET(20) NUMBITS(1) [
        /// use the post-divider.
        UseThePostDivider = 0,
        /// bypass of the post-divider.
        BypassOfThePostDivider = 1
    ],
    /// enable the output clock.
    CLKEN OFFSET(21) NUMBITS(1) [
        /// disable the output clock.
        DisableTheOutputClock = 0,
        /// enable the output clock.
        EnableTheOutputClock = 1
    ],
    /// free running mode.
    FRMEN OFFSET(22) NUMBITS(1) [
        /// free running mode is disable.
        FreeRunningModeIsDisable = 0,
        /// free running mode is enable.
        FreeRunningModeIsEnable = 1
    ],
    /// free running mode clockstable: Warning: Only make frm_clockstable =1 after the P
    FRMCLKSTABLE OFFSET(23) NUMBITS(1) [],
    /// skew mode.
    SKEWEN OFFSET(24) NUMBITS(1) [
        /// skew mode is disable.
        SkewModeIsDisable = 0,
        /// skew mode is enable.
        SkewModeIsEnable = 1
    ]
],
PLL0STAT [
    /// lock detector output (active high) Warning: The lock signal is only reliable bet
    LOCK OFFSET(0) NUMBITS(1) [],
    /// pre-divider ratio change acknowledge.
    PREDIVACK OFFSET(1) NUMBITS(1) [],
    /// feedback divider ratio change acknowledge.
    FEEDDIVACK OFFSET(2) NUMBITS(1) [],
    /// post-divider ratio change acknowledge.
    POSTDIVACK OFFSET(3) NUMBITS(1) [],
    /// free running detector output (active high).
    FRMDET OFFSET(4) NUMBITS(1) []
],
PLL0NDEC [
    /// pre-divider divider ratio (N-divider).
    NDIV OFFSET(0) NUMBITS(8) [],
    /// pre-divider ratio change request.
    NREQ OFFSET(8) NUMBITS(1) []
],
PLL0PDEC [
    /// post-divider divider ratio (P-divider)
    PDIV OFFSET(0) NUMBITS(5) [],
    /// feedback ratio change request.
    PREQ OFFSET(5) NUMBITS(1) []
],
PLL0SSCG0 [
    /// input word of the wrapper bit 31 to 0.
    MD_LBS OFFSET(0) NUMBITS(32) []
],
PLL0SSCG1 [
    /// input word of the wrapper bit 32.
    MD_MBS OFFSET(0) NUMBITS(1) [],
    /// md change request.
    MD_REQ OFFSET(1) NUMBITS(1) [],
    /// programmable modulation frequency fm = Fref/Nss mf[2:0] = 000 => Nss=512 (fm ~ 3
    MF OFFSET(2) NUMBITS(3) [],
    /// programmable frequency modulation depth Dfmodpk-pk = Fref*kss/Fcco = kss/(2*md[3
    MR OFFSET(5) NUMBITS(3) [],
    /// modulation waveform control Compensation for low pass filtering of the PLL to ge
    MC OFFSET(8) NUMBITS(2) [],
    /// to select an external mdiv value.
    MDIV_EXT OFFSET(10) NUMBITS(16) [],
    /// to select an external mreq value.
    MREQ OFFSET(26) NUMBITS(1) [],
    /// dithering between two modulation frequencies in a random way or in a pseudo rand
    DITHER OFFSET(27) NUMBITS(1) [],
    /// to select mdiv_ext and mreq_ext sel_ext = 0: mdiv ~ md[32:0], mreq = 1 sel_ext =
    SEL_EXT OFFSET(28) NUMBITS(1) []
],
FUNCRETENTIONCTRL [
    /// functional retention in power down only.
    FUNCRETENA OFFSET(0) NUMBITS(1) [
        /// disable functional retention.
        DisableFunctionalRetention = 0,
        /// enable functional retention.
        EnableFunctionalRetention = 1
    ],
    /// Start address divided by 4 inside SRAMX bank.
    RET_START OFFSET(1) NUMBITS(13) [],
    /// lenth of Scan chains to save.
    RET_LENTH OFFSET(14) NUMBITS(10) []
],
CPUCTRL [
    /// CPU1 clock enable.
    CPU1CLKEN OFFSET(3) NUMBITS(1) [
        /// The CPU1 clock is not enabled.
        TheCPU1ClockIsNotEnabled = 0,
        /// The CPU1 clock is enabled.
        TheCPU1ClockIsEnabled = 1
    ],
    /// CPU1 reset.
    CPU1RSTEN OFFSET(5) NUMBITS(1) [
        /// The CPU1 is not being reset.
        TheCPU1IsNotBeingReset = 0,
        /// The CPU1 is being reset.
        TheCPU1IsBeingReset = 1
    ]
],
CPBOOT [
    /// Coprocessor Boot Address for CPU1.
    CPBOOT OFFSET(0) NUMBITS(32) []
],
CPSTAT [
    /// The CPU0 sleeping state.
    CPU0SLEEPING OFFSET(0) NUMBITS(1) [
        /// the CPU is not sleeping.
        TheCPUIsNotSleeping = 0,
        /// the CPU is sleeping.
        TheCPUIsSleeping = 1
    ],
    /// The CPU1 sleeping state.
    CPU1SLEEPING OFFSET(1) NUMBITS(1) [
        /// the CPU is not sleeping.
        TheCPUIsNotSleeping = 0,
        /// the CPU is sleeping.
        TheCPUIsSleeping = 1
    ],
    /// The CPU0 lockup state.
    CPU0LOCKUP OFFSET(2) NUMBITS(1) [
        /// the CPU is not in lockup.
        TheCPUIsNotInLockup = 0,
        /// the CPU is in lockup.
        TheCPUIsInLockup = 1
    ],
    /// The CPU1 lockup state.
    CPU1LOCKUP OFFSET(3) NUMBITS(1) [
        /// the CPU is not in lockup.
        TheCPUIsNotInLockup = 0,
        /// the CPU is in lockup.
        TheCPUIsInLockup = 1
    ]
],
CLOCK_CTRL [
    /// Enable XTAL32MHz clock for Frequency Measure module.
    XTAL32MHZ_FREQM_ENA OFFSET(1) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable FRO 1MHz clock for Frequency Measure module and for UTICK.
    FRO1MHZ_UTICK_ENA OFFSET(2) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable FRO 12MHz clock for Frequency Measure module.
    FRO12MHZ_FREQM_ENA OFFSET(3) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable FRO 96MHz clock for Frequency Measure module.
    FRO_HF_FREQM_ENA OFFSET(4) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable clock_in clock for clock module.
    CLKIN_ENA OFFSET(5) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable FRO 1MHz clock for clock muxing in clock gen.
    FRO1MHZ_CLK_ENA OFFSET(6) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable FRO 12MHz clock for analog control of the FRO 192MHz.
    ANA_FRO12M_CLK_ENA OFFSET(7) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable clock for cristal oscilator calibration.
    XO_CAL_CLK_ENA OFFSET(8) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ],
    /// Enable clocks FRO_1MHz and FRO_12MHz for PLU deglitching.
    PLU_DEGLITCH_CLK_ENA OFFSET(9) NUMBITS(1) [
        /// The clock is not enabled.
        TheClockIsNotEnabled = 0,
        /// The clock is enabled.
        TheClockIsEnabled = 1
    ]
],
COMP_INT_CTRL [
    /// Analog Comparator interrupt enable control:.
    INT_ENABLE OFFSET(0) NUMBITS(1) [
        /// interrupt disable.
        InterruptDisable = 0,
        /// interrupt enable.
        InterruptEnable = 1
    ],
    /// Analog Comparator interrupt clear.
    INT_CLEAR OFFSET(1) NUMBITS(1) [
        /// No effect.
        NoEffect = 0,
        /// Clear the interrupt. Self-cleared bit.
        ClearTheInterruptSelfClearedBit = 1
    ],
    /// Comparator interrupt type selector:.
    INT_CTRL OFFSET(2) NUMBITS(3) [
        /// The analog comparator interrupt edge sensitive is disabled.
        TheAnalogComparatorInterruptEdgeSensitiveIsDisabled = 0,
        /// The analog comparator interrupt level sensitive is disabled.
        TheAnalogComparatorInterruptLevelSensitiveIsDisabled = 1,
        /// analog comparator interrupt is rising edge sensitive.
        AnalogComparatorInterruptIsRisingEdgeSensitive = 2,
        /// Analog Comparator interrupt is high level sensitive.
        AnalogComparatorInterruptIsHighLevelSensitive = 3,
        /// analog comparator interrupt is falling edge sensitive.
        AnalogComparatorInterruptIsFallingEdgeSensitive = 4,
        /// Analog Comparator interrupt is low level sensitive.
        AnalogComparatorInterruptIsLowLevelSensitive = 5,
        /// analog comparator interrupt is rising and falling edge sensitive.
        AnalogComparatorInterruptIsRisingAndFallingEdgeSensitive = 6
    ],
    /// Select which Analog comparator output (filtered our un-filtered) is used for int
    INT_SOURCE OFFSET(5) NUMBITS(1) [
        /// Select Analog Comparator filtered output as input for interrupt detection.
        SelectAnalogComparatorFilteredOutputAsInputForInterruptDetection = 0,
        /// Select Analog Comparator raw output (unfiltered) as input for interrupt detectio
        RAW_INT = 1
    ]
],
COMP_INT_STATUS [
    /// Interrupt status BEFORE Interrupt Enable.
    STATUS OFFSET(0) NUMBITS(1) [
        /// no interrupt pending.
        NoInterruptPending = 0,
        /// interrupt pending.
        InterruptPending = 1
    ],
    /// Interrupt status AFTER Interrupt Enable.
    INT_STATUS OFFSET(1) NUMBITS(1) [
        /// no interrupt pending.
        NoInterruptPending = 0,
        /// interrupt pending.
        InterruptPending = 1
    ],
    /// comparator analog output.
    VAL OFFSET(2) NUMBITS(1) [
        /// P+ is smaller than P-.
        PIsSmallerThanP = 0,
        /// P+ is greater than P-.
        PIsGreaterThanP = 1
    ]
],
AUTOCLKGATEOVERRIDE [
    /// Control automatic clock gating of ROM controller.
    ROM OFFSET(0) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAMX controller.
    RAMX_CTRL OFFSET(1) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAM0 controller.
    RAM0_CTRL OFFSET(2) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAM1 controller.
    RAM1_CTRL OFFSET(3) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAM2 controller.
    RAM2_CTRL OFFSET(4) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAM3 controller.
    RAM3_CTRL OFFSET(5) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of RAM4 controller.
    RAM4_CTRL OFFSET(6) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of synchronous bridge controller 0.
    SYNC0_APB OFFSET(7) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of synchronous bridge controller 1.
    SYNC1_APB OFFSET(8) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of CRCGEN controller.
    CRCGEN OFFSET(11) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of DMA0 controller.
    SDMA0 OFFSET(12) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of DMA1 controller.
    SDMA1 OFFSET(13) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of USB controller.
    USB0 OFFSET(14) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// Control automatic clock gating of synchronous system controller registers bank.
    SYSCON OFFSET(15) NUMBITS(1) [
        /// Automatic clock gating is not overridden.
        AutomaticClockGatingIsNotOverridden = 0,
        /// Automatic clock gating is overridden (Clock gating is disabled).
        AutomaticClockGatingIsOverriddenClockGatingIsDisabled = 1
    ],
    /// The value 0xC0DE must be written for AUTOCLKGATEOVERRIDE registers fields update
    ENABLEUPDATE OFFSET(16) NUMBITS(16) [
        /// Bit Fields 0 - 15 of this register are not updated
        BitFields015OfThisRegisterAreNotUpdated = 0,
        /// Bit Fields 0 - 15 of this register are updated
        BitFields015OfThisRegisterAreUpdated = 49374
    ]
],
GPIOPSYNC [
    /// Enable bypass of the first stage of synchonization inside GPIO_INT module.
    PSYNC OFFSET(0) NUMBITS(1) [
        /// use the first stage of synchonization inside GPIO_INT module.
        UseTheFirstStageOfSynchonizationInsideGPIO_INTModule = 0,
        /// bypass of the first stage of synchonization inside GPIO_INT module.
        BypassOfTheFirstStageOfSynchonizationInsideGPIO_INTModule = 1
    ]
],
DEBUG_LOCK_EN [
    /// Control write access to CODESECURITYPROTTEST, CODESECURITYPROTCPU0, CODESECURITY
    LOCK_ALL OFFSET(0) NUMBITS(4) [
        /// Any other value than b1010: disable write access to all 6 registers.
        AnyOtherValueThanB1010DisableWriteAccessToAll6Registers = 0,
        /// 1010: Enable write access to all 6 registers.
        _1010EnableWriteAccessToAll6Registers = 10
    ]
],
DEBUG_FEATURES [
    /// CPU0 Invasive debug control:.
    CPU0_DBGEN OFFSET(0) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Non Invasive debug control:.
    CPU0_NIDEN OFFSET(2) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Secure Invasive debug control:.
    CPU0_SPIDEN OFFSET(4) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Secure Non Invasive debug control:.
    CPU0_SPNIDEN OFFSET(6) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU1 Invasive debug control:.
    CPU1_DBGEN OFFSET(8) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU1 Non Invasive debug control:.
    CPU1_NIDEN OFFSET(10) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ]
],
DEBUG_FEATURES_DP [
    /// CPU0 (CPU0) Invasive debug control:.
    CPU0_DBGEN OFFSET(0) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Non Invasive debug control:.
    CPU0_NIDEN OFFSET(2) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Secure Invasive debug control:.
    CPU0_SPIDEN OFFSET(4) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU0 Secure Non Invasive debug control:.
    CPU0_SPNIDEN OFFSET(6) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU1 Invasive debug control:.
    CPU1_DBGEN OFFSET(8) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ],
    /// CPU1 Non Invasive debug control:.
    CPU1_NIDEN OFFSET(10) NUMBITS(2) [
        /// Any other value than b10: invasive debug is disable.
        AnyOtherValueThanB10InvasiveDebugIsDisable = 1,
        /// 10: Invasive debug is enabled.
        _10InvasiveDebugIsEnabled = 2
    ]
],
KEY_BLOCK [
    /// Write a value to block quiddikey/PUF all index.
    KEY_BLOCK OFFSET(0) NUMBITS(32) []
],
DEBUG_AUTH_BEACON [
    /// Set by the debug authentication code in ROM to pass the debug beacons (Credentia
    BEACON OFFSET(0) NUMBITS(32) []
],
CPUCFG [
    /// Enable CPU1.
    CPU1ENABLE OFFSET(2) NUMBITS(1) [
        /// CPU1 is disable (Processor in reset).
        CPU1IsDisableProcessorInReset = 0,
        /// CPU1 is enable.
        CPU1IsEnable = 1
    ]
],
DEVICE_ID0 [
    /// ROM revision.
    ROM_REV_MINOR OFFSET(20) NUMBITS(4) []
],
DIEID [
    /// Chip Metal Revision ID.
    REV_ID OFFSET(0) NUMBITS(4) [],
    /// Chip Number 0x426B.
    MCO_NUM_IN_DIE_ID OFFSET(4) NUMBITS(20) []
],
PRESETCTRLSET0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRLSET1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRLSET2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRLCLR0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRLCLR1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
PRESETCTRLCLR2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
pub AHBCLKCTRLSET0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRLSET1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRLSET2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRLCLR0 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRLCLR1 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
],
AHBCLKCTRLCLR2 [
    /// Data array value
    DATA OFFSET(0) NUMBITS(32) []
]
];
pub const SYSCON_BASE: StaticRef<SysconRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const SysconRegisters) };
