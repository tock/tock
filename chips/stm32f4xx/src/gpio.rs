use cortexm4;
use cortexm4::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::exti;
use crate::rcc;

/// General-purpose I/Os
#[repr(C)]
struct GpioRegisters {
    /// GPIO port mode register
    moder: ReadWrite<u32, MODER::Register>,
    /// GPIO port output type register
    otyper: ReadWrite<u32, OTYPER::Register>,
    /// GPIO port output speed register
    ospeedr: ReadWrite<u32, OSPEEDR::Register>,
    /// GPIO port pull-up/pull-down register
    pupdr: ReadWrite<u32, PUPDR::Register>,
    /// GPIO port input data register
    idr: ReadOnly<u32, IDR::Register>,
    /// GPIO port output data register
    odr: ReadWrite<u32, ODR::Register>,
    /// GPIO port bit set/reset register
    bsrr: WriteOnly<u32, BSRR::Register>,
    /// GPIO port configuration lock register
    lckr: ReadWrite<u32, LCKR::Register>,
    /// GPIO alternate function low register
    afrl: ReadWrite<u32, AFRL::Register>,
    /// GPIO alternate function high register
    afrh: ReadWrite<u32, AFRH::Register>,
}

register_bitfields![u32,
    MODER [
        /// Port x configuration bits (y = 0..15)
        MODER15 OFFSET(30) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER14 OFFSET(28) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER13 OFFSET(26) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER12 OFFSET(24) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER11 OFFSET(22) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER10 OFFSET(20) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER9 OFFSET(18) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER8 OFFSET(16) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER7 OFFSET(14) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER6 OFFSET(12) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER5 OFFSET(10) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER4 OFFSET(8) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER3 OFFSET(6) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER2 OFFSET(4) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER1 OFFSET(2) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        MODER0 OFFSET(0) NUMBITS(2) []
    ],
    OTYPER [
        /// Port x configuration bits (y = 0..15)
        OT15 OFFSET(15) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT14 OFFSET(14) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT13 OFFSET(13) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT12 OFFSET(12) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT11 OFFSET(11) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT10 OFFSET(10) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT9 OFFSET(9) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT8 OFFSET(8) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT7 OFFSET(7) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT6 OFFSET(6) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT5 OFFSET(5) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT4 OFFSET(4) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT3 OFFSET(3) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT2 OFFSET(2) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT1 OFFSET(1) NUMBITS(1) [],
        /// Port x configuration bits (y = 0..15)
        OT0 OFFSET(0) NUMBITS(1) []
    ],
    OSPEEDR [
        /// Port x configuration bits (y = 0..15)
        OSPEEDR15 OFFSET(30) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR14 OFFSET(28) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR13 OFFSET(26) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR12 OFFSET(24) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR11 OFFSET(22) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR10 OFFSET(20) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR9 OFFSET(18) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR8 OFFSET(16) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR7 OFFSET(14) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR6 OFFSET(12) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR5 OFFSET(10) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR4 OFFSET(8) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR3 OFFSET(6) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR2 OFFSET(4) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR1 OFFSET(2) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        OSPEEDR0 OFFSET(0) NUMBITS(2) []
    ],
    PUPDR [
        /// Port x configuration bits (y = 0..15)
        PUPDR15 OFFSET(30) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR14 OFFSET(28) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR13 OFFSET(26) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR12 OFFSET(24) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR11 OFFSET(22) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR10 OFFSET(20) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR9 OFFSET(18) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR8 OFFSET(16) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR7 OFFSET(14) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR6 OFFSET(12) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR5 OFFSET(10) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR4 OFFSET(8) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR3 OFFSET(6) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR2 OFFSET(4) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR1 OFFSET(2) NUMBITS(2) [],
        /// Port x configuration bits (y = 0..15)
        PUPDR0 OFFSET(0) NUMBITS(2) []
    ],
    IDR [
        /// Port input data (y = 0..15)
        IDR15 OFFSET(15) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR14 OFFSET(14) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR13 OFFSET(13) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR12 OFFSET(12) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR11 OFFSET(11) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR10 OFFSET(10) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR9 OFFSET(9) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR8 OFFSET(8) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR7 OFFSET(7) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR6 OFFSET(6) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR5 OFFSET(5) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR4 OFFSET(4) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR3 OFFSET(3) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR2 OFFSET(2) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR1 OFFSET(1) NUMBITS(1) [],
        /// Port input data (y = 0..15)
        IDR0 OFFSET(0) NUMBITS(1) []
    ],
    ODR [
        /// Port output data (y = 0..15)
        ODR15 OFFSET(15) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR14 OFFSET(14) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR13 OFFSET(13) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR12 OFFSET(12) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR11 OFFSET(11) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR10 OFFSET(10) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR9 OFFSET(9) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR8 OFFSET(8) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR7 OFFSET(7) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR6 OFFSET(6) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR5 OFFSET(5) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR4 OFFSET(4) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR3 OFFSET(3) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR2 OFFSET(2) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR1 OFFSET(1) NUMBITS(1) [],
        /// Port output data (y = 0..15)
        ODR0 OFFSET(0) NUMBITS(1) []
    ],
    BSRR [
        /// Port x reset bit y (y = 0..15)
        BR15 OFFSET(31) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR14 OFFSET(30) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR13 OFFSET(29) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR12 OFFSET(28) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR11 OFFSET(27) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR10 OFFSET(26) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR9 OFFSET(25) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR8 OFFSET(24) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR7 OFFSET(23) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR6 OFFSET(22) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR5 OFFSET(21) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR4 OFFSET(20) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR3 OFFSET(19) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR2 OFFSET(18) NUMBITS(1) [],
        /// Port x reset bit y (y = 0..15)
        BR1 OFFSET(17) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BR0 OFFSET(16) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS15 OFFSET(15) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS14 OFFSET(14) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS13 OFFSET(13) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS12 OFFSET(12) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS11 OFFSET(11) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS10 OFFSET(10) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS9 OFFSET(9) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS8 OFFSET(8) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS7 OFFSET(7) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS6 OFFSET(6) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS5 OFFSET(5) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS4 OFFSET(4) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS3 OFFSET(3) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS2 OFFSET(2) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS1 OFFSET(1) NUMBITS(1) [],
        /// Port x set bit y (y= 0..15)
        BS0 OFFSET(0) NUMBITS(1) []
    ],
    LCKR [
        /// Port x lock bit y (y= 0..15)
        LCKK OFFSET(16) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK15 OFFSET(15) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK14 OFFSET(14) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK13 OFFSET(13) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK12 OFFSET(12) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK11 OFFSET(11) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK10 OFFSET(10) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK9 OFFSET(9) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK8 OFFSET(8) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK7 OFFSET(7) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK6 OFFSET(6) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK5 OFFSET(5) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK4 OFFSET(4) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK3 OFFSET(3) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK2 OFFSET(2) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK1 OFFSET(1) NUMBITS(1) [],
        /// Port x lock bit y (y= 0..15)
        LCK0 OFFSET(0) NUMBITS(1) []
    ],
    AFRL [
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL7 OFFSET(28) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL6 OFFSET(24) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL5 OFFSET(20) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL4 OFFSET(16) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL3 OFFSET(12) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL2 OFFSET(8) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL1 OFFSET(4) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 0..7)
        AFRL0 OFFSET(0) NUMBITS(4) []
    ],
    AFRH [
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH15 OFFSET(28) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH14 OFFSET(24) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH13 OFFSET(20) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH12 OFFSET(16) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH11 OFFSET(12) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH10 OFFSET(8) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH9 OFFSET(4) NUMBITS(4) [],
        /// Alternate function selection for port x bit y (y = 8..15)
        AFRH8 OFFSET(0) NUMBITS(4) []
    ]
];

const GPIOH_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40021C00 as *const GpioRegisters) };

const GPIOG_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40021800 as *const GpioRegisters) };

const GPIOF_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40021400 as *const GpioRegisters) };

const GPIOE_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40021000 as *const GpioRegisters) };

const GPIOD_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40020C00 as *const GpioRegisters) };

const GPIOC_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40020800 as *const GpioRegisters) };

const GPIOB_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40020400 as *const GpioRegisters) };

const GPIOA_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40020000 as *const GpioRegisters) };

/// STM32F446RE has eight GPIO ports labeled from A-H [^1]. This is represented
/// by three bits.
///
/// [^1]: Figure 3. STM32F446xC/E block diagram, page 16 of the datasheet
#[repr(u32)]
pub enum PortId {
    A = 0b000,
    B = 0b001,
    C = 0b010,
    D = 0b011,
    E = 0b100,
    F = 0b101,
    G = 0b110,
    H = 0b111,
}

/// Name of the GPIO pin on the STM32F446RE.
///
/// The "Pinout and pin description" section [^1] of the STM32F446RE datasheet
/// shows the mapping between the names and the hardware pins on different chip
/// packages.
///
/// The first three bits represent the port and last four bits represent the
/// pin.
///
/// [^1]: Section 4, Pinout and pin description, pages 41-45
#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PinId {
    PA00 = 0b0000000, PA01 = 0b0000001, PA02 = 0b0000010, PA03 = 0b0000011,
    PA04 = 0b0000100, PA05 = 0b0000101, PA06 = 0b0000110, PA07 = 0b0000111,
    PA08 = 0b0001000, PA09 = 0b0001001, PA10 = 0b0001010, PA11 = 0b0001011,
    PA12 = 0b0001100, PA13 = 0b0001101, PA14 = 0b0001110, PA15 = 0b0001111,

    PB00 = 0b0010000, PB01 = 0b0010001, PB02 = 0b0010010, PB03 = 0b0010011,
    PB04 = 0b0010100, PB05 = 0b0010101, PB06 = 0b0010110, PB07 = 0b0010111,
    PB08 = 0b0011000, PB09 = 0b0011001, PB10 = 0b0011010, PB11 = 0b0011011,
    PB12 = 0b0011100, PB13 = 0b0011101, PB14 = 0b0011110, PB15 = 0b0011111,

    PC00 = 0b0100000, PC01 = 0b0100001, PC02 = 0b0100010, PC03 = 0b0100011,
    PC04 = 0b0100100, PC05 = 0b0100101, PC06 = 0b0100110, PC07 = 0b0100111,
    PC08 = 0b0101000, PC09 = 0b0101001, PC10 = 0b0101010, PC11 = 0b0101011,
    PC12 = 0b0101100, PC13 = 0b0101101, PC14 = 0b0101110, PC15 = 0b0101111,

    PD00 = 0b0110000, PD01 = 0b0110001, PD02 = 0b0110010, PD03 = 0b0110011,
    PD04 = 0b0110100, PD05 = 0b0110101, PD06 = 0b0110110, PD07 = 0b0110111,
    PD08 = 0b0111000, PD09 = 0b0111001, PD10 = 0b0111010, PD11 = 0b0111011,
    PD12 = 0b0111100, PD13 = 0b0111101, PD14 = 0b0111110, PD15 = 0b0111111,

    PE00 = 0b1000000, PE01 = 0b1000001, PE02 = 0b1000010, PE03 = 0b1000011,
    PE04 = 0b1000100, PE05 = 0b1000101, PE06 = 0b1000110, PE07 = 0b1000111,
    PE08 = 0b1001000, PE09 = 0b1001001, PE10 = 0b1001010, PE11 = 0b1001011,
    PE12 = 0b1001100, PE13 = 0b1001101, PE14 = 0b1001110, PE15 = 0b1001111,

    PF00 = 0b1010000, PF01 = 0b1010001, PF02 = 0b1010010, PF03 = 0b1010011,
    PF04 = 0b1010100, PF05 = 0b1010101, PF06 = 0b1010110, PF07 = 0b1010111,
    PF08 = 0b1011000, PF09 = 0b1011001, PF10 = 0b1011010, PF11 = 0b1011011,
    PF12 = 0b1011100, PF13 = 0b1011101, PF14 = 0b1011110, PF15 = 0b1011111,

    PG00 = 0b1100000, PG01 = 0b1100001, PG02 = 0b1100010, PG03 = 0b1100011,
    PG04 = 0b1100100, PG05 = 0b1100101, PG06 = 0b1100110, PG07 = 0b1100111,
    PG08 = 0b1101000, PG09 = 0b1101001, PG10 = 0b1101010, PG11 = 0b1101011,
    PG12 = 0b1101100, PG13 = 0b1101101, PG14 = 0b1101110, PG15 = 0b1101111,

    PH00 = 0b1110000, PH01 = 0b1110001,
}

impl<'a> GpioPorts<'a> {
    pub fn get_pin(&self, pinid: PinId) -> Option<&Pin<'a>> {
        let mut port_num: u8 = pinid as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;

        let mut pin_num: u8 = pinid as u8;
        // Mask top 3 bits, so can get only the suffix
        pin_num &= 0b0001111;

        self.pins[usize::from(port_num)][usize::from(pin_num)].as_ref()
    }

    pub fn get_port(&self, pinid: PinId) -> &Port {
        let mut port_num: u8 = pinid as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;
        &self.ports[usize::from(port_num)]
    }

    pub fn get_port_from_port_id(&self, portid: PortId) -> &Port {
        &self.ports[portid as usize]
    }
}

impl PinId {
    // extract the last 4 bits. [3:0] is the pin number, [6:4] is the port
    // number
    pub fn get_pin_number(&self) -> u8 {
        let mut pin_num = *self as u8;

        pin_num = pin_num & 0b00001111;
        pin_num
    }

    // extract bits [6:4], which is the port number
    pub fn get_port_number(&self) -> u8 {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;
        port_num
    }
}

enum_from_primitive! {
    #[repr(u32)]
    #[derive(PartialEq)]
    /// GPIO pin mode [^1]
    ///
    /// [^1]: Section 7.1.4, page 187 of reference manual
    pub enum Mode {
        Input = 0b00,
        GeneralPurposeOutputMode = 0b01,
        AlternateFunctionMode = 0b10,
        AnalogMode = 0b11,
    }
}

/// Alternate functions that may be assigned to a `Pin`.
///
/// GPIO pins on the STM32F446RE may serve multiple functions. In addition to
/// the default functionality, each pin can be assigned up to sixteen different
/// alternate functions. The various functions for each pin are described in
/// "Alternate Function"" section of the STM32F446RE datasheet[^1].
///
/// Alternate Function bit mapping is shown here[^2].
///
/// [^1]: Section 4, Pinout and pin description, Table 11. Alternate function,
///       pages 59-66
///
/// [^2]: Section 7.4.9, page 192 of Reference Manual
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
    /// GPIO pin internal pull-up and pull-down [^1]
    ///
    /// [^1]: Section 7.4.4, page 189 of reference manual
    enum PullUpPullDown {
        NoPullUpPullDown = 0b00,
        PullUp = 0b01,
        PullDown = 0b10,
    }
}

pub struct Port<'a> {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock<'a>,
}

macro_rules! declare_gpio_pins {
    ($($pin:ident)*, $exti:expr) => {
        [
            $(Some(Pin::new(PinId::$pin, $exti)), )*
        ]
    }
}

// Note: This would probably be better structured as each port holding
// the pins associated with it, but here they are kept separate for
// historical reasons. If writing new GPIO code, look elsewhere for
// a template on how to structure the relationship between ports and pins.
// We need to use `Option<Pin>`, instead of just `Pin` because GPIOH has
// only two pins - PH00 and PH01, rather than the usual sixteen pins.
pub struct GpioPorts<'a> {
    ports: [Port<'a>; 8],
    pub pins: [[Option<Pin<'a>>; 16]; 8],
}

impl<'a> GpioPorts<'a> {
    pub fn new(rcc: &'a rcc::Rcc, exti: &'a exti::Exti<'a>) -> Self {
        Self {
            ports: [
                Port {
                    registers: GPIOA_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOA),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOB_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOB),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOC_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOC),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOD_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOD),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOE_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOE),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOF_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOF),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOG_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOG),
                        rcc,
                    )),
                },
                Port {
                    registers: GPIOH_BASE,
                    clock: PortClock(rcc::PeripheralClock::new(
                        rcc::PeripheralClockType::AHB1(rcc::HCLK1::GPIOH),
                        rcc,
                    )),
                },
            ],
            pins: [
                declare_gpio_pins! {
                    PA00 PA01 PA02 PA03 PA04 PA05 PA06 PA07
                    PA08 PA09 PA10 PA11 PA12 PA13 PA14 PA15, exti
                },
                declare_gpio_pins! {
                    PB00 PB01 PB02 PB03 PB04 PB05 PB06 PB07
                    PB08 PB09 PB10 PB11 PB12 PB13 PB14 PB15, exti
                },
                declare_gpio_pins! {
                    PC00 PC01 PC02 PC03 PC04 PC05 PC06 PC07
                    PC08 PC09 PC10 PC11 PC12 PC13 PC14 PC15, exti
                },
                declare_gpio_pins! {
                    PD00 PD01 PD02 PD03 PD04 PD05 PD06 PD07
                    PD08 PD09 PD10 PD11 PD12 PD13 PD14 PD15, exti
                },
                declare_gpio_pins! {
                    PE00 PE01 PE02 PE03 PE04 PE05 PE06 PE07
                    PE08 PE09 PE10 PE11 PE12 PE13 PE14 PE15, exti
                },
                declare_gpio_pins! {
                    PF00 PF01 PF02 PF03 PF04 PF05 PF06 PF07
                    PF08 PF09 PF10 PF11 PF12 PF13 PF14 PF15, exti
                },
                declare_gpio_pins! {
                    PG00 PG01 PG02 PG03 PG04 PG05 PG06 PG07
                    PG08 PG09 PG10 PG11 PG12 PG13 PG14 PG15, exti
                },
                [
                    Some(Pin::new(PinId::PH00, exti)),
                    Some(Pin::new(PinId::PH01, exti)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
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
    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }
}

struct PortClock<'a>(rcc::PeripheralClock<'a>);

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

// `exti_lineid` is used to configure EXTI settings for the Pin.
pub struct Pin<'a> {
    pinid: PinId,
    ports_ref: OptionalCell<&'a GpioPorts<'a>>,
    exti: &'a exti::Exti<'a>,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
    exti_lineid: OptionalCell<exti::LineId>,
}

impl<'a> Pin<'a> {
    pub const fn new(pinid: PinId, exti: &'a exti::Exti<'a>) -> Self {
        Self {
            pinid,
            ports_ref: OptionalCell::empty(),
            exti,
            client: OptionalCell::empty(),
            exti_lineid: OptionalCell::empty(),
        }
    }

    pub fn set_ports_ref(&self, ports: &'a GpioPorts<'a>) {
        self.ports_ref.set(ports);
    }

    pub fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| client.fired());
    }

    pub fn get_mode(&self) -> Mode {
        let port = self.ports_ref.expect("").get_port(self.pinid);

        let val = match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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

    pub fn set_alternate_function(&self, af: AlternateFunction) {
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
            0b0000 => port.registers.afrl.modify(AFRL::AFRL0.val(af as u32)),
            0b0001 => port.registers.afrl.modify(AFRL::AFRL1.val(af as u32)),
            0b0010 => port.registers.afrl.modify(AFRL::AFRL2.val(af as u32)),
            0b0011 => port.registers.afrl.modify(AFRL::AFRL3.val(af as u32)),
            0b0100 => port.registers.afrl.modify(AFRL::AFRL4.val(af as u32)),
            0b0101 => port.registers.afrl.modify(AFRL::AFRL5.val(af as u32)),
            0b0110 => port.registers.afrl.modify(AFRL::AFRL6.val(af as u32)),
            0b0111 => port.registers.afrl.modify(AFRL::AFRL7.val(af as u32)),
            0b1000 => port.registers.afrh.modify(AFRH::AFRH8.val(af as u32)),
            0b1001 => port.registers.afrh.modify(AFRH::AFRH9.val(af as u32)),
            0b1010 => port.registers.afrh.modify(AFRH::AFRH10.val(af as u32)),
            0b1011 => port.registers.afrh.modify(AFRH::AFRH11.val(af as u32)),
            0b1100 => port.registers.afrh.modify(AFRH::AFRH12.val(af as u32)),
            0b1101 => port.registers.afrh.modify(AFRH::AFRH13.val(af as u32)),
            0b1110 => port.registers.afrh.modify(AFRH::AFRH14.val(af as u32)),
            0b1111 => port.registers.afrh.modify(AFRH::AFRH15.val(af as u32)),
            _ => {}
        }
    }

    pub fn get_pinid(&self) -> PinId {
        self.pinid
    }

    pub fn set_exti_lineid(&self, lineid: exti::LineId) {
        self.exti_lineid.set(lineid);
    }

    fn set_mode_output_pushpull(&self) {
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        let val = match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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
        let port = self.ports_ref.expect("").get_port(self.pinid);

        match self.pinid.get_pin_number() {
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

impl hil::gpio::Pin for Pin<'_> {}
impl<'a> hil::gpio::InterruptPin<'a> for Pin<'a> {}

impl hil::gpio::Configure for Pin<'_> {
    /// Output mode default is push-pull
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::GeneralPurposeOutputMode);
        self.set_mode_output_pushpull();
        hil::gpio::Configuration::Output
    }

    /// Input mode default is no internal pull-up, no pull-down (i.e.,
    /// floating). Also upon setting the mode as input, the internal schmitt
    /// trigger is automatically activated. Schmitt trigger is deactivated in
    /// AnalogMode.
    fn make_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Input);
        hil::gpio::Configuration::Input
    }

    /// According to AN4899, Section 6.1, setting to AnalogMode, disables
    /// internal schmitt trigger. We do not disable clock to the GPIO port,
    /// because there could be other pins active on the port.
    fn deactivate_to_low_power(&self) {
        self.set_mode(Mode::AnalogMode);
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::AnalogMode);
        hil::gpio::Configuration::LowPower
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::AnalogMode);
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
            Mode::GeneralPurposeOutputMode => hil::gpio::Configuration::Output,
            Mode::AnalogMode => hil::gpio::Configuration::LowPower,
            Mode::AlternateFunctionMode => hil::gpio::Configuration::Function,
        }
    }

    fn is_input(&self) -> bool {
        self.get_mode() == Mode::Input
    }

    fn is_output(&self) -> bool {
        self.get_mode() == Mode::GeneralPurposeOutputMode
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
    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        unsafe {
            atomic(|| {
                self.exti_lineid.map(|lineid| {
                    let l = lineid.clone();

                    // disable the interrupt
                    self.exti.mask_interrupt(l);
                    self.exti.clear_pending(l);

                    match mode {
                        hil::gpio::InterruptEdge::EitherEdge => {
                            self.exti.select_rising_trigger(l);
                            self.exti.select_falling_trigger(l);
                        }
                        hil::gpio::InterruptEdge::RisingEdge => {
                            self.exti.select_rising_trigger(l);
                            self.exti.deselect_falling_trigger(l);
                        }
                        hil::gpio::InterruptEdge::FallingEdge => {
                            self.exti.deselect_rising_trigger(l);
                            self.exti.select_falling_trigger(l);
                        }
                    }

                    self.exti.unmask_interrupt(l);
                });
            });
        }
    }

    fn disable_interrupts(&self) {
        unsafe {
            atomic(|| {
                self.exti_lineid.map(|lineid| {
                    let l = lineid.clone();
                    self.exti.mask_interrupt(l);
                    self.exti.clear_pending(l);
                });
            });
        }
    }

    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        self.exti_lineid
            .map_or(false, |&mut lineid| self.exti.is_pending(lineid))
    }
}
