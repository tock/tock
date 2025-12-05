// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use crate::clocks::Stm32l4Clocks;
use kernel::platform::chip::ClockInterface;

pub struct PeripheralClock<'a> {
    pub clock: PeripheralClockType,
    clocks: &'a dyn Stm32l4Clocks,
}

/// Bus + Clock name for the peripherals
pub enum PeripheralClockType {
    AHB1(HCLK1),
    AHB2(HCLK2),
    APB1(PCLK1),
    APB2(PCLK2),
}

/// Peripherals clocked by HCLK1
pub enum HCLK1 {
    DMA1,
    DMA2,
}

/// Peripherals clocked by HCLK2
pub enum HCLK2 {
    GPIOH,
    GPIOG,
    GPIOF,
    GPIOE,
    GPIOD,
    GPIOC,
    GPIOB,
    GPIOA,
}

/// Peripherals clocked by PCLK1
pub enum PCLK1 {
    USART2,
    USART3,
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    SYSCFG,
    USART1,
}

impl<'a> PeripheralClock<'a> {
    pub const fn new(clock: PeripheralClockType, clocks: &'a dyn Stm32l4Clocks) -> Self {
        Self { clock, clocks }
    }

    pub fn get_frequency(&self) -> u32 {
        let rcc = self.clocks.get_rcc();
        let hclk_freq = self.clocks.get_ahb_frequency();
        match self.clock {
            PeripheralClockType::AHB1(_) | PeripheralClockType::AHB2(_) => hclk_freq as u32,
            PeripheralClockType::APB1(_) => {
                let prescaler = rcc.get_apb1_prescaler();
                (hclk_freq / usize::from(prescaler)) as u32
            }
            PeripheralClockType::APB2(_) => {
                let prescaler = rcc.get_apb2_prescaler();
                (hclk_freq / usize::from(prescaler)) as u32
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
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOH => rcc.is_enabled_gpioh_clock(),
                HCLK2::GPIOG => rcc.is_enabled_gpiog_clock(),
                HCLK2::GPIOF => rcc.is_enabled_gpiof_clock(),
                HCLK2::GPIOE => rcc.is_enabled_gpioe_clock(),
                HCLK2::GPIOD => rcc.is_enabled_gpiod_clock(),
                HCLK2::GPIOC => rcc.is_enabled_gpioc_clock(),
                HCLK2::GPIOB => rcc.is_enabled_gpiob_clock(),
                HCLK2::GPIOA => rcc.is_enabled_gpioa_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::USART2 => rcc.is_enabled_usart2_clock(),
                PCLK1::USART3 => rcc.is_enabled_usart3_clock(),
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::SYSCFG => rcc.is_enabled_syscfg_clock(),
                PCLK2::USART1 => rcc.is_enabled_usart1_clock(),
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
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOH => {
                    rcc.enable_gpioh_clock();
                }
                HCLK2::GPIOG => {
                    rcc.enable_gpiog_clock();
                }
                HCLK2::GPIOF => {
                    rcc.enable_gpiof_clock();
                }
                HCLK2::GPIOE => {
                    rcc.enable_gpioe_clock();
                }
                HCLK2::GPIOD => {
                    rcc.enable_gpiod_clock();
                }
                HCLK2::GPIOC => {
                    rcc.enable_gpioc_clock();
                }
                HCLK2::GPIOB => {
                    rcc.enable_gpiob_clock();
                }
                HCLK2::GPIOA => {
                    rcc.enable_gpioa_clock();
                }
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::USART2 => {
                    rcc.enable_usart2_clock();
                }
                PCLK1::USART3 => {
                    rcc.enable_usart3_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::SYSCFG => {
                    rcc.enable_syscfg_clock();
                }
                PCLK2::USART1 => {
                    rcc.enable_usart1_clock();
                }
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
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::GPIOH => {
                    rcc.disable_gpioh_clock();
                }
                HCLK2::GPIOG => {
                    rcc.disable_gpiog_clock();
                }
                HCLK2::GPIOF => {
                    rcc.disable_gpiof_clock();
                }
                HCLK2::GPIOE => {
                    rcc.disable_gpioe_clock();
                }
                HCLK2::GPIOD => {
                    rcc.disable_gpiod_clock();
                }
                HCLK2::GPIOC => {
                    rcc.disable_gpioc_clock();
                }
                HCLK2::GPIOB => {
                    rcc.disable_gpiob_clock();
                }
                HCLK2::GPIOA => {
                    rcc.disable_gpioa_clock();
                }
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::USART2 => {
                    rcc.disable_usart2_clock();
                }
                PCLK1::USART3 => {
                    rcc.disable_usart3_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::SYSCFG => {
                    rcc.disable_syscfg_clock();
                }
                PCLK2::USART1 => {
                    rcc.disable_usart1_clock();
                }
            },
        }
    }
}
