use cortexm7::support::atomic;
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

/// Imxrt1050-evkb has 5 GPIO ports labeled from 1-5 [^1]. This is represented
/// by three bits.
///
/// [^1]: 12.5.1 GPIO memory map, page 1009 of the Reference Manual.
#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GpioPort {
    GPIO1 = 0b000,
    GPIO2 = 0b001,
    GPIO3 = 0b010,
    GPIO4 = 0b011,
    GPIO5 = 0b100,
}

/// Creates a GPIO ID
///
/// Low 6 bits are the GPIO offset; the '17' in GPIO2[17]
/// Next 3 bits are the GPIO port; the '2' in GPIO2[17]
const fn gpio_id(port: GpioPort, offset: u16) -> u16 {
    ((port as u16) << 6) | offset & 0x3F
}

/// GPIO Pin Identifiers
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum PinId {
    // GPIO1
    AdB0_00 = gpio_id(GpioPort::GPIO1, 0),
    AdB0_01 = gpio_id(GpioPort::GPIO1, 1),
    AdB0_02 = gpio_id(GpioPort::GPIO1, 2),
    AdB0_03 = gpio_id(GpioPort::GPIO1, 3),
    AdB0_04 = gpio_id(GpioPort::GPIO1, 4),
    AdB0_05 = gpio_id(GpioPort::GPIO1, 5),
    AdB0_06 = gpio_id(GpioPort::GPIO1, 6),
    AdB0_07 = gpio_id(GpioPort::GPIO1, 7),
    AdB0_08 = gpio_id(GpioPort::GPIO1, 8),
    AdB0_09 = gpio_id(GpioPort::GPIO1, 9),
    AdB0_10 = gpio_id(GpioPort::GPIO1, 10),
    AdB0_11 = gpio_id(GpioPort::GPIO1, 11),
    AdB0_12 = gpio_id(GpioPort::GPIO1, 12),
    AdB0_13 = gpio_id(GpioPort::GPIO1, 13),
    AdB0_14 = gpio_id(GpioPort::GPIO1, 14),
    AdB0_15 = gpio_id(GpioPort::GPIO1, 15),

    AdB1_00 = gpio_id(GpioPort::GPIO1, 16),
    AdB1_01 = gpio_id(GpioPort::GPIO1, 17),
    AdB1_02 = gpio_id(GpioPort::GPIO1, 18),
    AdB1_03 = gpio_id(GpioPort::GPIO1, 19),
    AdB1_04 = gpio_id(GpioPort::GPIO1, 20),
    AdB1_05 = gpio_id(GpioPort::GPIO1, 21),
    AdB1_06 = gpio_id(GpioPort::GPIO1, 22),
    AdB1_07 = gpio_id(GpioPort::GPIO1, 23),
    AdB1_08 = gpio_id(GpioPort::GPIO1, 24),
    AdB1_09 = gpio_id(GpioPort::GPIO1, 25),
    AdB1_10 = gpio_id(GpioPort::GPIO1, 26),
    AdB1_11 = gpio_id(GpioPort::GPIO1, 27),
    AdB1_12 = gpio_id(GpioPort::GPIO1, 28),
    AdB1_13 = gpio_id(GpioPort::GPIO1, 29),
    AdB1_14 = gpio_id(GpioPort::GPIO1, 30),
    AdB1_15 = gpio_id(GpioPort::GPIO1, 31),

    // GPIO2
    B0_00 = gpio_id(GpioPort::GPIO2, 0),
    B0_01 = gpio_id(GpioPort::GPIO2, 1),
    B0_02 = gpio_id(GpioPort::GPIO2, 2),
    B0_03 = gpio_id(GpioPort::GPIO2, 3),
    B0_04 = gpio_id(GpioPort::GPIO2, 4),
    B0_05 = gpio_id(GpioPort::GPIO2, 5),
    B0_06 = gpio_id(GpioPort::GPIO2, 6),
    B0_07 = gpio_id(GpioPort::GPIO2, 7),
    B0_08 = gpio_id(GpioPort::GPIO2, 8),
    B0_09 = gpio_id(GpioPort::GPIO2, 9),
    B0_10 = gpio_id(GpioPort::GPIO2, 10),
    B0_11 = gpio_id(GpioPort::GPIO2, 11),
    B0_12 = gpio_id(GpioPort::GPIO2, 12),
    B0_13 = gpio_id(GpioPort::GPIO2, 13),
    B0_14 = gpio_id(GpioPort::GPIO2, 14),
    B0_15 = gpio_id(GpioPort::GPIO2, 15),

    B1_00 = gpio_id(GpioPort::GPIO2, 16),
    B1_01 = gpio_id(GpioPort::GPIO2, 17),
    B1_02 = gpio_id(GpioPort::GPIO2, 18),
    B1_03 = gpio_id(GpioPort::GPIO2, 19),
    B1_04 = gpio_id(GpioPort::GPIO2, 20),
    B1_05 = gpio_id(GpioPort::GPIO2, 21),
    B1_06 = gpio_id(GpioPort::GPIO2, 22),
    B1_07 = gpio_id(GpioPort::GPIO2, 23),
    B1_08 = gpio_id(GpioPort::GPIO2, 24),
    B1_09 = gpio_id(GpioPort::GPIO2, 25),
    B1_10 = gpio_id(GpioPort::GPIO2, 26),
    B1_11 = gpio_id(GpioPort::GPIO2, 27),
    B1_12 = gpio_id(GpioPort::GPIO2, 28),
    B1_13 = gpio_id(GpioPort::GPIO2, 29),
    B1_14 = gpio_id(GpioPort::GPIO2, 30),
    B1_15 = gpio_id(GpioPort::GPIO2, 31),

    // GPIO3
    SdB1_00 = gpio_id(GpioPort::GPIO3, 0),
    SdB1_01 = gpio_id(GpioPort::GPIO3, 1),
    SdB1_02 = gpio_id(GpioPort::GPIO3, 2),
    SdB1_03 = gpio_id(GpioPort::GPIO3, 3),
    SdB1_04 = gpio_id(GpioPort::GPIO3, 4),
    SdB1_05 = gpio_id(GpioPort::GPIO3, 5),
    SdB1_06 = gpio_id(GpioPort::GPIO3, 6),
    SdB1_07 = gpio_id(GpioPort::GPIO3, 7),
    SdB1_08 = gpio_id(GpioPort::GPIO3, 8),
    SdB1_09 = gpio_id(GpioPort::GPIO3, 9),
    SdB1_10 = gpio_id(GpioPort::GPIO3, 10),
    SdB1_11 = gpio_id(GpioPort::GPIO3, 11),

    SdB0_00 = gpio_id(GpioPort::GPIO3, 12),
    SdB0_01 = gpio_id(GpioPort::GPIO3, 13),
    SdB0_02 = gpio_id(GpioPort::GPIO3, 14),
    SdB0_03 = gpio_id(GpioPort::GPIO3, 15),
    SdB0_04 = gpio_id(GpioPort::GPIO3, 16),
    SdB0_05 = gpio_id(GpioPort::GPIO3, 17),

    Emc32 = gpio_id(GpioPort::GPIO3, 18),
    Emc33 = gpio_id(GpioPort::GPIO3, 19),
    Emc34 = gpio_id(GpioPort::GPIO3, 20),
    Emc35 = gpio_id(GpioPort::GPIO3, 21),
    Emc36 = gpio_id(GpioPort::GPIO3, 22),
    Emc37 = gpio_id(GpioPort::GPIO3, 23),
    Emc38 = gpio_id(GpioPort::GPIO3, 24),
    Emc39 = gpio_id(GpioPort::GPIO3, 25),
    Emc40 = gpio_id(GpioPort::GPIO3, 26),
    Emc41 = gpio_id(GpioPort::GPIO3, 27),

    // GPIO4
    Emc00 = gpio_id(GpioPort::GPIO4, 0),
    Emc01 = gpio_id(GpioPort::GPIO4, 1),
    Emc02 = gpio_id(GpioPort::GPIO4, 2),
    Emc03 = gpio_id(GpioPort::GPIO4, 3),
    Emc04 = gpio_id(GpioPort::GPIO4, 4),
    Emc05 = gpio_id(GpioPort::GPIO4, 5),
    Emc06 = gpio_id(GpioPort::GPIO4, 6),
    Emc07 = gpio_id(GpioPort::GPIO4, 7),
    Emc08 = gpio_id(GpioPort::GPIO4, 8),
    Emc09 = gpio_id(GpioPort::GPIO4, 9),
    Emc10 = gpio_id(GpioPort::GPIO4, 10),
    Emc11 = gpio_id(GpioPort::GPIO4, 11),
    Emc12 = gpio_id(GpioPort::GPIO4, 12),
    Emc13 = gpio_id(GpioPort::GPIO4, 13),
    Emc14 = gpio_id(GpioPort::GPIO4, 14),
    Emc15 = gpio_id(GpioPort::GPIO4, 15),
    Emc16 = gpio_id(GpioPort::GPIO4, 16),
    Emc17 = gpio_id(GpioPort::GPIO4, 17),
    Emc18 = gpio_id(GpioPort::GPIO4, 18),
    Emc19 = gpio_id(GpioPort::GPIO4, 19),
    Emc20 = gpio_id(GpioPort::GPIO4, 20),
    Emc21 = gpio_id(GpioPort::GPIO4, 21),
    Emc22 = gpio_id(GpioPort::GPIO4, 22),
    Emc23 = gpio_id(GpioPort::GPIO4, 23),
    Emc24 = gpio_id(GpioPort::GPIO4, 24),
    Emc25 = gpio_id(GpioPort::GPIO4, 25),
    Emc26 = gpio_id(GpioPort::GPIO4, 26),
    Emc27 = gpio_id(GpioPort::GPIO4, 27),
    Emc28 = gpio_id(GpioPort::GPIO4, 28),
    Emc29 = gpio_id(GpioPort::GPIO4, 29),
    Emc30 = gpio_id(GpioPort::GPIO4, 30),
    Emc31 = gpio_id(GpioPort::GPIO4, 31),

    // GPIO5
    Wakeup = gpio_id(GpioPort::GPIO5, 0),
    PmicOnReq = gpio_id(GpioPort::GPIO5, 1),
    PmicStbyReq = gpio_id(GpioPort::GPIO5, 2),
}

impl PinId {
    /// Returns the port number as a base 0 index
    ///
    /// GPIO5 -> 4
    /// GPIO2 -> 1
    /// GPIO1 -> 0
    const fn port(self) -> usize {
        (self as u16 >> 6) as usize
    }
    /// Returns the pin offset, half-closed range [0, 32)
    const fn offset(self) -> usize {
        (self as usize) & 0x3F
    }
}

/// GPIO pin mode
/// In order to set alternate functions such as LPI2C or LPUART,
/// you will need to use iomuxc enable_sw_mux_ctl_pad_gpio with
/// the specific MUX_MODE according to the reference manual (Chapter 11).
/// For the gpio mode, input or output we set the GDIR pin accordingly [^1]
///
/// [^1]: 12.4.3. GPIO Programming, page 1008 of the Reference Manual
pub enum Mode {
    Input = 0b00,
    Output = 0b01,
}

pub struct Port<'a> {
    registers: StaticRef<GpioRegisters>,
    clock: PortClock<'a>,
    pins: [Pin<'a>; 32],
}

impl<'a> Port<'a> {
    const fn new(registers: StaticRef<GpioRegisters>, clock: PortClock<'a>) -> Self {
        Self {
            registers,
            clock,
            pins: [
                Pin::new(registers, 00),
                Pin::new(registers, 01),
                Pin::new(registers, 02),
                Pin::new(registers, 03),
                Pin::new(registers, 04),
                Pin::new(registers, 05),
                Pin::new(registers, 06),
                Pin::new(registers, 07),
                Pin::new(registers, 08),
                Pin::new(registers, 09),
                Pin::new(registers, 10),
                Pin::new(registers, 11),
                Pin::new(registers, 12),
                Pin::new(registers, 13),
                Pin::new(registers, 14),
                Pin::new(registers, 15),
                Pin::new(registers, 16),
                Pin::new(registers, 17),
                Pin::new(registers, 18),
                Pin::new(registers, 19),
                Pin::new(registers, 20),
                Pin::new(registers, 21),
                Pin::new(registers, 22),
                Pin::new(registers, 23),
                Pin::new(registers, 24),
                Pin::new(registers, 25),
                Pin::new(registers, 26),
                Pin::new(registers, 27),
                Pin::new(registers, 28),
                Pin::new(registers, 29),
                Pin::new(registers, 30),
                Pin::new(registers, 31),
            ],
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

    pub fn handle_interrupt(&self) {
        let imr_val: u32 = self.registers.imr.get();

        // Read the `ISR` register and toggle the appropriate bits in
        // `isr`. Once that is done, write the value of `isr` back. We
        // can have a situation where memory value of `ISR` could have
        // changed due to an external interrupt. `ISR` is a read/clear write
        // 1 register (`rc_w1`). So, we only clear bits whose value has been
        // transferred to `isr`.
        let isr_val = unsafe {
            atomic(|| {
                let isr_val = self.registers.isr.get();
                self.registers.isr.set(isr_val);
                isr_val
            })
        };

        BitOffsets(isr_val)
            .filter(|offset| imr_val & (1 << offset) != 0)
            .for_each(|offset| {
                self.pins[offset as usize]
                    .client
                    .map(|client| client.fired());
            });
    }
}

pub struct Ports<'a>([Port<'a>; 5]);

impl<'a> Ports<'a> {
    pub const fn new(ccm: &'a ccm::Ccm) -> Self {
        Ports([
            Port::new(
                GPIO1_BASE,
                PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
            ),
            Port::new(
                GPIO2_BASE,
                PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
            ),
            Port::new(
                GPIO3_BASE,
                PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
            ),
            Port::new(
                GPIO4_BASE,
                PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
            ),
            Port::new(
                GPIO5_BASE,
                PortClock(ccm::PeripheralClock::ccgr1(ccm, ccm::HCLK1::GPIO1)),
            ),
        ])
    }
    pub const fn pin(&self, pin: PinId) -> &Pin<'a> {
        &self.0[pin.port()].pins[pin.offset()]
    }
    pub const fn port(&self, port: GpioPort) -> &Port<'a> {
        &self.0[port as usize]
    }
}

struct PortClock<'a>(ccm::PeripheralClock<'a>);

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
    registers: StaticRef<GpioRegisters>,
    offset: usize,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
}

trait U32Ext {
    fn set_bit(self, offset: usize) -> Self;
    fn clear_bit(self, offset: usize) -> Self;
    fn is_bit_set(self, offset: usize) -> bool;
}

impl U32Ext for u32 {
    #[inline(always)]
    fn set_bit(self, offset: usize) -> u32 {
        self | (1 << offset)
    }
    #[inline(always)]
    fn clear_bit(self, offset: usize) -> u32 {
        self & !(1 << offset)
    }
    #[inline(always)]
    fn is_bit_set(self, offset: usize) -> bool {
        (self & (1 << offset)) != 0
    }
}

impl<'a> Pin<'a> {
    const fn new(registers: StaticRef<GpioRegisters>, offset: usize) -> Self {
        Pin {
            registers,
            offset,
            client: OptionalCell::empty(),
        }
    }

    fn get_mode(&self) -> Mode {
        if self.registers.gdir.get().is_bit_set(self.offset) {
            Mode::Output
        } else {
            Mode::Input
        }
    }

    fn set_mode(&self, mode: Mode) {
        let gdir = self.registers.gdir.get();
        let gdir = match mode {
            Mode::Input => gdir.clear_bit(self.offset),
            Mode::Output => gdir.set_bit(self.offset),
        };
        self.registers.gdir.set(gdir);
    }

    fn set_output_high(&self) {
        self.registers.dr_set.set(1 << self.offset);
    }

    fn set_output_low(&self) {
        self.registers.dr_clear.set(1 << self.offset);
    }

    fn is_output_high(&self) -> bool {
        self.registers.dr.get().is_bit_set(self.offset)
    }

    fn toggle_output(&self) -> bool {
        self.registers.dr_toggle.set(1 << self.offset);
        self.is_output_high()
    }

    fn read_input(&self) -> bool {
        self.registers.psr.get().is_bit_set(self.offset)
    }

    fn mask_interrupt(&self) {
        let imr = self.registers.imr.get();
        let imr = imr.clear_bit(self.offset);
        self.registers.imr.set(imr);
    }

    fn unmask_interrupt(&self) {
        let imr = self.registers.imr.get();
        let imr = imr.set_bit(self.offset);
        self.registers.imr.set(imr);
    }

    fn clear_pending(&self) {
        self.registers.isr.set(1 << self.offset); // W1C
    }

    fn set_edge_sensitive(&self, sensitive: hil::gpio::InterruptEdge) {
        use hil::gpio::InterruptEdge::*;
        const RISING_EDGE_SENSITIVE: u32 = 0b10;
        const FALLING_EDGE_SENSITIVE: u32 = 0b11;

        let edge_sel = self.registers.edge_sel.get();
        let icr_offset = (self.offset % 16) * 2;

        let sensitive = match sensitive {
            EitherEdge => {
                let edge_sel = edge_sel.set_bit(self.offset);
                self.registers.edge_sel.set(edge_sel);
                // A high EDGE_SEL disregards the corresponding ICR[1|2] setting
                return;
            }
            RisingEdge => RISING_EDGE_SENSITIVE << icr_offset,
            FallingEdge => FALLING_EDGE_SENSITIVE << icr_offset,
        };

        let edge_sel = edge_sel.clear_bit(self.offset);
        self.registers.edge_sel.set(edge_sel);

        let icr_mask = 0b11 << icr_offset;
        if self.offset < 16 {
            let icr1 = self.registers.icr1.get();
            let icr1 = (icr1 & !icr_mask) | sensitive;
            self.registers.icr1.set(icr1);
        } else {
            let icr2 = self.registers.icr2.get();
            let icr2 = (icr2 & !icr_mask) | sensitive;
            self.registers.icr2.set(icr2);
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
                self.set_edge_sensitive(mode);

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
        self.registers.isr.get().is_bit_set(self.offset)
    }
}

/// An iterator that returns the offsets of each high bit
///
/// Each offset is returned only once. There is no guarantee
/// for iteration order.
struct BitOffsets(u32);

impl Iterator for BitOffsets {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let offset = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(offset)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let popcnt = self.0.count_ones() as usize;
        (popcnt, Some(popcnt))
    }
}

impl ExactSizeIterator for BitOffsets {}

#[cfg(test)]
mod tests {
    use super::BitOffsets;
    use std::collections::HashSet;

    #[test]
    fn bit_offsets() {
        fn check(offsets: BitOffsets, expected: impl Iterator<Item = u32> + Clone) {
            let size = expected.clone().count();
            assert_eq!(offsets.len(), size);

            let word = offsets.0;
            let expected: HashSet<_> = expected.collect();
            let actual: HashSet<_> = offsets.collect();
            let mut ordered_expected: Vec<_> = expected.iter().cloned().collect();
            ordered_expected.sort_unstable();
            let mut ordered_actual: Vec<_> = actual.iter().cloned().collect();
            ordered_actual.sort_unstable();
            assert_eq!(
                expected, actual,
                "\n  Ordered left: {:?}\n Ordered right: {:?}\n Word: {:#b}",
                ordered_expected, ordered_actual, word
            );
        }

        assert_eq!(BitOffsets(0).next(), None);
        check(BitOffsets(u32::max_value()), 0..32);
        check(BitOffsets(0x5555_5555), (0..32).step_by(2));
        check(BitOffsets(0xAAAA_AAAA), (0..32).skip(1).step_by(2));
    }
}
