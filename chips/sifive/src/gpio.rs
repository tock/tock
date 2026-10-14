// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! General Purpose Input/Output driver.

use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    Field, FieldValue, ReadOnly, ReadWrite, register_bitfields, register_structs,
};

register_structs! {
    pub GpioRegisters {
        /// Pin value.
        (0x000 => value: ReadOnly<u32, pins::Register>),
        /// Pin Input Enable Register
        (0x004 => input_en: ReadWrite<u32, pins::Register>),
        /// Pin Output Enable Register
        (0x008 => output_en: ReadWrite<u32, pins::Register>),
        /// Output Port Value Register
        (0x00c => port: ReadWrite<u32, pins::Register>),
        /// Internal Pull-Up Enable Register
        (0x010 => pullup: ReadWrite<u32, pins::Register>),
        /// Drive Strength Register
        (0x014 => drive: ReadWrite<u32, pins::Register>),
        /// Rise Interrupt Enable Register
        (0x018 => rise_ie: ReadWrite<u32, pins::Register>),
        /// Rise Interrupt Pending Register
        (0x01c => rise_ip: ReadWrite<u32, pins::Register>),
        /// Fall Interrupt Enable Register
        (0x020 => fall_ie: ReadWrite<u32, pins::Register>),
        /// Fall Interrupt Pending Register
        (0x024 => fall_ip: ReadWrite<u32, pins::Register>),
        /// High Interrupt Enable Register
        (0x028 => high_ie: ReadWrite<u32, pins::Register>),
        /// High Interrupt Pending Register
        (0x02c => high_ip: ReadWrite<u32, pins::Register>),
        /// Low Interrupt Enable Register
        (0x030 => low_ie: ReadWrite<u32, pins::Register>),
        /// Low Interrupt Pending Register
        (0x034 => low_ip: ReadWrite<u32, pins::Register>),
        /// HW I/O Function Enable Register
        (0x038 => iof_en: ReadWrite<u32, pins::Register>),
        /// HW I/O Function Select Register
        (0x03c => iof_sel: ReadWrite<u32, pins::Register>),
        /// Output XOR (invert) Register
        (0x040 => out_xor: ReadWrite<u32, pins::Register>),
        (0x044 => @END),
    }
}

register_bitfields![u32,
    pub pins [
        pin0 0,
        pin1 1,
        pin2 2,
        pin3 3,
        pin4 4,
        pin5 5,
        pin6 6,
        pin7 7,
        pin8 8,
        pin9 9,
        pin10 10,
        pin11 11,
        pin12 12,
        pin13 13,
        pin14 14,
        pin15 15,
        pin16 16,
        pin17 17,
        pin18 18,
        pin19 19,
        pin20 20,
        pin21 21,
        pin22 22,
        pin23 23,
        pin24 24,
        pin25 25,
        pin26 26,
        pin27 27,
        pin28 28,
        pin29 29,
        pin30 30,
        pin31 31
    ]
];

pub struct GpioPin<'a> {
    registers: StaticRef<GpioRegisters>,
    pin: Field<u32, pins::Register>,
    set: FieldValue<u32, pins::Register>,
    clear: FieldValue<u32, pins::Register>,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
}

impl<'a> GpioPin<'a> {
    pub const fn new(
        base: StaticRef<GpioRegisters>,
        pin: Field<u32, pins::Register>,
        set: FieldValue<u32, pins::Register>,
        clear: FieldValue<u32, pins::Register>,
    ) -> GpioPin<'a> {
        GpioPin {
            registers: base,
            pin,
            set,
            clear,
            client: OptionalCell::empty(),
        }
    }

    /// Configure this pin as IO Function 0. What that maps to is chip- and pin-
    /// specific.
    pub fn iof0(&self) {
        let regs = self.registers;

        regs.out_xor.modify(self.clear);
        regs.iof_sel.modify(self.clear);
        regs.iof_en.modify(self.set);
    }

    /// Configure this pin as IO Function 1. What that maps to is chip- and pin-
    /// specific.
    pub fn iof1(&self) {
        let regs = self.registers;

        regs.out_xor.modify(self.clear);
        regs.iof_sel.modify(self.set);
        regs.iof_en.modify(self.set);
    }

    /// There are separate interrupts in PLIC for each pin, so the interrupt
    /// handler only needs to exist on each pin.
    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Clear the pending GPIO interrupt.
        regs.rise_ip.modify(self.set);
        regs.fall_ip.modify(self.set);
        regs.high_ip.modify(self.set);
        regs.low_ip.modify(self.set);

        self.client.map(|client| {
            client.fired();
        });
    }
}

impl hil::gpio::Configure for GpioPin<'_> {
    fn configuration(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        if regs.iof_en.is_set(self.pin) {
            return hil::gpio::Configuration::Function;
        }

        let output = regs.output_en.is_set(self.pin);
        let input = regs.output_en.is_set(self.pin);

        match (input, output) {
            (true, true) => hil::gpio::Configuration::InputOutput,
            (true, false) => hil::gpio::Configuration::Input,
            (false, true) => hil::gpio::Configuration::Output,
            (false, false) => hil::gpio::Configuration::LowPower,
        }
    }

    fn set_floating_state(&self, mode: hil::gpio::FloatingState) {
        let regs = self.registers;

        match mode {
            hil::gpio::FloatingState::PullUp => {
                regs.pullup.modify(self.set);
            }
            hil::gpio::FloatingState::PullDown => {
                regs.pullup.modify(self.clear);
            }
            hil::gpio::FloatingState::PullNone => {
                regs.pullup.modify(self.clear);
            }
        }
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        let regs = self.registers;
        if regs.pullup.is_set(self.pin) {
            hil::gpio::FloatingState::PullUp
        } else {
            hil::gpio::FloatingState::PullDown
        }
    }

    fn deactivate_to_low_power(&self) {
        self.disable_input();
        self.disable_output();
    }

    fn make_output(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.drive.modify(self.clear);
        regs.out_xor.modify(self.clear);
        regs.output_en.modify(self.set);
        regs.iof_en.modify(self.clear);

        self.configuration()
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        let regs = self.registers;
        regs.output_en.modify(self.clear);
        self.configuration()
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.input_en.modify(self.set);
        regs.iof_en.modify(self.clear);

        self.configuration()
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        let regs = self.registers;
        regs.input_en.modify(self.clear);
        self.configuration()
    }
}

impl hil::gpio::Input for GpioPin<'_> {
    fn read(&self) -> bool {
        let regs = self.registers;

        regs.value.is_set(self.pin)
    }
}

impl hil::gpio::Output for GpioPin<'_> {
    fn toggle(&self) -> bool {
        let regs = self.registers;

        let current_outputs = regs.port.extract();
        if current_outputs.is_set(self.pin) {
            regs.port.modify_no_read(current_outputs, self.clear);
        } else {
            regs.port.modify_no_read(current_outputs, self.set);
        }
        regs.port.extract().is_set(self.pin)
    }

    fn set(&self) {
        let regs = self.registers;

        regs.port.modify(self.set);
    }

    fn clear(&self) {
        let regs = self.registers;

        regs.port.modify(self.clear);
    }
}

impl<'a> hil::gpio::Interrupt<'a> for GpioPin<'a> {
    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        let regs = self.registers;

        regs.pullup.modify(self.clear);
        regs.input_en.modify(self.set);
        regs.iof_en.modify(self.clear);

        match mode {
            hil::gpio::InterruptEdge::RisingEdge => {
                regs.rise_ie.modify(self.set);
            }
            hil::gpio::InterruptEdge::FallingEdge => {
                regs.fall_ie.modify(self.set);
            }
            hil::gpio::InterruptEdge::EitherEdge => {
                regs.rise_ie.modify(self.set);
                regs.fall_ie.modify(self.set);
            }
        }
    }

    fn disable_interrupts(&self) {
        let regs = self.registers;

        regs.rise_ie.modify(self.clear);
        regs.fall_ie.modify(self.clear);
    }

    fn is_pending(&self) -> bool {
        let regs = self.registers;

        regs.rise_ip.is_set(self.pin) || regs.fall_ip.is_set(self.pin)
    }
}
