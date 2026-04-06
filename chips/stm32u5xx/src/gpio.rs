// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::debug;
use kernel::hil::gpio;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;

use crate::exti::{Exti, LineId};

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
        /// GPIO port configuration lock register
        (0x01C => lckr: ReadWrite<u32>),
        /// GPIO alternate function low register
        (0x020 => afrl: ReadWrite<u32>),
        /// GPIO alternate function high register
        (0x024 => afrh: ReadWrite<u32>),
        (0x028 => @END),
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PinId {
    Pin00 = 0, Pin01 = 1, Pin02 = 2, Pin03 = 3,
    Pin04 = 4, Pin05 = 5, Pin06 = 6, Pin07 = 7,
    Pin08 = 8, Pin09 = 9, Pin10 = 10, Pin11 = 11,
    Pin12 = 12, Pin13 = 13, Pin14 = 14, Pin15 = 15,
}

#[derive(Copy, Clone, PartialEq)]
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
    exti: &'a Exti<'a>,
    port_id: u32,
    client: OptionalCell<&'a dyn gpio::Client>,
    exti_lineid: OptionalCell<LineId>,
}

impl<'a> Pin<'a> {
    pub const fn new(
        base: StaticRef<GpioRegisters>,
        pin: usize,
        exti: &'a Exti<'a>,
        port_id: u32,
    ) -> Pin<'a> {
        Pin {
            registers: base,
            pin: pin,
            pin_mask: 1 << pin,
            exti,
            port_id,
            client: OptionalCell::empty(),
            exti_lineid: OptionalCell::empty(),
        }
    }

    pub fn set_mode(&self, mode: Mode) {
        let offset = self.pin * 2;
        let mut val = self.registers.moder.get();
        val &= !(0x3 << offset);
        val |= (mode as u32) << offset;
        self.registers.moder.set(val);
    }

    pub fn set_speed_high(&self) {
        let offset = self.pin * 2;
        let mut val = self.registers.ospeedr.get();
        val |= 3 << offset;
        self.registers.ospeedr.set(val);
    }

    pub fn set_alternate_function(&self, func: u32) {
        if self.pin < 8 {
            let offset = self.pin * 4;
            let mut val = self.registers.afrl.get();
            val &= !(0xF << offset);
            val |= (func & 0xF) << offset;
            self.registers.afrl.set(val);
        } else {
            let offset = (self.pin - 8) * 4;
            let mut val = self.registers.afrh.get();
            val &= !(0xF << offset);
            val |= (func & 0xF) << offset;
            self.registers.afrh.set(val);
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| client.fired());
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

impl<'a> gpio::Interrupt<'a> for Pin<'a> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let line_num = self.pin;
        if line_num < 16 {
            debug!("GPIO: Enabling interrupts for Pin {} on Port {}", line_num, self.port_id);
            let line = unsafe { core::mem::transmute::<u8, LineId>(line_num as u8) };
            self.exti_lineid.set(line);
            
            self.client.map(|client| {
                self.exti.register_client(line, client);
            });

            // 1. Route the port to the line
            self.exti.select_port(line, self.port_id);
            
            // 2. MARK THE LINE AS SECURE (Required for U5 Secure Mode!)
            self.exti.set_secure(line);

            self.exti.mask_interrupt(line);
            self.exti.clear_pending(line);

            match mode {
                gpio::InterruptEdge::EitherEdge => {
                    self.exti.select_rising_trigger(line);
                    self.exti.select_falling_trigger(line);
                }
                gpio::InterruptEdge::RisingEdge => {
                    self.exti.select_rising_trigger(line);
                    self.exti.deselect_falling_trigger(line);
                }
                gpio::InterruptEdge::FallingEdge => {
                    self.exti.deselect_rising_trigger(line);
                    self.exti.select_falling_trigger(line);
                }
            }
            self.exti.unmask_interrupt(line);
        }
    }

    fn disable_interrupts(&self) {
        self.exti_lineid.map(|line| {
            self.exti.mask_interrupt(line);
            self.exti.clear_pending(line);
        });
    }

    fn is_pending(&self) -> bool {
        self.exti_lineid.map_or(false, |line| self.exti.is_pending(line))
    }
}

pub struct Port<'a> {
    registers: StaticRef<GpioRegisters>,
    exti: &'a Exti<'a>,
    port_id: u32,
}

impl<'a> Port<'a> {
    pub const fn new(base: StaticRef<GpioRegisters>, exti: &'a Exti<'a>, port_id: u32) -> Port<'a> {
        Port {
            registers: base,
            exti,
            port_id,
        }
    }

    pub fn pin(&self, pin: PinId) -> Pin<'a> {
        Pin::new(self.registers, pin as usize, self.exti, self.port_id)
    }
}
