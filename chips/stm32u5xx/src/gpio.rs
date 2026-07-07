// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::hil::gpio;
use kernel::hil::gpio::Input;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

use crate::exti::{Exti, LineId};

register_structs! {
    pub GpioRegisters {
        /// GPIO port mode register
        (0x000 => pub moder: ReadWrite<u32, MODER::Register>),
        /// GPIO port output type register
        (0x004 => pub otyper: ReadWrite<u32, OTYPER::Register>),
        /// GPIO port output speed register
        (0x008 => pub ospeedr: ReadWrite<u32, OSPEEDR::Register>),
        /// GPIO port pull-up/pull-down register
        (0x00C => pub pupdr: ReadWrite<u32, PUPDR::Register>),
        /// GPIO port input data register
        (0x010 => pub idr: ReadOnly<u32, IDR::Register>),
        /// GPIO port output data register
        (0x014 => pub odr: ReadWrite<u32, ODR::Register>),
        /// GPIO port bit set/reset register
        (0x018 => pub bsrr: WriteOnly<u32, BSRR::Register>),
        /// GPIO port configuration lock register
        (0x01C => pub lckr: ReadWrite<u32>),
        /// GPIO alternate function low register
        (0x020 => pub afrl: ReadWrite<u32, AFRL::Register>),
        /// GPIO alternate function high register
        (0x024 => pub afrh: ReadWrite<u32, AFRH::Register>),
        (0x028 => @END),
    }
}

register_bitfields![u32,
    pub MODER [
        MODER0 OFFSET(0) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER1 OFFSET(2) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER2 OFFSET(4) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER3 OFFSET(6) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER4 OFFSET(8) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER5 OFFSET(10) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER6 OFFSET(12) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER7 OFFSET(14) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER8 OFFSET(16) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER9 OFFSET(18) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER10 OFFSET(20) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER11 OFFSET(22) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER12 OFFSET(24) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER13 OFFSET(26) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER14 OFFSET(28) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ],
        MODER15 OFFSET(30) NUMBITS(2) [
            Input = 0,
            Output = 1,
            AlternateFunction = 2,
            Analog = 3
        ]
    ],
    pub OTYPER [
        OT0 OFFSET(0) NUMBITS(1) [],
        OT1 OFFSET(1) NUMBITS(1) [],
        OT2 OFFSET(2) NUMBITS(1) [],
        OT3 OFFSET(3) NUMBITS(1) [],
        OT4 OFFSET(4) NUMBITS(1) [],
        OT5 OFFSET(5) NUMBITS(1) [],
        OT6 OFFSET(6) NUMBITS(1) [],
        OT7 OFFSET(7) NUMBITS(1) [],
        OT8 OFFSET(8) NUMBITS(1) [],
        OT9 OFFSET(9) NUMBITS(1) [],
        OT10 OFFSET(10) NUMBITS(1) [],
        OT11 OFFSET(11) NUMBITS(1) [],
        OT12 OFFSET(12) NUMBITS(1) [],
        OT13 OFFSET(13) NUMBITS(1) [],
        OT14 OFFSET(14) NUMBITS(1) [],
        OT15 OFFSET(15) NUMBITS(1) []
    ],
    pub OSPEEDR [
        OSPEEDR0 OFFSET(0) NUMBITS(2) [],
        OSPEEDR1 OFFSET(2) NUMBITS(2) [],
        OSPEEDR2 OFFSET(4) NUMBITS(2) [],
        OSPEEDR3 OFFSET(6) NUMBITS(2) [],
        OSPEEDR4 OFFSET(8) NUMBITS(2) [],
        OSPEEDR5 OFFSET(10) NUMBITS(2) [],
        OSPEEDR6 OFFSET(12) NUMBITS(2) [],
        OSPEEDR7 OFFSET(14) NUMBITS(2) [],
        OSPEEDR8 OFFSET(16) NUMBITS(2) [],
        OSPEEDR9 OFFSET(18) NUMBITS(2) [],
        OSPEEDR10 OFFSET(20) NUMBITS(2) [],
        OSPEEDR11 OFFSET(22) NUMBITS(2) [],
        OSPEEDR12 OFFSET(24) NUMBITS(2) [],
        OSPEEDR13 OFFSET(26) NUMBITS(2) [],
        OSPEEDR14 OFFSET(28) NUMBITS(2) [],
        OSPEEDR15 OFFSET(30) NUMBITS(2) []
    ],
    pub PUPDR [
        PUPDR0 OFFSET(0) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR1 OFFSET(2) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR2 OFFSET(4) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR3 OFFSET(6) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR4 OFFSET(8) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR5 OFFSET(10) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR6 OFFSET(12) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR7 OFFSET(14) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR8 OFFSET(16) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR9 OFFSET(18) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR10 OFFSET(20) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR11 OFFSET(22) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR12 OFFSET(24) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR13 OFFSET(26) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR14 OFFSET(28) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ],
        PUPDR15 OFFSET(30) NUMBITS(2) [
            None = 0,
            PullUp = 1,
            PullDown = 2
        ]
    ],
    pub IDR [
        ID0 OFFSET(0) NUMBITS(1) [],
        ID1 OFFSET(1) NUMBITS(1) [],
        ID2 OFFSET(2) NUMBITS(1) [],
        ID3 OFFSET(3) NUMBITS(1) [],
        ID4 OFFSET(4) NUMBITS(1) [],
        ID5 OFFSET(5) NUMBITS(1) [],
        ID6 OFFSET(6) NUMBITS(1) [],
        ID7 OFFSET(7) NUMBITS(1) [],
        ID8 OFFSET(8) NUMBITS(1) [],
        ID9 OFFSET(9) NUMBITS(1) [],
        ID10 OFFSET(10) NUMBITS(1) [],
        ID11 OFFSET(11) NUMBITS(1) [],
        ID12 OFFSET(12) NUMBITS(1) [],
        ID13 OFFSET(13) NUMBITS(1) [],
        ID14 OFFSET(14) NUMBITS(1) [],
        ID15 OFFSET(15) NUMBITS(1) []
    ],
    pub ODR [
        OD0 OFFSET(0) NUMBITS(1) [],
        OD1 OFFSET(1) NUMBITS(1) [],
        OD2 OFFSET(2) NUMBITS(1) [],
        OD3 OFFSET(3) NUMBITS(1) [],
        OD4 OFFSET(4) NUMBITS(1) [],
        OD5 OFFSET(5) NUMBITS(1) [],
        OD6 OFFSET(6) NUMBITS(1) [],
        OD7 OFFSET(7) NUMBITS(1) [],
        OD8 OFFSET(8) NUMBITS(1) [],
        OD9 OFFSET(9) NUMBITS(1) [],
        OD10 OFFSET(10) NUMBITS(1) [],
        OD11 OFFSET(11) NUMBITS(1) [],
        OD12 OFFSET(12) NUMBITS(1) [],
        OD13 OFFSET(13) NUMBITS(1) [],
        OD14 OFFSET(14) NUMBITS(1) [],
        OD15 OFFSET(15) NUMBITS(1) []
    ],
    pub BSRR [
        BS0 OFFSET(0) NUMBITS(1) [],
        BS1 OFFSET(1) NUMBITS(1) [],
        BS2 OFFSET(2) NUMBITS(1) [],
        BS3 OFFSET(3) NUMBITS(1) [],
        BS4 OFFSET(4) NUMBITS(1) [],
        BS5 OFFSET(5) NUMBITS(1) [],
        BS6 OFFSET(6) NUMBITS(1) [],
        BS7 OFFSET(7) NUMBITS(1) [],
        BS8 OFFSET(8) NUMBITS(1) [],
        BS9 OFFSET(9) NUMBITS(1) [],
        BS10 OFFSET(10) NUMBITS(1) [],
        BS11 OFFSET(11) NUMBITS(1) [],
        BS12 OFFSET(12) NUMBITS(1) [],
        BS13 OFFSET(13) NUMBITS(1) [],
        BS14 OFFSET(14) NUMBITS(1) [],
        BS15 OFFSET(15) NUMBITS(1) [],
        BR0 OFFSET(16) NUMBITS(1) [],
        BR1 OFFSET(17) NUMBITS(1) [],
        BR2 OFFSET(18) NUMBITS(1) [],
        BR3 OFFSET(19) NUMBITS(1) [],
        BR4 OFFSET(20) NUMBITS(1) [],
        BR5 OFFSET(21) NUMBITS(1) [],
        BR6 OFFSET(22) NUMBITS(1) [],
        BR7 OFFSET(23) NUMBITS(1) [],
        BR8 OFFSET(24) NUMBITS(1) [],
        BR9 OFFSET(25) NUMBITS(1) [],
        BR10 OFFSET(26) NUMBITS(1) [],
        BR11 OFFSET(27) NUMBITS(1) [],
        BR12 OFFSET(28) NUMBITS(1) [],
        BR13 OFFSET(29) NUMBITS(1) [],
        BR14 OFFSET(30) NUMBITS(1) [],
        BR15 OFFSET(31) NUMBITS(1) []
    ],
    pub AFRL [
        AF0 OFFSET(0) NUMBITS(4) [],
        AF1 OFFSET(4) NUMBITS(4) [],
        AF2 OFFSET(8) NUMBITS(4) [],
        AF3 OFFSET(12) NUMBITS(4) [],
        AF4 OFFSET(16) NUMBITS(4) [],
        AF5 OFFSET(20) NUMBITS(4) [],
        AF6 OFFSET(24) NUMBITS(4) [],
        AF7 OFFSET(28) NUMBITS(4) []
    ],
    pub AFRH [
        AF8 OFFSET(0) NUMBITS(4) [],
        AF9 OFFSET(4) NUMBITS(4) [],
        AF10 OFFSET(8) NUMBITS(4) [],
        AF11 OFFSET(12) NUMBITS(4) [],
        AF12 OFFSET(16) NUMBITS(4) [],
        AF13 OFFSET(20) NUMBITS(4) [],
        AF14 OFFSET(24) NUMBITS(4) [],
        AF15 OFFSET(28) NUMBITS(4) []
    ]
];

pub const GPIO_A_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x52020000 as *const GpioRegisters) };

pub const GPIO_B_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x52020400 as *const GpioRegisters) };

pub const GPIO_C_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x52020800 as *const GpioRegisters) };

#[derive(Copy, Clone, PartialEq)]
pub enum PinId {
    Pin00 = 0,
    Pin01 = 1,
    Pin02 = 2,
    Pin03 = 3,
    Pin04 = 4,
    Pin05 = 5,
    Pin06 = 6,
    Pin07 = 7,
    Pin08 = 8,
    Pin09 = 9,
    Pin10 = 10,
    Pin11 = 11,
    Pin12 = 12,
    Pin13 = 13,
    Pin14 = 14,
    Pin15 = 15,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Input = 0,
    Output = 1,
    AlternateFunction = 2,
    Analog = 3,
}

pub enum PullUpPullDown {
    None = 0,
    PullUp = 1,
    PullDown = 2,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GpioPort {
    PortA = 0,
    PortB = 1,
    PortC = 2,
    PortD = 3,
    PortE = 4,
    PortF = 5,
    PortG = 6,
    PortH = 7,
    PortI = 8,
    PortJ = 9,
}

pub struct Pin<'a> {
    registers: StaticRef<GpioRegisters>,
    pin: PinId,
    exti: &'a Exti<'a>,
    port_id: GpioPort,
    client: OptionalCell<&'a dyn gpio::Client>,
    exti_lineid: OptionalCell<LineId>,
}

impl<'a> Pin<'a> {
    // Only our own crate can create pins
    pub(crate) const fn new(
        base: StaticRef<GpioRegisters>,
        pin: PinId,
        exti: &'a Exti<'a>,
        port_id: GpioPort,
    ) -> Pin<'a> {
        Pin {
            registers: base,
            pin,
            exti,
            port_id,
            client: OptionalCell::empty(),
            exti_lineid: OptionalCell::empty(),
        }
    }
    /// Sets the mode of the pin.
    ///
    /// This is a low-level function intended for board-level muxing.
    /// For general GPIO usage, use the `kernel::hil::gpio::Configure` trait.
    pub fn set_mode(&self, mode: Mode) {
        match self.pin {
            PinId::Pin00 => self.registers.moder.modify(MODER::MODER0.val(mode as u32)),
            PinId::Pin01 => self.registers.moder.modify(MODER::MODER1.val(mode as u32)),
            PinId::Pin02 => self.registers.moder.modify(MODER::MODER2.val(mode as u32)),
            PinId::Pin03 => self.registers.moder.modify(MODER::MODER3.val(mode as u32)),
            PinId::Pin04 => self.registers.moder.modify(MODER::MODER4.val(mode as u32)),
            PinId::Pin05 => self.registers.moder.modify(MODER::MODER5.val(mode as u32)),
            PinId::Pin06 => self.registers.moder.modify(MODER::MODER6.val(mode as u32)),
            PinId::Pin07 => self.registers.moder.modify(MODER::MODER7.val(mode as u32)),
            PinId::Pin08 => self.registers.moder.modify(MODER::MODER8.val(mode as u32)),
            PinId::Pin09 => self.registers.moder.modify(MODER::MODER9.val(mode as u32)),
            PinId::Pin10 => self.registers.moder.modify(MODER::MODER10.val(mode as u32)),
            PinId::Pin11 => self.registers.moder.modify(MODER::MODER11.val(mode as u32)),
            PinId::Pin12 => self.registers.moder.modify(MODER::MODER12.val(mode as u32)),
            PinId::Pin13 => self.registers.moder.modify(MODER::MODER13.val(mode as u32)),
            PinId::Pin14 => self.registers.moder.modify(MODER::MODER14.val(mode as u32)),
            PinId::Pin15 => self.registers.moder.modify(MODER::MODER15.val(mode as u32)),
        }
    }
    /// Sets the output speed to 'Very High'.
    ///
    /// This is a low-level function intended for high-speed peripherals
    /// like USART or SPI.
    pub fn set_speed_high(&self) {
        match self.pin {
            PinId::Pin00 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR0.val(3)),
            PinId::Pin01 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR1.val(3)),
            PinId::Pin02 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR2.val(3)),
            PinId::Pin03 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR3.val(3)),
            PinId::Pin04 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR4.val(3)),
            PinId::Pin05 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR5.val(3)),
            PinId::Pin06 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR6.val(3)),
            PinId::Pin07 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR7.val(3)),
            PinId::Pin08 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR8.val(3)),
            PinId::Pin09 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR9.val(3)),
            PinId::Pin10 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR10.val(3)),
            PinId::Pin11 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR11.val(3)),
            PinId::Pin12 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR12.val(3)),
            PinId::Pin13 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR13.val(3)),
            PinId::Pin14 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR14.val(3)),
            PinId::Pin15 => self.registers.ospeedr.modify(OSPEEDR::OSPEEDR15.val(3)),
        }
    }

    /// Configures the pin for an Alternate Function (AF).
    ///
    /// Refer to the STM32U5 datasheet for the AF mapping table.
    /// This is a low-level function intended for peripheral initialization.
    pub fn set_alternate_function(&self, func: u32) {
        match self.pin {
            PinId::Pin00 => self.registers.afrl.modify(AFRL::AF0.val(func)),
            PinId::Pin01 => self.registers.afrl.modify(AFRL::AF1.val(func)),
            PinId::Pin02 => self.registers.afrl.modify(AFRL::AF2.val(func)),
            PinId::Pin03 => self.registers.afrl.modify(AFRL::AF3.val(func)),
            PinId::Pin04 => self.registers.afrl.modify(AFRL::AF4.val(func)),
            PinId::Pin05 => self.registers.afrl.modify(AFRL::AF5.val(func)),
            PinId::Pin06 => self.registers.afrl.modify(AFRL::AF6.val(func)),
            PinId::Pin07 => self.registers.afrl.modify(AFRL::AF7.val(func)),
            PinId::Pin08 => self.registers.afrh.modify(AFRH::AF8.val(func)),
            PinId::Pin09 => self.registers.afrh.modify(AFRH::AF9.val(func)),
            PinId::Pin10 => self.registers.afrh.modify(AFRH::AF10.val(func)),
            PinId::Pin11 => self.registers.afrh.modify(AFRH::AF11.val(func)),
            PinId::Pin12 => self.registers.afrh.modify(AFRH::AF12.val(func)),
            PinId::Pin13 => self.registers.afrh.modify(AFRH::AF13.val(func)),
            PinId::Pin14 => self.registers.afrh.modify(AFRH::AF14.val(func)),
            PinId::Pin15 => self.registers.afrh.modify(AFRH::AF15.val(func)),
        }
    }

    fn get_mode(&self) -> Mode {
        let val = match self.pin {
            PinId::Pin00 => self.registers.moder.read(MODER::MODER0),
            PinId::Pin01 => self.registers.moder.read(MODER::MODER1),
            PinId::Pin02 => self.registers.moder.read(MODER::MODER2),
            PinId::Pin03 => self.registers.moder.read(MODER::MODER3),
            PinId::Pin04 => self.registers.moder.read(MODER::MODER4),
            PinId::Pin05 => self.registers.moder.read(MODER::MODER5),
            PinId::Pin06 => self.registers.moder.read(MODER::MODER6),
            PinId::Pin07 => self.registers.moder.read(MODER::MODER7),
            PinId::Pin08 => self.registers.moder.read(MODER::MODER8),
            PinId::Pin09 => self.registers.moder.read(MODER::MODER9),
            PinId::Pin10 => self.registers.moder.read(MODER::MODER10),
            PinId::Pin11 => self.registers.moder.read(MODER::MODER11),
            PinId::Pin12 => self.registers.moder.read(MODER::MODER12),
            PinId::Pin13 => self.registers.moder.read(MODER::MODER13),
            PinId::Pin14 => self.registers.moder.read(MODER::MODER14),
            PinId::Pin15 => self.registers.moder.read(MODER::MODER15),
        };
        match val {
            0 => Mode::Input,
            1 => Mode::Output,
            2 => Mode::AlternateFunction,
            _ => Mode::Analog,
        }
    }

    fn set_pull(&self, pull: PullUpPullDown) {
        match self.pin {
            PinId::Pin00 => self.registers.pupdr.modify(PUPDR::PUPDR0.val(pull as u32)),
            PinId::Pin01 => self.registers.pupdr.modify(PUPDR::PUPDR1.val(pull as u32)),
            PinId::Pin02 => self.registers.pupdr.modify(PUPDR::PUPDR2.val(pull as u32)),
            PinId::Pin03 => self.registers.pupdr.modify(PUPDR::PUPDR3.val(pull as u32)),
            PinId::Pin04 => self.registers.pupdr.modify(PUPDR::PUPDR4.val(pull as u32)),
            PinId::Pin05 => self.registers.pupdr.modify(PUPDR::PUPDR5.val(pull as u32)),
            PinId::Pin06 => self.registers.pupdr.modify(PUPDR::PUPDR6.val(pull as u32)),
            PinId::Pin07 => self.registers.pupdr.modify(PUPDR::PUPDR7.val(pull as u32)),
            PinId::Pin08 => self.registers.pupdr.modify(PUPDR::PUPDR8.val(pull as u32)),
            PinId::Pin09 => self.registers.pupdr.modify(PUPDR::PUPDR9.val(pull as u32)),
            PinId::Pin10 => self.registers.pupdr.modify(PUPDR::PUPDR10.val(pull as u32)),
            PinId::Pin11 => self.registers.pupdr.modify(PUPDR::PUPDR11.val(pull as u32)),
            PinId::Pin12 => self.registers.pupdr.modify(PUPDR::PUPDR12.val(pull as u32)),
            PinId::Pin13 => self.registers.pupdr.modify(PUPDR::PUPDR13.val(pull as u32)),
            PinId::Pin14 => self.registers.pupdr.modify(PUPDR::PUPDR14.val(pull as u32)),
            PinId::Pin15 => self.registers.pupdr.modify(PUPDR::PUPDR15.val(pull as u32)),
        }
    }

    fn get_pull(&self) -> PullUpPullDown {
        let val = match self.pin {
            PinId::Pin00 => self.registers.pupdr.read(PUPDR::PUPDR0),
            PinId::Pin01 => self.registers.pupdr.read(PUPDR::PUPDR1),
            PinId::Pin02 => self.registers.pupdr.read(PUPDR::PUPDR2),
            PinId::Pin03 => self.registers.pupdr.read(PUPDR::PUPDR3),
            PinId::Pin04 => self.registers.pupdr.read(PUPDR::PUPDR4),
            PinId::Pin05 => self.registers.pupdr.read(PUPDR::PUPDR5),
            PinId::Pin06 => self.registers.pupdr.read(PUPDR::PUPDR6),
            PinId::Pin07 => self.registers.pupdr.read(PUPDR::PUPDR7),
            PinId::Pin08 => self.registers.pupdr.read(PUPDR::PUPDR8),
            PinId::Pin09 => self.registers.pupdr.read(PUPDR::PUPDR9),
            PinId::Pin10 => self.registers.pupdr.read(PUPDR::PUPDR10),
            PinId::Pin11 => self.registers.pupdr.read(PUPDR::PUPDR11),
            PinId::Pin12 => self.registers.pupdr.read(PUPDR::PUPDR12),
            PinId::Pin13 => self.registers.pupdr.read(PUPDR::PUPDR13),
            PinId::Pin14 => self.registers.pupdr.read(PUPDR::PUPDR14),
            PinId::Pin15 => self.registers.pupdr.read(PUPDR::PUPDR15),
        };
        match val {
            1 => PullUpPullDown::PullUp,
            2 => PullUpPullDown::PullDown,
            _ => PullUpPullDown::None,
        }
    }
}

impl gpio::Configure for Pin<'_> {
    fn configuration(&self) -> gpio::Configuration {
        match self.get_mode() {
            Mode::Input => gpio::Configuration::Input,
            Mode::Output => gpio::Configuration::Output,
            Mode::AlternateFunction => gpio::Configuration::Function,
            Mode::Analog => gpio::Configuration::LowPower,
        }
    }

    fn make_output(&self) -> gpio::Configuration {
        self.set_mode(Mode::Output);
        gpio::Configuration::Output
    }

    fn disable_output(&self) -> gpio::Configuration {
        self.set_mode(Mode::Input);
        gpio::Configuration::Input
    }

    fn make_input(&self) -> gpio::Configuration {
        self.set_mode(Mode::Input);
        gpio::Configuration::Input
    }

    fn disable_input(&self) -> gpio::Configuration {
        self.set_mode(Mode::Analog);
        gpio::Configuration::LowPower
    }

    /// Deactivates the pin to its lowest power state.
    ///
    /// According to RM0456 (STM32U5 Reference Manual), Section 13.3.12
    /// (Analog configuration), setting a pin to Analog mode deactivates
    /// the Schmitt trigger input, providing zero consumption for every
    /// analog value of the I/O pin. We do not disable the clock to
    /// the entire GPIO port here because other pins on the same
    /// port may still be in use.
    fn deactivate_to_low_power(&self) {
        self.set_mode(Mode::Analog);
    }

    fn set_floating_state(&self, state: gpio::FloatingState) {
        match state {
            gpio::FloatingState::PullUp => self.set_pull(PullUpPullDown::PullUp),
            gpio::FloatingState::PullDown => self.set_pull(PullUpPullDown::PullDown),
            gpio::FloatingState::PullNone => self.set_pull(PullUpPullDown::None),
        }
    }

    fn floating_state(&self) -> gpio::FloatingState {
        match self.get_pull() {
            PullUpPullDown::PullUp => gpio::FloatingState::PullUp,
            PullUpPullDown::PullDown => gpio::FloatingState::PullDown,
            PullUpPullDown::None => gpio::FloatingState::PullNone,
        }
    }
}

impl gpio::Input for Pin<'_> {
    fn read(&self) -> bool {
        match self.pin {
            PinId::Pin00 => self.registers.idr.is_set(IDR::ID0),
            PinId::Pin01 => self.registers.idr.is_set(IDR::ID1),
            PinId::Pin02 => self.registers.idr.is_set(IDR::ID2),
            PinId::Pin03 => self.registers.idr.is_set(IDR::ID3),
            PinId::Pin04 => self.registers.idr.is_set(IDR::ID4),
            PinId::Pin05 => self.registers.idr.is_set(IDR::ID5),
            PinId::Pin06 => self.registers.idr.is_set(IDR::ID6),
            PinId::Pin07 => self.registers.idr.is_set(IDR::ID7),
            PinId::Pin08 => self.registers.idr.is_set(IDR::ID8),
            PinId::Pin09 => self.registers.idr.is_set(IDR::ID9),
            PinId::Pin10 => self.registers.idr.is_set(IDR::ID10),
            PinId::Pin11 => self.registers.idr.is_set(IDR::ID11),
            PinId::Pin12 => self.registers.idr.is_set(IDR::ID12),
            PinId::Pin13 => self.registers.idr.is_set(IDR::ID13),
            PinId::Pin14 => self.registers.idr.is_set(IDR::ID14),
            PinId::Pin15 => self.registers.idr.is_set(IDR::ID15),
        }
    }
}

impl gpio::Output for Pin<'_> {
    fn set(&self) {
        match self.pin {
            PinId::Pin00 => self.registers.bsrr.write(BSRR::BS0::SET),
            PinId::Pin01 => self.registers.bsrr.write(BSRR::BS1::SET),
            PinId::Pin02 => self.registers.bsrr.write(BSRR::BS2::SET),
            PinId::Pin03 => self.registers.bsrr.write(BSRR::BS3::SET),
            PinId::Pin04 => self.registers.bsrr.write(BSRR::BS4::SET),
            PinId::Pin05 => self.registers.bsrr.write(BSRR::BS5::SET),
            PinId::Pin06 => self.registers.bsrr.write(BSRR::BS6::SET),
            PinId::Pin07 => self.registers.bsrr.write(BSRR::BS7::SET),
            PinId::Pin08 => self.registers.bsrr.write(BSRR::BS8::SET),
            PinId::Pin09 => self.registers.bsrr.write(BSRR::BS9::SET),
            PinId::Pin10 => self.registers.bsrr.write(BSRR::BS10::SET),
            PinId::Pin11 => self.registers.bsrr.write(BSRR::BS11::SET),
            PinId::Pin12 => self.registers.bsrr.write(BSRR::BS12::SET),
            PinId::Pin13 => self.registers.bsrr.write(BSRR::BS13::SET),
            PinId::Pin14 => self.registers.bsrr.write(BSRR::BS14::SET),
            PinId::Pin15 => self.registers.bsrr.write(BSRR::BS15::SET),
        }
    }

    fn clear(&self) {
        match self.pin {
            PinId::Pin00 => self.registers.bsrr.write(BSRR::BR0::SET),
            PinId::Pin01 => self.registers.bsrr.write(BSRR::BR1::SET),
            PinId::Pin02 => self.registers.bsrr.write(BSRR::BR2::SET),
            PinId::Pin03 => self.registers.bsrr.write(BSRR::BR3::SET),
            PinId::Pin04 => self.registers.bsrr.write(BSRR::BR4::SET),
            PinId::Pin05 => self.registers.bsrr.write(BSRR::BR5::SET),
            PinId::Pin06 => self.registers.bsrr.write(BSRR::BR6::SET),
            PinId::Pin07 => self.registers.bsrr.write(BSRR::BR7::SET),
            PinId::Pin08 => self.registers.bsrr.write(BSRR::BR8::SET),
            PinId::Pin09 => self.registers.bsrr.write(BSRR::BR9::SET),
            PinId::Pin10 => self.registers.bsrr.write(BSRR::BR10::SET),
            PinId::Pin11 => self.registers.bsrr.write(BSRR::BR11::SET),
            PinId::Pin12 => self.registers.bsrr.write(BSRR::BR12::SET),
            PinId::Pin13 => self.registers.bsrr.write(BSRR::BR13::SET),
            PinId::Pin14 => self.registers.bsrr.write(BSRR::BR14::SET),
            PinId::Pin15 => self.registers.bsrr.write(BSRR::BR15::SET),
        }
    }

    fn toggle(&self) -> bool {
        match self.pin {
            PinId::Pin00 => {
                let val = self.registers.odr.is_set(ODR::OD0);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin01 => {
                let val = self.registers.odr.is_set(ODR::OD1);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin02 => {
                let val = self.registers.odr.is_set(ODR::OD2);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin03 => {
                let val = self.registers.odr.is_set(ODR::OD3);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin04 => {
                let val = self.registers.odr.is_set(ODR::OD4);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin05 => {
                let val = self.registers.odr.is_set(ODR::OD5);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin06 => {
                let val = self.registers.odr.is_set(ODR::OD6);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin07 => {
                let val = self.registers.odr.is_set(ODR::OD7);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin08 => {
                let val = self.registers.odr.is_set(ODR::OD8);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin09 => {
                let val = self.registers.odr.is_set(ODR::OD9);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin10 => {
                let val = self.registers.odr.is_set(ODR::OD10);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin11 => {
                let val = self.registers.odr.is_set(ODR::OD11);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin12 => {
                let val = self.registers.odr.is_set(ODR::OD12);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin13 => {
                let val = self.registers.odr.is_set(ODR::OD13);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin14 => {
                let val = self.registers.odr.is_set(ODR::OD14);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
            PinId::Pin15 => {
                let val = self.registers.odr.is_set(ODR::OD15);
                if val {
                    self.clear();
                } else {
                    self.set();
                }
            }
        }
        self.read()
    }
}

impl<'a> gpio::Interrupt<'a> for Pin<'a> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let line = LineId::from(self.pin);
        self.exti_lineid.set(line);

        self.client.map(|client| {
            self.exti.register_client(line, client);
        });

        // 1. Route the port to the line
        self.exti.select_port(line, self.port_id as u32);

        // 2. Configure the EXTI line as Secure.
        // On the STM32U5, the EXTI controller is TrustZone-aware. Since the Tock
        // kernel is running in the Secure state, we must explicitly mark the
        // interrupt line as Secure in the EXTI_SECCFGR1 register. If we omit this,
        // the hardware firewall will block the interrupt signal from reaching
        // the Secure CPU context.
        self.exti.set_secure(line);

        self.exti.mask_interrupt(line);
        self.exti.clear_pending(line);

        match mode {
            gpio::InterruptEdge::EitherEdge => {
                self.exti.select_rising_trigger(line);
                self.exti.select_falling_trigger(line);
            }
            gpio::InterruptEdge::RisingEdge => {
                self.exti.select_rising_trigger(line);
                self.exti.deselect_falling_trigger(line);
            }
            gpio::InterruptEdge::FallingEdge => {
                self.exti.deselect_rising_trigger(line);
                self.exti.select_falling_trigger(line);
            }
        }
        self.exti.unmask_interrupt(line);
    }

    fn disable_interrupts(&self) {
        self.exti_lineid.map(|line| {
            self.exti.mask_interrupt(line);
            self.exti.clear_pending(line);
        });
    }

    fn is_pending(&self) -> bool {
        self.exti_lineid
            .map_or(false, |line| self.exti.is_pending(line))
    }
}

impl From<PinId> for LineId {
    fn from(pin: PinId) -> Self {
        match pin {
            PinId::Pin00 => LineId::Line00,
            PinId::Pin01 => LineId::Line01,
            PinId::Pin02 => LineId::Line02,
            PinId::Pin03 => LineId::Line03,
            PinId::Pin04 => LineId::Line04,
            PinId::Pin05 => LineId::Line05,
            PinId::Pin06 => LineId::Line06,
            PinId::Pin07 => LineId::Line07,
            PinId::Pin08 => LineId::Line08,
            PinId::Pin09 => LineId::Line09,
            PinId::Pin10 => LineId::Line10,
            PinId::Pin11 => LineId::Line11,
            PinId::Pin12 => LineId::Line12,
            PinId::Pin13 => LineId::Line13,
            PinId::Pin14 => LineId::Line14,
            PinId::Pin15 => LineId::Line15,
        }
    }
}

/// Represents a collection of 16 GPIO pins.
pub struct Port<'a> {
    registers: StaticRef<GpioRegisters>,
    exti: &'a Exti<'a>,
    port_id: GpioPort,
}

impl<'a> Port<'a> {
    /// Creates a new Port instance.
    pub const fn new(
        base: StaticRef<GpioRegisters>,
        exti: &'a Exti<'a>,
        port_id: GpioPort,
    ) -> Self {
        Port {
            registers: base,
            exti,
            port_id,
        }
    }

    /// Returns a Pin instance for a specific physical pin on this port.
    pub fn pin(&self, pin: PinId) -> Pin<'a> {
        Pin::new(self.registers, pin, self.exti, self.port_id)
    }
}
