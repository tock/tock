// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

// The PWR peripheral is needed for setting the voltage scale and enabling the EPOD booster
use crate::pwr::{Pwr, VoltageScale};

// Configuration structures and enumerations
pub mod config;
use config::{HseMode, Pll, PllInput, RccConfig};

// Enumerations for some RCC fields
pub mod values;
use values::{
    Adcdacsel, Adfsel, Dacsel, Fdcansel, I2c3sel, I2csel, Iclksel, Lptim2sel, Lptimsel, Lpusartsel,
    MsiRange, Octospisel, PllDiv, PllMboost, PllSource, Pllrge, Rngsel, Rtcsel, Saessel, Saisel,
    Sdmmcsel, Spi1sel, Spi2sel, Spi3sel, Sysclk, Usart1sel, Usartsel,
};

// Helper for frequencies
pub mod hertz;
use hertz::Hertz;

/// All clock frequencies
#[derive(Copy, Clone)]
pub struct Clocks {
    pub sys: Hertz,
    pub hclk1: Hertz,
    pub hclk2: Hertz,
    pub hclk3: Hertz,
    pub pclk1: Hertz,
    pub pclk2: Hertz,
    pub pclk3: Hertz,
    pub pclk1_tim: Hertz,
    pub pclk2_tim: Hertz,
    pub msik: Option<Hertz>,
    pub hsi48: Option<Hertz>,
    pub rtc: Option<Hertz>,
    pub lsi: Option<Hertz>,
    pub hse: Option<Hertz>,
    pub hsi: Option<Hertz>,
    pub pll1_p: Option<Hertz>,
    pub pll1_q: Option<Hertz>,
    pub pll1_r: Option<Hertz>,
    pub pll2_p: Option<Hertz>,
    pub pll2_q: Option<Hertz>,
    pub pll2_r: Option<Hertz>,
    pub pll3_p: Option<Hertz>,
    pub pll3_q: Option<Hertz>,
    pub pll3_r: Option<Hertz>,
}

// Registers and fields
register_structs! {
    pub RccRegisters {
        /// Control register
        (0x000 => cr: ReadWrite<u32, CR::Register>),
        (0x004 => _reserved0),
        (0x008 => icscr1: ReadWrite<u32, ICSCR1::Register>),
        (0x00C => _reserved1),
        /// Clock configuration register 1
        (0x01C => cfgr1: ReadWrite<u32, CFGR1::Register>),
        /// Clock configuration register 2
        (0x020 => cfgr2: ReadWrite<u32, CFGR2::Register>),
        /// Clock configuration register 2
        (0x024 => cfgr3: ReadWrite<u32, CFGR3::Register>),
        /// RCC PLL1 configuration register
        (0x028 => pll1cfgr: ReadWrite<u32, PLLxCFGR::Register>),
        /// RCC PLL2 configuration register
        (0x02C => pll2cfgr: ReadWrite<u32, PLLxCFGR::Register>),
        /// RCC PLL3 configuration register
        (0x030 => pll3cfgr: ReadWrite<u32, PLLxCFGR::Register>),
        /// RCC PLL1 dividers configuration register
        (0x034 => pll1divr: ReadWrite<u32, PLLxDIVR::Register>),
        (0x038 => _reserved2),
        /// RCC PLL2 dividers configuration register
        (0x03C => pll2divr: ReadWrite<u32, PLLxDIVR::Register>),
        (0x040 => _reserved3),
        /// RCC PLL3 dividers configuration register
        (0x044 => pll3divr: ReadWrite<u32, PLLxDIVR::Register>),
        (0x048 => _reserved4),
        /// AHB1 peripheral clock enable register
        (0x088 => ahb1enr: ReadWrite<u32, AHB1ENR::Register>),
        /// AHB2 peripheral clock enable register 1
        (0x08C => ahb2enr1: ReadWrite<u32, AHB2ENR1::Register>),
        (0x090 => _reserved5: [u32; 1]),
        /// AHB3 peripheral clock enable register
        (0x094 => ahb3enr: ReadWrite<u32, AHB3ENR::Register>),
        (0x098 => _reserved6: [u32; 1]),
        /// APB1 peripheral clock enable register 1
        (0x09C => apb1enr1: ReadWrite<u32, APB1ENR1::Register>),
        (0x0A0 => _reserved7: [u32; 1]), //this would be APB1ENR2, but unused for now
        /// APB2 peripheral clock enable register
        (0x0A4 => apb2enr: ReadWrite<u32, APB2ENR::Register>),
        /// APB3 peripheral clock enable register
        (0x0A8 => apb3enr: ReadWrite<u32, APB3ENR::Register>),
        (0x0AC => _reserved8: [u32; 13]),
        /// Peripherals independent clock configuration register 1
        (0x0E0 => ccipr1: ReadWrite<u32, CCIPR1::Register>),
        /// Peripherals independent clock configuration register 2
        (0x0E4 => ccipr2: ReadWrite<u32, CCIPR2::Register>),
        /// Peripherals independent clock configuration register 3
        (0x0E8 => ccipr3: ReadWrite<u32, CCIPR3::Register>),
        (0x0EC => _reserved9),
        /// RCC backup domain control registe
        (0x0F0 => bdcr: ReadWrite<u32, BDCR::Register>),
        (0x0F4 => @END),
    }
}
register_bitfields![u32,
    pub CR [
        MSISON OFFSET(0) NUMBITS(1) [],
        MSISRDY OFFSET(2) NUMBITS(1) [],
        MSIPLLEN OFFSET(3) NUMBITS(1) [],
        MSIKON OFFSET(4) NUMBITS(1) [],
        MSIKRDY OFFSET(5) NUMBITS(1) [],
        MSIPLLFAST OFFSET(7) NUMBITS(1) [],
        HSION OFFSET(8) NUMBITS(1) [],
        HSIRDY OFFSET(10) NUMBITS(1) [],
        HSI48ON OFFSET(12) NUMBITS(1) [],
        HSI48RDY OFFSET(13) NUMBITS(1) [],
        HSEON OFFSET(16) NUMBITS(1) [],
        HSERDY OFFSET(17) NUMBITS(1) [],
        HSEBYP OFFSET(18) NUMBITS(1) [],
        HSEEXT OFFSET(20) NUMBITS(1) [
            ANALOG = 0,
            DIGITAL = 1,
        ],
        PLL1ON OFFSET(24) NUMBITS(1) [],
        PLL1RDY OFFSET(25) NUMBITS(1) [],
        PLL2ON OFFSET(26) NUMBITS(1) [],
        PLL2RDY OFFSET(27) NUMBITS(1) [],
        PLL3ON OFFSET(28) NUMBITS(1) [],
        PLL3RDY OFFSET(29) NUMBITS(1) [],
    ],
    pub ICSCR1 [
        MSISRANGE OFFSET(28) NUMBITS(4) [],
        MSIKRANGE OFFSET(24) NUMBITS(4) [],
        MSIRGSEL OFFSET(23) NUMBITS(1) [
            CSR = 0,
            ICSCR1 = 1,
        ],
    ],
    pub CFGR1 [
        SW OFFSET(0) NUMBITS(2) [],
        SWS OFFSET(2) NUMBITS(2) [],
    ],
    pub CFGR2 [
        HPRE OFFSET(0) NUMBITS(4) [
            DIV1 = 0,
            DIV2 = 0b1000,
            DIV4 = 0b1001,
            DIV8 = 0b1010,
            DIV16 = 0b1011,
            // no DIV32
            DIV64 = 0b1100,
            DIV128 = 0b1101,
            DIV256 = 0b1110,
            DIV512 = 0b1111,
        ],
        PPRE1 OFFSET(4) NUMBITS(3) [
            DIV1 = 0,
            DIV2 = 0b100,
            DIV4 = 0b101,
            DIV8 = 0b110,
            DIV16 = 0b111,
        ],
        PPRE2 OFFSET(8) NUMBITS(3) [
            DIV1 = 0,
            DIV2 = 0b100,
            DIV4 = 0b101,
            DIV8 = 0b110,
            DIV16 = 0b111,
        ],
    ],
    pub CFGR3 [
        PPRE3 OFFSET(4) NUMBITS(3) [
            DIV1 = 0,
            DIV2 = 0b100,
            DIV4 = 0b101,
            DIV8 = 0b110,
            DIV16 = 0b111,
        ],
    ],
    pub PLLxCFGR [
        PLLxREN OFFSET(18) NUMBITS(1) [],
        PLLxQEN OFFSET(17) NUMBITS(1) [],
        PLLxPEN OFFSET(16) NUMBITS(1) [],
        PLLxMBOOST OFFSET(12) NUMBITS(4) [],
        PLLxM OFFSET(8) NUMBITS(4) [],
        PLLxFRACEN OFFSET(4) NUMBITS(1) [],
        PLLxRGE OFFSET(2) NUMBITS(2) [],
        PLLxSRC OFFSET(0) NUMBITS(2) []
    ],
    pub PLLxDIVR [
        PLLxR OFFSET(24) NUMBITS(7) [],
        PLLxQ OFFSET(16) NUMBITS(7) [],
        PLLxP OFFSET(9) NUMBITS(7) [],
        PLLxN OFFSET(0) NUMBITS(9) []
    ],
    pub AHB1ENR [
        GPDMA1EN OFFSET(0) NUMBITS(1) [],
        CRCEN OFFSET(12) NUMBITS(1) []
    ],
    pub AHB2ENR1 [
        GPIOAEN OFFSET(0) NUMBITS(1) [],
        GPIOBEN OFFSET(1) NUMBITS(1) [],
        GPIOCEN OFFSET(2) NUMBITS(1) [],
        GPIODEN OFFSET(3) NUMBITS(1) [],
        GPIOEEN OFFSET(4) NUMBITS(1) [],
        GPIOFEN OFFSET(5) NUMBITS(1) [],
        GPIOGEN OFFSET(6) NUMBITS(1) [],
        GPIOHEN OFFSET(7) NUMBITS(1) [],
        GPIOIEN OFFSET(8) NUMBITS(1) [],
        GPIOJEN OFFSET(9) NUMBITS(1) [],
        ADC12EN OFFSET(10) NUMBITS(1) [],
        AESEN   OFFSET(16) NUMBITS(1) [],
        HASHEN  OFFSET(17) NUMBITS(1) [],
        TRNGEN  OFFSET(18) NUMBITS(1) [],
        PKAEN   OFFSET(19) NUMBITS(1) []
    ],
    pub AHB3ENR [
        DAC1EN OFFSET(6) NUMBITS(1) [],
        PWREN OFFSET(2) NUMBITS(1) []
    ],
    pub APB1ENR1 [
        I2C1EN OFFSET(21) NUMBITS(1) [],
        TIM3EN OFFSET(1) NUMBITS(1) [],
        TIM2EN OFFSET(0) NUMBITS(1) []
    ],
    pub APB2ENR [
        USART1EN OFFSET(14) NUMBITS(1) [],
        SPI1EN OFFSET(12) NUMBITS(1) []
    ],
    pub APB3ENR [
        RTCAPBEN OFFSET(21) NUMBITS(1) [],
        SYSCFGEN OFFSET(1) NUMBITS(1) []
    ],
    pub CCIPR1 [
        ICLKSEL OFFSET(26) NUMBITS(2) [],
        FDCAN1SEL OFFSET(24) NUMBITS(2) [],
        SPI1SEL OFFSET(20) NUMBITS(2) [],
        LPTIM2SEL OFFSET(18) NUMBITS(2) [],
        SPI2SEL OFFSET(16) NUMBITS(2) [],
        I2C4SEL OFFSET(14) NUMBITS(2) [],
        I2C2SEL OFFSET(12) NUMBITS(2) [],
        I2C1SEL OFFSET(10) NUMBITS(2) [],
        UART5SEL OFFSET(8) NUMBITS(2) [],
        UART4SEL OFFSET(6) NUMBITS(2) [],
        USART3SEL OFFSET(4) NUMBITS(2) [],
        USART2SEL OFFSET(2) NUMBITS(2) [],
        USART1SEL OFFSET(0) NUMBITS(2) []
    ],
    pub CCIPR2 [
        OCTOSPISEL OFFSET(20) NUMBITS(2) [],
        SDMMCSEL OFFSET(14) NUMBITS(1) [],
        RNGSEL OFFSET(12) NUMBITS(2) [],
        SAESSEL OFFSET(11) NUMBITS(1) [],
        SAI1SEL OFFSET(5) NUMBITS(3) []
    ],
    pub CCIPR3 [
        ADF1SEL OFFSET(16) NUMBITS(3) [],
        DAC1SEL OFFSET(15) NUMBITS(1) [],
        ADCDACSEL OFFSET(12) NUMBITS(3) [],
        LPTIM1SEL OFFSET(10) NUMBITS(2) [],
        I2C3SEL OFFSET(6) NUMBITS(2) [],
        SPI3SEL OFFSET(3) NUMBITS(2) [],
        LPUART1SEL OFFSET(0) NUMBITS(3) []
    ],
    pub BDCR [
        LSIRDY OFFSET(27) NUMBITS(1) [],
        LSION OFFSET(26) NUMBITS(1) [],
        RTCEN OFFSET(15) NUMBITS(1) [],
        RTCSEL OFFSET(8) NUMBITS(2) []
    ],
];
/// Base address for RCC in Nonsecure mode
pub const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x46020C00 as *const RccRegisters) };

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

impl Rcc {
    pub const fn new(base: StaticRef<RccRegisters>) -> Self {
        Self { registers: base }
    }

    /// Configure and start all necessary clocks
    // Adapted from embassy-rs/embassy/embassy-stm32/src/rcc/u5.rs
    pub fn init(&self, config: RccConfig, pwr: &Pwr) -> Clocks {
        // Configure the clock to a safe default state before starting configuration:
        // power range 1 (most powerful), HSI as SYSCLK source (16MHz)
        pwr.set_voltage_scaling(VoltageScale::Range1);
        self.enable_hsi16();
        self.set_sysclk_source(Sysclk::Hsi);

        let msis = config.msis.map(|range| {
            // Check MSI output per RM0456 § 11.4.10
            if let VoltageScale::Range4 = config.voltage_range {
                assert!(Self::msirange_to_hertz(range).0 <= 24_000_000);
            }

            // RM0456 § 11.8.2: spin until MSIS is off or MSIS is ready before setting its range
            loop {
                let cr = self.registers.cr.extract();
                if !cr.is_set(CR::MSISON) || cr.is_set(CR::MSISRDY) {
                    break;
                }
            }

            self.registers
                .icscr1
                .modify(ICSCR1::MSISRANGE.val(range as u32) + ICSCR1::MSIRGSEL::ICSCR1);

            self.registers
                .cr
                .write(CR::MSIPLLEN::CLEAR + CR::MSISON::SET);

            while !self.registers.cr.is_set(CR::MSISRDY) {}

            Self::msirange_to_hertz(range)
        });

        let msik = config.msik.map(|range| {
            if let VoltageScale::Range4 = config.voltage_range {
                assert!(Self::msirange_to_hertz(range).0 <= 24_000_000);
            }

            loop {
                let cr = self.registers.cr.extract();
                if !cr.is_set(CR::MSIKON) || cr.is_set(CR::MSIKRDY) {
                    break;
                }
            }

            self.registers
                .icscr1
                .modify(ICSCR1::MSIKRANGE.val(range as u32) + ICSCR1::MSIRGSEL::ICSCR1);

            self.registers.cr.modify(CR::MSIKON::SET);

            while !self.registers.cr.is_set(CR::MSIKRDY) {}

            Self::msirange_to_hertz(range)
        });

        let hsi = config.hsi.then_some(Hertz(16_000_000));

        let hse = config.hse.map(|hse| {
            // Check frequency limits per RM456 § 11.4.10
            match config.voltage_range {
                VoltageScale::Range1 | VoltageScale::Range2 | VoltageScale::Range3 => {
                    assert!(hse.freq.0 <= 50_000_000);
                }
                VoltageScale::Range4 => {
                    assert!(hse.freq.0 <= 25_000_000);
                }
            }

            // Enable HSE and wait for it to stabilize
            self.registers.cr.modify(
                CR::HSEON::SET
                    + match hse.mode {
                        HseMode::Oscillator => CR::HSEBYP::CLEAR,
                        _ => CR::HSEBYP::SET,
                    }
                    + match hse.mode {
                        HseMode::Oscillator | HseMode::Bypass => CR::HSEEXT::ANALOG,
                        HseMode::BypassDigital => CR::HSEEXT::DIGITAL,
                    },
            );

            while !self.registers.cr.is_set(CR::HSERDY) {}

            hse.freq
        });

        let hsi48 = if config.hsi48 {
            self.enable_hsi48();
            Some(Hertz(48_000_000))
        } else {
            None
        };

        let lsi = if config.lsi {
            self.enable_lsi();
            Some(Hertz(32_000))
        } else {
            None
        };

        let pll_input = PllInput {
            hsi,
            hse,
            msi: msis,
        };
        let pll1 = config.pll1.map_or_else(
            || {
                self.change_pll_enable(PllInstance::Pll1, false);
                PllOutput::default()
            },
            |c| self.init_pll(PllInstance::Pll1, Some(c), &pll_input, config.voltage_range),
        );
        let pll2 = config.pll2.map_or_else(
            || {
                self.change_pll_enable(PllInstance::Pll2, false);
                PllOutput::default()
            },
            |c| self.init_pll(PllInstance::Pll2, Some(c), &pll_input, config.voltage_range),
        );
        let pll3 = config.pll3.map_or_else(
            || {
                self.change_pll_enable(PllInstance::Pll3, false);
                PllOutput::default()
            },
            |c| self.init_pll(PllInstance::Pll3, Some(c), &pll_input, config.voltage_range),
        );

        // Verify that sysclk is valid before attempting to change the clock source
        // This ensures that, even in case of an error, the clock remains in a safe state
        let sys_clk = match config.sys {
            Sysclk::Hse => hse.unwrap(),
            Sysclk::Hsi => hsi.unwrap(),
            Sysclk::Msis => msis.unwrap(),
            Sysclk::Pll1R => pll1.r.unwrap(),
        };

        let hclk = sys_clk / config.ahb_pre;

        let hclk_max = match config.voltage_range {
            VoltageScale::Range1 => Hertz::mhz(160),
            VoltageScale::Range2 => Hertz::mhz(110),
            VoltageScale::Range3 => Hertz::mhz(55),
            VoltageScale::Range4 => Hertz::mhz(25),
        };
        assert!(hclk <= hclk_max);

        // If needed, enable the EPOD booster to reach the target clock speed, per § 10.5.4
        if sys_clk >= Hertz::mhz(55) {
            pwr.enable_epod_booster();
        }

        // Set the requested power mode
        pwr.set_voltage_scaling(config.voltage_range);

        // Configure the bus prescalers
        self.registers.cfgr2.modify(
            CFGR2::HPRE.val(config.ahb_pre as u32)
                + CFGR2::PPRE1.val(config.apb1_pre as u32)
                + CFGR2::PPRE2.val(config.apb2_pre as u32),
        );
        self.registers
            .cfgr3
            .modify(CFGR3::PPRE3.val(config.apb3_pre as u32));

        // Switch the clock source
        self.set_sysclk_source(config.sys);

        let (pclk1, pclk1_tim) = Self::calc_pclk(hclk, config.apb1_pre);
        let (pclk2, pclk2_tim) = Self::calc_pclk(hclk, config.apb2_pre);
        let (pclk3, _) = Self::calc_pclk(hclk, config.apb3_pre);

        let rtc = match config.mux.rtcsel {
            Rtcsel::Disable => None,
            Rtcsel::Lsi => {
                assert!(config.lsi);
                lsi
            }
            Rtcsel::Lse => None, // not implemented
            Rtcsel::Hse => None, // not implemented
        };

        // Set clock sources according to the mux configuration
        config.mux.init(self);

        // Return a structure containing all effective clock frequencies
        Clocks {
            sys: sys_clk,
            hclk1: hclk,
            hclk2: hclk,
            hclk3: hclk,
            pclk1,
            pclk2,
            pclk3,
            pclk1_tim,
            pclk2_tim,
            msik,
            hsi48,
            rtc,
            lsi,
            hse,
            hsi,
            pll1_p: pll1.p,
            pll1_q: pll1.q,
            pll1_r: pll1.r,
            pll2_p: pll2.p,
            pll2_q: pll2.q,
            pll2_r: pll2.r,
            pll3_p: pll3.p,
            pll3_q: pll3.q,
            pll3_r: pll3.r,
        }
    }

    /// Start the 16MHz internal oscillator
    fn enable_hsi16(&self) {
        self.registers.cr.modify(CR::HSION::SET);

        // Wait for oscillator ready
        while !self.registers.cr.is_set(CR::HSIRDY) {}
    }

    /// Start the 48MHz internal oscillator
    fn enable_hsi48(&self) {
        self.registers.cr.modify(CR::HSI48ON::SET);

        // Wait for oscillator ready
        while !self.registers.cr.is_set(CR::HSI48RDY) {}
    }

    /// Start the 20kHz internal oscillator
    fn enable_lsi(&self) {
        self.registers.bdcr.modify(BDCR::LSION::SET);

        // Wait for oscillator ready
        while !self.registers.bdcr.is_set(BDCR::LSIRDY) {}
    }

    /// Change the clock source for SYSCLK
    fn set_sysclk_source(&self, clk: Sysclk) {
        self.registers.cfgr1.modify(CFGR1::SW.val(clk as u32));

        // Wait for the value in the "status" register to match what we set
        while self.registers.cfgr1.read(CFGR1::SWS) != clk as u32 {}
    }

    /// Enable or disable a PLL
    fn change_pll_enable(&self, instance: PllInstance, enabled: bool) {
        self.registers.cr.modify(match instance {
            PllInstance::Pll1 => CR::PLL1ON.val(enabled as u32),
            PllInstance::Pll2 => CR::PLL2ON.val(enabled as u32),
            PllInstance::Pll3 => CR::PLL3ON.val(enabled as u32),
        });

        while self.registers.cr.is_set(match instance {
            PllInstance::Pll1 => CR::PLL1RDY,
            PllInstance::Pll2 => CR::PLL2RDY,
            PllInstance::Pll3 => CR::PLL3RDY,
        }) != enabled
        {}
    }

    /// Configure and start a PLL
    // Adapted from embassy-rs/embassy/embassy-stm32/src/rcc/u5.rs
    fn init_pll(
        &self,
        instance: PllInstance,
        config: Option<Pll>,
        input: &PllInput,
        voltage_range: VoltageScale,
    ) -> PllOutput {
        // Disable PLL
        self.change_pll_enable(instance, false);

        let Some(pll) = config else {
            return PllOutput::default();
        };

        let src_freq = match pll.source {
            PllSource::Disable => panic!("must not select PLL source as DISABLE"),
            PllSource::Hse => input.hse.unwrap(),
            PllSource::Hsi => input.hsi.unwrap(),
            PllSource::Msis => input.msi.unwrap(),
        };

        // Calculate the reference clock, which is the source divided by m
        let ref_freq = src_freq / pll.prediv;

        // Check limits per RM0456 § 11.4.6
        assert!(Hertz::mhz(4) <= ref_freq && ref_freq <= Hertz::mhz(16));

        // Check PLL clocks per RM0456 § 11.4.10
        let (vco_min, vco_max, out_max) = match voltage_range {
            VoltageScale::Range1 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(208)),
            VoltageScale::Range2 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(110)),
            VoltageScale::Range3 => (Hertz::mhz(128), Hertz::mhz(330), Hertz::mhz(55)),
            VoltageScale::Range4 => panic!("PLL is unavailable in voltage range 4"),
        };

        // Calculate the PLL VCO clock
        let vco_freq = ref_freq * pll.mul;
        assert!(vco_freq >= vco_min && vco_freq <= vco_max);

        // Calculate output clocks
        let p = pll.divp.map(|div| vco_freq / div);
        let q = pll.divq.map(|div| vco_freq / div);
        let r = pll.divr.map(|div| vco_freq / div);
        for freq in [p, q, r] {
            if let Some(freq) = freq {
                assert!(freq <= out_max);
            }
        }

        let divr = match instance {
            PllInstance::Pll1 => &self.registers.pll1divr,
            PllInstance::Pll2 => &self.registers.pll2divr,
            PllInstance::Pll3 => &self.registers.pll3divr,
        };
        divr.write(
            PLLxDIVR::PLLxN.val(pll.mul as u32)
                + PLLxDIVR::PLLxP.val(pll.divp.unwrap_or(PllDiv::Div1) as u32)
                + PLLxDIVR::PLLxQ.val(pll.divq.unwrap_or(PllDiv::Div1) as u32)
                + PLLxDIVR::PLLxR.val(pll.divr.unwrap_or(PllDiv::Div1) as u32),
        );

        let input_range = match ref_freq.0 {
            ..=8_000_000 => Pllrge::Freq4to8mhz,
            _ => Pllrge::Freq8to16mhz,
        };

        let (pllxcfgr, mboost) = match instance {
            PllInstance::Pll1 => {
                // § 10.5.4: if we're targeting >= 55 MHz, we must configure PLL1MBOOST to a prescaler
                // value that results in an output between 4 and 16 MHz for the PWR EPOD boost
                let mboost = if r.unwrap() >= Hertz::mhz(55) {
                    // source_clk can be up to 50 MHz, so there's just a few cases:
                    match src_freq.0 {
                        ..=16_000_000 => PllMboost::Div1, // Bypass, giving EPOD 4-16 MHz
                        16_000_001..=32_000_000 => PllMboost::Div2, // Divide by 2, giving EPOD 8-16 MHz
                        _ => PllMboost::Div4, // Divide by 4, giving EPOD 8-12.5 MHz
                    }
                } else {
                    PllMboost::Div1
                };

                (&self.registers.pll1cfgr, mboost)
            }
            PllInstance::Pll2 => (&self.registers.pll2cfgr, PllMboost::Div1),
            PllInstance::Pll3 => (&self.registers.pll3cfgr, PllMboost::Div1),
        };

        pllxcfgr.write(
            PLLxCFGR::PLLxMBOOST.val(mboost as u32)
                + PLLxCFGR::PLLxPEN.val(pll.divp.is_some() as u32)
                + PLLxCFGR::PLLxQEN.val(pll.divq.is_some() as u32)
                + PLLxCFGR::PLLxREN.val(pll.divr.is_some() as u32)
                + PLLxCFGR::PLLxM.val(pll.prediv as u32)
                + PLLxCFGR::PLLxSRC.val(pll.prediv as u32)
                + PLLxCFGR::PLLxSRC.val(pll.source as u32)
                + PLLxCFGR::PLLxRGE.val(input_range as u32),
        );

        // Enable PLL
        self.change_pll_enable(instance, true);

        PllOutput { p, q, r }
    }

    /// Set the clock source for various peripherals
    pub fn set_clock_sources(
        &self,
        // BDCR
        rtcsel: Rtcsel,
        // CCIPR1
        fdcansel: Fdcansel,
        i2c1sel: I2csel,
        i2c2sel: I2csel,
        i2c4sel: I2csel,
        iclksel: Iclksel,
        lptim2sel: Lptim2sel,
        spi1sel: Spi1sel,
        spi2sel: Spi2sel,
        uart4sel: Usartsel,
        uart5sel: Usartsel,
        usart1sel: Usart1sel,
        usart3sel: Usartsel,
        // CCIPR2
        octospisel: Octospisel,
        rngsel: Rngsel,
        saessel: Saessel,
        sai1sel: Saisel,
        sdmmcsel: Sdmmcsel,
        // CCIPR3
        adcdacsel: Adcdacsel,
        adf1sel: Adfsel,
        dac1sel: Dacsel,
        i2c3sel: I2c3sel,
        lptim1sel: Lptimsel,
        lpuart1sel: Lpusartsel,
        spi3sel: Spi3sel,
    ) {
        self.registers.bdcr.modify(BDCR::RTCSEL.val(rtcsel as u32));

        // There is no USART2 on STM32U535/545
        self.registers.ccipr1.modify(
            CCIPR1::FDCAN1SEL.val(fdcansel as u32)
                + CCIPR1::I2C1SEL.val(i2c1sel as u32)
                + CCIPR1::I2C2SEL.val(i2c2sel as u32)
                + CCIPR1::I2C4SEL.val(i2c4sel as u32)
                + CCIPR1::ICLKSEL.val(iclksel as u32)
                + CCIPR1::LPTIM2SEL.val(lptim2sel as u32)
                + CCIPR1::SPI1SEL.val(spi1sel as u32)
                + CCIPR1::SPI2SEL.val(spi2sel as u32)
                + CCIPR1::UART4SEL.val(uart4sel as u32)
                + CCIPR1::UART5SEL.val(uart5sel as u32)
                + CCIPR1::USART1SEL.val(usart1sel as u32)
                + CCIPR1::USART3SEL.val(usart3sel as u32),
        );

        self.registers.ccipr2.modify(
            CCIPR2::OCTOSPISEL.val(octospisel as u32)
                + CCIPR2::RNGSEL.val(rngsel as u32)
                + CCIPR2::SAESSEL.val(saessel as u32)
                + CCIPR2::SAI1SEL.val(sai1sel as u32)
                + CCIPR2::SDMMCSEL.val(sdmmcsel as u32),
        );

        self.registers.ccipr3.modify(
            CCIPR3::ADCDACSEL.val(adcdacsel as u32)
                + CCIPR3::ADF1SEL.val(adf1sel as u32)
                + CCIPR3::DAC1SEL.val(dac1sel as u32)
                + CCIPR3::I2C3SEL.val(i2c3sel as u32)
                + CCIPR3::LPTIM1SEL.val(lptim1sel as u32)
                + CCIPR3::LPUART1SEL.val(lpuart1sel as u32)
                + CCIPR3::SPI3SEL.val(spi3sel as u32),
        );
    }

    // Enable clock routing to various peripherals
    pub fn enable_dma1(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPDMA1EN::SET);
    }
    pub fn enable_gpioa(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::GPIOAEN::SET);
    }
    pub fn enable_gpiob(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::GPIOBEN::SET);
    }
    pub fn enable_gpioc(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::GPIOCEN::SET);
    }
    pub fn enable_usart1(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::SET);
    }
    pub fn enable_tim2(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM2EN::SET);
    }
    pub fn enable_tim3(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM3EN::SET);
    }
    pub fn enable_syscfg(&self) {
        self.registers.apb3enr.modify(APB3ENR::SYSCFGEN::SET);
    }
    pub fn enable_pwr(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::PWREN::SET);
    }
    pub fn enable_rtc_apb(&self) {
        self.registers.apb3enr.modify(APB3ENR::RTCAPBEN::SET);
    }
    pub fn enable_rtc(&self) {
        self.registers.bdcr.modify(BDCR::RTCEN::SET);
    }
    pub fn enable_adc1(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::ADC12EN::SET);
    }
    pub fn enable_dac1(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::DAC1EN::SET);
    }
    pub fn enable_hash(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::HASHEN::SET);
    }
    pub fn enable_trng(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::TRNGEN::SET);
    }
    pub fn enable_crc(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::CRCEN::SET);
    }
    pub fn enable_aes(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::AESEN::SET);
    }
    pub fn enable_pka(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::PKAEN::SET);
    }
    pub fn enable_spi1(&self) {
        self.registers.apb2enr.modify(APB2ENR::SPI1EN::SET);
    }
    pub fn enable_i2c1(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::I2C1EN::SET);
    }

    /// Get the effective frequency for PCLK (and PCLK used for timers) from an input frequency and prescaler
    // For timers, the hardware automatically multiplies PCLK by 2 if the prescaler (divider) is not 1
    fn calc_pclk<D>(hclk: Hertz, ppre: D) -> (Hertz, Hertz)
    where
        Hertz: core::ops::Div<D, Output = Hertz>,
    {
        let pclk = hclk / ppre;
        let pclk_tim = if hclk == pclk { pclk } else { pclk * 2u32 };
        (pclk, pclk_tim)
    }

    /// Convert `MsiRange` to its effective frequency
    // Adapted from embassy-rs/embassy/embassy-stm32/src/rcc/u5.rs
    fn msirange_to_hertz(range: MsiRange) -> Hertz {
        match range {
            MsiRange::Range48mhz => Hertz(48_000_000),
            MsiRange::Range24mhz => Hertz(24_000_000),
            MsiRange::Range16mhz => Hertz(16_000_000),
            MsiRange::Range12mhz => Hertz(12_000_000),
            MsiRange::Range4mhz => Hertz(4_000_000),
            MsiRange::Range2mhz => Hertz(2_000_000),
            MsiRange::Range133mhz => Hertz(1_330_000),
            MsiRange::Range1mhz => Hertz(1_000_000),
            MsiRange::Range3072mhz => Hertz(3_072_000),
            MsiRange::Range1536mhz => Hertz(1_536_000),
            MsiRange::Range1024mhz => Hertz(1_024_000),
            MsiRange::Range768khz => Hertz(768_000),
            MsiRange::Range400khz => Hertz(400_000),
            MsiRange::Range200khz => Hertz(200_000),
            MsiRange::Range133khz => Hertz(133_000),
            MsiRange::Range100khz => Hertz(100_000),
        }
    }
}

/// Used internally: taken by `Rcc::change_pll_enable` and `Rcc::init_pll`
#[derive(Clone, Copy)]
enum PllInstance {
    Pll1 = 0,
    Pll2 = 1,
    Pll3 = 2,
}

/// Used internally: returned by `Rcc::init_pll`, used in `Rcc::init`
#[derive(Default)]
struct PllOutput {
    pub p: Option<Hertz>,
    pub q: Option<Hertz>,
    pub r: Option<Hertz>,
}
