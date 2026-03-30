// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::hil::gpio;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub GpioRegisters {
        /// GPIO port mode register
        (0x000 => moder: ReadWrite<u32>),
        /// GPIO port output type register
        (0x004 => otyper: ReadWrite<u32>),
        /// GPIO port output speed register
        (0x008 => ospeedr: ReadWrite<u32>),
        /// GPIO port pull-up/pull-down register
        (0x00C => pupdr: ReadWrite<u32>),
        /// GPIO port input data register
        (0x010 => idr: ReadWrite<u32>),
        /// GPIO port output data register
        (0x014 => odr: ReadWrite<u32>),
        /// GPIO port bit set/reset register
        (0x018 => bsrr: ReadWrite<u32>),
        (0x01C => @END),
    }
}

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

pub struct Pin<'a> {
    registers: StaticRef<GpioRegisters>,
    pin: usize,
    pin_mask: u32,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Pin<'a> {
    pub const fn new(base: StaticRef<GpioRegisters>, pin: usize) -> Pin<'a> {
        Pin {
            registers: base,
            pin: pin,
            pin_mask: 1 << pin,
            _marker: core::marker::PhantomData,
        }
    }

    fn set_mode(&self, mode: Mode) {
        let offset = self.pin * 2;
        let mut val = self.registers.moder.get();
        val &= !(0x3 << offset);
        val |= (mode as u32) << offset;
        self.registers.moder.set(val);
    }

    fn get_mode(&self) -> Mode {
        let offset = self.pin * 2;
        let val = (self.registers.moder.get() >> offset) & 0x3;
        match val {
            0 => Mode::Input,
            1 => Mode::Output,
            2 => Mode::AlternateFunction,
            _ => Mode::Analog,
        }
    }

    fn set_pull(&self, pull: PullUpPullDown) {
        let offset = self.pin * 2;
        let mut val = self.registers.pupdr.get();
        val &= !(0x3 << offset);
        val |= (pull as u32) << offset;
        self.registers.pupdr.set(val);
    }

    fn get_pull(&self) -> PullUpPullDown {
        let offset = self.pin * 2;
        let val = (self.registers.pupdr.get() >> offset) & 0x3;
        match val {
            1 => PullUpPullDown::PullUp,
            2 => PullUpPullDown::PullDown,
            _ => PullUpPullDown::None,
        }
    }
}

impl<'a> gpio::Configure for Pin<'a> {
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

impl<'a> gpio::Input for Pin<'a> {
    fn read(&self) -> bool {
        (self.registers.idr.get() & self.pin_mask) != 0
    }
}

impl<'a> gpio::Output for Pin<'a> {
    fn set(&self) {
        self.registers.bsrr.set(self.pin_mask);
    }

    fn clear(&self) {
        self.registers.bsrr.set(self.pin_mask << 16);
    }

    fn toggle(&self) -> bool {
        let val = self.registers.odr.get();
        self.registers.odr.set(val ^ self.pin_mask);
        (self.registers.odr.get() & self.pin_mask) != 0
    }
}
