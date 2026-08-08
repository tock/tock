// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Adapted from embassy-rs/embassy/embassy-stm32/src/rcc/u5.rs

use super::hertz::Hertz;
use super::{
    Rcc,
    values::{
        AHBPrescaler, APBPrescaler, Adcdacsel, Adfsel, Dacsel, Fdcansel, I2c3sel, I2csel, Iclksel,
        Lptim2sel, Lptimsel, Lpusartsel, MsiRange, Octospisel, PllDiv, PllMul, PllPreDiv,
        PllSource, Rngsel, Rtcsel, Saessel, Saisel, Sdmmcsel, Spi1sel, Spi2sel, Spi3sel, Sysclk,
        Usart1sel, Usartsel,
    },
};
use crate::pwr::VoltageScale;

#[derive(Copy, Clone)]
pub struct RccConfig {
    /// The voltage range influences the maximum clock frequencies for different parts of the device
    ///
    /// In particular, system clocks exceeding 110 MHz require `RANGE1`, and system clocks exceeding
    /// 55 MHz require at least `RANGE2`
    ///
    /// See RM0456 § 10.5.4 for a general overview and § 11.4.10 for clock source frequency limits
    pub voltage_range: VoltageScale,

    // Base clock sources
    pub msis: Option<MsiRange>,
    pub msik: Option<MsiRange>,
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub hsi48: bool,

    // PLL
    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    pub pll3: Option<Pll>,

    // SYSCLK, buses
    pub sys: Sysclk,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,

    // Per-peripheral kernel clock selection muxes
    pub mux: ClockMuxConfig,
}

/// Configuration for peripheral clock sources
#[derive(Copy, Clone, Default)]
pub struct ClockMuxConfig {
    pub rtcsel: Rtcsel,
    pub fdcan1sel: Fdcansel,
    pub i2c1sel: I2csel,
    pub i2c2sel: I2csel,
    pub i2c4sel: I2csel,
    pub iclksel: Iclksel,
    pub lptim2sel: Lptim2sel,
    pub spi1sel: Spi1sel,
    pub spi2sel: Spi2sel,
    pub uart4sel: Usartsel,
    pub uart5sel: Usartsel,
    pub usart1sel: Usart1sel,
    pub usart3sel: Usartsel,
    pub octospisel: Octospisel,
    pub rngsel: Rngsel,
    pub saessel: Saessel,
    pub sai1sel: Saisel,
    pub sdmmcsel: Sdmmcsel,
    pub adcdacsel: Adcdacsel,
    pub adf1sel: Adfsel,
    pub dac1sel: Dacsel,
    pub i2c3sel: I2c3sel,
    pub lptim1sel: Lptimsel,
    pub lpuart1sel: Lpusartsel,
    pub spi3sel: Spi3sel,
}
impl ClockMuxConfig {
    pub(crate) fn init(&self, rcc: &Rcc) {
        rcc.set_clock_sources(
            // BDCR
            self.rtcsel,
            // CCIPR1
            self.fdcan1sel,
            self.i2c1sel,
            self.i2c2sel,
            self.i2c4sel,
            self.iclksel,
            self.lptim2sel,
            self.spi1sel,
            self.spi2sel,
            self.uart4sel,
            self.uart5sel,
            self.usart1sel,
            self.usart3sel,
            // CCIPR2
            self.octospisel,
            self.rngsel,
            self.saessel,
            self.sai1sel,
            self.sdmmcsel,
            // CCIPR3
            self.adcdacsel,
            self.adf1sel,
            self.dac1sel,
            self.i2c3sel,
            self.lptim1sel,
            self.lpuart1sel,
            self.spi3sel,
        );
    }
}

#[derive(Copy, Clone)]
pub struct Hse {
    pub freq: Hertz,
    pub mode: HseMode,
}

#[derive(Copy, Clone)]
pub enum HseMode {
    /// Crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// External analog clock (low swing) (HSEBYP=1, HSEEXT=0)
    Bypass,
    /// External digital clock (full swing) (HSEBYP=1, HSEEXT=1)
    BypassDigital,
}

#[derive(Clone, Copy)]
pub struct Pll {
    /// The clock source for the PLL.
    pub source: PllSource,
    /// The PLL pre-divider.
    ///
    /// The clock speed of the `source` divided by `m` must be between 4 and 16 MHz.
    pub prediv: PllPreDiv,
    /// The PLL multiplier.
    ///
    /// The multiplied clock – `source` divided by `m` times `n` – must be between 128 and 544
    /// MHz. The upper limit may be lower depending on the `Config { voltage_range }`.
    pub mul: PllMul,
    /// The divider for the P output.
    ///
    /// The P output is one of several options
    /// that can be used to feed the SAI/MDF/ADF Clock mux's.
    pub divp: Option<PllDiv>,
    /// The divider for the Q output.
    ///
    /// The Q ouput is one of severals options that can be used to feed the 48MHz clocks
    /// and the OCTOSPI clock. It may also be used on the MDF/ADF clock mux's.
    pub divq: Option<PllDiv>,
    /// The divider for the R output.
    ///
    /// When used to drive the system clock, `source` divided by `m` times `n` divided by `r`
    /// must not exceed 160 MHz. System clocks above 55 MHz require a non-default
    /// `Config { voltage_range }`.
    pub divr: Option<PllDiv>,
}

pub struct PllInput {
    pub hsi: Option<Hertz>,
    pub hse: Option<Hertz>,
    pub msi: Option<Hertz>,
}
