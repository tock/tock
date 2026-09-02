// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2024.
//
// Author: Radu Matei <radu.matei.05.21@gmail.com>
//         Alberto Udrea <albertoudrea4@gmail.com>

//! Programmable Input Output (PIO) hardware.
//!
//! The driver itself lives in the `rp2xxx` crate, shared with the other RP2
//! chip. What is specific to this chip is here: where the two PIO blocks are,
//! how many there are, where each one's interrupt registers start, and which
//! GPIO alternate function selects them.
//!
//! Refer to the RP2040 Datasheet, Section 3 for more information.
//! RP2040 Datasheet [1].
//!
//! [1]: https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf

use crate::gpio::{GpioFunction, RPGpioPin};
use kernel::utilities::StaticRef;
use rp2xxx::pio::{PioIrqRegisters, PioRegisters};

pub use rp2xxx::pio::{
    InterruptSources, LoadedProgram, PioBlock, PioFifoJoin, PioMovStatusType, PioPin, PioRxClient,
    PioSmClient, PioTxClient, ProgramError, RelocatedProgram, SMNumber, StateMachine,
    StateMachineConfiguration,
};

/// There are 2 PIO blocks on the RP2040.
#[derive(Clone, Copy, PartialEq)]
pub enum PIONumber {
    PIO0 = 0,
    PIO1 = 1,
}

impl PioBlock for PIONumber {}

/// The shared PIO driver, with this chip's block numbering filled in.
pub type Pio = rp2xxx::pio::Pio<PIONumber>;

const PIO_0_BASE_ADDRESS: usize = 0x50200000;
const PIO_1_BASE_ADDRESS: usize = 0x50300000;

/// Where a block's interrupt registers start, relative to the block.
///
/// They follow the state machine registers directly on this chip. The RP2350
/// puts sixteen `RXFn_PUTGETn` registers and a `GPIOBASE` in between, so its
/// offset is larger.
const IRQ_OFFSET: usize = 0x128;

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

/// Point a pin at one of this chip's PIO blocks.
///
/// Which alternate function selects a PIO block, and how many blocks there
/// are, are facts about this chip rather than about the state machines, so
/// this stays here rather than moving into the shared driver.
///
/// Takes the block rather than the `Pio` driving it, because a pin can be
/// pointed at a block without a driver in hand: `PioPad::select_pio` has
/// exactly that.
pub fn gpio_init(block: PIONumber, pin: &RPGpioPin) {
    pin.set_function(match block {
        PIONumber::PIO0 => GpioFunction::PIO0,
        PIONumber::PIO1 => GpioFunction::PIO1,
    });
}

impl PioPin for RPGpioPin<'_> {
    fn pin_number(&self) -> u32 {
        self.pin() as u32
    }
}
