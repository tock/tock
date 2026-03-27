// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Minimal GPIO support for STM32U5xx for Tock bring-up.

use crate::clocks::{phclk, Stm32u5Clocks};
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

// ---------- Registers ----------

#[repr(C)]
struct GpioRegisters {
    moder: ReadWrite<u32, MODER::Register>,
    otyper: ReadWrite<u32, OTYPER::Register>,
    ospeedr: ReadWrite<u32, OSPEEDR::Register>,
    pupdr: ReadWrite<u32, PUPDR::Register>,
    idr: ReadOnly<u32, IDR::Register>,
    odr: ReadWrite<u32, ODR::Register>,
    bsrr: WriteOnly<u32, BSRR::Register>,
    lckr: ReadWrite<u32, LCKR::Register>,
    afrl: ReadWrite<u32, AFRL::Register>,
    afrh: ReadWrite<u32, AFRH::Register>,
}

register_bitfields![u32,
    MODER [
        MODER15 OFFSET(30) NUMBITS(2) [],
        MODER14 OFFSET(28) NUMBITS(2) [],
        MODER13 OFFSET(26) NUMBITS(2) [],
        MODER12 OFFSET(24) NUMBITS(2) [],
        MODER11 OFFSET(22) NUMBITS(2) [],
        MODER10 OFFSET(20) NUMBITS(2) [],
        MODER9  OFFSET(18) NUMBITS(2) [],
        MODER8  OFFSET(16) NUMBITS(2) [],
        MODER7  OFFSET(14) NUMBITS(2) [],
        MODER6  OFFSET(12) NUMBITS(2) [],
        MODER5  OFFSET(10) NUMBITS(2) [],
        MODER4  OFFSET(8)  NUMBITS(2) [],
        MODER3  OFFSET(6)  NUMBITS(2) [],
        MODER2  OFFSET(4)  NUMBITS(2) [],
        MODER1  OFFSET(2)  NUMBITS(2) [],
        MODER0  OFFSET(0)  NUMBITS(2) [],
    ],
    OTYPER [
        OT15 OFFSET(15) NUMBITS(1) [],
        OT14 OFFSET(14) NUMBITS(1) [],
        OT13 OFFSET(13) NUMBITS(1) [],
        OT12 OFFSET(12) NUMBITS(1) [],
        OT11 OFFSET(11) NUMBITS(1) [],
        OT10 OFFSET(10) NUMBITS(1) [],
        OT9  OFFSET(9)  NUMBITS(1) [],
        OT8  OFFSET(8)  NUMBITS(1) [],
        OT7  OFFSET(7)  NUMBITS(1) [],
        OT6  OFFSET(6)  NUMBITS(1) [],
        OT5  OFFSET(5)  NUMBITS(1) [],
        OT4  OFFSET(4)  NUMBITS(1) [],
        OT3  OFFSET(3)  NUMBITS(1) [],
        OT2  OFFSET(2)  NUMBITS(1) [],
        OT1  OFFSET(1)  NUMBITS(1) [],
        OT0  OFFSET(0)  NUMBITS(1) [],
    ],
    OSPEEDR [
        OSPEEDR15 OFFSET(30) NUMBITS(2) [],
        OSPEEDR14 OFFSET(28) NUMBITS(2) [],
        OSPEEDR13 OFFSET(26) NUMBITS(2) [],
        OSPEEDR12 OFFSET(24) NUMBITS(2) [],
        OSPEEDR11 OFFSET(22) NUMBITS(2) [],
        OSPEEDR10 OFFSET(20) NUMBITS(2) [],
        OSPEEDR9  OFFSET(18) NUMBITS(2) [],
        OSPEEDR8  OFFSET(16) NUMBITS(2) [],
        OSPEEDR7  OFFSET(14) NUMBITS(2) [],
        OSPEEDR6  OFFSET(12) NUMBITS(2) [],
        OSPEEDR5  OFFSET(10) NUMBITS(2) [],
        OSPEEDR4  OFFSET(8)  NUMBITS(2) [],
        OSPEEDR3  OFFSET(6)  NUMBITS(2) [],
        OSPEEDR2  OFFSET(4)  NUMBITS(2) [],
        OSPEEDR1  OFFSET(2)  NUMBITS(2) [],
        OSPEEDR0  OFFSET(0)  NUMBITS(2) [],
    ],
    PUPDR [
        PUPDR15 OFFSET(30) NUMBITS(2) [],
        PUPDR14 OFFSET(28) NUMBITS(2) [],
        PUPDR13 OFFSET(26) NUMBITS(2) [],
        PUPDR12 OFFSET(24) NUMBITS(2) [],
        PUPDR11 OFFSET(22) NUMBITS(2) [],
        PUPDR10 OFFSET(20) NUMBITS(2) [],
        PUPDR9  OFFSET(18) NUMBITS(2) [],
        PUPDR8  OFFSET(16) NUMBITS(2) [],
        PUPDR7  OFFSET(14) NUMBITS(2) [],
        PUPDR6  OFFSET(12) NUMBITS(2) [],
        PUPDR5  OFFSET(10) NUMBITS(2) [],
        PUPDR4  OFFSET(8)  NUMBITS(2) [],
        PUPDR3  OFFSET(6)  NUMBITS(2) [],
        PUPDR2  OFFSET(4)  NUMBITS(2) [],
        PUPDR1  OFFSET(2)  NUMBITS(2) [],
        PUPDR0  OFFSET(0)  NUMBITS(2) [],
    ],
    IDR [
        IDR15 OFFSET(15) NUMBITS(1) [],
        IDR14 OFFSET(14) NUMBITS(1) [],
        IDR13 OFFSET(13) NUMBITS(1) [],
        IDR12 OFFSET(12) NUMBITS(1) [],
        IDR11 OFFSET(11) NUMBITS(1) [],
        IDR10 OFFSET(10) NUMBITS(1) [],
        IDR9  OFFSET(9)  NUMBITS(1) [],
        IDR8  OFFSET(8)  NUMBITS(1) [],
        IDR7  OFFSET(7)  NUMBITS(1) [],
        IDR6  OFFSET(6)  NUMBITS(1) [],
        IDR5  OFFSET(5)  NUMBITS(1) [],
        IDR4  OFFSET(4)  NUMBITS(1) [],
        IDR3  OFFSET(3)  NUMBITS(1) [],
        IDR2  OFFSET(2)  NUMBITS(1) [],
        IDR1  OFFSET(1)  NUMBITS(1) [],
        IDR0  OFFSET(0)  NUMBITS(1) [],
    ],
    ODR [
        ODR15 OFFSET(15) NUMBITS(1) [],
        ODR14 OFFSET(14) NUMBITS(1) [],
        ODR13 OFFSET(13) NUMBITS(1) [],
        ODR12 OFFSET(12) NUMBITS(1) [],
        ODR11 OFFSET(11) NUMBITS(1) [],
        ODR10 OFFSET(10) NUMBITS(1) [],
        ODR9  OFFSET(9)  NUMBITS(1) [],
        ODR8  OFFSET(8)  NUMBITS(1) [],
        ODR7  OFFSET(7)  NUMBITS(1) [],
        ODR6  OFFSET(6)  NUMBITS(1) [],
        ODR5  OFFSET(5)  NUMBITS(1) [],
        ODR4  OFFSET(4)  NUMBITS(1) [],
        ODR3  OFFSET(3)  NUMBITS(1) [],
        ODR2  OFFSET(2)  NUMBITS(1) [],
        ODR1  OFFSET(1)  NUMBITS(1) [],
        ODR0  OFFSET(0)  NUMBITS(1) [],
    ],
    BSRR [
        BR15 OFFSET(31) NUMBITS(1) [],
        BR14 OFFSET(30) NUMBITS(1) [],
        BR13 OFFSET(29) NUMBITS(1) [],
        BR12 OFFSET(28) NUMBITS(1) [],
        BR11 OFFSET(27) NUMBITS(1) [],
        BR10 OFFSET(26) NUMBITS(1) [],
        BR9  OFFSET(25) NUMBITS(1) [],
        BR8  OFFSET(24) NUMBITS(1) [],
        BR7  OFFSET(23) NUMBITS(1) [],
        BR6  OFFSET(22) NUMBITS(1) [],
        BR5  OFFSET(21) NUMBITS(1) [],
        BR4  OFFSET(20) NUMBITS(1) [],
        BR3  OFFSET(19) NUMBITS(1) [],
        BR2  OFFSET(18) NUMBITS(1) [],
        BR1  OFFSET(17) NUMBITS(1) [],
        BR0 OFFSET(16) NUMBITS(1) [],
        BS15 OFFSET(15) NUMBITS(1) [],
        BS14 OFFSET(14) NUMBITS(1) [],
        BS13 OFFSET(13) NUMBITS(1) [],
        BS12 OFFSET(12) NUMBITS(1) [],
        BS11 OFFSET(11) NUMBITS(1) [],
        BS10 OFFSET(10) NUMBITS(1) [],
        BS9  OFFSET(9)  NUMBITS(1) [],
        BS8  OFFSET(8)  NUMBITS(1) [],
        BS7  OFFSET(7)  NUMBITS(1) [],
        BS6  OFFSET(6)  NUMBITS(1) [],
        BS5  OFFSET(5)  NUMBITS(1) [],
        BS4  OFFSET(4)  NUMBITS(1) [],
        BS3  OFFSET(3)  NUMBITS(1) [],
        BS2  OFFSET(2)  NUMBITS(1) [],
        BS1  OFFSET(1) NUMBITS(1) [],
        BS0 OFFSET(0)  NUMBITS(1) [],
    ],
    LCKR [
        LCKK  OFFSET(16) NUMBITS(1) [],
        LCK15 OFFSET(15) NUMBITS(1) [],
        LCK14 OFFSET(14) NUMBITS(1) [],
        LCK13 OFFSET(13) NUMBITS(1) [],
        LCK12 OFFSET(12) NUMBITS(1) [],
        LCK11 OFFSET(11) NUMBITS(1) [],
        LCK10 OFFSET(10) NUMBITS(1) [],
        LCK9  OFFSET(9)  NUMBITS(1) [],
        LCK8  OFFSET(8)  NUMBITS(1) [],
        LCK7  OFFSET(7)  NUMBITS(1) [],
        LCK6  OFFSET(6)  NUMBITS(1) [],
        LCK5  OFFSET(5)  NUMBITS(1) [],
        LCK4  OFFSET(4)  NUMBITS(1) [],
        LCK3  OFFSET(3)  NUMBITS(1) [],
        LCK2  OFFSET(2)  NUMBITS(1) [],
        LCK1  OFFSET(1)  NUMBITS(1) [],
        LCK0  OFFSET(0)  NUMBITS(1) [],
    ],
    AFRL [
        AFRL7 OFFSET(28) NUMBITS(4) [],
        AFRL6 OFFSET(24) NUMBITS(4) [],
        AFRL5 OFFSET(20) NUMBITS(4) [],
        AFRL4 OFFSET(16) NUMBITS(4) [],
        AFRL3 OFFSET(12) NUMBITS(4) [],
        AFRL2 OFFSET(8)  NUMBITS(4) [],
        AFRL1 OFFSET(4)  NUMBITS(4) [],
        AFRL0 OFFSET(0)  NUMBITS(4) [],
    ],
    AFRH [
        AFRH15 OFFSET(28) NUMBITS(4) [],
        AFRH14 OFFSET(24) NUMBITS(4) [],
        AFRH13 OFFSET(20) NUMBITS(4) [],
        AFRH12 OFFSET(16) NUMBITS(4) [],
        AFRH11 OFFSET(12) NUMBITS(4) [],
        AFRH10 OFFSET(8)  NUMBITS(4) [],
        AFRH9  OFFSET(4)  NUMBITS(4) [],
        AFRH8  OFFSET(0)  NUMBITS(4) [],
    ]
];

// Non-secure GPIO base addresses (RM0456)
const GPIOA_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x42020000 as *const GpioRegisters) };
const GPIOB_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x42020400 as *const GpioRegisters) };
const GPIOC_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x42020800 as *const GpioRegisters) };

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PortId {
    A = 0b000,
    B = 0b001,
    C = 0b010,
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum PinId {
    PA00 = 0b0000000,
    PA01 = 0b0000001,
    PA02 = 0b0000010,
    PA03 = 0b0000011,
    PA04 = 0b0000100,
    PA05 = 0b0000101,
    PA06 = 0b0000110,
    PA07 = 0b0000111,
    PA08 = 0b0001000,
    PA09 = 0b0001001,
    PA10 = 0b0001010,
    PA11 = 0b0001011,
    PA12 = 0b0001100,
    PA13 = 0b0001101,
    PA14 = 0b0001110,
    PA15 = 0b0001111,
    PB00 = 0b0010000,
    PB01 = 0b0010001,
    PB02 = 0b0010010,
    PB03 = 0b0010011,
    PB04 = 0b0010100,
    PB05 = 0b0010101,
    PB06 = 0b0010110,
    PB07 = 0b0010111,
    PB08 = 0b0011000,
    PB09 = 0b0011001,
    PB10 = 0b0011010,
    PB11 = 0b0011011,
    PB12 = 0b0011100,
    PB13 = 0b0011101,
    PB14 = 0b0011110,
    PB15 = 0b0011111,
    PC00 = 0b0100000,
    PC01 = 0b0100001,
    PC02 = 0b0100010,
    PC03 = 0b0100011,
    PC04 = 0b0100100,
    PC05 = 0b0100101,
    PC06 = 0b0100110,
    PC07 = 0b0100111,
    PC08 = 0b0101000,
    PC09 = 0b0101001,
    PC10 = 0b0101010,
    PC11 = 0b0101011,
    PC12 = 0b0101100,
    PC13 = 0b0101101,
    PC14 = 0b0101110,
    PC15 = 0b0101111,
}

impl<'a> GpioPorts<'a> {
    pub fn get_pin(&self, pin_id: PinId) -> Option<&Pin<'a>> {
        let mut port_num: u8 = pin_id as u8;

        port_num >>= 4; // Upper 3 bits are port number
        let mut pin_num: u8 = pin_id as u8;
        pin_num &= 0b0000_1111; // Lower 4 bits are pin number

        self.pins[usize::from(port_num)][usize::from(pin_num)].as_ref()
    }

    pub fn get_port(&self, pin_id: PinId) -> &Port {
        let mut port_num: u8 = pin_id as u8;

        port_num >>= 4; // Upper 3 bits are port number

        &self.ports[usize::from(port_num)]
    }

    pub fn get_port_from_port_id(&self, port_id: PortId) -> &Port {
        &self.ports[usize::from(port_id as u8)]
    }
}

impl PinId {
    pub fn get_pin_number(&self) -> u8 {
        let mut pin_num = *self as u8;
        pin_num &= 0b0000_1111; // Lower 4 bits are
        pin_num
    }

    /// Return true if this pin index belongs to the low AFRL register.
    pub fn is_low_bank(&self) -> bool {
        self.get_pin_number() < 8
    }

    pub fn get_port_number(&self) -> u8 {
        let mut port_num = *self as u8;
        port_num >>= 4; // Upper 3 bits are port number
        port_num
    }
}

enum_from_primitive! {
    #[repr(u32)]
    #[derive(PartialEq)]

    pub enum Mode {
        Input = 0b00,
        Output = 0b01,
        AlternateFunction = 0b10,
        Analog = 0b11,
    }
}

#[repr(u32)]
pub enum AlternateFunction {
    AF0 = 0b0000,
    AF1 = 0b0001,
    AF2 = 0b0010,
    AF3 = 0b0011,
    AF4 = 0b0100,
    AF5 = 0b0101,
    AF6 = 0b0110,
    AF7 = 0b0111,
    AF8 = 0b1000,
    AF9 = 0b1001,
    AF10 = 0b1010,
    AF11 = 0b1011,
    AF12 = 0b1100,
    AF13 = 0b1101,
    AF14 = 0b1110,
    AF15 = 0b1111,
}

enum_from_primitive! {
    #[repr(u32)]

    enum PullUpPullDown {
        NoPullUpPullDown = 0b00,
        PullUp = 0b01,
        PullDown = 0b10
    }
}

pub struct Port<'a> {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock<'a>,
}

macro_rules! declare_gpio_pins {
    ($($pin:ident)*) => {
        [
            $(Some(Pin::new(PinId::$pin)), )*
        ]
    }
}
pub struct GpioPorts<'a> {
    pub ports: [Port<'a>; 3],
    pub pins: [[Option<Pin<'a>>; 16]; 3],
}

impl<'a> GpioPorts<'a> {
    pub fn new(clocks: &'a dyn Stm32u5Clocks) -> Self {
        Self {
            ports: [
                Port {
                    registers: GPIOA_BASE,
                    clock: PortClock(phclk::PeripheralClock::new(
                        phclk::PeripheralClockType::AHB1(phclk::HCLK1::GPIOA),
                        clocks,
                    )),
                },
                Port {
                    registers: GPIOB_BASE,
                    clock: PortClock(phclk::PeripheralClock::new(
                        phclk::PeripheralClockType::AHB1(phclk::HCLK1::GPIOB),
                        clocks,
                    )),
                },
                Port {
                    registers: GPIOC_BASE,
                    clock: PortClock(phclk::PeripheralClock::new(
                        phclk::PeripheralClockType::AHB1(phclk::HCLK1::GPIOC),
                        clocks,
                    )),
                },
            ],
            pins: [
                declare_gpio_pins! {
                    PA00 PA01 PA02 PA03 PA04 PA05 PA06 PA07
                    PA08 PA09 PA10 PA11 PA12 PA13 PA14 PA15
                },
                declare_gpio_pins! {
                    PB00 PB01 PB02 PB03 PB04 PB05 PB06 PB07
                    PB08 PB09 PB10 PB11 PB12 PB13 PB14 PB15
                },
                declare_gpio_pins! {
                    PC00 PC01 PC02 PC03 PC04 PC05 PC06 PC07
                    PC08 PC09 PC10 PC11 PC12 PC13 PC14 PC15
                },
            ],
        }
    }

    pub fn setup_circular_deps(&'a self) {
        for pin_group in self.pins.iter() {
            for pin in pin_group {
                pin.as_ref().map(|p| p.set_ports_ref(self));
            }
        }
    }
}

impl Port<'_> {
    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }
}

struct PortClock<'a>(crate::clocks::phclk::PeripheralClock<'a>);

impl ClockInterface for PortClock<'_> {
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

pub struct Pin<'a> {
    pin_id: PinId,
    ports_ref: OptionalCell<&'a GpioPorts<'a>>,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
}

impl<'a> Pin<'a> {
    pub const fn new(pin_id: PinId) -> Self {
        Self {
            pin_id,
            ports_ref: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }
    pub fn set_ports_ref(&self, ports: &'a GpioPorts<'a>) {
        self.ports_ref.set(ports);
    }

    pub fn get_mode(&self) -> Mode {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id); // Unwrap fail =

        let val = match self.pin_id.get_pin_number() {
            0b0000 => port.registers.moder.read(MODER::MODER0),
            0b0001 => port.registers.moder.read(MODER::MODER1),
            0b0010 => port.registers.moder.read(MODER::MODER2),
            0b0011 => port.registers.moder.read(MODER::MODER3),
            0b0100 => port.registers.moder.read(MODER::MODER4),
            0b0101 => port.registers.moder.read(MODER::MODER5),
            0b0110 => port.registers.moder.read(MODER::MODER6),
            0b0111 => port.registers.moder.read(MODER::MODER7),
            0b1000 => port.registers.moder.read(MODER::MODER8),
            0b1001 => port.registers.moder.read(MODER::MODER9),
            0b1010 => port.registers.moder.read(MODER::MODER10),
            0b1011 => port.registers.moder.read(MODER::MODER11),
            0b1100 => port.registers.moder.read(MODER::MODER12),
            0b1101 => port.registers.moder.read(MODER::MODER13),
            0b1110 => port.registers.moder.read(MODER::MODER14),
            0b1111 => port.registers.moder.read(MODER::MODER15),
            _ => 0,
        };
        Mode::from_u32(val).unwrap_or(Mode::Input)
    }

    pub fn set_mode(&self, mode: Mode) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id); // Unwrap fail =

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.moder.modify(MODER::MODER0.val(mode as u32)),
            0b0001 => port.registers.moder.modify(MODER::MODER1.val(mode as u32)),
            0b0010 => port.registers.moder.modify(MODER::MODER2.val(mode as u32)),
            0b0011 => port.registers.moder.modify(MODER::MODER3.val(mode as u32)),
            0b0100 => port.registers.moder.modify(MODER::MODER4.val(mode as u32)),
            0b0101 => port.registers.moder.modify(MODER::MODER5.val(mode as u32)),
            0b0110 => port.registers.moder.modify(MODER::MODER6.val(mode as u32)),
            0b0111 => port.registers.moder.modify(MODER::MODER7.val(mode as u32)),
            0b1000 => port.registers.moder.modify(MODER::MODER8.val(mode as u32)),
            0b1001 => port.registers.moder.modify(MODER::MODER9.val(mode as u32)),
            0b1010 => port.registers.moder.modify(MODER::MODER10.val(mode as u32)),
            0b1011 => port.registers.moder.modify(MODER::MODER11.val(mode as u32)),
            0b1100 => port.registers.moder.modify(MODER::MODER12.val(mode as u32)),
            0b1101 => port.registers.moder.modify(MODER::MODER13.val(mode as u32)),
            0b1110 => port.registers.moder.modify(MODER::MODER14.val(mode as u32)),
            0b1111 => port.registers.moder.modify(MODER::MODER15.val(mode as u32)),
            _ => {}
        }
    }

    /// Configure this pin for an alternate function and set the AFR nibble.
    pub fn set_alternate_function(&self, af: AlternateFunction) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);
        let pin_num = self.pin_id.get_pin_number();

        // Select alternate-function mode.
        self.set_mode(Mode::AlternateFunction);

        if pin_num < 8 {
            match pin_num {
                0 => port.registers.afrl.modify(AFRL::AFRL0.val(af as u32)),
                1 => port.registers.afrl.modify(AFRL::AFRL1.val(af as u32)),
                2 => port.registers.afrl.modify(AFRL::AFRL2.val(af as u32)),
                3 => port.registers.afrl.modify(AFRL::AFRL3.val(af as u32)),
                4 => port.registers.afrl.modify(AFRL::AFRL4.val(af as u32)),
                5 => port.registers.afrl.modify(AFRL::AFRL5.val(af as u32)),
                6 => port.registers.afrl.modify(AFRL::AFRL6.val(af as u32)),
                7 => port.registers.afrl.modify(AFRL::AFRL7.val(af as u32)),
                _ => {}
            }
        } else {
            match pin_num {
                8 => port.registers.afrh.modify(AFRH::AFRH8.val(af as u32)),
                9 => port.registers.afrh.modify(AFRH::AFRH9.val(af as u32)),
                10 => port.registers.afrh.modify(AFRH::AFRH10.val(af as u32)),
                11 => port.registers.afrh.modify(AFRH::AFRH11.val(af as u32)),
                12 => port.registers.afrh.modify(AFRH::AFRH12.val(af as u32)),
                13 => port.registers.afrh.modify(AFRH::AFRH13.val(af as u32)),
                14 => port.registers.afrh.modify(AFRH::AFRH14.val(af as u32)),
                15 => port.registers.afrh.modify(AFRH::AFRH15.val(af as u32)),
                _ => {}
            }
        }
    }

    pub fn get_pinid(&self) -> PinId {
        self.pin_id
    }

    fn set_mode_output_pushpull(&self) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id); // Unwrap fail =

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.otyper.modify(OTYPER::OT0::CLEAR),
            0b0001 => port.registers.otyper.modify(OTYPER::OT1::CLEAR),
            0b0010 => port.registers.otyper.modify(OTYPER::OT2::CLEAR),
            0b0011 => port.registers.otyper.modify(OTYPER::OT3::CLEAR),
            0b0100 => port.registers.otyper.modify(OTYPER::OT4::CLEAR),
            0b0101 => port.registers.otyper.modify(OTYPER::OT5::CLEAR),
            0b0110 => port.registers.otyper.modify(OTYPER::OT6::CLEAR),
            0b0111 => port.registers.otyper.modify(OTYPER::OT7::CLEAR),
            0b1000 => port.registers.otyper.modify(OTYPER::OT8::CLEAR),
            0b1001 => port.registers.otyper.modify(OTYPER::OT9::CLEAR),
            0b1010 => port.registers.otyper.modify(OTYPER::OT10::CLEAR),
            0b1011 => port.registers.otyper.modify(OTYPER::OT11::CLEAR),
            0b1100 => port.registers.otyper.modify(OTYPER::OT12::CLEAR),
            0b1101 => port.registers.otyper.modify(OTYPER::OT13::CLEAR),
            0b1110 => port.registers.otyper.modify(OTYPER::OT14::CLEAR),
            0b1111 => port.registers.otyper.modify(OTYPER::OT15::CLEAR),
            _ => {}
        }
    }

    pub fn set_speed(&self) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id); // Unwrap fail =

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR0.val(0b11)),
            0b0001 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR1.val(0b11)),
            0b0010 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR2.val(0b11)),
            0b0011 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR3.val(0b11)),
            0b0100 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR4.val(0b11)),
            0b0101 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR5.val(0b11)),
            0b0110 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR6.val(0b11)),
            0b0111 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR7.val(0b11)),
            0b1000 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR8.val(0b11)),
            0b1001 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR9.val(0b11)),
            0b1010 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR10.val(0b11)),
            0b1011 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR11.val(0b11)),
            0b1100 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR12.val(0b11)),
            0b1101 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR13.val(0b11)),
            0b1110 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR14.val(0b11)),
            0b1111 => port.registers.ospeedr.modify(OSPEEDR::OSPEEDR15.val(0b11)),
            _ => {}
        }
    }

    pub fn set_mode_output_opendrain(&self) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.otyper.modify(OTYPER::OT0::SET),
            0b0001 => port.registers.otyper.modify(OTYPER::OT1::SET),
            0b0010 => port.registers.otyper.modify(OTYPER::OT2::SET),
            0b0011 => port.registers.otyper.modify(OTYPER::OT3::SET),
            0b0100 => port.registers.otyper.modify(OTYPER::OT4::SET),
            0b0101 => port.registers.otyper.modify(OTYPER::OT5::SET),
            0b0110 => port.registers.otyper.modify(OTYPER::OT6::SET),
            0b0111 => port.registers.otyper.modify(OTYPER::OT7::SET),
            0b1000 => port.registers.otyper.modify(OTYPER::OT8::SET),
            0b1001 => port.registers.otyper.modify(OTYPER::OT9::SET),
            0b1010 => port.registers.otyper.modify(OTYPER::OT10::SET),
            0b1011 => port.registers.otyper.modify(OTYPER::OT11::SET),
            0b1100 => port.registers.otyper.modify(OTYPER::OT12::SET),
            0b1101 => port.registers.otyper.modify(OTYPER::OT13::SET),
            0b1110 => port.registers.otyper.modify(OTYPER::OT14::SET),
            0b1111 => port.registers.otyper.modify(OTYPER::OT15::SET),
            _ => {}
        }
    }

    fn get_pullup_pulldown(&self) -> PullUpPullDown {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        let val = match self.pin_id.get_pin_number() {
            0b0000 => port.registers.pupdr.read(PUPDR::PUPDR0),
            0b0001 => port.registers.pupdr.read(PUPDR::PUPDR1),
            0b0010 => port.registers.pupdr.read(PUPDR::PUPDR2),
            0b0011 => port.registers.pupdr.read(PUPDR::PUPDR3),
            0b0100 => port.registers.pupdr.read(PUPDR::PUPDR4),
            0b0101 => port.registers.pupdr.read(PUPDR::PUPDR5),
            0b0110 => port.registers.pupdr.read(PUPDR::PUPDR6),
            0b0111 => port.registers.pupdr.read(PUPDR::PUPDR7),
            0b1000 => port.registers.pupdr.read(PUPDR::PUPDR8),
            0b1001 => port.registers.pupdr.read(PUPDR::PUPDR9),
            0b1010 => port.registers.pupdr.read(PUPDR::PUPDR10),
            0b1011 => port.registers.pupdr.read(PUPDR::PUPDR11),
            0b1100 => port.registers.pupdr.read(PUPDR::PUPDR12),
            0b1101 => port.registers.pupdr.read(PUPDR::PUPDR13),
            0b1110 => port.registers.pupdr.read(PUPDR::PUPDR14),
            0b1111 => port.registers.pupdr.read(PUPDR::PUPDR15),
            _ => 0,
        };

        PullUpPullDown::from_u32(val).unwrap_or(PullUpPullDown::NoPullUpPullDown)
    }

    fn set_pullup_pulldown(&self, pupd: PullUpPullDown) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.pupdr.modify(PUPDR::PUPDR0.val(pupd as u32)),
            0b0001 => port.registers.pupdr.modify(PUPDR::PUPDR1.val(pupd as u32)),
            0b0010 => port.registers.pupdr.modify(PUPDR::PUPDR2.val(pupd as u32)),
            0b0011 => port.registers.pupdr.modify(PUPDR::PUPDR3.val(pupd as u32)),
            0b0100 => port.registers.pupdr.modify(PUPDR::PUPDR4.val(pupd as u32)),
            0b0101 => port.registers.pupdr.modify(PUPDR::PUPDR5.val(pupd as u32)),
            0b0110 => port.registers.pupdr.modify(PUPDR::PUPDR6.val(pupd as u32)),
            0b0111 => port.registers.pupdr.modify(PUPDR::PUPDR7.val(pupd as u32)),
            0b1000 => port.registers.pupdr.modify(PUPDR::PUPDR8.val(pupd as u32)),
            0b1001 => port.registers.pupdr.modify(PUPDR::PUPDR9.val(pupd as u32)),
            0b1010 => port.registers.pupdr.modify(PUPDR::PUPDR10.val(pupd as u32)),
            0b1011 => port.registers.pupdr.modify(PUPDR::PUPDR11.val(pupd as u32)),
            0b1100 => port.registers.pupdr.modify(PUPDR::PUPDR12.val(pupd as u32)),
            0b1101 => port.registers.pupdr.modify(PUPDR::PUPDR13.val(pupd as u32)),
            0b1110 => port.registers.pupdr.modify(PUPDR::PUPDR14.val(pupd as u32)),
            0b1111 => port.registers.pupdr.modify(PUPDR::PUPDR15.val(pupd as u32)),
            _ => {}
        }
    }

    fn set_output_high(&self) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.bsrr.write(BSRR::BS0::SET),
            0b0001 => port.registers.bsrr.write(BSRR::BS1::SET),
            0b0010 => port.registers.bsrr.write(BSRR::BS2::SET),
            0b0011 => port.registers.bsrr.write(BSRR::BS3::SET),
            0b0100 => port.registers.bsrr.write(BSRR::BS4::SET),
            0b0101 => port.registers.bsrr.write(BSRR::BS5::SET),
            0b0110 => port.registers.bsrr.write(BSRR::BS6::SET),
            0b0111 => port.registers.bsrr.write(BSRR::BS7::SET),
            0b1000 => port.registers.bsrr.write(BSRR::BS8::SET),
            0b1001 => port.registers.bsrr.write(BSRR::BS9::SET),
            0b1010 => port.registers.bsrr.write(BSRR::BS10::SET),
            0b1011 => port.registers.bsrr.write(BSRR::BS11::SET),
            0b1100 => port.registers.bsrr.write(BSRR::BS12::SET),
            0b1101 => port.registers.bsrr.write(BSRR::BS13::SET),
            0b1110 => port.registers.bsrr.write(BSRR::BS14::SET),
            0b1111 => port.registers.bsrr.write(BSRR::BS15::SET),
            _ => {}
        }
    }

    fn set_output_low(&self) {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.bsrr.write(BSRR::BR0::SET),
            0b0001 => port.registers.bsrr.write(BSRR::BR1::SET),
            0b0010 => port.registers.bsrr.write(BSRR::BR2::SET),
            0b0011 => port.registers.bsrr.write(BSRR::BR3::SET),
            0b0100 => port.registers.bsrr.write(BSRR::BR4::SET),
            0b0101 => port.registers.bsrr.write(BSRR::BR5::SET),
            0b0110 => port.registers.bsrr.write(BSRR::BR6::SET),
            0b0111 => port.registers.bsrr.write(BSRR::BR7::SET),
            0b1000 => port.registers.bsrr.write(BSRR::BR8::SET),
            0b1001 => port.registers.bsrr.write(BSRR::BR9::SET),
            0b1010 => port.registers.bsrr.write(BSRR::BR10::SET),
            0b1011 => port.registers.bsrr.write(BSRR::BR11::SET),
            0b1100 => port.registers.bsrr.write(BSRR::BR12::SET),
            0b1101 => port.registers.bsrr.write(BSRR::BR13::SET),
            0b1110 => port.registers.bsrr.write(BSRR::BR14::SET),
            0b1111 => port.registers.bsrr.write(BSRR::BR15::SET),
            _ => {}
        }
    }

    fn is_output_high(&self) -> bool {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.odr.is_set(ODR::ODR0),
            0b0001 => port.registers.odr.is_set(ODR::ODR1),
            0b0010 => port.registers.odr.is_set(ODR::ODR2),
            0b0011 => port.registers.odr.is_set(ODR::ODR3),
            0b0100 => port.registers.odr.is_set(ODR::ODR4),
            0b0101 => port.registers.odr.is_set(ODR::ODR5),
            0b0110 => port.registers.odr.is_set(ODR::ODR6),
            0b0111 => port.registers.odr.is_set(ODR::ODR7),
            0b1000 => port.registers.odr.is_set(ODR::ODR8),
            0b1001 => port.registers.odr.is_set(ODR::ODR9),
            0b1010 => port.registers.odr.is_set(ODR::ODR10),
            0b1011 => port.registers.odr.is_set(ODR::ODR11),
            0b1100 => port.registers.odr.is_set(ODR::ODR12),
            0b1101 => port.registers.odr.is_set(ODR::ODR13),
            0b1110 => port.registers.odr.is_set(ODR::ODR14),
            0b1111 => port.registers.odr.is_set(ODR::ODR15),
            _ => false,
        }
    }

    fn toggle_output(&self) -> bool {
        if self.is_output_high() {
            self.set_output_low();
            false
        } else {
            self.set_output_high();
            true
        }
    }

    fn read_input(&self) -> bool {
        let port = self.ports_ref.unwrap_or_panic().get_port(self.pin_id);

        match self.pin_id.get_pin_number() {
            0b0000 => port.registers.idr.is_set(IDR::IDR0),
            0b0001 => port.registers.idr.is_set(IDR::IDR1),
            0b0010 => port.registers.idr.is_set(IDR::IDR2),
            0b0011 => port.registers.idr.is_set(IDR::IDR3),
            0b0100 => port.registers.idr.is_set(IDR::IDR4),
            0b0101 => port.registers.idr.is_set(IDR::IDR5),
            0b0110 => port.registers.idr.is_set(IDR::IDR6),
            0b0111 => port.registers.idr.is_set(IDR::IDR7),
            0b1000 => port.registers.idr.is_set(IDR::IDR8),
            0b1001 => port.registers.idr.is_set(IDR::IDR9),
            0b1010 => port.registers.idr.is_set(IDR::IDR10),
            0b1011 => port.registers.idr.is_set(IDR::IDR11),
            0b1100 => port.registers.idr.is_set(IDR::IDR12),
            0b1101 => port.registers.idr.is_set(IDR::IDR13),
            0b1110 => port.registers.idr.is_set(IDR::IDR14),
            0b1111 => port.registers.idr.is_set(IDR::IDR15),
            _ => false,
        }
    }
}

impl hil::gpio::Configure for Pin<'_> {
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Output);
        self.set_mode_output_pushpull();
        hil::gpio::Configuration::Output
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Input);
        hil::gpio::Configuration::Input
    }

    fn deactivate_to_low_power(&self) {
        self.set_mode(Mode::Analog);
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Analog);
        hil::gpio::Configuration::LowPower
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Analog);
        hil::gpio::Configuration::LowPower
    }

    fn set_floating_state(&self, mode: hil::gpio::FloatingState) {
        match mode {
            hil::gpio::FloatingState::PullUp => self.set_pullup_pulldown(PullUpPullDown::PullUp),
            hil::gpio::FloatingState::PullDown => {
                self.set_pullup_pulldown(PullUpPullDown::PullDown)
            }
            hil::gpio::FloatingState::PullNone => {
                self.set_pullup_pulldown(PullUpPullDown::NoPullUpPullDown)
            }
        }
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        match self.get_pullup_pulldown() {
            PullUpPullDown::PullUp => hil::gpio::FloatingState::PullUp,
            PullUpPullDown::PullDown => hil::gpio::FloatingState::PullDown,
            PullUpPullDown::NoPullUpPullDown => hil::gpio::FloatingState::PullNone,
        }
    }

    fn configuration(&self) -> hil::gpio::Configuration {
        match self.get_mode() {
            Mode::Input => hil::gpio::Configuration::Input,
            Mode::Output => hil::gpio::Configuration::Output,
            Mode::Analog => hil::gpio::Configuration::LowPower,
            Mode::AlternateFunction => hil::gpio::Configuration::Function,
        }
    }

    fn is_input(&self) -> bool {
        self.get_mode() == Mode::Input
    }

    fn is_output(&self) -> bool {
        self.get_mode() == Mode::Output
    }
}

impl hil::gpio::Output for Pin<'_> {
    fn set(&self) {
        self.set_output_high();
    }

    fn clear(&self) {
        self.set_output_low();
    }

    fn toggle(&self) -> bool {
        self.toggle_output()
    }
}

impl hil::gpio::Input for Pin<'_> {
    fn read(&self) -> bool {
        self.read_input()
    }
}

impl<'a> hil::gpio::Interrupt<'a> for Pin<'a> {
    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, _mode: hil::gpio::InterruptEdge) {
        // EXTI is not yet wired up for this minimal bring-up, so interrupts are unsupported.
    }

    fn disable_interrupts(&self) {}

    fn is_pending(&self) -> bool {
        false
    }
}
