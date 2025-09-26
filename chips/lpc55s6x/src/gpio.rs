// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! General‑Purpose Input/Output (GPIO) driver for the LPC55S6x family.
//!
//! Features supported:
//! - 64 GPIO pins across Port0 and Port1
//! - Direction control (input/output) with atomic set/clear/not registers
//! - Pin read/write/toggle operations
//! - Integration with IOCON, InputMux, and PINT for flexible pin configuration
//! - Interrupt support on rising, falling, or both edges
//! - Safe abstractions for pin initialization and peripheral setup
//!
//! The `Pins` struct bundles all pins and ensures they are initialized with
//! the required InputMux, IOCON, and PINT connections. Each `GpioPin` can be
//! retrieved by its `LPCPin` enum value and used directly in drivers.
//!
//! Reference: *LPC55S6x/LPC55S2x/LPC552x User Manual* (NXP).

use kernel::hil::gpio;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

register_structs! {
    pub GpioRegisters {
        (0x0000 => _reserved0: [u8; 0x2000]),
        (0x2000 => dir_0: ReadWrite<u32, DIR::Register>),
        (0x2004 => dir_1: ReadWrite<u32, DIR::Register>),
        (0x2008 => _reserved1: [u8; 0x78]),
        (0x2080 => mask_0: ReadWrite<u32, MASK::Register>),
        (0x2084 => mask_1: ReadWrite<u32, MASK::Register>),
        (0x2088 => _reserved2: [u8; 0x78]),
        (0x2100 => pin_0: ReadWrite<u32, PIN::Register>),
        (0x2104 => pin_1: ReadWrite<u32, PIN::Register>),
        (0x2108 => _reserved3: [u8; 0x78]),
        (0x2180 => mpin_0: ReadWrite<u32, MPIN::Register>),
        (0x2184 => mpin_1: ReadWrite<u32, MPIN::Register>),
        (0x2188 => _reserved4: [u8; 0x78]),
        (0x2200 => set_0: WriteOnly<u32, SET::Register>),
        (0x2204 => set_1: WriteOnly<u32, SET::Register>),
        (0x2208 => _reserved5: [u8; 0x78]),
        (0x2280 => clr_0: WriteOnly<u32, CLR::Register>),
        (0x2284 => clr_1: WriteOnly<u32, CLR::Register>),
        (0x2288 => _reserved6: [u8; 0x78]),
        (0x2300 => not_0: WriteOnly<u32, NOT::Register>),
        (0x2304 => not_1: WriteOnly<u32, NOT::Register>),
        (0x2308 => _reserved7: [u8; 0x78]),
        (0x2380 => dirset_0: WriteOnly<u32, DIRSET::Register>),
        (0x2384 => dirset_1: WriteOnly<u32, DIRSET::Register>),
        (0x2388 => _reserved8: [u8; 0x78]),
        (0x2400 => dirclr_0: WriteOnly<u32, DIRCLR::Register>),
        (0x2404 => dirclr_1: WriteOnly<u32, DIRCLR::Register>),
        (0x2408 => _reserved9: [u8; 0x78]),
        (0x2480 => dirnot_0: WriteOnly<u32, DIRNOT::Register>),
        (0x2484 => dirnot_1: WriteOnly<u32, DIRNOT::Register>),
        (0x2488 => @END),
    }
}

register_bitfields![u32,
    DIR [ DIRP OFFSET(0) NUMBITS(32) [] ], MASK [ MASKP OFFSET(0) NUMBITS(32) [] ],
    PIN [ PORT OFFSET(0) NUMBITS(32) [] ], MPIN [ MPORTP OFFSET(0) NUMBITS(32) [] ],
    SET [ SETP OFFSET(0) NUMBITS(32) [] ], CLR [ CLRP OFFSET(0) NUMBITS(32) [] ],
    NOT [ NOTP OFFSET(0) NUMBITS(32) [] ], DIRSET [ DIRSETP OFFSET(0) NUMBITS(32) [] ],
    DIRCLR [ DIRCLRP OFFSET(0) NUMBITS(32) [] ], DIRNOT [ DIRNOTP OFFSET(0) NUMBITS(32) [] ]
];

pub(crate) const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x5008_C000 as *const GpioRegisters) };

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LPCPin {
    P0_0 = 0,
    P0_1 = 1,
    P0_2 = 2,
    P0_3 = 3,
    P0_4 = 4,
    P0_5 = 5,
    P0_6 = 6,
    P0_7 = 7,
    P0_8 = 8,
    P0_9 = 9,
    P0_10 = 10,
    P0_11 = 11,
    P0_12 = 12,
    P0_13 = 13,
    P0_14 = 14,
    P0_15 = 15,
    P0_16 = 16,
    P0_17 = 17,
    P0_18 = 18,
    P0_19 = 19,
    P0_20 = 20,
    P0_21 = 21,
    P0_22 = 22,
    P0_23 = 23,
    P0_24 = 24,
    P0_25 = 25,
    P0_26 = 26,
    P0_27 = 27,
    P0_28 = 28,
    P0_29 = 29,
    P0_30 = 30,
    P0_31 = 31,
    P1_0 = 32,
    P1_1 = 33,
    P1_2 = 34,
    P1_3 = 35,
    P1_4 = 36,
    P1_5 = 37,
    P1_6 = 38,
    P1_7 = 39,
    P1_8 = 40,
    P1_9 = 41,
    P1_10 = 42,
    P1_11 = 43,
    P1_12 = 44,
    P1_13 = 45,
    P1_14 = 46,
    P1_15 = 47,
    P1_16 = 48,
    P1_17 = 49,
    P1_18 = 50,
    P1_19 = 51,
    P1_20 = 52,
    P1_21 = 53,
    P1_22 = 54,
    P1_23 = 55,
    P1_24 = 56,
    P1_25 = 57,
    P1_26 = 58,
    P1_27 = 59,
    P1_28 = 60,
    P1_29 = 61,
    P1_30 = 62,
    P1_31 = 63,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Port {
    Port0 = 0,
    Port1 = 1,
}

pub struct Pins<'a> {
    pub pins: [GpioPin<'a>; 64],
    pub inputmux: Inputmux,
    pub iocon: Iocon,
    pub pint: Pint<'a>,
}

impl<'a> Pins<'a> {
    pub const fn new() -> Self {
        let inputmux = Inputmux::new();
        let iocon = Iocon::new();
        let pint = Pint::new();
        Self {
            pins: [
                GpioPin::new(LPCPin::P0_0),
                GpioPin::new(LPCPin::P0_1),
                GpioPin::new(LPCPin::P0_2),
                GpioPin::new(LPCPin::P0_3),
                GpioPin::new(LPCPin::P0_4),
                GpioPin::new(LPCPin::P0_5),
                GpioPin::new(LPCPin::P0_6),
                GpioPin::new(LPCPin::P0_7),
                GpioPin::new(LPCPin::P0_8),
                GpioPin::new(LPCPin::P0_9),
                GpioPin::new(LPCPin::P0_10),
                GpioPin::new(LPCPin::P0_11),
                GpioPin::new(LPCPin::P0_12),
                GpioPin::new(LPCPin::P0_13),
                GpioPin::new(LPCPin::P0_14),
                GpioPin::new(LPCPin::P0_15),
                GpioPin::new(LPCPin::P0_16),
                GpioPin::new(LPCPin::P0_17),
                GpioPin::new(LPCPin::P0_18),
                GpioPin::new(LPCPin::P0_19),
                GpioPin::new(LPCPin::P0_20),
                GpioPin::new(LPCPin::P0_21),
                GpioPin::new(LPCPin::P0_22),
                GpioPin::new(LPCPin::P0_23),
                GpioPin::new(LPCPin::P0_24),
                GpioPin::new(LPCPin::P0_25),
                GpioPin::new(LPCPin::P0_26),
                GpioPin::new(LPCPin::P0_27),
                GpioPin::new(LPCPin::P0_28),
                GpioPin::new(LPCPin::P0_29),
                GpioPin::new(LPCPin::P0_30),
                GpioPin::new(LPCPin::P0_31),
                GpioPin::new(LPCPin::P1_0),
                GpioPin::new(LPCPin::P1_1),
                GpioPin::new(LPCPin::P1_2),
                GpioPin::new(LPCPin::P1_3),
                GpioPin::new(LPCPin::P1_4),
                GpioPin::new(LPCPin::P1_5),
                GpioPin::new(LPCPin::P1_6),
                GpioPin::new(LPCPin::P1_7),
                GpioPin::new(LPCPin::P1_8),
                GpioPin::new(LPCPin::P1_9),
                GpioPin::new(LPCPin::P1_10),
                GpioPin::new(LPCPin::P1_11),
                GpioPin::new(LPCPin::P1_12),
                GpioPin::new(LPCPin::P1_13),
                GpioPin::new(LPCPin::P1_14),
                GpioPin::new(LPCPin::P1_15),
                GpioPin::new(LPCPin::P1_16),
                GpioPin::new(LPCPin::P1_17),
                GpioPin::new(LPCPin::P1_18),
                GpioPin::new(LPCPin::P1_19),
                GpioPin::new(LPCPin::P1_20),
                GpioPin::new(LPCPin::P1_21),
                GpioPin::new(LPCPin::P1_22),
                GpioPin::new(LPCPin::P1_23),
                GpioPin::new(LPCPin::P1_24),
                GpioPin::new(LPCPin::P1_25),
                GpioPin::new(LPCPin::P1_26),
                GpioPin::new(LPCPin::P1_27),
                GpioPin::new(LPCPin::P1_28),
                GpioPin::new(LPCPin::P1_29),
                GpioPin::new(LPCPin::P1_30),
                GpioPin::new(LPCPin::P1_31),
            ],
            inputmux,
            iocon,
            pint,
        }
    }
    /// Returns a reference to the GPIO pin corresponding to the given `LPCPin`.
    ///
    /// # Panics
    /// This function will **never panic** because:
    /// - `searched_pin` is an `LPCPin` enum, which only has valid, in‑range discriminants.
    /// - Each valid `LPCPin` maps to a slot in `self.pins`, so the index is always within bounds.
    /// - During initialization, all entries in `self.pins` are guaranteed to be populated (`Some`),
    ///   so calling `.unwrap()` is safe.
    ///
    /// Therefore, both the array indexing and the `unwrap()` are guaranteed not to fail.
    pub fn get_pin(&self, searched_pin: LPCPin) -> &GpioPin<'a> {
        &self.pins[searched_pin as usize]
    }

    pub fn handle_interrupt(&self) {
        self.pint.handle_interrupt();

        for pin in self.pins.iter() {
            pin.handle_interrupt();
        }
    }

    pub fn set_inputmux(&'a self) {
        for pin in self.pins.iter() {
            pin.set_inputmux(&self.inputmux);
        }
    }

    pub fn set_iocon(&'a self) {
        for pin in self.pins.iter() {
            pin.set_iocon(&self.iocon);
        }
    }

    pub fn set_pint(&'a self) {
        for pin in self.pins.iter() {
            pin.set_pint(&self.pint);
        }
    }

    pub fn init(&'a self) {
        self.set_inputmux();
        self.set_iocon();
        self.set_pint();
    }
}

pub struct GpioPin<'a> {
    registers: StaticRef<GpioRegisters>,
    port: Port,
    pin: u8,
    client: OptionalCell<&'a dyn gpio::Client>,
    inputmux: OptionalCell<&'a Inputmux>,
    iocon: OptionalCell<&'a Iocon>,
    pint: OptionalCell<&'a Pint<'a>>,
}

pub use kernel::hil::gpio::{Configure, Input, Interrupt, Output, Pin};

use crate::inputmux::Inputmux;
use crate::iocon::Iocon;
use crate::pint::{Edge, Pint};

impl<'a> GpioPin<'a> {
    pub const fn new(pin_name: LPCPin) -> Self {
        let pin_num = pin_name as u8;
        let port = match pin_num / 32 {
            0 => Port::Port0,
            1 => Port::Port1,
            _ => panic!("Invalid pin number for LPCPin"),
        };
        Self {
            registers: GPIO_BASE,
            port,
            pin: pin_num % 32,
            client: OptionalCell::empty(),
            inputmux: OptionalCell::empty(),
            iocon: OptionalCell::empty(),
            pint: OptionalCell::empty(),
        }
    }

    fn pin_mask(&self) -> u32 {
        1 << self.pin
    }

    fn is_output(&self) -> bool {
        match self.port {
            Port::Port0 => (self.registers.dir_0.get() & self.pin_mask()) != 0,
            Port::Port1 => (self.registers.dir_1.get() & self.pin_mask()) != 0,
        }
    }

    pub fn get_pin_num(&self) -> usize {
        (self.port as usize * 32) + self.pin as usize
    }

    pub fn handle_interrupt(&self) {
        self.pint.map(|pint| {
            pint.handle_interrupt();
        });
    }

    pub fn set_inputmux(&self, inputmux: &'a Inputmux) {
        self.inputmux.set(inputmux);
    }
    pub fn set_iocon(&self, iocon: &'a Iocon) {
        self.iocon.set(iocon);
    }
    pub fn set_pint(&self, pint: &'a Pint<'a>) {
        self.pint.set(pint);
    }
}

impl gpio::Output for GpioPin<'_> {
    fn set(&self) {
        match self.port {
            Port::Port0 => self.registers.set_0.write(SET::SETP.val(self.pin_mask())),
            Port::Port1 => self.registers.set_1.write(SET::SETP.val(self.pin_mask())),
        }
    }

    fn clear(&self) {
        match self.port {
            Port::Port0 => self.registers.clr_0.write(CLR::CLRP.val(self.pin_mask())),
            Port::Port1 => self.registers.clr_1.write(CLR::CLRP.val(self.pin_mask())),
        }
    }

    fn toggle(&self) -> bool {
        match self.port {
            Port::Port0 => self.registers.not_0.write(NOT::NOTP.val(self.pin_mask())),
            Port::Port1 => self.registers.not_1.write(NOT::NOTP.val(self.pin_mask())),
        }
        self.read()
    }
}

impl gpio::Input for GpioPin<'_> {
    fn read(&self) -> bool {
        match self.port {
            Port::Port0 => self.registers.pin_0.get() & self.pin_mask() != 0,
            Port::Port1 => self.registers.pin_1.get() & self.pin_mask() != 0,
        }
    }
}

impl gpio::Configure for GpioPin<'_> {
    fn make_output(&self) -> gpio::Configuration {
        match self.port {
            Port::Port0 => self
                .registers
                .dirset_0
                .write(DIRSET::DIRSETP.val(self.pin_mask())),
            Port::Port1 => self
                .registers
                .dirset_1
                .write(DIRSET::DIRSETP.val(self.pin_mask())),
        }
        gpio::Configuration::Output
    }

    fn make_input(&self) -> gpio::Configuration {
        match self.port {
            Port::Port0 => self
                .registers
                .dirclr_0
                .write(DIRCLR::DIRCLRP.val(self.pin_mask())),
            Port::Port1 => self
                .registers
                .dirclr_1
                .write(DIRCLR::DIRCLRP.val(self.pin_mask())),
        }
        gpio::Configuration::Input
    }

    fn configuration(&self) -> gpio::Configuration {
        if self.is_output() {
            gpio::Configuration::Output
        } else {
            gpio::Configuration::Input
        }
    }

    fn set_floating_state(&self, state: kernel::hil::gpio::FloatingState) {
        let pins = [
            LPCPin::P0_0,
            LPCPin::P0_1,
            LPCPin::P0_2,
            LPCPin::P0_3,
            LPCPin::P0_4,
            LPCPin::P0_5,
            LPCPin::P0_6,
            LPCPin::P0_7,
            LPCPin::P0_8,
            LPCPin::P0_9,
            LPCPin::P0_10,
            LPCPin::P0_11,
            LPCPin::P0_12,
            LPCPin::P0_13,
            LPCPin::P0_14,
            LPCPin::P0_15,
            LPCPin::P0_16,
            LPCPin::P0_17,
            LPCPin::P0_18,
            LPCPin::P0_19,
            LPCPin::P0_20,
            LPCPin::P0_21,
            LPCPin::P0_22,
            LPCPin::P0_23,
            LPCPin::P0_24,
            LPCPin::P0_25,
            LPCPin::P0_26,
            LPCPin::P0_27,
            LPCPin::P0_28,
            LPCPin::P0_29,
            LPCPin::P0_30,
            LPCPin::P0_31,
            LPCPin::P1_0,
            LPCPin::P1_1,
            LPCPin::P1_2,
            LPCPin::P1_3,
            LPCPin::P1_4,
            LPCPin::P1_5,
            LPCPin::P1_6,
            LPCPin::P1_7,
            LPCPin::P1_8,
            LPCPin::P1_9,
            LPCPin::P1_10,
            LPCPin::P1_11,
            LPCPin::P1_12,
            LPCPin::P1_13,
            LPCPin::P1_14,
            LPCPin::P1_15,
            LPCPin::P1_16,
            LPCPin::P1_17,
            LPCPin::P1_18,
            LPCPin::P1_19,
            LPCPin::P1_20,
            LPCPin::P1_21,
            LPCPin::P1_22,
            LPCPin::P1_23,
            LPCPin::P1_24,
            LPCPin::P1_25,
            LPCPin::P1_26,
            LPCPin::P1_27,
            LPCPin::P1_28,
            LPCPin::P1_29,
            LPCPin::P1_30,
            LPCPin::P1_31,
        ];

        for pin in pins.iter() {
            match state {
                gpio::FloatingState::PullNone => {
                    self.iocon.map(|iocon| {
                        iocon.set_pull_none(*pin);
                    });
                }
                gpio::FloatingState::PullUp => {
                    self.iocon.map(|iocon| {
                        iocon.set_pull_up(*pin);
                    });
                }
                gpio::FloatingState::PullDown => {
                    self.iocon.map(|iocon| {
                        iocon.set_pull_down(*pin);
                    });
                }
            }
        }
    }
    fn floating_state(&self) -> gpio::FloatingState {
        gpio::FloatingState::PullNone
    }
    fn disable_input(&self) -> gpio::Configuration {
        self.make_output()
    }
    fn disable_output(&self) -> gpio::Configuration {
        self.make_input()
    }
    fn deactivate_to_low_power(&self) {
        let _state = gpio::FloatingState::PullNone;
        self.make_input();
    }
}

impl<'a> gpio::Interrupt<'a> for GpioPin<'a> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
        self.pint.map(|pint| {
            pint.set_client(0, client);
        });
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        match mode {
            gpio::InterruptEdge::RisingEdge => {
                self.pint.map(|pint| {
                    pint.configure_interrupt(0, Edge::Rising);
                });
            }
            gpio::InterruptEdge::FallingEdge => {
                self.pint.map(|pint| {
                    pint.configure_interrupt(0, Edge::Falling);
                });
            }
            gpio::InterruptEdge::EitherEdge => {
                self.pint.map(|pint| {
                    pint.configure_interrupt(0, Edge::Both);
                });
            }
        }
    }

    fn disable_interrupts(&self) {
        self.pint.map(|pint| {
            pint.disable_interrupt(0);
        });
    }

    fn is_pending(&self) -> bool {
        self.pint.map_or(false, |pint| {
            let channel = 0;
            pint.is_pending(channel)
        })
    }
}
