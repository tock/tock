use cortexm7;
use cortexm7::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

// use crate::exti;
use crate::ccm;
use crate::iomuxc;

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
    dr_toggle: WriteOnly<u32, DR_TOGGLE::Register>
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

#[repr(u32)]
pub enum PortId {
    P1 = 0b000,
    P2 = 0b001,
    P3 = 0b010,
    P4 = 0b011,
    P5 = 0b100,
}


// Name of the GPIO pins 
#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PinId {
    P1_00 = 0b00000000, P1_01 = 0b00000001, P1_02 = 0b00000010, P1_03 = 0b00000011,
    P1_04 = 0b00000100, P1_05 = 0b00000101, P1_06 = 0b00000110, P1_07 = 0b00000111,
    P1_08 = 0b00001000, P1_09 = 0b00001001, P1_10 = 0b00001010, P1_11 = 0b00001011,
    P1_12 = 0b00001100, P1_13 = 0b00001101, P1_14 = 0b00001110, P1_15 = 0b00001111,
    P1_16 = 0b00010000, P1_17 = 0b00010001, P1_18 = 0b00010010, P1_19 = 0b00010011,
    P1_20 = 0b00010100, P1_21 = 0b00010101, P1_22 = 0b00010110, P1_23 = 0b00010111,
    P1_24 = 0b00011000, P1_25 = 0b00011001, P1_26 = 0b00011010, P1_27 = 0b00011011,
    P1_28 = 0b00011100, P1_29 = 0b00011101, P1_30 = 0b00011110, P1_31 = 0b00011111,

    P2_00 = 0b00100000, P2_01 = 0b00100001, P2_02 = 0b00100010, P2_03 = 0b00100011,
    P2_04 = 0b00100100, P2_05 = 0b00100101, P2_06 = 0b00100110, P2_07 = 0b00100111,
    P2_08 = 0b00101000, P2_09 = 0b00101001, P2_10 = 0b00101010, P2_11 = 0b00101011,
    P2_12 = 0b00101100, P2_13 = 0b00101101, P2_14 = 0b00101110, P2_15 = 0b00101111,
    P2_16 = 0b00110000, P2_17 = 0b00110001, P2_18 = 0b00110010, P2_19 = 0b00110011,
    P2_20 = 0b00110100, P2_21 = 0b00110101, P2_22 = 0b00110110, P2_23 = 0b00110111,
    P2_24 = 0b00111000, P2_25 = 0b00111001, P2_26 = 0b00111010, P2_27 = 0b00111011,
    P2_28 = 0b00111100, P2_29 = 0b00111101, P2_30 = 0b00111110, P2_31 = 0b00111111,

    P3_00 = 0b01000000, P3_01 = 0b01000001, P3_02 = 0b01000010, P3_03 = 0b01000011,
    P3_04 = 0b01000100, P3_05 = 0b01000101, P3_06 = 0b01000110, P3_07 = 0b01000111,
    P3_08 = 0b01001000, P3_09 = 0b01001001, P3_10 = 0b01001010, P3_11 = 0b01001011,
    P3_12 = 0b01001100, P3_13 = 0b01001101, P3_14 = 0b01001110, P3_15 = 0b01001111,
    P3_16 = 0b01010000, P3_17 = 0b01010001, P3_18 = 0b01010010, P3_19 = 0b01010011,
    P3_20 = 0b01010100, P3_21 = 0b01010101, P3_22 = 0b01010110, P3_23 = 0b01010111,
    P3_24 = 0b01011000, P3_25 = 0b01011001, P3_26 = 0b01011010, P3_27 = 0b01011011,
    P3_28 = 0b01011100, P3_29 = 0b01011101, P3_30 = 0b01011110, P3_31 = 0b01011111,

    P4_00 = 0b01100000, P4_01 = 0b01100001, P4_02 = 0b01100010, P4_03 = 0b01100011,
    P4_04 = 0b01100100, P4_05 = 0b01100101, P4_06 = 0b01100110, P4_07 = 0b01100111,
    P4_08 = 0b01101000, P4_09 = 0b01101001, P4_10 = 0b01101010, P4_11 = 0b01101011,
    P4_12 = 0b01101100, P4_13 = 0b01101101, P4_14 = 0b01101110, P4_15 = 0b01101111,
    P4_16 = 0b01110000, P4_17 = 0b01110001, P4_18 = 0b01110010, P4_19 = 0b01110011,
    P4_20 = 0b01110100, P4_21 = 0b01110101, P4_22 = 0b01110110, P4_23 = 0b01110111,
    P4_24 = 0b01111000, P4_25 = 0b01111001, P4_26 = 0b01111010, P4_27 = 0b01111011,
    P4_28 = 0b01111100, P4_29 = 0b01111101, P4_30 = 0b01111110, P4_31 = 0b01111111,

    P5_00 = 0b10000000, P5_01 = 0b10000001, P5_02 = 0b10000010, P5_03 = 0b10000011,
    P5_04 = 0b10000100, P5_05 = 0b10000101, P5_06 = 0b10000110, P5_07 = 0b10000111,
    P5_08 = 0b10001000, P5_09 = 0b10001001, P5_10 = 0b10001010, P5_11 = 0b10001011,
    P5_12 = 0b10001100, P5_13 = 0b10001101, P5_14 = 0b10001110, P5_15 = 0b10001111,
    P5_16 = 0b10010000, P5_17 = 0b10010001, P5_18 = 0b10010010, P5_19 = 0b10010011,
    P5_20 = 0b10010100, P5_21 = 0b10010101, P5_22 = 0b10010110, P5_23 = 0b10010111,
    P5_24 = 0b10011000, P5_25 = 0b10011001, P5_26 = 0b10011010, P5_27 = 0b10011011,
    P5_28 = 0b10011100, P5_29 = 0b10011101, P5_30 = 0b10011110, P5_31 = 0b10011111,
}

impl PinId {
    pub fn get_pin(&self) -> &Option<Pin<'static>> {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 5;

        let mut pin_num: u8 = *self as u8;
        // Mask top 3 bits, so can get only the suffix
        pin_num &= 0b00011111;

        unsafe {&PIN[usize::from(port_num)][usize::from(pin_num)] }
    }

    pub fn get_pin_mut(&self) -> &mut Option<Pin<'static>> {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 5;

        let mut pin_num: u8 = *self as u8;
        // Mask top 3 bits, so can get only the suffix
        pin_num &= 0b00011111;

        unsafe { &mut PIN[usize::from(port_num)][usize::from(pin_num)] }
    }

    pub fn get_port(&self) -> &Port {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 5;
        unsafe { &PORT[usize::from(port_num)] }
    }

    // extract the last 4 bits. [3:0] is the pin number, [6:4] is the port
    // number
    pub fn get_pin_number(&self) -> u8 {
        let mut pin_num = *self as u8;

        pin_num = pin_num & 0b00011111;
        pin_num
    }

    // extract bits [6:4], which is the port number
    pub fn get_port_number(&self) -> u8 {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 5;
        port_num
    }
}

/// GPIO pin mode [^1]
///
/// [^1]: Section 7.1.4, page 187 of reference manual
enum_from_primitive! {
    #[repr(u32)]
    #[derive(PartialEq)]
    pub enum Mode {
        Input = 0b0,
        Output = 0b1
    }
}

/// Aici ar fi venit Alternative Functions...
#[repr(u32)]
pub enum AlternateFunction {
    None = 0
}

/// GPIO pin internal pull-up and pull-down [^1]
enum_from_primitive! {
    #[repr(u32)]
    enum PullUpPullDown {
        Pus0_100kOhmPullDown = 0b00,    // 100K Ohm Pull Down
        Pus1_47kOhmPullUp = 0b01,       // 47K Ohm Pull Up
        Pus2_100kOhmPullUp = 0b10,      // 100K Ohm Pull Up
        Pus3_22kOhmPullUp = 0b11,       // 22K Ohm Pull Up
    }
}

pub struct Port {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock,
}

pub static mut PORT: [Port; 1] = [
    Port {
        registers: GPIO1_BASE,
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

// no `exti_lineid` for the moment
pub struct Pin<'a> {
    pinid: PinId,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
    // exti_lineid: OptionalCell<exti::LineId>,
}

macro_rules! declare_gpio_pins {
    ($($pin:ident)*) => {
        [
            $(Some(Pin::new(PinId::$pin)), )*
        ]
    }
}

pub static mut PIN: [[Option<Pin<'static>>; 32]; 5] = [
    declare_gpio_pins! {
        P1_00 P1_01 P1_02 P1_03 P1_04 P1_05 P1_06 P1_07
        P1_08 P1_09 P1_10 P1_11 P1_12 P1_13 P1_14 P1_15
        P1_16 P1_17 P1_18 P1_19 P1_20 P1_21 P1_22 P1_23
        P1_24 P1_25 P1_26 P1_27 P1_28 P1_29 P1_30 P1_31
    },
    declare_gpio_pins! {
        P2_00 P2_01 P2_02 P2_03 P2_04 P2_05 P2_06 P2_07
        P2_08 P2_09 P2_10 P2_11 P2_12 P2_13 P2_14 P2_15
        P2_16 P2_17 P2_18 P2_19 P2_20 P2_21 P2_22 P2_23
        P2_24 P2_25 P2_26 P2_27 P2_28 P2_29 P2_30 P2_31
    },    
    declare_gpio_pins! {
        P3_00 P3_01 P3_02 P3_03 P3_04 P3_05 P3_06 P3_07
        P3_08 P3_09 P3_10 P3_11 P3_12 P3_13 P3_14 P3_15
        P3_16 P3_17 P3_18 P3_19 P3_20 P3_21 P3_22 P3_23
        P3_24 P3_25 P3_26 P3_27 P3_28 P3_29 P3_30 P3_31
    },
    declare_gpio_pins! {
        P4_00 P4_01 P4_02 P4_03 P4_04 P4_05 P4_06 P4_07
        P4_08 P4_09 P4_10 P4_11 P4_12 P4_13 P4_14 P4_15
        P4_16 P4_17 P4_18 P4_19 P4_20 P4_21 P4_22 P4_23
        P4_24 P4_25 P4_26 P4_27 P4_28 P4_29 P4_30 P4_31
    },
    declare_gpio_pins! {
        P5_00 P5_01 P5_02 P5_03 P5_04 P5_05 P5_06 P5_07
        P5_08 P5_09 P5_10 P5_11 P5_12 P5_13 P5_14 P5_15
        P5_16 P5_17 P5_18 P5_19 P5_20 P5_21 P5_22 P5_23
        P5_24 P5_25 P5_26 P5_27 P5_28 P5_29 P5_30 P5_31
    },
];

impl Pin<'a> {
    const fn new(pinid: PinId) -> Pin<'a> {
        Pin {
            pinid: pinid,
            client: OptionalCell::empty(),
            // no exti for the moment
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
            0b01001 => port.registers.gdir.read(GDIR::GDIR9),
            _ => 0,
        };

        Mode::from_u32(val).unwrap_or(Mode::Input)
    }

    pub fn set_mode(&self, mode: Mode) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b01001 => {
                unsafe {
                    iomuxc::IOMUXC.enable_gpio1_09();
                }
                port.registers.gdir.modify(GDIR::GDIR9.val(mode as u32));
            },
            0b10000 => {
                unsafe {
                    iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio_ad_b1_00_alt3_mode();
                    iomuxc::IOMUXC.enable_lpi2c_scl_select_input();
                    iomuxc::IOMUXC.enable_sw_mux_ctl_pad_gpio_ad_b1_01_alt3_mode();
                    iomuxc::IOMUXC.enable_lpi2c_sda_select_input();
                    iomuxc::IOMUXC.enable_lpi2c1_scl_16();
                    iomuxc::IOMUXC.enable_lpi2c1_sda_17();
                }
            },
            _ => {}
        }
    }

    // no alternate function for the moment
    pub fn set_alternate_function(&self, _af: AlternateFunction) {
        let _port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            // 0b1001 => port.registers.afrh.modify(AFRH::AFRH9.val(af as u32)),
            _ => {}
        }
    }

    pub fn get_pinid(&self) -> PinId {
        self.pinid
    }

    // no exti line for tge moment
    // pub fn set_exti_lineid(&self, lineid: exti::LineId) {
    //     self.exti_lineid.set(lineid);
    // }

    // none for the momenent
    fn set_mode_output_pushpull(&self) {
        let _port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            // 0b1001 => port.registers.otyper.modify(OTYPER::OT9::CLEAR),
            _ => {}
        }
    }

    // oarecum inutile momentan
    fn get_pullup_pulldown(&self) -> PullUpPullDown {
        let _port = self.pinid.get_port();

        let val = match self.pinid.get_pin_number() {
            // 0b01001 => iomuxc::registers.sw_pad_ctl_pad_gpio_ad_b0_09.read(SW_PAD_CTL_PAD_GPIO_AD_B0_09::PUS),
            _ => 0,
        };

        PullUpPullDown::from_u32(val).unwrap_or(PullUpPullDown::Pus0_100kOhmPullDown)
    }

    // oarecum inutile momentan
    fn set_pullup_pulldown(&self, _pupd: PullUpPullDown) {
        let _port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            // 0b01001 => { 
            //     iomuxc::IOMUXC.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::PKE::SET);   
            //     iomuxc.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::PUS.val(pupd as u32));
            // },
            _ => {}
        }
    }

    fn set_output_high(&self) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b01001 => {
                port.registers.dr.write(DR::DR9::SET);
            },
            _ => {}
        }
    }

    fn set_output_low(&self) {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b01001 => port.registers.dr.write(DR::DR9::CLEAR),
            _ => {}
        }
    }

    fn is_output_high(&self) -> bool {
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b01001 => port.registers.dr.is_set(DR::DR9) , 
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
        let port = self.pinid.get_port();

        match self.pinid.get_pin_number() {
            0b1001 => port.registers.dr.is_set(DR::DR9),
            _ => false,
        }
    }
}

impl hil::gpio::Pin for Pin<'a> {}
impl hil::gpio::InterruptPin for Pin<'a> {}

impl hil::gpio::Configure for Pin<'a> {
    /// Output mode default is push-pull
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_mode(Mode::Output);
        // self.set_mode_output_pushpull();
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
        // self.set_mode(Mode::AnalogMode);
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        // self.set_mode(Mode::AnalogMode);
        hil::gpio::Configuration::LowPower
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // self.set_mode(Mode::AnalogMode);
        hil::gpio::Configuration::LowPower
    }

    fn set_floating_state(&self, mode: hil::gpio::FloatingState) {
        match mode {
            hil::gpio::FloatingState::PullUp => self.set_pullup_pulldown(PullUpPullDown::Pus2_100kOhmPullUp),
            hil::gpio::FloatingState::PullDown => {
                self.set_pullup_pulldown(PullUpPullDown::Pus2_100kOhmPullUp)
            }
            hil::gpio::FloatingState::PullNone => {
                self.set_pullup_pulldown(PullUpPullDown::Pus0_100kOhmPullDown)
            }
        }
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        // match self.get_pullup_pulldown() {
        //     PullUpPullDown::PullUp => hil::gpio::FloatingState::PullUp,
        //     PullUpPullDown::PullDown => hil::gpio::FloatingState::PullDown,
            // PullUpPullDown::NoPullUpPullDown => hil::gpio::FloatingState::PullNone,
        // }
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

impl hil::gpio::Output for Pin<'a> {
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

impl hil::gpio::Input for Pin<'a> {
    fn read(&self) -> bool {
        self.read_input()
    }
}

impl hil::gpio::Interrupt for Pin<'a> {
    fn enable_interrupts(&self, _mode: hil::gpio::InterruptEdge) {
        // unsafe {
        //     atomic(|| {
        //         self.exti_lineid.map(|lineid| {
        //             let l = lineid.clone();

        //             // disable the interrupt
        //             exti::EXTI.mask_interrupt(l);
        //             exti::EXTI.clear_pending(l);

        //             match mode {
        //                 hil::gpio::InterruptEdge::EitherEdge => {
        //                     exti::EXTI.select_rising_trigger(l);
        //                     exti::EXTI.select_falling_trigger(l);
        //                 }
        //                 hil::gpio::InterruptEdge::RisingEdge => {
        //                     exti::EXTI.select_rising_trigger(l);
        //                     exti::EXTI.deselect_falling_trigger(l);
        //                 }
        //                 hil::gpio::InterruptEdge::FallingEdge => {
        //                     exti::EXTI.deselect_rising_trigger(l);
        //                     exti::EXTI.select_falling_trigger(l);
        //                 }
        //             }

        //             exti::EXTI.unmask_interrupt(l);
        //         });
        //     });
        // }
    }

    fn disable_interrupts(&self) {
        // unsafe {
        //     atomic(|| {
        //         self.exti_lineid.map(|lineid| {
        //             let l = lineid.clone();
        //             exti::EXTI.mask_interrupt(l);
        //             exti::EXTI.clear_pending(l);
        //         });
        //     });
        // }
    }

    fn set_client(&self, client: &'static dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        // unsafe {
        //     self.exti_lineid
        //         .map_or(false, |&mut lineid| exti::EXTI.is_pending(lineid))
        // }
        false
    }
}
