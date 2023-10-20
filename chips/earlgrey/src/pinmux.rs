// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mux'ing between physical pads and GPIO or other peripherals.

use kernel::hil::gpio;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, FieldValue, LocalRegisterCopy};
use kernel::utilities::StaticRef;

use crate::registers::pinmux_regs::{
    PinmuxRegisters, DIO_PAD_ATTR_REGWEN, MIO_OUTSEL_REGWEN, MIO_PAD_ATTR_REGWEN,
    MIO_PERIPH_INSEL_REGWEN,
};
use crate::registers::top_earlgrey::{
    DirectPads, MuxedPads, PinmuxInsel, PinmuxOutsel, PinmuxPeripheralIn, PINMUX_AON_BASE_ADDR,
};

pub const PINMUX_BASE: StaticRef<PinmuxRegisters> =
    unsafe { StaticRef::new(PINMUX_AON_BASE_ADDR as *const PinmuxRegisters) };

// To avoid code duplication for MIO/DIO we introduce
// one register layout for both types of IO. In the future this code
// should be replaced by official improved auto generated definitions.
// OpenTitan documentation reference:
// <https://opentitan.org/book/hw/ip/pinmux/doc/registers.html#fields-6>
// <https://opentitan.org/book/hw/ip/pinmux/doc/registers.html#fields-8>
register_bitfields![u32,
    pub(crate) PAD_ATTR [
        INVERT OFFSET(0) NUMBITS(1) [],
        VIRTUAL_OPEN_DRAIN_EN OFFSET(1) NUMBITS(1) [],
        PULL_EN OFFSET(2) NUMBITS(1) [],
        PULL OFFSET(3) NUMBITS(1) [
            DOWN = 0,
            UP = 1,
        ],
        KEEPER_EN OFFSET(4) NUMBITS(1) [],
        SCHMITT_EN OFFSET(5) NUMBITS(1) [],
        OPEN_DRAIN_EN OFFSET(6) NUMBITS(1) [],
        SLEW_RATE OFFSET(16) NUMBITS(2) [],
        DRIVE_STRENGTH OFFSET(20) NUMBITS(4) [],
    ],
];

type PadAttribute = LocalRegisterCopy<u32, PAD_ATTR::Register>;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Pad {
    Mio(MuxedPads),
    Dio(DirectPads),
}

impl Pad {
    /// Extract value of attributes using common layout
    fn pad_attr(&self) -> PadAttribute {
        PadAttribute::new(match *self {
            Self::Mio(mio) => PINMUX_BASE.mio_pad_attr[mio as usize].get(),
            Self::Dio(dio) => PINMUX_BASE.dio_pad_attr[dio as usize].get(),
        })
    }

    /// Modify value of pad attribute using common MIO/DIO register layout
    fn modify_pad_attr(&self, flags: FieldValue<u32, PAD_ATTR::Register>) {
        let mut attr = self.pad_attr();
        attr.modify(flags);
        match *self {
            Self::Mio(mio) => &PINMUX_BASE.mio_pad_attr[mio as usize].set(attr.get()),
            Self::Dio(dio) => &PINMUX_BASE.dio_pad_attr[dio as usize].set(attr.get()),
        };
    }

    pub fn set_floating_state(&self, mode: gpio::FloatingState) {
        self.modify_pad_attr(match mode {
            gpio::FloatingState::PullUp => PAD_ATTR::PULL_EN::SET + PAD_ATTR::PULL::UP,
            gpio::FloatingState::PullDown => PAD_ATTR::PULL_EN::SET + PAD_ATTR::PULL::DOWN,
            gpio::FloatingState::PullNone => PAD_ATTR::PULL_EN::CLEAR + PAD_ATTR::PULL::CLEAR,
        });
    }

    pub fn set_output_open_drain(&self) {
        self.modify_pad_attr(PAD_ATTR::OPEN_DRAIN_EN::SET);
    }

    pub fn set_output_push_pull(&self) {
        self.modify_pad_attr(PAD_ATTR::OPEN_DRAIN_EN::CLEAR);
    }

    pub fn set_invert_sense(&self, invert: bool) {
        if invert {
            self.modify_pad_attr(PAD_ATTR::INVERT::SET)
        } else {
            self.modify_pad_attr(PAD_ATTR::INVERT::CLEAR)
        }
    }

    pub fn floating_state(&self) -> gpio::FloatingState {
        let pad_attr: PadAttribute = self.pad_attr();
        if pad_attr.matches_all(PAD_ATTR::PULL::UP + PAD_ATTR::PULL_EN::SET) {
            gpio::FloatingState::PullUp
        } else if pad_attr.matches_all(PAD_ATTR::PULL::DOWN + PAD_ATTR::PULL_EN::SET) {
            gpio::FloatingState::PullDown
        } else {
            gpio::FloatingState::PullNone
        }
    }

    /// Prohibits any further changes to input/output/open-drain or pullup configuration.
    pub fn lock_pad_attributes(&self) {
        match *self {
            Self::Mio(mio) => PINMUX_BASE.mio_pad_attr_regwen[(mio as u32) as usize]
                .write(MIO_PAD_ATTR_REGWEN::EN_0::CLEAR),
            Self::Dio(dio) => PINMUX_BASE.dio_pad_attr_regwen[(dio as u32) as usize]
                .write(DIO_PAD_ATTR_REGWEN::EN_0::CLEAR),
        };
    }
}

// Configuration of PINMUX multiplexers for I/O
// OpenTitan Documentation reference:
// https://opentitan.org/book/hw/ip/pinmux/doc/programmers_guide.html#pinmux-configuration

trait SelectOutput {
    /// Connect particular pad to internal peripheral
    fn connect_output(self, output: PinmuxOutsel);

    /// Connect particular pad output to always low
    fn connect_low(self);

    /// Connect particular pad output to always high
    fn connect_high(self);

    /// This function disconnect pad from peripheral
    /// and set it to High-Impedance state
    fn connect_high_z(self);

    /// Lock selection of output for particular pad
    fn lock(self);
}

impl SelectOutput for MuxedPads {
    fn connect_output(self, output: PinmuxOutsel) {
        PINMUX_BASE.mio_outsel[self as usize].set(output as u32)
    }

    fn connect_low(self) {
        PINMUX_BASE.mio_outsel[self as usize].set(PinmuxOutsel::ConstantZero as u32)
    }

    fn connect_high(self) {
        PINMUX_BASE.mio_outsel[self as usize].set(PinmuxOutsel::ConstantOne as u32)
    }

    fn connect_high_z(self) {
        PINMUX_BASE.mio_outsel[self as usize].set(PinmuxOutsel::ConstantHighZ as u32)
    }

    fn lock(self) {
        PINMUX_BASE.mio_outsel_regwen[self as usize].write(MIO_OUTSEL_REGWEN::EN_0::CLEAR);
    }
}

trait SelectInput {
    /// Connect internal peripheral input to particular pad
    fn connect_input(self, input: PinmuxInsel);

    /// Connect internal peripherals input to always low
    fn connect_low(self);

    /// Connect internal peripherals input to always high
    fn connect_high(self);

    /// Lock input configurations
    fn lock(self);
}

/// MuxedPads names and values overlap with PinmuxInsel,
/// function below is used to convert it to valid PinmuxInsel.
/// OpenTitan documentation reference:
/// <https://opentitan.org/book/hw/ip/pinmux/doc/programmers_guide.html#pinmux-configuration>
impl From<MuxedPads> for PinmuxInsel {
    fn from(pad: MuxedPads) -> Self {
        // Add 2 to skip constant ConstantZero and ConstantOne.
        match PinmuxInsel::try_from(pad as u32 + 2) {
            Ok(select) => select,
            Err(_) => PinmuxInsel::ConstantZero,
        }
    }
}

impl SelectInput for PinmuxPeripheralIn {
    fn connect_input(self, input: PinmuxInsel) {
        PINMUX_BASE.mio_periph_insel[self as usize].set(input as u32)
    }

    fn connect_low(self) {
        PINMUX_BASE.mio_periph_insel[self as usize].set(PinmuxInsel::ConstantZero as u32)
    }

    fn connect_high(self) {
        PINMUX_BASE.mio_periph_insel[self as usize].set(PinmuxInsel::ConstantOne as u32)
    }

    fn lock(self) {
        PINMUX_BASE.mio_periph_insel_regwen[self as usize]
            .write(MIO_PERIPH_INSEL_REGWEN::EN_0::CLEAR);
    }
}
