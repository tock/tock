// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Programmable Input Output (PIO) hardware.
//!
//! The driver itself lives in the `rp2xxx` crate, shared with the other RP2
//! chip. What is specific to this chip is here: where the three PIO blocks
//! are, how many there are, where each one's interrupt registers start, and
//! which GPIO alternate function selects them.
//!
//! The interrupt registers begin further into the block than they do on the
//! RP2040. This chip inserts sixteen `RXFn_PUTGETn` registers and a
//! `GPIOBASE` at +0x168 first, so `INTR` lands at +0x16c rather than +0x128.
//!
//! `GPIOBASE` selects which thirty two GPIOs a block can reach. It resets to
//! zero, meaning GPIO 0 to 31, which covers every pin an RP2350A brings out,
//! and nothing here writes it.
//!
//! Refer to the RP2350 Datasheet, Section 11.
//! RP2350 Datasheet [1].
//!
//! [1]: https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf

use crate::gpio::{GpioFunction, RPGpioPin};
use kernel::utilities::StaticRef;
use rp2xxx::pio::{PioIrqRegisters, PioRegisters};

pub use rp2xxx::pio::{
    InterruptSources, LoadedProgram, PioBlock, PioFifoJoin, PioMovStatusType, PioPin, PioRxClient,
    PioSmClient, PioTxClient, ProgramError, RelocatedProgram, SMNumber, StateMachine,
    StateMachineConfiguration,
};

/// There are 3 PIO blocks on the RP2350, one more than the RP2040 has.
#[derive(Clone, Copy, PartialEq)]
pub enum PIONumber {
    PIO0 = 0,
    PIO1 = 1,
    PIO2 = 2,
}

impl PioBlock for PIONumber {}

/// The shared PIO driver, with this chip's block numbering filled in.
pub type Pio = rp2xxx::pio::Pio<PIONumber>;

const PIO_0_BASE_ADDRESS: usize = 0x50200000;
const PIO_1_BASE_ADDRESS: usize = 0x50300000;
const PIO_2_BASE_ADDRESS: usize = 0x50400000;

/// Where a block's interrupt registers start, relative to the block.
///
/// The RP2040 puts them at +0x128, directly after the state machine
/// registers. This chip has sixteen `RXFn_PUTGETn` registers and a `GPIOBASE`
/// in between.
const IRQ_OFFSET: usize = 0x16c;

const fn regs(base: usize) -> StaticRef<PioRegisters> {
    unsafe { StaticRef::new(base as *const PioRegisters) }
}

const fn irq_regs(base: usize) -> StaticRef<PioIrqRegisters> {
    unsafe { StaticRef::new((base + IRQ_OFFSET) as *const PioIrqRegisters) }
}

fn new_pio(block: PIONumber, base: usize) -> Pio {
    Pio::new(
        block,
        regs(base),
        irq_regs(base),
        regs(base + 0x1000),
        regs(base + 0x2000),
        regs(base + 0x3000),
    )
}

/// Create a driver for PIO0.
pub fn new_pio0() -> Pio {
    new_pio(PIONumber::PIO0, PIO_0_BASE_ADDRESS)
}

/// Create a driver for PIO1.
pub fn new_pio1() -> Pio {
    new_pio(PIONumber::PIO1, PIO_1_BASE_ADDRESS)
}

/// Create a driver for PIO2.
pub fn new_pio2() -> Pio {
    new_pio(PIONumber::PIO2, PIO_2_BASE_ADDRESS)
}

/// Point a pin at the PIO block a driver drives.
///
/// Which alternate function selects a PIO block, and how many blocks there
/// are, are facts about this chip rather than about the state machines.
pub fn gpio_init(pio: &Pio, pin: &RPGpioPin) {
    pin.set_function(match pio.number() {
        PIONumber::PIO0 => GpioFunction::PIO0,
        PIONumber::PIO1 => GpioFunction::PIO1,
        PIONumber::PIO2 => GpioFunction::PIO2,
    });
}

impl PioPin for RPGpioPin<'_> {
    fn pin_number(&self) -> u32 {
        self.pin() as u32
    }
}
