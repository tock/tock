// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! The clock module for STM32WLE5xx chips.
//!
//! This is highly similar to the one for STM32L4xx chips. This clock
//! implementation provides the minimal functionality required to enable
//! peripherals and configure speeds (as tested for I2C and UART). This
//! is still highly a work in progress and documentation comments here
//! describing the usage will be updated as development continues.

use crate::clocks::Stm32wle5xxClocks;
use crate::rcc::{APBPrescaler, RtcClockSource};
use kernel::platform::chip::ClockInterface;

pub struct PeripheralClock<'a> {
    pub clock: PeripheralClockType,
    clocks: &'a dyn Stm32wle5xxClocks,
}

/// Bus + Clock name for the peripherals
pub enum PeripheralClockType {
    AHB1(HCLK1),
    AHB2(HCLK2),
    AHB3(HCLK3),
    APB1(PCLK1),
    APB2(PCLK2),
    APB3(PCLK3),
    RTC,
}

/// Peripherals clocked by HCLK1
pub enum HCLK1 {
    DMA1,
    DMA2,
    DMAMUX1,
    CRC,
}

/// Peripherals clocked by HCLK3
pub enum HCLK3 {
    PKA,
    AES,
    RNG,
    HSEM,
    FLASH,
}

/// Peripherals clocked by HCLK2
pub enum HCLK2 {
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOH,
}

/// Peripherals clocked by PCLK1
pub enum PCLK1 {
    TIM2,
    RTCAPB,
    WWDG,
    SPI2S2,
    USART2,
    I2C1,
    I2C2,
    I2C3,
    DAC,
    LPTIM1,
    LPUART1,
    LPTIM2,
    LPTIM3,
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    ADC,
    TIM1,
    SPI1,
    USART1,
    TIM16,
    TIM17,
}

/// Peripherals clocked by PCLK3
pub enum PCLK3 {
    SUBGHZSPI,
}

impl<'a> PeripheralClock<'a> {
    pub const fn new(clock: PeripheralClockType, clocks: &'a dyn Stm32wle5xxClocks) -> Self {
        Self { clock, clocks }
    }

    pub fn get_frequency(&self) -> u32 {
        #[inline(always)]
        fn tim_freq(hclk_freq: usize, prescaler: APBPrescaler) -> usize {
            match prescaler {
                APBPrescaler::DivideBy1 | APBPrescaler::DivideBy2 | APBPrescaler::DivideBy4 => {
                    hclk_freq
                }
                _ => hclk_freq / usize::from(prescaler) * 4,
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
                    PCLK1::TIM2 => tim_freq(hclk_freq, prescaler) as u32,
                    _ => (hclk_freq / usize::from(prescaler)) as u32,
                }
            }
            PeripheralClockType::APB2(_) => {
                let prescaler = rcc.get_apb2_prescaler();
                (hclk_freq / usize::from(prescaler)) as u32
            }
            //TODO: implement clock frequency retrieval for RTC and PWR peripherals
            PeripheralClockType::RTC => todo!(),
            PeripheralClockType::APB3(ref v) => {
                let prescaler = rcc.get_apb3_prescaler();
                match v {
                    PCLK3::SUBGHZSPI => (hclk_freq / usize::from(prescaler)) as u32,
                }
            }
        }
    }
}

impl ClockInterface for PeripheralClock<'_> {
    fn is_enabled(&self) -> bool {
        let rcc = self.clocks.get_rcc();
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => rcc.is_enabled_dma1_clock(),
                HCLK1::DMA2 => rcc.is_enabled_dma2_clock(),
                HCLK1::CRC => unimplemented!(),
                HCLK1::DMAMUX1 => unimplemented!(),
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOA => rcc.is_enabled_gpioa_clock(),
                HCLK2::GPIOB => rcc.is_enabled_gpiob_clock(),
                HCLK2::GPIOC => rcc.is_enabled_gpioc_clock(),
                HCLK2::GPIOH => rcc.is_enabled_gpioh_clock(),
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::AES => unimplemented!(),
                HCLK3::PKA => unimplemented!(),
                HCLK3::RNG => rcc.is_enabled_rng_clock(),
                HCLK3::HSEM => unimplemented!(),
                HCLK3::FLASH => unimplemented!(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => rcc.is_enabled_tim2_clock(),
                PCLK1::USART2 => rcc.is_enabled_usart2_clock(),
                PCLK1::I2C1 => rcc.is_enabled_i2c1_clock(),
                PCLK1::I2C2 => rcc.is_enabled_i2c2_clock(),
                PCLK1::DAC => rcc.is_enabled_dac_clock(),
                PCLK1::RTCAPB => unimplemented!(),
                PCLK1::LPTIM1 => unimplemented!(),
                PCLK1::LPTIM2 => unimplemented!(),
                PCLK1::LPTIM3 => unimplemented!(),
                PCLK1::LPUART1 => unimplemented!(),
                PCLK1::WWDG => unimplemented!(),
                PCLK1::SPI2S2 => unimplemented!(),
                PCLK1::I2C3 => unimplemented!(),
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => rcc.is_enabled_usart1_clock(),
                PCLK2::SPI1 => rcc.is_enabled_spi1_clock(),
                PCLK2::ADC => rcc.is_enabled_adc1_clock(),
                PCLK2::TIM1 => unimplemented!(),
                PCLK2::TIM16 => unimplemented!(),
                PCLK2::TIM17 => unimplemented!(),
            },
            PeripheralClockType::RTC => rcc.is_enabled_rtc_clock(),
            PeripheralClockType::APB3(ref v) => match v {
                PCLK3::SUBGHZSPI => rcc.is_enabled_subghzspi_clock(),
            },
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
                HCLK1::CRC => {
                    unimplemented!()
                }
                HCLK1::DMAMUX1 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOA => {
                    rcc.enable_gpioa_clock();
                }
                HCLK2::GPIOB => {
                    rcc.enable_gpiob_clock();
                }
                HCLK2::GPIOC => {
                    rcc.enable_gpioc_clock();
                }
                HCLK2::GPIOH => {
                    rcc.enable_gpioh_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::AES => {
                    unimplemented!()
                }
                HCLK3::PKA => {
                    unimplemented!()
                }
                HCLK3::RNG => {
                    rcc.enable_rng_clock();
                }
                HCLK3::HSEM => {
                    unimplemented!()
                }
                HCLK3::FLASH => {
                    unimplemented!()
                }
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    rcc.enable_tim2_clock();
                }
                PCLK1::USART2 => {
                    rcc.enable_usart2_clock();
                }
                PCLK1::I2C1 => {
                    rcc.enable_i2c1_clock();
                }
                PCLK1::I2C2 => {
                    rcc.enable_i2c2_clock();
                }
                PCLK1::DAC => {
                    rcc.enable_dac_clock();
                }
                PCLK1::RTCAPB => {
                    unimplemented!()
                }
                PCLK1::LPTIM1 => {
                    unimplemented!()
                }
                PCLK1::LPTIM2 => {
                    unimplemented!()
                }
                PCLK1::LPTIM3 => {
                    unimplemented!()
                }
                PCLK1::LPUART1 => {
                    unimplemented!()
                }
                PCLK1::WWDG => {
                    unimplemented!()
                }
                PCLK1::SPI2S2 => {
                    unimplemented!()
                }
                PCLK1::I2C3 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => rcc.enable_usart1_clock(),
                PCLK2::SPI1 => rcc.enable_spi1_clock(),
                PCLK2::ADC => {
                    rcc.enable_adc1_clock();
                }
                PCLK2::TIM1 => {
                    unimplemented!()
                }
                PCLK2::TIM16 => {
                    unimplemented!()
                }
                PCLK2::TIM17 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::RTC => rcc.enable_rtc_clock(RtcClockSource::LSI),
            PeripheralClockType::APB3(ref v) => match v {
                PCLK3::SUBGHZSPI => rcc.enable_subghzspi_clock(),
            },
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
                HCLK1::CRC => {
                    unimplemented!()
                }
                HCLK1::DMAMUX1 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOA => {
                    rcc.disable_gpioa_clock();
                }
                HCLK2::GPIOB => {
                    rcc.disable_gpiob_clock();
                }
                HCLK2::GPIOC => {
                    rcc.disable_gpioc_clock();
                }
                HCLK2::GPIOH => {
                    rcc.disable_gpioh_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::AES => {
                    unimplemented!()
                }
                HCLK3::PKA => {
                    unimplemented!()
                }
                HCLK3::RNG => {
                    rcc.disable_rng_clock();
                }
                HCLK3::HSEM => {
                    unimplemented!()
                }
                HCLK3::FLASH => {
                    unimplemented!()
                }
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    rcc.disable_tim2_clock();
                }
                PCLK1::USART2 => {
                    rcc.disable_usart2_clock();
                }
                PCLK1::DAC => {
                    rcc.disable_dac_clock();
                }
                PCLK1::I2C1 => {
                    rcc.disable_i2c1_clock();
                }
                PCLK1::I2C2 => {
                    rcc.disable_i2c2_clock();
                }
                PCLK1::RTCAPB => {
                    unimplemented!()
                }
                PCLK1::LPTIM1 => {
                    unimplemented!()
                }
                PCLK1::LPTIM2 => {
                    unimplemented!()
                }
                PCLK1::LPTIM3 => {
                    unimplemented!()
                }
                PCLK1::LPUART1 => {
                    unimplemented!()
                }
                PCLK1::WWDG => {
                    unimplemented!()
                }
                PCLK1::SPI2S2 => {
                    unimplemented!()
                }
                PCLK1::I2C3 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => rcc.disable_usart1_clock(),
                PCLK2::SPI1 => rcc.disable_spi1_clock(),
                PCLK2::ADC => {
                    rcc.disable_adc1_clock();
                }
                PCLK2::TIM1 => {
                    unimplemented!()
                }
                PCLK2::TIM16 => {
                    unimplemented!()
                }
                PCLK2::TIM17 => {
                    unimplemented!()
                }
            },
            PeripheralClockType::RTC => rcc.disable_rtc_clock(),
            PeripheralClockType::APB3(ref v) => match v {
                PCLK3::SUBGHZSPI => rcc.disable_subghzspi_clock(),
            },
        }
    }
}
