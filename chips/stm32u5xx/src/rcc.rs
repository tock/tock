// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub RccRegisters {
        /// Control register
        (0x000 => cr: ReadWrite<u32>),
        (0x004 => _reserved0: [u32; 33]),
        /// AHB1 peripheral clock enable register
        (0x088 => ahb1enr: ReadWrite<u32, AHB1ENR::Register>),
        /// AHB2 peripheral clock enable register 1
        (0x08C => ahb2enr1: ReadWrite<u32, AHB2ENR1::Register>),
        (0x090 => _reserved1: [u32; 3]),
        /// APB1 peripheral clock enable register 1
        (0x09C => apb1enr1: ReadWrite<u32, APB1ENR1::Register>),
        (0x0A0 => _reserved2: [u32; 1]),
        /// APB2 peripheral clock enable register
        (0x0A4 => apb2enr: ReadWrite<u32, APB2ENR::Register>),
        /// APB3 peripheral clock enable register
        (0x0A8 => apb3enr: ReadWrite<u32, APB3ENR::Register>),
        (0x0AC => _reserved3: [u32; 13]),
        /// Peripherals independent clock configuration register 1
        (0x0E0 => ccipr1: ReadWrite<u32, CCIPR1::Register>),
        (0x0E4 => @END),
    }
}

register_bitfields![u32,
    pub AHB1ENR [
        GPDMA1EN OFFSET(0) NUMBITS(1) []
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
        HASHEN OFFSET(17) NUMBITS(1) [],
    ],
    pub APB1ENR1 [
        TIM2EN OFFSET(0) NUMBITS(1) []
    ],
    pub APB2ENR [
        USART1EN OFFSET(14) NUMBITS(1) []
    ],
    pub APB3ENR [
        SYSCFGEN OFFSET(1) NUMBITS(1) []
    ],
    pub CCIPR1 [
        USART1SEL OFFSET(0) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ]
    ]
];

/// Base address for RCC in Secure mode.
pub const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x46020C00 as *const RccRegisters) };

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

impl Rcc {
    pub const fn new(base: StaticRef<RccRegisters>) -> Rcc {
        Rcc { registers: base }
    }

    pub fn enable_dma1(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPDMA1EN::SET);
    }

    pub fn enable_gpioa(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::GPIOAEN::SET);
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

    pub fn enable_syscfg(&self) {
        self.registers.apb3enr.modify(APB3ENR::SYSCFGEN::SET);
    }

    pub fn set_usart1_source_pclk(&self) {
        self.registers.ccipr1.modify(CCIPR1::USART1SEL::PCLK);
    }

    pub fn enable_hash(&self) {
        self.registers.ahb2enr1.modify(AHB2ENR1::HASHEN::SET);
    }
}
