use cortexm7::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::ccm;

/// General-purpose I/Os
#[repr(C)]
struct GpioRegisters {
    // GPIO data register
    dr: ReadWrite<u32, DR::Register>,
    // GPIO direction register
    gdir: ReadWrite<u32, GDIR::Register>,
    // GPIO pad status register
    psr: ReadOnly<u32, PSR::Register>,
    // GPIO Interrupt configuration register 1
    icr1: ReadWrite<u32, ICR1::Register>,
    // GPIO Interrupt configuration register 2
    icr2: ReadWrite<u32, ICR2::Register>,
    // GPIO interrupt mask register
    imr: ReadWrite<u32, IMR::Register>,
    // GPIO interrupt status register -- W1C - Write 1 to clear
    isr: ReadWrite<u32, ISR::Register>,
    // GPIO edge select register
    edge_sel: ReadWrite<u32, EDGE_SEL::Register>,
    _reserved1: [u8; 100],
    // GPIO data register set
    dr_set: WriteOnly<u32, DR_SET::Register>,
    // GPIO data register clear
    dr_clear: WriteOnly<u32, DR_CLEAR::Register>,
    // GPIO data register toggle
    dr_toggle: WriteOnly<u32, DR_TOGGLE::Register>,
}

register_bitfields![u32,
    DR [
        // the value of the GPIO output when the signal is configured as an output
        DR31 OFFSET(31) NUMBITS(1) [],
        DR30 OFFSET(30) NUMBITS(1) [],
        DR29 OFFSET(29) NUMBITS(1) [],
        DR28 OFFSET(28) NUMBITS(1) [],
        DR27 OFFSET(27) NUMBITS(1) [],
        DR26 OFFSET(26) NUMBITS(1) [],
        DR25 OFFSET(25) NUMBITS(1) [],
        DR24 OFFSET(24) NUMBITS(1) [],
        DR23 OFFSET(23) NUMBITS(1) [],
        DR22 OFFSET(22) NUMBITS(1) [],
        DR21 OFFSET(21) NUMBITS(1) [],
        DR20 OFFSET(20) NUMBITS(1) [],
        DR19 OFFSET(19) NUMBITS(1) [],
        DR18 OFFSET(18) NUMBITS(1) [],
        DR17 OFFSET(17) NUMBITS(1) [],
        DR16 OFFSET(16) NUMBITS(1) [],
        DR15 OFFSET(15) NUMBITS(1) [],
        DR14 OFFSET(14) NUMBITS(1) [],
        DR13 OFFSET(13) NUMBITS(1) [],
        DR12 OFFSET(12) NUMBITS(1) [],
        DR11 OFFSET(11) NUMBITS(1) [],
        DR10 OFFSET(10) NUMBITS(1) [],
        DR9 OFFSET(9) NUMBITS(1) [],
        DR8 OFFSET(8) NUMBITS(1) [],
        DR7 OFFSET(7) NUMBITS(1) [],
        DR6 OFFSET(6) NUMBITS(1) [],
        DR5 OFFSET(5) NUMBITS(1) [],
        DR4 OFFSET(4) NUMBITS(1) [],
        DR3 OFFSET(3) NUMBITS(1) [],
        DR2 OFFSET(2) NUMBITS(1) [],
        DR1 OFFSET(1) NUMBITS(1) [],
        DR0 OFFSET(0) NUMBITS(1) []
    ],

    GDIR [
        // bit n of this register defines the direction of the GPIO[n] signal
        GDIR31 OFFSET(31) NUMBITS(1) [],
        GDIR30 OFFSET(30) NUMBITS(1) [],
        GDIR29 OFFSET(29) NUMBITS(1) [],
        GDIR28 OFFSET(28) NUMBITS(1) [],
        GDIR27 OFFSET(27) NUMBITS(1) [],
        GDIR26 OFFSET(26) NUMBITS(1) [],
        GDIR25 OFFSET(25) NUMBITS(1) [],
        GDIR24 OFFSET(24) NUMBITS(1) [],
        GDIR23 OFFSET(23) NUMBITS(1) [],
        GDIR22 OFFSET(22) NUMBITS(1) [],
        GDIR21 OFFSET(21) NUMBITS(1) [],
        GDIR20 OFFSET(20) NUMBITS(1) [],
        GDIR19 OFFSET(19) NUMBITS(1) [],
        GDIR18 OFFSET(18) NUMBITS(1) [],
        GDIR17 OFFSET(17) NUMBITS(1) [],
        GDIR16 OFFSET(16) NUMBITS(1) [],
        GDIR15 OFFSET(15) NUMBITS(1) [],
        GDIR14 OFFSET(14) NUMBITS(1) [],
        GDIR13 OFFSET(13) NUMBITS(1) [],
        GDIR12 OFFSET(12) NUMBITS(1) [],
        GDIR11 OFFSET(11) NUMBITS(1) [],
        GDIR10 OFFSET(10) NUMBITS(1) [],
        GDIR9 OFFSET(9) NUMBITS(1) [],
        GDIR8 OFFSET(8) NUMBITS(1) [],
        GDIR7 OFFSET(7) NUMBITS(1) [],
        GDIR6 OFFSET(6) NUMBITS(1) [],
        GDIR5 OFFSET(5) NUMBITS(1) [],
        GDIR4 OFFSET(4) NUMBITS(1) [],
        GDIR3 OFFSET(3) NUMBITS(1) [],
        GDIR2 OFFSET(2) NUMBITS(1) [],
        GDIR1 OFFSET(1) NUMBITS(1) [],
        GDIR0 OFFSET(0) NUMBITS(1) []
    ],

    PSR [
        // bit n of this register returns the state of the corresponding GPIO[n] signal
        PSR31 OFFSET(31) NUMBITS(1) [],
        PSR30 OFFSET(30) NUMBITS(1) [],
        PSR29 OFFSET(29) NUMBITS(1) [],
        PSR28 OFFSET(28) NUMBITS(1) [],
        PSR27 OFFSET(27) NUMBITS(1) [],
        PSR26 OFFSET(26) NUMBITS(1) [],
        PSR25 OFFSET(25) NUMBITS(1) [],
        PSR24 OFFSET(24) NUMBITS(1) [],
        PSR23 OFFSET(23) NUMBITS(1) [],
        PSR22 OFFSET(22) NUMBITS(1) [],
        PSR21 OFFSET(21) NUMBITS(1) [],
        PSR20 OFFSET(20) NUMBITS(1) [],
        PSR19 OFFSET(19) NUMBITS(1) [],
        PSR18 OFFSET(18) NUMBITS(1) [],
        PSR17 OFFSET(17) NUMBITS(1) [],
        PSR16 OFFSET(16) NUMBITS(1) [],
        PSR15 OFFSET(15) NUMBITS(1) [],
        PSR14 OFFSET(14) NUMBITS(1) [],
        PSR13 OFFSET(13) NUMBITS(1) [],
        PSR12 OFFSET(12) NUMBITS(1) [],
        PSR11 OFFSET(11) NUMBITS(1) [],
        PSR10 OFFSET(10) NUMBITS(1) [],
        PSR9 OFFSET(9) NUMBITS(1) [],
        PSR8 OFFSET(8) NUMBITS(1) [],
        PSR7 OFFSET(7) NUMBITS(1) [],
        PSR6 OFFSET(6) NUMBITS(1) [],
        PSR5 OFFSET(5) NUMBITS(1) [],
        PSR4 OFFSET(4) NUMBITS(1) [],
        PSR3 OFFSET(3) NUMBITS(1) [],
        PSR2 OFFSET(2) NUMBITS(1) [],
        PSR1 OFFSET(1) NUMBITS(1) [],
        PSR0 OFFSET(0) NUMBITS(1) []
    ],

    ICR1 [
        // IRCn of this register defines interrupt condition for signal n
        ICR15 OFFSET(15) NUMBITS(2) [],
        ICR14 OFFSET(14) NUMBITS(2) [],
        ICR13 OFFSET(13) NUMBITS(2) [],
        ICR12 OFFSET(12) NUMBITS(2) [],
        ICR11 OFFSET(11) NUMBITS(2) [],
        ICR10 OFFSET(10) NUMBITS(2) [],
        ICR9 OFFSET(9) NUMBITS(2) [],
        ICR8 OFFSET(8) NUMBITS(2) [],
        ICR7 OFFSET(7) NUMBITS(2) [],
        ICR6 OFFSET(6) NUMBITS(2) [],
        ICR5 OFFSET(5) NUMBITS(2) [],
        ICR4 OFFSET(4) NUMBITS(2) [],
        ICR3 OFFSET(3) NUMBITS(2) [],
        ICR2 OFFSET(2) NUMBITS(2) [],
        ICR1 OFFSET(1) NUMBITS(2) [],
        ICR0 OFFSET(0) NUMBITS(2) []
    ],

    ICR2 [
        // IRCn of this register defines interrupt condition for signal n
        ICR31 OFFSET(31) NUMBITS(2) [],
        ICR30 OFFSET(30) NUMBITS(2) [],
        ICR29 OFFSET(29) NUMBITS(2) [],
        ICR28 OFFSET(28) NUMBITS(2) [],
        ICR27 OFFSET(27) NUMBITS(2) [],
        ICR26 OFFSET(26) NUMBITS(2) [],
        ICR25 OFFSET(25) NUMBITS(2) [],
        ICR24 OFFSET(24) NUMBITS(2) [],
        ICR23 OFFSET(23) NUMBITS(2) [],
        ICR22 OFFSET(22) NUMBITS(2) [],
        ICR21 OFFSET(21) NUMBITS(2) [],
        ICR20 OFFSET(20) NUMBITS(2) [],
        ICR19 OFFSET(19) NUMBITS(2) [],
        ICR18 OFFSET(18) NUMBITS(2) [],
        ICR17 OFFSET(17) NUMBITS(2) [],
        ICR16 OFFSET(16) NUMBITS(2) []
    ],

    IMR [
        // enable or disable the interrupt function on each of the 32 GPIO signals
        IMR31 OFFSET(31) NUMBITS(1) [],
        IMR30 OFFSET(30) NUMBITS(1) [],
        IMR29 OFFSET(29) NUMBITS(1) [],
        IMR28 OFFSET(28) NUMBITS(1) [],
        IMR27 OFFSET(27) NUMBITS(1) [],
        IMR26 OFFSET(26) NUMBITS(1) [],
        IMR25 OFFSET(25) NUMBITS(1) [],
        IMR24 OFFSET(24) NUMBITS(1) [],
        IMR23 OFFSET(23) NUMBITS(1) [],
        IMR22 OFFSET(22) NUMBITS(1) [],
        IMR21 OFFSET(21) NUMBITS(1) [],
        IMR20 OFFSET(20) NUMBITS(1) [],
        IMR19 OFFSET(19) NUMBITS(1) [],
        IMR18 OFFSET(18) NUMBITS(1) [],
        IMR17 OFFSET(17) NUMBITS(1) [],
        IMR16 OFFSET(16) NUMBITS(1) [],
        IMR15 OFFSET(15) NUMBITS(1) [],
        IMR14 OFFSET(14) NUMBITS(1) [],
        IMR13 OFFSET(13) NUMBITS(1) [],
        IMR12 OFFSET(12) NUMBITS(1) [],
        IMR11 OFFSET(11) NUMBITS(1) [],
        IMR10 OFFSET(10) NUMBITS(1) [],
        IMR9 OFFSET(9) NUMBITS(1) [],
        IMR8 OFFSET(8) NUMBITS(1) [],
        IMR7 OFFSET(7) NUMBITS(1) [],
        IMR6 OFFSET(6) NUMBITS(1) [],
        IMR5 OFFSET(5) NUMBITS(1) [],
        IMR4 OFFSET(4) NUMBITS(1) [],
        IMR3 OFFSET(3) NUMBITS(1) [],
        IMR2 OFFSET(2) NUMBITS(1) [],
        IMR1 OFFSET(1) NUMBITS(1) [],
        IMR0 OFFSET(0) NUMBITS(1) []
    ],

    ISR [
        // Bit n of this register is asserted (active high) when the active condition is detected
        // on the GPIO input and waiting for service
        ISR31 OFFSET(31) NUMBITS(1) [],
        ISR30 OFFSET(30) NUMBITS(1) [],
        ISR29 OFFSET(29) NUMBITS(1) [],
        ISR28 OFFSET(28) NUMBITS(1) [],
        ISR27 OFFSET(27) NUMBITS(1) [],
        ISR26 OFFSET(26) NUMBITS(1) [],
        ISR25 OFFSET(25) NUMBITS(1) [],
        ISR24 OFFSET(24) NUMBITS(1) [],
        ISR23 OFFSET(23) NUMBITS(1) [],
        ISR22 OFFSET(22) NUMBITS(1) [],
        ISR21 OFFSET(21) NUMBITS(1) [],
        ISR20 OFFSET(20) NUMBITS(1) [],
        ISR19 OFFSET(19) NUMBITS(1) [],
        ISR18 OFFSET(18) NUMBITS(1) [],
        ISR17 OFFSET(17) NUMBITS(1) [],
        ISR16 OFFSET(16) NUMBITS(1) [],
        ISR15 OFFSET(15) NUMBITS(1) [],
        ISR14 OFFSET(14) NUMBITS(1) [],
        ISR13 OFFSET(13) NUMBITS(1) [],
        ISR12 OFFSET(12) NUMBITS(1) [],
        ISR11 OFFSET(11) NUMBITS(1) [],
        ISR10 OFFSET(10) NUMBITS(1) [],
        ISR9 OFFSET(9) NUMBITS(1) [],
        ISR8 OFFSET(8) NUMBITS(1) [],
        ISR7 OFFSET(7) NUMBITS(1) [],
        ISR6 OFFSET(6) NUMBITS(1) [],
        ISR5 OFFSET(5) NUMBITS(1) [],
        ISR4 OFFSET(4) NUMBITS(1) [],
        ISR3 OFFSET(3) NUMBITS(1) [],
        ISR2 OFFSET(2) NUMBITS(1) [],
        ISR1 OFFSET(1) NUMBITS(1) [],
        ISR0 OFFSET(0) NUMBITS(1) []
    ],

    EDGE_SEL [
        // When EDGE_SELn is set, the GPIO disregards the ICRn setting
        EDGE_SEL31 OFFSET(31) NUMBITS(1) [],
        EDGE_SEL30 OFFSET(30) NUMBITS(1) [],
        EDGE_SEL29 OFFSET(29) NUMBITS(1) [],
        EDGE_SEL28 OFFSET(28) NUMBITS(1) [],
        EDGE_SEL27 OFFSET(27) NUMBITS(1) [],
        EDGE_SEL26 OFFSET(26) NUMBITS(1) [],
        EDGE_SEL25 OFFSET(25) NUMBITS(1) [],
        EDGE_SEL24 OFFSET(24) NUMBITS(1) [],
        EDGE_SEL23 OFFSET(23) NUMBITS(1) [],
        EDGE_SEL22 OFFSET(22) NUMBITS(1) [],
        EDGE_SEL21 OFFSET(21) NUMBITS(1) [],
        EDGE_SEL20 OFFSET(20) NUMBITS(1) [],
        EDGE_SEL19 OFFSET(19) NUMBITS(1) [],
        EDGE_SEL18 OFFSET(18) NUMBITS(1) [],
        EDGE_SEL17 OFFSET(17) NUMBITS(1) [],
        EDGE_SEL16 OFFSET(16) NUMBITS(1) [],
        EDGE_SEL15 OFFSET(15) NUMBITS(1) [],
        EDGE_SEL14 OFFSET(14) NUMBITS(1) [],
        EDGE_SEL13 OFFSET(13) NUMBITS(1) [],
        EDGE_SEL12 OFFSET(12) NUMBITS(1) [],
        EDGE_SEL11 OFFSET(11) NUMBITS(1) [],
        EDGE_SEL10 OFFSET(10) NUMBITS(1) [],
        EDGE_SEL9 OFFSET(9) NUMBITS(1) [],
        EDGE_SEL8 OFFSET(8) NUMBITS(1) [],
        EDGE_SEL7 OFFSET(7) NUMBITS(1) [],
        EDGE_SEL6 OFFSET(6) NUMBITS(1) [],
        EDGE_SEL5 OFFSET(5) NUMBITS(1) [],
        EDGE_SEL4 OFFSET(4) NUMBITS(1) [],
        EDGE_SEL3 OFFSET(3) NUMBITS(1) [],
        EDGE_SEL2 OFFSET(2) NUMBITS(1) [],
        EDGE_SEL1 OFFSET(1) NUMBITS(1) [],
        EDGE_SEL0 OFFSET(0) NUMBITS(1) []
    ],

    DR_SET [
        // The set register of DR
        DR_SET31 OFFSET(31) NUMBITS(1) [],
        DR_SET30 OFFSET(30) NUMBITS(1) [],
        DR_SET29 OFFSET(29) NUMBITS(1) [],
        DR_SET28 OFFSET(28) NUMBITS(1) [],
        DR_SET27 OFFSET(27) NUMBITS(1) [],
        DR_SET26 OFFSET(26) NUMBITS(1) [],
        DR_SET25 OFFSET(25) NUMBITS(1) [],
        DR_SET24 OFFSET(24) NUMBITS(1) [],
        DR_SET23 OFFSET(23) NUMBITS(1) [],
        DR_SET22 OFFSET(22) NUMBITS(1) [],
        DR_SET21 OFFSET(21) NUMBITS(1) [],
        DR_SET20 OFFSET(20) NUMBITS(1) [],
        DR_SET19 OFFSET(19) NUMBITS(1) [],
        DR_SET18 OFFSET(18) NUMBITS(1) [],
        DR_SET17 OFFSET(17) NUMBITS(1) [],
        DR_SET16 OFFSET(16) NUMBITS(1) [],
        DR_SET15 OFFSET(15) NUMBITS(1) [],
        DR_SET14 OFFSET(14) NUMBITS(1) [],
        DR_SET13 OFFSET(13) NUMBITS(1) [],
        DR_SET12 OFFSET(12) NUMBITS(1) [],
        DR_SET11 OFFSET(11) NUMBITS(1) [],
        DR_SET10 OFFSET(10) NUMBITS(1) [],
        DR_SET9 OFFSET(9) NUMBITS(1) [],
        DR_SET8 OFFSET(8) NUMBITS(1) [],
        DR_SET7 OFFSET(7) NUMBITS(1) [],
        DR_SET6 OFFSET(6) NUMBITS(1) [],
        DR_SET5 OFFSET(5) NUMBITS(1) [],
        DR_SET4 OFFSET(4) NUMBITS(1) [],
        DR_SET3 OFFSET(3) NUMBITS(1) [],
        DR_SET2 OFFSET(2) NUMBITS(1) [],
        DR_SET1 OFFSET(1) NUMBITS(1) [],
        DR_SET0 OFFSET(0) NUMBITS(1) []
    ],

    DR_CLEAR [
        // The clear register of DR
        DR_CLEAR31 OFFSET(31) NUMBITS(1) [],
        DR_CLEAR30 OFFSET(30) NUMBITS(1) [],
        DR_CLEAR29 OFFSET(29) NUMBITS(1) [],
        DR_CLEAR28 OFFSET(28) NUMBITS(1) [],
        DR_CLEAR27 OFFSET(27) NUMBITS(1) [],
        DR_CLEAR26 OFFSET(26) NUMBITS(1) [],
        DR_CLEAR25 OFFSET(25) NUMBITS(1) [],
        DR_CLEAR24 OFFSET(24) NUMBITS(1) [],
        DR_CLEAR23 OFFSET(23) NUMBITS(1) [],
        DR_CLEAR22 OFFSET(22) NUMBITS(1) [],
        DR_CLEAR21 OFFSET(21) NUMBITS(1) [],
        DR_CLEAR20 OFFSET(20) NUMBITS(1) [],
        DR_CLEAR19 OFFSET(19) NUMBITS(1) [],
        DR_CLEAR18 OFFSET(18) NUMBITS(1) [],
        DR_CLEAR17 OFFSET(17) NUMBITS(1) [],
        DR_CLEAR16 OFFSET(16) NUMBITS(1) [],
        DR_CLEAR15 OFFSET(15) NUMBITS(1) [],
        DR_CLEAR14 OFFSET(14) NUMBITS(1) [],
        DR_CLEAR13 OFFSET(13) NUMBITS(1) [],
        DR_CLEAR12 OFFSET(12) NUMBITS(1) [],
        DR_CLEAR11 OFFSET(11) NUMBITS(1) [],
        DR_CLEAR10 OFFSET(10) NUMBITS(1) [],
        DR_CLEAR9 OFFSET(9) NUMBITS(1) [],
        DR_CLEAR8 OFFSET(8) NUMBITS(1) [],
        DR_CLEAR7 OFFSET(7) NUMBITS(1) [],
        DR_CLEAR6 OFFSET(6) NUMBITS(1) [],
        DR_CLEAR5 OFFSET(5) NUMBITS(1) [],
        DR_CLEAR4 OFFSET(4) NUMBITS(1) [],
        DR_CLEAR3 OFFSET(3) NUMBITS(1) [],
        DR_CLEAR2 OFFSET(2) NUMBITS(1) [],
        DR_CLEAR1 OFFSET(1) NUMBITS(1) [],
        DR_CLEAR0 OFFSET(0) NUMBITS(1) []
    ],

    DR_TOGGLE [
        // The toggle register of DR
        DR_TOGGLE31 OFFSET(31) NUMBITS(1) [],
        DR_TOGGLE30 OFFSET(30) NUMBITS(1) [],
        DR_TOGGLE29 OFFSET(29) NUMBITS(1) [],
        DR_TOGGLE28 OFFSET(28) NUMBITS(1) [],
        DR_TOGGLE27 OFFSET(27) NUMBITS(1) [],
        DR_TOGGLE26 OFFSET(26) NUMBITS(1) [],
        DR_TOGGLE25 OFFSET(25) NUMBITS(1) [],
        DR_TOGGLE24 OFFSET(24) NUMBITS(1) [],
        DR_TOGGLE23 OFFSET(23) NUMBITS(1) [],
        DR_TOGGLE22 OFFSET(22) NUMBITS(1) [],
        DR_TOGGLE21 OFFSET(21) NUMBITS(1) [],
        DR_TOGGLE20 OFFSET(20) NUMBITS(1) [],
        DR_TOGGLE19 OFFSET(19) NUMBITS(1) [],
        DR_TOGGLE18 OFFSET(18) NUMBITS(1) [],
        DR_TOGGLE17 OFFSET(17) NUMBITS(1) [],
        DR_TOGGLE16 OFFSET(16) NUMBITS(1) [],
        DR_TOGGLE15 OFFSET(15) NUMBITS(1) [],
        DR_TOGGLE14 OFFSET(14) NUMBITS(1) [],
        DR_TOGGLE13 OFFSET(13) NUMBITS(1) [],
        DR_TOGGLE12 OFFSET(12) NUMBITS(1) [],
        DR_TOGGLE11 OFFSET(11) NUMBITS(1) [],
        DR_TOGGLE10 OFFSET(10) NUMBITS(1) [],
        DR_TOGGLE9 OFFSET(9) NUMBITS(1) [],
        DR_TOGGLE8 OFFSET(8) NUMBITS(1) [],
        DR_TOGGLE7 OFFSET(7) NUMBITS(1) [],
        DR_TOGGLE6 OFFSET(6) NUMBITS(1) [],
        DR_TOGGLE5 OFFSET(5) NUMBITS(1) [],
        DR_TOGGLE4 OFFSET(4) NUMBITS(1) [],
        DR_TOGGLE3 OFFSET(3) NUMBITS(1) [],
        DR_TOGGLE2 OFFSET(2) NUMBITS(1) [],
        DR_TOGGLE1 OFFSET(1) NUMBITS(1) [],
        DR_TOGGLE0 OFFSET(0) NUMBITS(1) []
    ]
];

const GPIO1_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401B8000 as *const GpioRegisters) };

const GPIO2_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401BC000 as *const GpioRegisters) };

const GPIO3_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401C0000 as *const GpioRegisters) };

const GPIO4_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401C4000 as *const GpioRegisters) };

const GPIO5_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x400C0000 as *const GpioRegisters) };

enum_from_primitive! {
    #[repr(u8)]
    #[derive(PartialEq)]

    /// Imxrt1050-evkb has 5 GPIO ports labeled from 1-5 [^1]. This is represented
    /// by three bits.
    ///
    /// [^1]: 12.5.1 GPIO memory map, page 1009 of the Reference Manual.
    pub enum GpioPort {
        GPIO1 = 0b000,
        GPIO2 = 0b001,
        GPIO3 = 0b010,
        GPIO4 = 0b011,
        GPIO5 = 0b100,
    }
}

// Name of the GPIO pins
// For imxrt1050, the pins are organised in pads. In order to use the pins
// efficiently, we use the following codification: 9 bits to identify a pin.
// - The first 3 bits identify the Pad (Emc, AdB0, AdB1, B0, B1, SdB0, SdB1) [^1]
// - The last 6 bits identifiy the Pin number (1 for Emc01)
// In order to identify the GPIO port, we make an association between the Pad and
// Pin number in order to get the port. For example, Emc00-Emc31 belong to GPIO4,
// while Emc32-Emc41 belong to GPIO3.
//
// [^1]: Naming of the pads: 11.7. IOMUXC memory map, page 380 of the Reference Manual
#[rustfmt::skip]
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum PinId {
    Emc00 = 0b000000000, Emc01 = 0b000000001, Emc02 = 0b000000010, Emc03 = 0b0000000011,
    Emc04 = 0b000000100, Emc05 = 0b000000101, Emc06 = 0b000000110, Emc07 = 0b000000111,
    Emc08 = 0b000001000, Emc09 = 0b000001001, Emc10 = 0b000001010, Emc11 = 0b000001011,
    Emc12 = 0b000001100, Emc13 = 0b000001101, Emc14 = 0b000001110, Emc15 = 0b000001111,
    Emc16 = 0b000010000, Emc17 = 0b000010001, Emc18 = 0b000010010, Emc19 = 0b000010011,
    Emc20 = 0b000010100, Emc21 = 0b000010101, Emc22 = 0b000010110, Emc23 = 0b000010111,
    Emc24 = 0b000011000, Emc25 = 0b000011001, Emc26 = 0b000011010, Emc27 = 0b000011011,
    Emc28 = 0b000011100, Emc29 = 0b000011101, Emc30 = 0b000011110, Emc31 = 0b000011111,
    Emc32 = 0b000100000, Emc33 = 0b000100001, Emc34 = 0b000100010, Emc35 = 0b000100011,
    Emc36 = 0b000100100, Emc37 = 0b000100101, Emc38 = 0b000100110, Emc39 = 0b000100111,
    Emc40 = 0b000101000, Emc41 = 0b000101001, 

    AdB0_00 = 0b001000000, AdB0_01 = 0b001000001, AdB0_02 = 0b001000010, AdB0_03 = 0b001000011,
    AdB0_04 = 0b001000100, AdB0_05 = 0b001000101, AdB0_06 = 0b001000110, AdB0_07 = 0b001000111,
    AdB0_08 = 0b001001000, AdB0_09 = 0b001001001, AdB0_10 = 0b001001010, AdB0_11 = 0b001001011,
    AdB0_12 = 0b001001100, AdB0_13 = 0b001001101, AdB0_14 = 0b001001110, AdB0_15 = 0b001001111,
    
    AdB1_00 = 0b010000000, AdB1_01 = 0b010000001, AdB1_02 = 0b010000010, AdB1_03 = 0b010000011,
    AdB1_04 = 0b010000100, AdB1_05 = 0b010000101, AdB1_06 = 0b010000110, AdB1_07 = 0b010000111,
    AdB1_08 = 0b010001000, AdB1_09 = 0b010001001, AdB1_10 = 0b010001010, AdB1_11 = 0b010001011,
    AdB1_12 = 0b010001100, AdB1_13 = 0b010001101, AdB1_14 = 0b010001110, AdB1_15 = 0b010001111,

    B0_00 = 0b011000000, B0_01 = 0b011000001, B0_02 = 0b011000010, B0_03 = 0b011000011,
    B0_04 = 0b011000100, B0_05 = 0b011000101, B0_06 = 0b011000110, B0_07 = 0b011000111,
    B0_08 = 0b011001000, B0_09 = 0b011001001, B0_10 = 0b011001010, B0_11 = 0b011001011,
    B0_12 = 0b011001100, B0_13 = 0b011001101, B0_14 = 0b011001110, B0_15 = 0b011001111,

    B1_00 = 0b100000000, B1_01 = 0b100000001, B1_02 = 0b100000010, B1_03 = 0b100000011,
    B1_04 = 0b100000100, B1_05 = 0b100000101, B1_06 = 0b100000110, B1_07 = 0b100000111,
    B1_08 = 0b100001000, B1_09 = 0b100001001, B1_10 = 0b100001010, B1_11 = 0b100001011,
    B1_12 = 0b100001100, B1_13 = 0b100001101, B1_14 = 0b100001110, B1_15 = 0b100001111,

    SdB0_00 = 0b101000000, SdB0_01 = 0b101000001, SdB0_02 = 0b101000010, SdB0_03 = 0b101000011,
    SdB0_04 = 0b101000100, SdB0_05 = 0b101000101, 

    SdB1_00 = 0b110000000, SdB1_01 = 0b110000001, SdB1_02 = 0b110000010, SdB1_03 = 0b110000011,
    SdB1_04 = 0b110000100, SdB1_05 = 0b110000101, SdB1_06 = 0b110000110, SdB1_07 = 0b110000111,
    SdB1_08 = 0b110001000, SdB1_09 = 0b110001001, SdB1_10 = 0b110001010, SdB1_11 = 0b110001011,
    SdB1_12 = 0b110001100,

    Wakeup = 0b111000000, PmicOnReq = 0b111000001, PmicStbyReq = 0b111000010, 
}

impl PinId {
    pub fn get_port_number(&self) -> GpioPort {
        let mut pad_num: u16 = *self as u16;
        pad_num >>= 6;
        let mut pin_num: u8 = *self as u8;
        pin_num &= 0b00111111;

        match pad_num {
            0b000 => {
                if pin_num < 32 {
                    GpioPort::GPIO4
                } else {
                    GpioPort::GPIO3
                }
            }
            0b001 => GpioPort::GPIO1,
            0b010 => GpioPort::GPIO1,
            0b011 => GpioPort::GPIO2,
            0b100 => GpioPort::GPIO2,
            0b101 => GpioPort::GPIO3,
            0b110 => GpioPort::GPIO3,
            0b111 => GpioPort::GPIO5,
            _ => GpioPort::GPIO1,
        }
    }

    pub fn get_pad_number(&self) -> u16 {
        let mut pad_num: u16 = *self as u16;
        pad_num >>= 6;
        pad_num
    }

    pub fn get_pin(&self) -> &Option<Pin<'static>> {
        let mut pad_num: u16 = *self as u16;

        // Right shift pad_num by 6 bits, so we can get rid of pin bits
        pad_num >>= 6;

        let mut pin_num: u8 = *self as u8;
        // Mask top 2 bits, so can get only the suffix
        pin_num &= 0b00111111;

        unsafe { &PIN[usize::from(pad_num)][usize::from(pin_num)] }
    }

    #[allow(clippy::mut_from_ref)]
    // This function is inherently unsafe, but no more unsafe than multiple accesses
    // to `pub static mut PIN` made directly, so okay to ignore this clippy lint
    // so long as the function is marked unsafe.
    pub unsafe fn get_pin_mut(&self) -> &mut Option<Pin<'static>> {
        let mut pad_num: u16 = *self as u16;

        // Right shift pad_num by 6 bits, so we can get rid of pin bits
        pad_num >>= 6;

        let mut pin_num: u8 = *self as u8;
        // Mask top 2 bits, so can get only the suffix
        pin_num &= 0b00111111;

        &mut PIN[usize::from(pad_num)][usize::from(pin_num)]
    }

    pub fn get_port(&self) -> &Port {
        let port_num: GpioPort = self.get_port_number();

        match port_num {
            GpioPort::GPIO1 => unsafe { &PORT[0] },
            GpioPort::GPIO2 => unsafe { &PORT[1] },
            GpioPort::GPIO3 => unsafe { &PORT[2] },
            GpioPort::GPIO4 => unsafe { &PORT[3] },
            GpioPort::GPIO5 => unsafe { &PORT[4] },
        }
    }

    // extract the last 6 bits. [6:0] is the pin number, [9:7] is the pad
    // number
    pub fn get_pin_number(&self) -> u8 {
        let mut pin_num = *self as u8;

        pin_num = pin_num & 0b00111111;
        pin_num
    }
}

enum_from_primitive! {
    #[repr(u32)]
    #[derive(PartialEq)]

    /// GPIO pin mode
    /// In order to set alternate functions such as LPI2C or LPUART,
    /// you will need to use iomuxc enable_sw_mux_ctl_pad_gpio with
    /// the specific MUX_MODE according to the reference manual (Chapter 11).
    /// For the gpio mode, input or output we set the GDIR pin accordingly [^1]
    ///
    /// [^1]: 12.4.3. GPIO Programming, page 1008 of the Reference Manual
    pub enum Mode {
        Input = 0b00,
        Output = 0b01
    }
}

pub struct Port {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock,
}

pub static mut PORT: [Port; 5] = [
    Port {
        registers: GPIO1_BASE,
        clock: PortClock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPIO1)),
    },
    Port {
        registers: GPIO2_BASE,
        clock: PortClock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPIO1)),
    },
    Port {
        registers: GPIO3_BASE,
        clock: PortClock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPIO1)),
    },
    Port {
        registers: GPIO4_BASE,
        clock: PortClock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPIO1)),
    },
    Port {
        registers: GPIO5_BASE,
        clock: PortClock(ccm::PeripheralClock::CCGR1(ccm::HCLK1::GPIO1)),
    },
];

impl Port {
    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn handle_interrupt(&self, gpio_port: GpioPort) {
        let mut isr_val: u32 = 0;
        let mut imr_val: u32 = self.registers.imr.get();

        // Read the `ISR` register and toggle the appropriate bits in
        // `isr`. Once that is done, write the value of `isr` back. We
        // can have a situation where memory value of `ISR` could have
        // changed due to an external interrupt. `ISR` is a read/clear write
        // 1 register (`rc_w1`). So, we only clear bits whose value has been
        // transferred to `isr`.
        unsafe {
            atomic(|| {
                isr_val = self.registers.isr.get();
                self.registers.isr.set(isr_val);
            });
        }

        let mut flagged_bit = 0;

        // stay in loop until we have processed all the flagged event bits
        while isr_val != 0 && imr_val != 0 {
            if (isr_val & 0b1) != 0 && (imr_val & 0b1) != 0 {
                let mut pin_num = usize::from(flagged_bit as u8);

                // depending on the gpio_port and the pin_number in gpio port
                // we determine the actual pin from the PIN array
                let pad_num = match gpio_port {
                    GpioPort::GPIO1 => {
                        if pin_num < 16 {
                            1
                        } else {
                            pin_num -= 16;
                            2
                        }
                    }
                    GpioPort::GPIO2 => {
                        if pin_num < 16 {
                            3
                        } else {
                            pin_num -= 16;
                            4
                        }
                    }
                    GpioPort::GPIO3 => {
                        if pin_num < 12 {
                            6
                        } else if pin_num < 18 {
                            pin_num -= 18;
                            5
                        } else {
                            pin_num += 32;
                            0
                        }
                    }
                    GpioPort::GPIO4 => 0,
                    GpioPort::GPIO5 => 7,
                };

                unsafe {
                    let pin = &PIN[pad_num][(pin_num as usize)];
                    match pin {
                        Some(val) => {
                            val.handle_interrupt();
                        }
                        None => {
                            panic!(
                                "Tried to access wrong pin from internal array {} {}",
                                pad_num, pin_num
                            );
                        }
                    }
                }
            }
            // move to next bit
            flagged_bit += 1;
            isr_val >>= 1;
            imr_val >>= 1;
        }
    }
}

struct PortClock(ccm::PeripheralClock);

impl ClockInterface for PortClock {
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
    pinid: PinId,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
}

macro_rules! declare_gpio_pins {
    ($($pin:ident)*) => {
        [
            $(Some(Pin::new(PinId::$pin)), )*
        ]
    }
}

// We need to use `Option<Pin>`, instead of just `Pin` because AdB0 for
// example has only sixteen pins - from AdB0_00 to AdB0_15, rather than
// the 42 pins needed for Emc.
pub static mut PIN: [[Option<Pin<'static>>; 42]; 8] = [
    declare_gpio_pins! {
        Emc00 Emc01 Emc02 Emc03 Emc04 Emc05 Emc06 Emc07
        Emc08 Emc09 Emc10 Emc11 Emc12 Emc13 Emc14 Emc15
        Emc16 Emc17 Emc18 Emc19 Emc20 Emc21 Emc22 Emc23
        Emc24 Emc25 Emc26 Emc27 Emc28 Emc29 Emc30 Emc31
        Emc32 Emc33 Emc34 Emc35 Emc36 Emc37 Emc38 Emc39
        Emc40 Emc41
    },
    [
        Some(Pin::new(PinId::AdB0_00)),
        Some(Pin::new(PinId::AdB0_01)),
        Some(Pin::new(PinId::AdB0_02)),
        Some(Pin::new(PinId::AdB0_03)),
        Some(Pin::new(PinId::AdB0_04)),
        Some(Pin::new(PinId::AdB0_05)),
        Some(Pin::new(PinId::AdB0_06)),
        Some(Pin::new(PinId::AdB0_07)),
        Some(Pin::new(PinId::AdB0_08)),
        Some(Pin::new(PinId::AdB0_09)),
        Some(Pin::new(PinId::AdB0_10)),
        Some(Pin::new(PinId::AdB0_11)),
        Some(Pin::new(PinId::AdB0_12)),
        Some(Pin::new(PinId::AdB0_13)),
        Some(Pin::new(PinId::AdB0_14)),
        Some(Pin::new(PinId::AdB0_15)),
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
    [
        Some(Pin::new(PinId::AdB1_00)),
        Some(Pin::new(PinId::AdB1_01)),
        Some(Pin::new(PinId::AdB1_02)),
        Some(Pin::new(PinId::AdB1_03)),
        Some(Pin::new(PinId::AdB1_04)),
        Some(Pin::new(PinId::AdB1_05)),
        Some(Pin::new(PinId::AdB1_06)),
        Some(Pin::new(PinId::AdB1_07)),
        Some(Pin::new(PinId::AdB1_08)),
        Some(Pin::new(PinId::AdB1_09)),
        Some(Pin::new(PinId::AdB1_10)),
        Some(Pin::new(PinId::AdB1_11)),
        Some(Pin::new(PinId::AdB1_12)),
        Some(Pin::new(PinId::AdB1_13)),
        Some(Pin::new(PinId::AdB1_14)),
        Some(Pin::new(PinId::AdB1_15)),
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
    [
        Some(Pin::new(PinId::B0_00)),
        Some(Pin::new(PinId::B0_01)),
        Some(Pin::new(PinId::B0_02)),
        Some(Pin::new(PinId::B0_03)),
        Some(Pin::new(PinId::B0_04)),
        Some(Pin::new(PinId::B0_05)),
        Some(Pin::new(PinId::B0_06)),
        Some(Pin::new(PinId::B0_07)),
        Some(Pin::new(PinId::B0_08)),
        Some(Pin::new(PinId::B0_09)),
        Some(Pin::new(PinId::B0_10)),
        Some(Pin::new(PinId::B0_11)),
        Some(Pin::new(PinId::B0_12)),
        Some(Pin::new(PinId::B0_13)),
        Some(Pin::new(PinId::B0_14)),
        Some(Pin::new(PinId::B0_15)),
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
    [
        Some(Pin::new(PinId::B1_00)),
        Some(Pin::new(PinId::B1_01)),
        Some(Pin::new(PinId::B1_02)),
        Some(Pin::new(PinId::B1_03)),
        Some(Pin::new(PinId::B1_04)),
        Some(Pin::new(PinId::B1_05)),
        Some(Pin::new(PinId::B1_06)),
        Some(Pin::new(PinId::B1_07)),
        Some(Pin::new(PinId::B1_08)),
        Some(Pin::new(PinId::B1_09)),
        Some(Pin::new(PinId::B1_10)),
        Some(Pin::new(PinId::B1_11)),
        Some(Pin::new(PinId::B1_12)),
        Some(Pin::new(PinId::B1_13)),
        Some(Pin::new(PinId::B1_14)),
        Some(Pin::new(PinId::B1_15)),
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
    [
        Some(Pin::new(PinId::SdB0_00)),
        Some(Pin::new(PinId::SdB0_01)),
        Some(Pin::new(PinId::SdB0_02)),
        Some(Pin::new(PinId::SdB0_03)),
        Some(Pin::new(PinId::SdB0_04)),
        Some(Pin::new(PinId::SdB0_05)),
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
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(Pin::new(PinId::SdB1_00)),
        Some(Pin::new(PinId::SdB1_01)),
        Some(Pin::new(PinId::SdB1_02)),
        Some(Pin::new(PinId::SdB1_03)),
        Some(Pin::new(PinId::SdB1_04)),
        Some(Pin::new(PinId::SdB1_05)),
        Some(Pin::new(PinId::SdB1_06)),
        Some(Pin::new(PinId::SdB1_07)),
        Some(Pin::new(PinId::SdB1_08)),
        Some(Pin::new(PinId::SdB1_09)),
        Some(Pin::new(PinId::SdB1_10)),
        Some(Pin::new(PinId::SdB1_11)),
        Some(Pin::new(PinId::SdB1_12)),
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
        None,
    ],
    [
        Some(Pin::new(PinId::Wakeup)),
        Some(Pin::new(PinId::PmicOnReq)),
        Some(Pin::new(PinId::PmicStbyReq)),
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
];

impl<'a> Pin<'a> {
    const fn new(pinid: PinId) -> Pin<'a> {
        Pin {
            pinid: pinid,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| client.fired());
    }

    pub fn get_mode(&self) -> Mode {
        let port = self.pinid.get_port();

        let val = match self.pinid.get_pin_number() {
            0b000000 => port.registers.gdir.read(GDIR::GDIR0),
            0b000001 => port.registers.gdir.read(GDIR::GDIR1),
            0b000010 => port.registers.gdir.read(GDIR::GDIR2),
            0b000011 => port.registers.gdir.read(GDIR::GDIR3),
            0b000100 => port.registers.gdir.read(GDIR::GDIR4),
            0b000101 => port.registers.gdir.read(GDIR::GDIR5),
            0b000110 => port.registers.gdir.read(GDIR::GDIR6),
            0b000111 => port.registers.gdir.read(GDIR::GDIR7),
            0b001000 => port.registers.gdir.read(GDIR::GDIR8),
            0b001001 => port.registers.gdir.read(GDIR::GDIR9),
            0b001010 => port.registers.gdir.read(GDIR::GDIR10),
            0b001011 => port.registers.gdir.read(GDIR::GDIR11),
            0b001100 => port.registers.gdir.read(GDIR::GDIR12),
            0b001101 => port.registers.gdir.read(GDIR::GDIR13),
            0b001110 => port.registers.gdir.read(GDIR::GDIR14),
            0b001111 => port.registers.gdir.read(GDIR::GDIR15),
            0b010000 => port.registers.gdir.read(GDIR::GDIR16),
            0b010001 => port.registers.gdir.read(GDIR::GDIR17),
            0b010010 => port.registers.gdir.read(GDIR::GDIR18),
            0b010011 => port.registers.gdir.read(GDIR::GDIR19),
            0b010100 => port.registers.gdir.read(GDIR::GDIR20),
            0b010101 => port.registers.gdir.read(GDIR::GDIR21),
            0b010110 => port.registers.gdir.read(GDIR::GDIR22),
            0b010111 => port.registers.gdir.read(GDIR::GDIR23),
            0b011000 => port.registers.gdir.read(GDIR::GDIR24),
            0b011001 => port.registers.gdir.read(GDIR::GDIR25),
            0b011010 => port.registers.gdir.read(GDIR::GDIR26),
            0b011011 => port.registers.gdir.read(GDIR::GDIR27),
            0b011100 => port.registers.gdir.read(GDIR::GDIR28),
            0b011101 => port.registers.gdir.read(GDIR::GDIR29),
            0b011110 => port.registers.gdir.read(GDIR::GDIR30),
            0b011111 => port.registers.gdir.read(GDIR::GDIR31),
            _ => 0,
        };

        Mode::from_u32(val).unwrap_or(Mode::Input)
    }

    pub fn set_mode(&self, mode: Mode) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b000000 => {
                port.registers.gdir.modify(GDIR::GDIR0.val(mode as u32));
            }
            0b000001 => {
                port.registers.gdir.modify(GDIR::GDIR1.val(mode as u32));
            }
            0b000010 => {
                port.registers.gdir.modify(GDIR::GDIR2.val(mode as u32));
            }
            0b000011 => {
                port.registers.gdir.modify(GDIR::GDIR3.val(mode as u32));
            }
            0b000100 => {
                port.registers.gdir.modify(GDIR::GDIR4.val(mode as u32));
            }
            0b000101 => {
                port.registers.gdir.modify(GDIR::GDIR5.val(mode as u32));
            }
            0b000110 => {
                port.registers.gdir.modify(GDIR::GDIR6.val(mode as u32));
            }
            0b000111 => {
                port.registers.gdir.modify(GDIR::GDIR7.val(mode as u32));
            }
            0b001000 => {
                port.registers.gdir.modify(GDIR::GDIR8.val(mode as u32));
            }
            0b001001 => {
                port.registers.gdir.modify(GDIR::GDIR9.val(mode as u32));
            }
            0b001010 => {
                port.registers.gdir.modify(GDIR::GDIR10.val(mode as u32));
            }
            0b001011 => {
                port.registers.gdir.modify(GDIR::GDIR11.val(mode as u32));
            }
            0b001100 => {
                port.registers.gdir.modify(GDIR::GDIR12.val(mode as u32));
            }
            0b001101 => {
                port.registers.gdir.modify(GDIR::GDIR13.val(mode as u32));
            }
            0b001110 => {
                port.registers.gdir.modify(GDIR::GDIR14.val(mode as u32));
            }
            0b001111 => {
                port.registers.gdir.modify(GDIR::GDIR15.val(mode as u32));
            }
            0b010000 => {
                port.registers.gdir.modify(GDIR::GDIR16.val(mode as u32));
            }
            0b010001 => {
                port.registers.gdir.modify(GDIR::GDIR17.val(mode as u32));
            }
            0b010010 => {
                port.registers.gdir.modify(GDIR::GDIR18.val(mode as u32));
            }
            0b010011 => {
                port.registers.gdir.modify(GDIR::GDIR19.val(mode as u32));
            }
            0b010100 => {
                port.registers.gdir.modify(GDIR::GDIR20.val(mode as u32));
            }
            0b010101 => {
                port.registers.gdir.modify(GDIR::GDIR21.val(mode as u32));
            }
            0b010110 => {
                port.registers.gdir.modify(GDIR::GDIR22.val(mode as u32));
            }
            0b010111 => {
                port.registers.gdir.modify(GDIR::GDIR23.val(mode as u32));
            }
            0b011000 => {
                port.registers.gdir.modify(GDIR::GDIR24.val(mode as u32));
            }
            0b011001 => {
                port.registers.gdir.modify(GDIR::GDIR25.val(mode as u32));
            }
            0b011010 => {
                port.registers.gdir.modify(GDIR::GDIR26.val(mode as u32));
            }
            0b011011 => {
                port.registers.gdir.modify(GDIR::GDIR27.val(mode as u32));
            }
            0b011100 => {
                port.registers.gdir.modify(GDIR::GDIR28.val(mode as u32));
            }
            0b011101 => {
                port.registers.gdir.modify(GDIR::GDIR29.val(mode as u32));
            }
            0b011110 => {
                port.registers.gdir.modify(GDIR::GDIR30.val(mode as u32));
            }
            0b011111 => {
                port.registers.gdir.modify(GDIR::GDIR31.val(mode as u32));
            }
            _ => {}
        }
    }

    pub fn get_pinid(&self) -> PinId {
        self.pinid
    }

    fn set_output_high(&self) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b000000 => {
                port.registers.dr.write(DR::DR0::SET);
            }
            0b000001 => {
                port.registers.dr.write(DR::DR1::SET);
            }
            0b000010 => {
                port.registers.dr.write(DR::DR2::SET);
            }
            0b000011 => {
                port.registers.dr.write(DR::DR3::SET);
            }
            0b000100 => {
                port.registers.dr.write(DR::DR4::SET);
            }
            0b000101 => {
                port.registers.dr.write(DR::DR5::SET);
            }
            0b000110 => {
                port.registers.dr.write(DR::DR6::SET);
            }
            0b000111 => {
                port.registers.dr.write(DR::DR7::SET);
            }
            0b001000 => {
                port.registers.dr.write(DR::DR8::SET);
            }
            0b001001 => {
                port.registers.dr.write(DR::DR9::SET);
            }
            0b001010 => {
                port.registers.dr.write(DR::DR10::SET);
            }
            0b001011 => {
                port.registers.dr.write(DR::DR11::SET);
            }
            0b001100 => {
                port.registers.dr.write(DR::DR12::SET);
            }
            0b001101 => {
                port.registers.dr.write(DR::DR13::SET);
            }
            0b001110 => {
                port.registers.dr.write(DR::DR14::SET);
            }
            0b001111 => {
                port.registers.dr.write(DR::DR15::SET);
            }
            0b010000 => {
                port.registers.dr.write(DR::DR16::SET);
            }
            0b010001 => {
                port.registers.dr.write(DR::DR17::SET);
            }
            0b010010 => {
                port.registers.dr.write(DR::DR18::SET);
            }
            0b010011 => {
                port.registers.dr.write(DR::DR19::SET);
            }
            0b010100 => {
                port.registers.dr.write(DR::DR20::SET);
            }
            0b010101 => {
                port.registers.dr.write(DR::DR21::SET);
            }
            0b010110 => {
                port.registers.dr.write(DR::DR22::SET);
            }
            0b010111 => {
                port.registers.dr.write(DR::DR23::SET);
            }
            0b011000 => {
                port.registers.dr.write(DR::DR24::SET);
            }
            0b011001 => {
                port.registers.dr.write(DR::DR25::SET);
            }
            0b011010 => {
                port.registers.dr.write(DR::DR26::SET);
            }
            0b011011 => {
                port.registers.dr.write(DR::DR27::SET);
            }
            0b011100 => {
                port.registers.dr.write(DR::DR28::SET);
            }
            0b011101 => {
                port.registers.dr.write(DR::DR29::SET);
            }
            0b011110 => {
                port.registers.dr.write(DR::DR30::SET);
            }
            0b011111 => {
                port.registers.dr.write(DR::DR31::SET);
            }
            _ => {}
        }
    }

    fn set_output_low(&self) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b000000 => {
                port.registers.dr.write(DR::DR0::CLEAR);
            }
            0b000001 => {
                port.registers.dr.write(DR::DR1::CLEAR);
            }
            0b000010 => {
                port.registers.dr.write(DR::DR2::CLEAR);
            }
            0b000011 => {
                port.registers.dr.write(DR::DR3::CLEAR);
            }
            0b000100 => {
                port.registers.dr.write(DR::DR4::CLEAR);
            }
            0b000101 => {
                port.registers.dr.write(DR::DR5::CLEAR);
            }
            0b000110 => {
                port.registers.dr.write(DR::DR6::CLEAR);
            }
            0b000111 => {
                port.registers.dr.write(DR::DR7::CLEAR);
            }
            0b001000 => {
                port.registers.dr.write(DR::DR8::CLEAR);
            }
            0b001001 => {
                port.registers.dr.write(DR::DR9::CLEAR);
            }
            0b001010 => {
                port.registers.dr.write(DR::DR10::CLEAR);
            }
            0b001011 => {
                port.registers.dr.write(DR::DR11::CLEAR);
            }
            0b001100 => {
                port.registers.dr.write(DR::DR12::CLEAR);
            }
            0b001101 => {
                port.registers.dr.write(DR::DR13::CLEAR);
            }
            0b001110 => {
                port.registers.dr.write(DR::DR14::CLEAR);
            }
            0b001111 => {
                port.registers.dr.write(DR::DR15::CLEAR);
            }
            0b010000 => {
                port.registers.dr.write(DR::DR16::CLEAR);
            }
            0b010001 => {
                port.registers.dr.write(DR::DR17::CLEAR);
            }
            0b010010 => {
                port.registers.dr.write(DR::DR18::CLEAR);
            }
            0b010011 => {
                port.registers.dr.write(DR::DR19::CLEAR);
            }
            0b010100 => {
                port.registers.dr.write(DR::DR20::CLEAR);
            }
            0b010101 => {
                port.registers.dr.write(DR::DR21::CLEAR);
            }
            0b010110 => {
                port.registers.dr.write(DR::DR22::CLEAR);
            }
            0b010111 => {
                port.registers.dr.write(DR::DR23::CLEAR);
            }
            0b011000 => {
                port.registers.dr.write(DR::DR24::CLEAR);
            }
            0b011001 => {
                port.registers.dr.write(DR::DR25::CLEAR);
            }
            0b011010 => {
                port.registers.dr.write(DR::DR26::CLEAR);
            }
            0b011011 => {
                port.registers.dr.write(DR::DR27::CLEAR);
            }
            0b011100 => {
                port.registers.dr.write(DR::DR28::CLEAR);
            }
            0b011101 => {
                port.registers.dr.write(DR::DR29::CLEAR);
            }
            0b011110 => {
                port.registers.dr.write(DR::DR30::CLEAR);
            }
            0b011111 => {
                port.registers.dr.write(DR::DR31::CLEAR);
            }
            _ => {}
        }
    }

    fn is_output_high(&self) -> bool {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b000000 => port.registers.dr.is_set(DR::DR0),
            0b000001 => port.registers.dr.is_set(DR::DR1),
            0b000010 => port.registers.dr.is_set(DR::DR2),
            0b000011 => port.registers.dr.is_set(DR::DR3),
            0b000100 => port.registers.dr.is_set(DR::DR4),
            0b000101 => port.registers.dr.is_set(DR::DR5),
            0b000110 => port.registers.dr.is_set(DR::DR6),
            0b000111 => port.registers.dr.is_set(DR::DR7),
            0b001000 => port.registers.dr.is_set(DR::DR8),
            0b001001 => port.registers.dr.is_set(DR::DR9),
            0b001010 => port.registers.dr.is_set(DR::DR10),
            0b001011 => port.registers.dr.is_set(DR::DR11),
            0b001100 => port.registers.dr.is_set(DR::DR12),
            0b001101 => port.registers.dr.is_set(DR::DR13),
            0b001110 => port.registers.dr.is_set(DR::DR14),
            0b001111 => port.registers.dr.is_set(DR::DR15),
            0b010000 => port.registers.dr.is_set(DR::DR16),
            0b010001 => port.registers.dr.is_set(DR::DR17),
            0b010010 => port.registers.dr.is_set(DR::DR18),
            0b010011 => port.registers.dr.is_set(DR::DR19),
            0b010100 => port.registers.dr.is_set(DR::DR20),
            0b010101 => port.registers.dr.is_set(DR::DR21),
            0b010110 => port.registers.dr.is_set(DR::DR22),
            0b010111 => port.registers.dr.is_set(DR::DR23),
            0b011000 => port.registers.dr.is_set(DR::DR24),
            0b011001 => port.registers.dr.is_set(DR::DR25),
            0b011010 => port.registers.dr.is_set(DR::DR26),
            0b011011 => port.registers.dr.is_set(DR::DR27),
            0b011100 => port.registers.dr.is_set(DR::DR28),
            0b011101 => port.registers.dr.is_set(DR::DR29),
            0b011110 => port.registers.dr.is_set(DR::DR30),
            0b011111 => port.registers.dr.is_set(DR::DR31),
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

    pub fn read_input(&self) -> bool {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b000000 => port.registers.dr.is_set(DR::DR0),
            0b000001 => port.registers.dr.is_set(DR::DR1),
            0b000010 => port.registers.dr.is_set(DR::DR2),
            0b000011 => port.registers.dr.is_set(DR::DR3),
            0b000100 => port.registers.dr.is_set(DR::DR4),
            0b000101 => port.registers.dr.is_set(DR::DR5),
            0b000110 => port.registers.dr.is_set(DR::DR6),
            0b000111 => port.registers.dr.is_set(DR::DR7),
            0b001000 => port.registers.dr.is_set(DR::DR8),
            0b001001 => port.registers.dr.is_set(DR::DR9),
            0b001010 => port.registers.dr.is_set(DR::DR10),
            0b001011 => port.registers.dr.is_set(DR::DR11),
            0b001100 => port.registers.dr.is_set(DR::DR12),
            0b001101 => port.registers.dr.is_set(DR::DR13),
            0b001110 => port.registers.dr.is_set(DR::DR14),
            0b001111 => port.registers.dr.is_set(DR::DR15),
            0b010000 => port.registers.dr.is_set(DR::DR16),
            0b010001 => port.registers.dr.is_set(DR::DR17),
            0b010010 => port.registers.dr.is_set(DR::DR18),
            0b010011 => port.registers.dr.is_set(DR::DR19),
            0b010100 => port.registers.dr.is_set(DR::DR20),
            0b010101 => port.registers.dr.is_set(DR::DR21),
            0b010110 => port.registers.dr.is_set(DR::DR22),
            0b010111 => port.registers.dr.is_set(DR::DR23),
            0b011000 => port.registers.dr.is_set(DR::DR24),
            0b011001 => port.registers.dr.is_set(DR::DR25),
            0b011010 => port.registers.dr.is_set(DR::DR26),
            0b011011 => port.registers.dr.is_set(DR::DR27),
            0b011100 => port.registers.dr.is_set(DR::DR28),
            0b011101 => port.registers.dr.is_set(DR::DR29),
            0b011110 => port.registers.dr.is_set(DR::DR30),
            0b011111 => port.registers.dr.is_set(DR::DR31),
            _ => false,
        }
    }

    fn mask_interrupt(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => port.registers.imr.write(IMR::IMR0::CLEAR),
            0b000001 => port.registers.imr.write(IMR::IMR1::CLEAR),
            0b000010 => port.registers.imr.write(IMR::IMR2::CLEAR),
            0b000011 => port.registers.imr.write(IMR::IMR3::CLEAR),
            0b000100 => port.registers.imr.write(IMR::IMR4::CLEAR),
            0b000101 => port.registers.imr.write(IMR::IMR5::CLEAR),
            0b000110 => port.registers.imr.write(IMR::IMR6::CLEAR),
            0b000111 => port.registers.imr.write(IMR::IMR7::CLEAR),
            0b001000 => port.registers.imr.write(IMR::IMR8::CLEAR),
            0b001001 => port.registers.imr.write(IMR::IMR9::CLEAR),
            0b001010 => port.registers.imr.write(IMR::IMR10::CLEAR),
            0b001011 => port.registers.imr.write(IMR::IMR11::CLEAR),
            0b001100 => port.registers.imr.write(IMR::IMR12::CLEAR),
            0b001101 => port.registers.imr.write(IMR::IMR13::CLEAR),
            0b001110 => port.registers.imr.write(IMR::IMR14::CLEAR),
            0b001111 => port.registers.imr.write(IMR::IMR15::CLEAR),
            0b010000 => port.registers.imr.write(IMR::IMR16::CLEAR),
            0b010001 => port.registers.imr.write(IMR::IMR17::CLEAR),
            0b010010 => port.registers.imr.write(IMR::IMR18::CLEAR),
            0b010011 => port.registers.imr.write(IMR::IMR19::CLEAR),
            0b010100 => port.registers.imr.write(IMR::IMR20::CLEAR),
            0b010101 => port.registers.imr.write(IMR::IMR21::CLEAR),
            0b010110 => port.registers.imr.write(IMR::IMR22::CLEAR),
            0b010111 => port.registers.imr.write(IMR::IMR23::CLEAR),
            0b011000 => port.registers.imr.write(IMR::IMR24::CLEAR),
            0b011001 => port.registers.imr.write(IMR::IMR25::CLEAR),
            0b011010 => port.registers.imr.write(IMR::IMR26::CLEAR),
            0b011011 => port.registers.imr.write(IMR::IMR27::CLEAR),
            0b011100 => port.registers.imr.write(IMR::IMR28::CLEAR),
            0b011101 => port.registers.imr.write(IMR::IMR29::CLEAR),
            0b011110 => port.registers.imr.write(IMR::IMR30::CLEAR),
            0b011111 => port.registers.imr.write(IMR::IMR31::CLEAR),
            _ => {}
        }
    }

    fn unmask_interrupt(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => port.registers.imr.write(IMR::IMR0::SET),
            0b000001 => port.registers.imr.write(IMR::IMR1::SET),
            0b000010 => port.registers.imr.write(IMR::IMR2::SET),
            0b000011 => port.registers.imr.write(IMR::IMR3::SET),
            0b000100 => port.registers.imr.write(IMR::IMR4::SET),
            0b000101 => port.registers.imr.write(IMR::IMR5::SET),
            0b000110 => port.registers.imr.write(IMR::IMR6::SET),
            0b000111 => port.registers.imr.write(IMR::IMR7::SET),
            0b001000 => port.registers.imr.write(IMR::IMR8::SET),
            0b001001 => port.registers.imr.write(IMR::IMR9::SET),
            0b001010 => port.registers.imr.write(IMR::IMR10::SET),
            0b001011 => port.registers.imr.write(IMR::IMR11::SET),
            0b001100 => port.registers.imr.write(IMR::IMR12::SET),
            0b001101 => port.registers.imr.write(IMR::IMR13::SET),
            0b001110 => port.registers.imr.write(IMR::IMR14::SET),
            0b001111 => port.registers.imr.write(IMR::IMR15::SET),
            0b010000 => port.registers.imr.write(IMR::IMR16::SET),
            0b010001 => port.registers.imr.write(IMR::IMR17::SET),
            0b010010 => port.registers.imr.write(IMR::IMR18::SET),
            0b010011 => port.registers.imr.write(IMR::IMR19::SET),
            0b010100 => port.registers.imr.write(IMR::IMR20::SET),
            0b010101 => port.registers.imr.write(IMR::IMR21::SET),
            0b010110 => port.registers.imr.write(IMR::IMR22::SET),
            0b010111 => port.registers.imr.write(IMR::IMR23::SET),
            0b011000 => port.registers.imr.write(IMR::IMR24::SET),
            0b011001 => port.registers.imr.write(IMR::IMR25::SET),
            0b011010 => port.registers.imr.write(IMR::IMR26::SET),
            0b011011 => port.registers.imr.write(IMR::IMR27::SET),
            0b011100 => port.registers.imr.write(IMR::IMR28::SET),
            0b011101 => port.registers.imr.write(IMR::IMR29::SET),
            0b011110 => port.registers.imr.write(IMR::IMR30::SET),
            0b011111 => port.registers.imr.write(IMR::IMR31::SET),
            _ => {}
        }
    }

    fn clear_pending(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => port.registers.isr.write(ISR::ISR0::SET),
            0b000001 => port.registers.isr.write(ISR::ISR1::SET),
            0b000010 => port.registers.isr.write(ISR::ISR2::SET),
            0b000011 => port.registers.isr.write(ISR::ISR3::SET),
            0b000100 => port.registers.isr.write(ISR::ISR4::SET),
            0b000101 => port.registers.isr.write(ISR::ISR5::SET),
            0b000110 => port.registers.isr.write(ISR::ISR6::SET),
            0b000111 => port.registers.isr.write(ISR::ISR7::SET),
            0b001000 => port.registers.isr.write(ISR::ISR8::SET),
            0b001001 => port.registers.isr.write(ISR::ISR9::SET),
            0b001010 => port.registers.isr.write(ISR::ISR10::SET),
            0b001011 => port.registers.isr.write(ISR::ISR11::SET),
            0b001100 => port.registers.isr.write(ISR::ISR12::SET),
            0b001101 => port.registers.isr.write(ISR::ISR13::SET),
            0b001110 => port.registers.isr.write(ISR::ISR14::SET),
            0b001111 => port.registers.isr.write(ISR::ISR15::SET),
            0b010000 => port.registers.isr.write(ISR::ISR16::SET),
            0b010001 => port.registers.isr.write(ISR::ISR17::SET),
            0b010010 => port.registers.isr.write(ISR::ISR18::SET),
            0b010011 => port.registers.isr.write(ISR::ISR19::SET),
            0b010100 => port.registers.isr.write(ISR::ISR20::SET),
            0b010101 => port.registers.isr.write(ISR::ISR21::SET),
            0b010110 => port.registers.isr.write(ISR::ISR22::SET),
            0b010111 => port.registers.isr.write(ISR::ISR23::SET),
            0b011000 => port.registers.isr.write(ISR::ISR24::SET),
            0b011001 => port.registers.isr.write(ISR::ISR25::SET),
            0b011010 => port.registers.isr.write(ISR::ISR26::SET),
            0b011011 => port.registers.isr.write(ISR::ISR27::SET),
            0b011100 => port.registers.isr.write(ISR::ISR28::SET),
            0b011101 => port.registers.isr.write(ISR::ISR29::SET),
            0b011110 => port.registers.isr.write(ISR::ISR30::SET),
            0b011111 => port.registers.isr.write(ISR::ISR31::SET),
            _ => {}
        }
    }

    // deselect either_edge first and then enable rising edge
    fn select_rising_trigger(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL0::CLEAR);
                port.registers.icr1.modify(ICR1::ICR0.val(0b10 as u32));
            }
            0b000001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL1::CLEAR);
                port.registers.icr1.modify(ICR1::ICR1.val(0b10 as u32));
            }
            0b000010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL2::CLEAR);
                port.registers.icr1.modify(ICR1::ICR2.val(0b10 as u32));
            }
            0b000011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL3::CLEAR);
                port.registers.icr1.modify(ICR1::ICR3.val(0b10 as u32));
            }
            0b000100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL4::CLEAR);
                port.registers.icr1.modify(ICR1::ICR4.val(0b10 as u32));
            }
            0b000101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL5::CLEAR);
                port.registers.icr1.modify(ICR1::ICR5.val(0b10 as u32));
            }
            0b000110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL6::CLEAR);
                port.registers.icr1.modify(ICR1::ICR6.val(0b10 as u32));
            }
            0b000111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL7::CLEAR);
                port.registers.icr1.modify(ICR1::ICR7.val(0b10 as u32));
            }
            0b001000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL8::CLEAR);
                port.registers.icr1.modify(ICR1::ICR8.val(0b10 as u32));
            }
            0b001001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL9::CLEAR);
                port.registers.icr1.modify(ICR1::ICR9.val(0b10 as u32));
            }
            0b001010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL10::CLEAR);
                port.registers.icr1.modify(ICR1::ICR10.val(0b10 as u32));
            }
            0b001011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL11::CLEAR);
                port.registers.icr1.modify(ICR1::ICR11.val(0b10 as u32));
            }
            0b001100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL12::CLEAR);
                port.registers.icr1.modify(ICR1::ICR12.val(0b10 as u32));
            }
            0b001101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL13::CLEAR);
                port.registers.icr1.modify(ICR1::ICR13.val(0b10 as u32));
            }
            0b001110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL14::CLEAR);
                port.registers.icr1.modify(ICR1::ICR14.val(0b10 as u32));
            }
            0b001111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL15::CLEAR);
                port.registers.icr1.modify(ICR1::ICR15.val(0b10 as u32));
            }
            0b010000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL16::CLEAR);
                port.registers.icr2.modify(ICR2::ICR16.val(0b10 as u32));
            }
            0b010001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL17::CLEAR);
                port.registers.icr2.modify(ICR2::ICR17.val(0b10 as u32));
            }
            0b010010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL18::CLEAR);
                port.registers.icr2.modify(ICR2::ICR18.val(0b10 as u32));
            }
            0b010011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL19::CLEAR);
                port.registers.icr2.modify(ICR2::ICR19.val(0b10 as u32));
            }
            0b010100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL20::CLEAR);
                port.registers.icr2.modify(ICR2::ICR20.val(0b10 as u32));
            }
            0b010101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL21::CLEAR);
                port.registers.icr2.modify(ICR2::ICR21.val(0b10 as u32));
            }
            0b010110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL22::CLEAR);
                port.registers.icr2.modify(ICR2::ICR22.val(0b10 as u32));
            }
            0b010111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL23::CLEAR);
                port.registers.icr2.modify(ICR2::ICR23.val(0b10 as u32));
            }
            0b011000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL24::CLEAR);
                port.registers.icr2.modify(ICR2::ICR24.val(0b10 as u32));
            }
            0b011001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL25::CLEAR);
                port.registers.icr2.modify(ICR2::ICR25.val(0b10 as u32));
            }
            0b011010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL26::CLEAR);
                port.registers.icr2.modify(ICR2::ICR26.val(0b10 as u32));
            }
            0b011011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL27::CLEAR);
                port.registers.icr2.modify(ICR2::ICR27.val(0b10 as u32));
            }
            0b011100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL28::CLEAR);
                port.registers.icr2.modify(ICR2::ICR28.val(0b10 as u32));
            }
            0b011101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL29::CLEAR);
                port.registers.icr2.modify(ICR2::ICR29.val(0b10 as u32));
            }
            0b011110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL30::CLEAR);
                port.registers.icr2.modify(ICR2::ICR30.val(0b10 as u32));
            }
            0b011111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL31::CLEAR);
                port.registers.icr2.modify(ICR2::ICR31.val(0b10 as u32));
            }
            _ => {}
        }
    }

    // deselect either_edge first and then enable falling edge
    fn select_falling_trigger(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL0::CLEAR);
                port.registers.icr1.modify(ICR1::ICR0.val(0b11 as u32));
            }
            0b000001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL1::CLEAR);
                port.registers.icr1.modify(ICR1::ICR1.val(0b11 as u32));
            }
            0b000010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL2::CLEAR);
                port.registers.icr1.modify(ICR1::ICR2.val(0b11 as u32));
            }
            0b000011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL3::CLEAR);
                port.registers.icr1.modify(ICR1::ICR3.val(0b11 as u32));
            }
            0b000100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL4::CLEAR);
                port.registers.icr1.modify(ICR1::ICR4.val(0b11 as u32));
            }
            0b000101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL5::CLEAR);
                port.registers.icr1.modify(ICR1::ICR5.val(0b11 as u32));
            }
            0b000110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL6::CLEAR);
                port.registers.icr1.modify(ICR1::ICR6.val(0b11 as u32));
            }
            0b000111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL7::CLEAR);
                port.registers.icr1.modify(ICR1::ICR7.val(0b11 as u32));
            }
            0b001000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL8::CLEAR);
                port.registers.icr1.modify(ICR1::ICR8.val(0b11 as u32));
            }
            0b001001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL9::CLEAR);
                port.registers.icr1.modify(ICR1::ICR9.val(0b11 as u32));
            }
            0b001010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL10::CLEAR);
                port.registers.icr1.modify(ICR1::ICR10.val(0b11 as u32));
            }
            0b001011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL11::CLEAR);
                port.registers.icr1.modify(ICR1::ICR11.val(0b11 as u32));
            }
            0b001100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL12::CLEAR);
                port.registers.icr1.modify(ICR1::ICR12.val(0b11 as u32));
            }
            0b001101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL13::CLEAR);
                port.registers.icr1.modify(ICR1::ICR13.val(0b11 as u32));
            }
            0b001110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL14::CLEAR);
                port.registers.icr1.modify(ICR1::ICR14.val(0b11 as u32));
            }
            0b001111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL15::CLEAR);
                port.registers.icr1.modify(ICR1::ICR15.val(0b11 as u32));
            }
            0b010000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL16::CLEAR);
                port.registers.icr2.modify(ICR2::ICR16.val(0b11 as u32));
            }
            0b010001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL17::CLEAR);
                port.registers.icr2.modify(ICR2::ICR17.val(0b11 as u32));
            }
            0b010010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL18::CLEAR);
                port.registers.icr2.modify(ICR2::ICR18.val(0b11 as u32));
            }
            0b010011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL19::CLEAR);
                port.registers.icr2.modify(ICR2::ICR19.val(0b11 as u32));
            }
            0b010100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL20::CLEAR);
                port.registers.icr2.modify(ICR2::ICR20.val(0b11 as u32));
            }
            0b010101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL21::CLEAR);
                port.registers.icr2.modify(ICR2::ICR21.val(0b11 as u32));
            }
            0b010110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL22::CLEAR);
                port.registers.icr2.modify(ICR2::ICR22.val(0b11 as u32));
            }
            0b010111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL23::CLEAR);
                port.registers.icr2.modify(ICR2::ICR23.val(0b11 as u32));
            }
            0b011000 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL24::CLEAR);
                port.registers.icr2.modify(ICR2::ICR24.val(0b11 as u32));
            }
            0b011001 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL25::CLEAR);
                port.registers.icr2.modify(ICR2::ICR25.val(0b11 as u32));
            }
            0b011010 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL26::CLEAR);
                port.registers.icr2.modify(ICR2::ICR26.val(0b11 as u32));
            }
            0b011011 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL27::CLEAR);
                port.registers.icr2.modify(ICR2::ICR27.val(0b11 as u32));
            }
            0b011100 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL28::CLEAR);
                port.registers.icr2.modify(ICR2::ICR28.val(0b11 as u32));
            }
            0b011101 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL29::CLEAR);
                port.registers.icr2.modify(ICR2::ICR29.val(0b11 as u32));
            }
            0b011110 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL30::CLEAR);
                port.registers.icr2.modify(ICR2::ICR30.val(0b11 as u32));
            }
            0b011111 => {
                port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL31::CLEAR);
                port.registers.icr2.modify(ICR2::ICR31.val(0b11 as u32));
            }
            _ => {}
        }
    }

    fn select_either_trigger(&self) {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL0::SET),
            0b000001 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL1::SET),
            0b000010 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL2::SET),
            0b000011 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL3::SET),
            0b000100 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL4::SET),
            0b000101 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL5::SET),
            0b000110 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL6::SET),
            0b000111 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL7::SET),
            0b001000 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL8::SET),
            0b001001 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL9::SET),
            0b001010 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL10::SET),
            0b001011 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL11::SET),
            0b001100 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL12::SET),
            0b001101 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL13::SET),
            0b001110 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL14::SET),
            0b001111 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL15::SET),
            0b010000 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL16::SET),
            0b010001 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL17::SET),
            0b010010 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL18::SET),
            0b010011 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL19::SET),
            0b010100 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL20::SET),
            0b010101 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL21::SET),
            0b010110 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL22::SET),
            0b010111 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL23::SET),
            0b011000 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL24::SET),
            0b011001 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL25::SET),
            0b011010 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL26::SET),
            0b011011 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL27::SET),
            0b011100 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL28::SET),
            0b011101 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL29::SET),
            0b011110 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL30::SET),
            0b011111 => port.registers.edge_sel.write(EDGE_SEL::EDGE_SEL31::SET),
            _ => {}
        }
    }
}

impl hil::gpio::Pin for Pin<'_> {}
impl<'a> hil::gpio::InterruptPin<'a> for Pin<'a> {}

impl hil::gpio::Configure for Pin<'_> {
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Output);
        hil::gpio::Configuration::Output
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Input);
        hil::gpio::Configuration::Input
    }

    fn deactivate_to_low_power(&self) {
        // Not implemented yet
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        // Not implemented yet
        hil::gpio::Configuration::LowPower
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // Not implemented yet
        hil::gpio::Configuration::LowPower
    }

    // PullUp or PullDown mode are set through the Iomux module
    fn set_floating_state(&self, _mode: hil::gpio::FloatingState) {}

    fn floating_state(&self) -> hil::gpio::FloatingState {
        hil::gpio::FloatingState::PullNone
    }

    fn configuration(&self) -> hil::gpio::Configuration {
        match self.get_mode() {
            Mode::Input => hil::gpio::Configuration::Input,
            Mode::Output => hil::gpio::Configuration::Output,
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
    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        unsafe {
            atomic(|| {
                // disable the interrupt
                self.mask_interrupt();
                self.clear_pending();

                match mode {
                    hil::gpio::InterruptEdge::EitherEdge => {
                        self.select_either_trigger();
                    }
                    hil::gpio::InterruptEdge::RisingEdge => {
                        self.select_rising_trigger();
                    }
                    hil::gpio::InterruptEdge::FallingEdge => {
                        self.select_falling_trigger();
                    }
                }

                self.unmask_interrupt();
            });
        }
    }

    fn disable_interrupts(&self) {
        unsafe {
            atomic(|| {
                self.mask_interrupt();
                self.clear_pending();
            });
        }
    }

    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        let port = self.pinid.get_port();
        match self.pinid.get_pin_number() {
            0b000000 => port.registers.isr.is_set(ISR::ISR0),
            0b000001 => port.registers.isr.is_set(ISR::ISR1),
            0b000010 => port.registers.isr.is_set(ISR::ISR2),
            0b000011 => port.registers.isr.is_set(ISR::ISR3),
            0b000100 => port.registers.isr.is_set(ISR::ISR4),
            0b000101 => port.registers.isr.is_set(ISR::ISR5),
            0b000110 => port.registers.isr.is_set(ISR::ISR6),
            0b000111 => port.registers.isr.is_set(ISR::ISR7),
            0b001000 => port.registers.isr.is_set(ISR::ISR8),
            0b001001 => port.registers.isr.is_set(ISR::ISR9),
            0b001010 => port.registers.isr.is_set(ISR::ISR10),
            0b001011 => port.registers.isr.is_set(ISR::ISR11),
            0b001100 => port.registers.isr.is_set(ISR::ISR12),
            0b001101 => port.registers.isr.is_set(ISR::ISR13),
            0b001110 => port.registers.isr.is_set(ISR::ISR14),
            0b001111 => port.registers.isr.is_set(ISR::ISR15),
            0b010000 => port.registers.isr.is_set(ISR::ISR16),
            0b010001 => port.registers.isr.is_set(ISR::ISR17),
            0b010010 => port.registers.isr.is_set(ISR::ISR18),
            0b010011 => port.registers.isr.is_set(ISR::ISR19),
            0b010100 => port.registers.isr.is_set(ISR::ISR20),
            0b010101 => port.registers.isr.is_set(ISR::ISR21),
            0b010110 => port.registers.isr.is_set(ISR::ISR22),
            0b010111 => port.registers.isr.is_set(ISR::ISR23),
            0b011000 => port.registers.isr.is_set(ISR::ISR24),
            0b011001 => port.registers.isr.is_set(ISR::ISR25),
            0b011010 => port.registers.isr.is_set(ISR::ISR26),
            0b011011 => port.registers.isr.is_set(ISR::ISR27),
            0b011100 => port.registers.isr.is_set(ISR::ISR28),
            0b011101 => port.registers.isr.is_set(ISR::ISR29),
            0b011110 => port.registers.isr.is_set(ISR::ISR30),
            0b011111 => port.registers.isr.is_set(ISR::ISR31),
            _ => false,
        }
    }
}
