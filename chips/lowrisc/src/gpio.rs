// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! General Purpose Input/Output driver.

use kernel::hil::gpio;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, Field, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

register_structs! {
    pub GpioRegisters {
        (0x00 => intr_state: ReadWrite<u32, pins::Register>),
        (0x04 => intr_enable: ReadWrite<u32, pins::Register>),
        (0x08 => intr_test: WriteOnly<u32, pins::Register>),
        (0x0c => alert_test: ReadOnly<u32>),
        (0x10 => data_in: ReadOnly<u32, pins::Register>),
        (0x14 => direct_out: ReadWrite<u32, pins::Register>),
        (0x18 => masked_out_lower: ReadWrite<u32, mask_half::Register>),
        (0x1c => masked_out_upper: ReadWrite<u32, mask_half::Register>),
        (0x20 => direct_oe: ReadWrite<u32, pins::Register>),
        (0x24 => masked_oe_lower: ReadWrite<u32, mask_half::Register>),
        (0x28 => masked_oe_upper: ReadWrite<u32, mask_half::Register>),
        (0x2c => intr_ctrl_en_rising: ReadWrite<u32, pins::Register>),
        (0x30 => intr_ctrl_en_falling: ReadWrite<u32, pins::Register>),
        (0x34 => intr_ctrl_en_lvlhigh: ReadWrite<u32, pins::Register>),
        (0x38 => intr_ctrl_en_lvllow: ReadWrite<u32, pins::Register>),
        (0x3c => ctrl_en_input_filter: ReadWrite<u32, pins::Register>),
        (0x40 => @END),
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
    ],
    mask_half [
        data OFFSET(0) NUMBITS(16) [],
        mask OFFSET(16) NUMBITS(16) []
    ]
];

pub type GpioBitfield = Field<u32, pins::Register>;

pub struct GpioPin<'a, PAD> {
    gpio_registers: StaticRef<GpioRegisters>,
    padctl: PAD,
    pin: Field<u32, pins::Register>,
    client: OptionalCell<&'a dyn gpio::Client>,
}

impl<'a, PAD> GpioPin<'a, PAD> {
    pub const fn new(
        gpio_base: StaticRef<GpioRegisters>,
        padctl: PAD,
        pin: Field<u32, pins::Register>,
    ) -> GpioPin<'a, PAD> {
        GpioPin {
            gpio_registers: gpio_base,
            padctl,
            pin,
            client: OptionalCell::empty(),
        }
    }

    #[inline(always)]
    fn half_set(
        val: bool,
        field: Field<u32, pins::Register>,
        lower: &ReadWrite<u32, mask_half::Register>,
        upper: &ReadWrite<u32, mask_half::Register>,
    ) {
        let shift = field.shift;
        let bit = u32::from(val);
        if shift < 16 {
            lower.write(mask_half::data.val(bit << shift) + mask_half::mask.val(1u32 << shift));
        } else {
            let upper_shift = shift - 16;
            upper.write(
                mask_half::data.val(bit << upper_shift) + mask_half::mask.val(1u32 << upper_shift),
            );
        }
    }

    pub fn handle_interrupt(&self) {
        let pin = self.pin;

        if self.gpio_registers.intr_state.is_set(pin) {
            self.gpio_registers.intr_state.modify(pin.val(1));
            self.client.map(|client| {
                client.fired();
            });
        }
    }
}

impl<PAD: gpio::Configure> gpio::Configure for GpioPin<'_, PAD> {
    fn configuration(&self) -> gpio::Configuration {
        match (
            self.padctl.configuration(),
            self.gpio_registers.direct_oe.is_set(self.pin),
        ) {
            (gpio::Configuration::InputOutput, true) => gpio::Configuration::InputOutput,
            (gpio::Configuration::InputOutput, false) => gpio::Configuration::Input,
            (gpio::Configuration::Input, false) => gpio::Configuration::Input,
            // This is configuration error we can't enable ouput
            // for GPIO pin connect to input only pad.
            (gpio::Configuration::Input, true) => gpio::Configuration::Function,
            // We curently dont support output only GPIO
            // OT register have only output_enable flag.
            (gpio::Configuration::Output, _) => gpio::Configuration::Function,
            (conf, _) => conf,
        }
    }

    fn set_floating_state(&self, mode: gpio::FloatingState) {
        self.padctl.set_floating_state(mode);
    }

    fn floating_state(&self) -> gpio::FloatingState {
        self.padctl.floating_state()
    }

    fn deactivate_to_low_power(&self) {
        self.disable_input();
        self.disable_output();
        self.padctl.deactivate_to_low_power();
    }

    fn make_output(&self) -> gpio::Configuration {
        // Re-connect in case we make output after switching from LowPower state.
        if let gpio::Configuration::InputOutput = self.padctl.make_output() {
            Self::half_set(
                true,
                self.pin,
                &self.gpio_registers.masked_oe_lower,
                &self.gpio_registers.masked_oe_upper,
            );
        }
        self.configuration()
    }

    fn disable_output(&self) -> gpio::Configuration {
        Self::half_set(
            false,
            self.pin,
            &self.gpio_registers.masked_oe_lower,
            &self.gpio_registers.masked_oe_upper,
        );
        self.configuration()
    }

    fn make_input(&self) -> gpio::Configuration {
        // Re-connect in case we make input after switching from LowPower state.
        self.padctl.make_input();
        self.configuration()
    }

    fn disable_input(&self) -> gpio::Configuration {
        self.configuration()
    }
}

impl<PAD> gpio::Input for GpioPin<'_, PAD> {
    fn read(&self) -> bool {
        self.gpio_registers.data_in.is_set(self.pin)
    }
}

impl<PAD> gpio::Output for GpioPin<'_, PAD> {
    fn toggle(&self) -> bool {
        let pin = self.pin;
        let new_state = !self.gpio_registers.direct_out.is_set(pin);

        Self::half_set(
            new_state,
            self.pin,
            &self.gpio_registers.masked_out_lower,
            &self.gpio_registers.masked_out_upper,
        );
        new_state
    }

    fn set(&self) {
        Self::half_set(
            true,
            self.pin,
            &self.gpio_registers.masked_out_lower,
            &self.gpio_registers.masked_out_upper,
        );
    }

    fn clear(&self) {
        Self::half_set(
            false,
            self.pin,
            &self.gpio_registers.masked_out_lower,
            &self.gpio_registers.masked_out_upper,
        );
    }
}

impl<'a, PAD> gpio::Interrupt<'a> for GpioPin<'a, PAD> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let pin = self.pin;

        match mode {
            gpio::InterruptEdge::RisingEdge => {
                self.gpio_registers.intr_ctrl_en_rising.modify(pin.val(1));
                self.gpio_registers.intr_ctrl_en_falling.modify(pin.val(0));
            }
            gpio::InterruptEdge::FallingEdge => {
                self.gpio_registers.intr_ctrl_en_rising.modify(pin.val(0));
                self.gpio_registers.intr_ctrl_en_falling.modify(pin.val(1));
            }
            gpio::InterruptEdge::EitherEdge => {
                self.gpio_registers.intr_ctrl_en_rising.modify(pin.val(1));
                self.gpio_registers.intr_ctrl_en_falling.modify(pin.val(1));
            }
        }
        self.gpio_registers.intr_state.modify(pin.val(1));
        self.gpio_registers.intr_enable.modify(pin.val(1));
    }

    fn disable_interrupts(&self) {
        let pin = self.pin;

        self.gpio_registers.intr_enable.modify(pin.val(0));
        // Clear any pending interrupt
        self.gpio_registers.intr_state.modify(pin.val(1));
    }

    fn is_pending(&self) -> bool {
        self.gpio_registers.intr_state.is_set(self.pin)
    }
}
