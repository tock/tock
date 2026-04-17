// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

use crate::clocks::{phclk, Stm32l4Clocks};
use crate::gpio;

/// System configuration controller (STM32L476)
/// Reference: RM0351 (SYSCFG chapter)
#[repr(C)]
struct SyscfgRegisters {
    /// memory remap register
    memrmp: ReadWrite<u32, MEMRMP::Register>,
    /// configuration register
    cfgr: ReadWrite<u32, CFGR::Register>,
    /// external interrupt configuration register 1
    exticr1: ReadWrite<u32, EXTICR1::Register>,
    /// external interrupt configuration register 2
    exticr2: ReadWrite<u32, EXTICR2::Register>,
    /// external interrupt configuration register 3
    exticr3: ReadWrite<u32, EXTICR3::Register>,
    /// external interrupt configuration register 4
    exticr4: ReadWrite<u32, EXTICR4::Register>,
    /// SRAM2 control and status register
    scsr: ReadWrite<u32, SCSR::Register>,
    /// configuration register 2
    cfgr2: ReadWrite<u32, CFGR2::Register>,
    /// SRAM2 write protection register
    swpr: ReadWrite<u32, SWPR::Register>,
    /// SRAM2 key register
    skr: ReadWrite<u32, SKR::Register>,
}

register_bitfields![u32,
    MEMRMP [
        /// Memory mapping selection
        MEM_MODE OFFSET(0) NUMBITS(3) []
    ],
    CFGR [
        /// FPU interrupt enable bits (aggregate placeholder)
        FPU_IE OFFSET(26) NUMBITS(6) [],
        /// Booster enable (improves performance at low VOS) (optional)
        BOOSTEN OFFSET(8) NUMBITS(1) []
    ],
    EXTICR1 [
        EXTI3 OFFSET(12) NUMBITS(4) [],
        EXTI2 OFFSET(8) NUMBITS(4) [],
        EXTI1 OFFSET(4) NUMBITS(4) [],
        EXTI0 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR2 [
        EXTI7 OFFSET(12) NUMBITS(4) [],
        EXTI6 OFFSET(8) NUMBITS(4) [],
        EXTI5 OFFSET(4) NUMBITS(4) [],
        EXTI4 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR3 [
        EXTI11 OFFSET(12) NUMBITS(4) [],
        EXTI10 OFFSET(8) NUMBITS(4) [],
        EXTI9 OFFSET(4) NUMBITS(4) [],
        EXTI8 OFFSET(0) NUMBITS(4) []
    ],
    EXTICR4 [
        EXTI15 OFFSET(12) NUMBITS(4) [],
        EXTI14 OFFSET(8) NUMBITS(4) [],
        EXTI13 OFFSET(4) NUMBITS(4) [],
        EXTI12 OFFSET(0) NUMBITS(4) []
    ],
    SCSR [
        /// SRAM2 erase
        SRAM2ER OFFSET(0) NUMBITS(1) [],
        /// SRAM2 busy (erase ongoing)
        SRAM2BSY OFFSET(1) NUMBITS(1) []
    ],
    CFGR2 [
        /// Cortex-M4 LOCKUP output enable
        LOCUP_LOCK OFFSET(0) NUMBITS(1) [],
        /// SRAM parity error address latch disable
        BYP_ADDR_PAR OFFSET(1) NUMBITS(1) []
    ],
    SWPR [
        /// SRAM2 pages write protection bits (0..23 for 24 pages on L4 family with 192KB SRAM2)
        P0 OFFSET(0) NUMBITS(1) [],
        P1 OFFSET(1) NUMBITS(1) [],
        P2 OFFSET(2) NUMBITS(1) [],
        P3 OFFSET(3) NUMBITS(1) [],
        P4 OFFSET(4) NUMBITS(1) [],
        P5 OFFSET(5) NUMBITS(1) [],
        P6 OFFSET(6) NUMBITS(1) [],
        P7 OFFSET(7) NUMBITS(1) [],
        P8 OFFSET(8) NUMBITS(1) [],
        P9 OFFSET(9) NUMBITS(1) [],
        P10 OFFSET(10) NUMBITS(1) [],
        P11 OFFSET(11) NUMBITS(1) [],
        P12 OFFSET(12) NUMBITS(1) [],
        P13 OFFSET(13) NUMBITS(1) [],
        P14 OFFSET(14) NUMBITS(1) [],
        P15 OFFSET(15) NUMBITS(1) [],
        P16 OFFSET(16) NUMBITS(1) [],
        P17 OFFSET(17) NUMBITS(1) [],
        P18 OFFSET(18) NUMBITS(1) [],
        P19 OFFSET(19) NUMBITS(1) [],
        P20 OFFSET(20) NUMBITS(1) [],
        P21 OFFSET(21) NUMBITS(1) [],
        P22 OFFSET(22) NUMBITS(1) [],
        P23 OFFSET(23) NUMBITS(1) []
    ],
    SKR [
        /// SRAM2 write protection unlock key
        KEY OFFSET(0) NUMBITS(8) []
    ]
];

const SYSCFG_BASE: StaticRef<SyscfgRegisters> =
    unsafe { StaticRef::new(0x40010000 as *const SyscfgRegisters) };

enum_from_primitive! {
    #[repr(u32)]
    /// SYSCFG EXTI configuration [^1]
    ///
    /// [^1]: Section 9.2.3, page 317 of reference manual
    enum ExtiCrId {
        PA = 0b0000,
        PB = 0b0001,
        PC = 0b0010,
        PD = 0b0011,
        PE = 0b0100,
        PF = 0b0101,
        PG = 0b0110,
        PH = 0b0111,
    }
}

pub struct Syscfg<'a> {
    registers: StaticRef<SyscfgRegisters>,
    clock: SyscfgClock<'a>,
}

impl<'a> Syscfg<'a> {
    pub const fn new(clocks: &'a dyn Stm32l4Clocks) -> Self {
        Self {
            registers: SYSCFG_BASE,
            clock: SyscfgClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB2(phclk::PCLK2::SYSCFG),
                clocks,
            )),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    /// Configures the SYSCFG_EXTICR{1, 2, 3, 4} registers
    pub fn configure_interrupt(&self, pinid: gpio::PinId) {
        let exticrid = self.get_exticrid_from_port_num(pinid.get_port_number());

        let pin_num = pinid.get_pin_number();
        match pin_num {
            // SYSCFG_EXTICR1
            0b0000 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI0.val(exticrid as u32)),
            0b0001 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI1.val(exticrid as u32)),
            0b0010 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI2.val(exticrid as u32)),
            0b0011 => self
                .registers
                .exticr1
                .modify(EXTICR1::EXTI3.val(exticrid as u32)),
            // SYSCFG_EXTICR2
            0b0100 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI4.val(exticrid as u32)),
            0b0101 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI5.val(exticrid as u32)),
            0b0110 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI6.val(exticrid as u32)),
            0b0111 => self
                .registers
                .exticr2
                .modify(EXTICR2::EXTI7.val(exticrid as u32)),
            // SYSCFG_EXTICR3
            0b1000 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI8.val(exticrid as u32)),
            0b1001 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI9.val(exticrid as u32)),
            0b1010 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI10.val(exticrid as u32)),
            0b1011 => self
                .registers
                .exticr3
                .modify(EXTICR3::EXTI11.val(exticrid as u32)),
            // SYSCFG_EXTICR4
            0b1100 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI12.val(exticrid as u32)),
            0b1101 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI13.val(exticrid as u32)),
            0b1110 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI14.val(exticrid as u32)),
            0b1111 => self
                .registers
                .exticr4
                .modify(EXTICR4::EXTI15.val(exticrid as u32)),
            _ => {}
        }
    }

    fn get_exticrid_from_port_num(&self, port_num: u8) -> ExtiCrId {
        ExtiCrId::from_u32(u32::from(port_num)).unwrap()
    }
}

struct SyscfgClock<'a>(phclk::PeripheralClock<'a>);

impl ClockInterface for SyscfgClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
