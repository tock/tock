// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub RccRegisters {
        /// Control register
        (0x000 => cr: ReadWrite<u32>),
        (0x004 => _reserved0: [u32; 34]),
        /// AHB2 peripheral clock enable register 1
        (0x08C => ahb2enr1: ReadWrite<u32>),
        (0x090 => _reserved1: [u32; 3]),
        /// APB1 peripheral clock enable register 1
        (0x09C => apb1enr1: ReadWrite<u32>),
        (0x0A0 => _reserved2: [u32; 1]),
        /// APB2 peripheral clock enable register
        (0x0A4 => apb2enr: ReadWrite<u32>),
        /// APB3 peripheral clock enable register
        (0x0A8 => apb3enr: ReadWrite<u32>),
        (0x0AC => _reserved3: [u32; 13]),
        /// Peripherals independent clock configuration register 1
        (0x0E0 => ccipr1: ReadWrite<u32>),
        (0x0E4 => @END),
    }
}

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

impl Rcc {
    pub const fn new(base: StaticRef<RccRegisters>) -> Rcc {
        Rcc { registers: base }
    }

    pub fn enable_gpioa(&self) {
        let val = self.registers.ahb2enr1.get();
        self.registers.ahb2enr1.set(val | 1);
    }

    pub fn enable_gpioc(&self) {
        let val = self.registers.ahb2enr1.get();
        self.registers.ahb2enr1.set(val | (1 << 2));
    }

    pub fn enable_usart1(&self) {
        let val = self.registers.apb2enr.get();
        self.registers.apb2enr.set(val | (1 << 14));
    }

    pub fn enable_tim2(&self) {
        let val = self.registers.apb1enr1.get();
        self.registers.apb1enr1.set(val | 1);
    }

    pub fn enable_syscfg(&self) {
        let val = self.registers.apb3enr.get();
        self.registers.apb3enr.set(val | (1 << 1));
    }

    pub fn set_usart1_source_pclk(&self) {
        let val = self.registers.ccipr1.get();
        self.registers.ccipr1.set(val & !3);
    }
}
