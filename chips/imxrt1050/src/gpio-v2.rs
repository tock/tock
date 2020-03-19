use cortexm7;
use cortexm7::support::atomic;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::exti;
use crate::ccm;

/// General-purpose I/Os
#[repr(C)]
struct GpioRegisters {
	// GPIO data register
	dr: ReadWrite<u32, DR::Register>,
    // GPIO direction register
    gdir: ReadWrite<u32, GDIR::Register>,
    // others unimplemented
	_reserved1: [u8; 12],
	// GPIO interrupt mask register
	imr: ReadWrite<u32, IMR::Register>,
	_reserved2: [u8; 116],
}

register_bitfields![u32,
    DR [
        // the value of the GPIO output when the signal is configured as an output 
        DR OFFSET(0) NUMBITS(32) [],
    ],

    GDIR [
    	// bit n of this register defines the direction of the GPIO[n] signal
    	GDIR OFFSET(0) NUMBITS(32) [],
    ],

    IMR [
    	// enable or disable the interrupt function on each of the 32 GPIO signals
    	IMR OFFSET(0) NUMBITS(32) [],
    ]
];

const GPIO1_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x401B8000 as *const GpioRegisters) };

// Yeah.. noo...
#[repr(u32)]
pub enum PortId {
    P1 = 0b000,
    P2 = 0b001,
    P3 = 0b010,
    P4 = 0b011,
    P5 = 0b100,
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PinId {
    PAX = 0b100000000,
}

impl PinId {
    pub fn get_pin(&self) -> &Option<Pin<'static>> {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;

        let mut pin_num: u8 = *self as u8;
        // Mask top 3 bits, so can get only the suffix
        pin_num &= 0b0001111;

        unsafe { &PIN[usize::from(port_num)][usize::from(pin_num)] }
    }

    pub fn get_pin_mut(&self) -> &mut Option<Pin<'static>> {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;

        let mut pin_num: u8 = *self as u8;
        // Mask top 3 bits, so can get only the suffix
        pin_num &= 0b0001111;

        unsafe { &mut PIN[usize::from(port_num)][usize::from(pin_num)] }
    }

    pub fn get_port(&self) -> &Port {
        let mut port_num: u8 = *self as u8;

        // Right shift p by 4 bits, so we can get rid of pin bits
        port_num >>= 4;
        unsafe { &PORT[usize::from(port_num)] }
    }

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


