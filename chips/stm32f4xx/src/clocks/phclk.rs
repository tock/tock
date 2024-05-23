// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::clocks::Stm32f4Clocks;
use crate::rcc::{APBPrescaler, Rcc, RtcClockSource};
use kernel::platform::chip::ClockInterface;

pub struct PeripheralClock<'a> {
    pub clock: PeripheralClockType,
    clocks: &'a dyn Stm32f4Clocks,
}

/// Bus + Clock name for the peripherals
pub enum PeripheralClockType {
    AHB1(HCLK1),
    AHB2(HCLK2),
    AHB3(HCLK3),
    APB1(PCLK1),
    APB2(PCLK2),
    RTC,
    PWR,
}

/// Peripherals clocked by HCLK1
pub enum HCLK1 {
    DMA1,
    DMA2,
    GPIOH,
    GPIOG,
    GPIOF,
    GPIOE,
    GPIOD,
    GPIOC,
    GPIOB,
    GPIOA,
}

/// Peripherals clocked by HCLK3
pub enum HCLK3 {
    FMC,
}

/// Peripherals clocked by HCLK2
pub enum HCLK2 {
    RNG,
    OTGFS,
}

/// Peripherals clocked by PCLK1
pub enum PCLK1 {
    TIM2,
    USART2,
    USART3,
    SPI3,
    I2C1,
    CAN1,
    DAC,
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    USART1,
    ADC1,
    SYSCFG,
}

impl<'a> PeripheralClock<'a> {
    pub const fn new(clock: PeripheralClockType, clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self { clock, clocks }
    }

    pub fn configure_rng_clock(&self) {
        self.clocks.get_rcc().configure_rng_clock();
    }

    pub fn get_frequency(&self) -> u32 {
        #[inline(always)]
        fn tim_freq(rcc: &Rcc, hclk_freq: usize, prescaler: APBPrescaler) -> usize {
            // Reference Manual RM0090 section 6.2
            // When TIMPRE bit of the RCC_DCKCFGR register is reset, if APBx prescaler is 1, then
            // TIMxCLK = PCLKx, otherwise TIMxCLK = 2x PCLKx.
            // When TIMPRE bit in the RCC_DCKCFGR register is set, if APBx prescaler is 1,2 or 4,
            // then TIMxCLK = HCLK, otherwise TIMxCLK = 4x PCLKx.
            if !rcc.is_enabled_tim_pre() {
                match prescaler {
                    APBPrescaler::DivideBy1 | APBPrescaler::DivideBy2 => hclk_freq,
                    _ => hclk_freq / usize::from(prescaler) * 2,
                }
            } else {
                match prescaler {
                    APBPrescaler::DivideBy1 | APBPrescaler::DivideBy2 | APBPrescaler::DivideBy4 => {
                        hclk_freq
                    }
                    _ => hclk_freq / usize::from(prescaler) * 4,
                }
            }
        }
        let rcc = self.clocks.get_rcc();
        let hclk_freq = self.clocks.get_ahb_frequency();
        match self.clock {
            PeripheralClockType::AHB1(_)
            | PeripheralClockType::AHB2(_)
            | PeripheralClockType::AHB3(_) => hclk_freq as u32,
            PeripheralClockType::APB1(ref v) => {
                let prescaler = rcc.get_apb1_prescaler();
                match v {
                    PCLK1::TIM2 => tim_freq(rcc, hclk_freq, prescaler) as u32,
                    _ => (hclk_freq / usize::from(prescaler)) as u32,
                }
            }
            PeripheralClockType::APB2(_) => {
                let prescaler = rcc.get_apb2_prescaler();
                (hclk_freq / usize::from(prescaler)) as u32
            }
            //TODO: implement clock frequency retrieval for RTC and PWR peripherals
            PeripheralClockType::RTC => todo!(),
            PeripheralClockType::PWR => todo!(),
        }
    }
}

impl<'a> ClockInterface for PeripheralClock<'a> {
    fn is_enabled(&self) -> bool {
        let rcc = self.clocks.get_rcc();
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => rcc.is_enabled_dma1_clock(),
                HCLK1::DMA2 => rcc.is_enabled_dma2_clock(),
                HCLK1::GPIOH => rcc.is_enabled_gpioh_clock(),
                HCLK1::GPIOG => rcc.is_enabled_gpiog_clock(),
                HCLK1::GPIOF => rcc.is_enabled_gpiof_clock(),
                HCLK1::GPIOE => rcc.is_enabled_gpioe_clock(),
                HCLK1::GPIOD => rcc.is_enabled_gpiod_clock(),
                HCLK1::GPIOC => rcc.is_enabled_gpioc_clock(),
                HCLK1::GPIOB => rcc.is_enabled_gpiob_clock(),
                HCLK1::GPIOA => rcc.is_enabled_gpioa_clock(),
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => rcc.is_enabled_rng_clock(),
                HCLK2::OTGFS => rcc.is_enabled_otgfs_clock(),
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => rcc.is_enabled_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => rcc.is_enabled_tim2_clock(),
                PCLK1::USART2 => rcc.is_enabled_usart2_clock(),
                PCLK1::USART3 => rcc.is_enabled_usart3_clock(),
                PCLK1::I2C1 => rcc.is_enabled_i2c1_clock(),
                PCLK1::SPI3 => rcc.is_enabled_spi3_clock(),
                PCLK1::CAN1 => rcc.is_enabled_can1_clock(),
                PCLK1::DAC => rcc.is_enabled_dac_clock(),
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => rcc.is_enabled_usart1_clock(),
                PCLK2::ADC1 => rcc.is_enabled_adc1_clock(),
                PCLK2::SYSCFG => rcc.is_enabled_syscfg_clock(),
            },
            PeripheralClockType::RTC => rcc.is_enabled_rtc_clock(),
            PeripheralClockType::PWR => rcc.is_enabled_pwr_clock(),
        }
    }

    fn enable(&self) {
        let rcc = self.clocks.get_rcc();
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => {
                    rcc.enable_dma1_clock();
                }
                HCLK1::DMA2 => {
                    rcc.enable_dma2_clock();
                }
                HCLK1::GPIOH => {
                    rcc.enable_gpioh_clock();
                }
                HCLK1::GPIOG => {
                    rcc.enable_gpiog_clock();
                }
                HCLK1::GPIOF => {
                    rcc.enable_gpiof_clock();
                }
                HCLK1::GPIOE => {
                    rcc.enable_gpioe_clock();
                }
                HCLK1::GPIOD => {
                    rcc.enable_gpiod_clock();
                }
                HCLK1::GPIOC => {
                    rcc.enable_gpioc_clock();
                }
                HCLK1::GPIOB => {
                    rcc.enable_gpiob_clock();
                }
                HCLK1::GPIOA => {
                    rcc.enable_gpioa_clock();
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => {
                    rcc.enable_rng_clock();
                }
                HCLK2::OTGFS => {
                    rcc.enable_otgfs_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => rcc.enable_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    rcc.enable_tim2_clock();
                }
                PCLK1::USART2 => {
                    rcc.enable_usart2_clock();
                }
                PCLK1::USART3 => {
                    rcc.enable_usart3_clock();
                }
                PCLK1::I2C1 => {
                    rcc.enable_i2c1_clock();
                }
                PCLK1::SPI3 => {
                    rcc.enable_spi3_clock();
                }
                PCLK1::CAN1 => {
                    rcc.enable_can1_clock();
                }
                PCLK1::DAC => {
                    rcc.enable_dac_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => {
                    rcc.enable_usart1_clock();
                }
                PCLK2::ADC1 => {
                    rcc.enable_adc1_clock();
                }
                PCLK2::SYSCFG => {
                    rcc.enable_syscfg_clock();
                }
            },
            PeripheralClockType::RTC => rcc.enable_rtc_clock(RtcClockSource::LSI),
            PeripheralClockType::PWR => rcc.enable_pwr_clock(),
        }
    }

    fn disable(&self) {
        let rcc = self.clocks.get_rcc();
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => {
                    rcc.disable_dma1_clock();
                }
                HCLK1::DMA2 => {
                    rcc.disable_dma2_clock();
                }
                HCLK1::GPIOH => {
                    rcc.disable_gpioh_clock();
                }
                HCLK1::GPIOG => {
                    rcc.disable_gpiog_clock();
                }
                HCLK1::GPIOF => {
                    rcc.disable_gpiof_clock();
                }
                HCLK1::GPIOE => {
                    rcc.disable_gpioe_clock();
                }
                HCLK1::GPIOD => {
                    rcc.disable_gpiod_clock();
                }
                HCLK1::GPIOC => {
                    rcc.disable_gpioc_clock();
                }
                HCLK1::GPIOB => {
                    rcc.disable_gpiob_clock();
                }
                HCLK1::GPIOA => {
                    rcc.disable_gpioa_clock();
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => {
                    rcc.disable_rng_clock();
                }
                HCLK2::OTGFS => {
                    rcc.disable_otgfs_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => rcc.disable_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    rcc.disable_tim2_clock();
                }
                PCLK1::USART2 => {
                    rcc.disable_usart2_clock();
                }
                PCLK1::USART3 => {
                    rcc.disable_usart3_clock();
                }
                PCLK1::I2C1 => {
                    rcc.disable_i2c1_clock();
                }
                PCLK1::SPI3 => {
                    rcc.disable_spi3_clock();
                }
                PCLK1::CAN1 => {
                    rcc.disable_can1_clock();
                }
                PCLK1::DAC => {
                    rcc.disable_dac_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => {
                    rcc.disable_usart1_clock();
                }
                PCLK2::ADC1 => {
                    rcc.disable_adc1_clock();
                }
                PCLK2::SYSCFG => {
                    rcc.disable_syscfg_clock();
                }
            },
            PeripheralClockType::RTC => rcc.disable_rtc_clock(),
            PeripheralClockType::PWR => rcc.disable_pwr_clock(),
        }
    }
}
