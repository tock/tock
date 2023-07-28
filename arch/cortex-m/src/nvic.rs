// Licensed under the Apache License, Version 2.0[0] or the MIT License.
// SPDX-License-Identifier: Apache-2.0[0] OR MIT
// Copyright Tock Contributors 2022.

//! Cortex-M NVIC
//!
//! Most NVIC configuration is in the NVIC registers:
//! <https://developer.arm.com/docs/100165/0201/nested-vectored-interrupt-controller/nvic-programmers-model/table-of-nvic-registers>
//!
//! Also part of the NVIC conceptually is the ICTR, which in older versions of
//! the ARM ARM was listed in the "Summary of system control and ID registers
//! not in the SCB" and newer ARM ARMs just file it in its own little private
//! sub-section with the NVIC documentation. Seems a configuration register
//! without a home, so we include it in the NVIC files as it's conceptually here.
//! <https://developer.arm.com/docs/ddi0337/latest/nested-vectored-interrupt-controller/nvic-programmers-model/interrupt-controller-type-register-ictr>

use kernel::platform::chip::Interrupts;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// Generates the CortexMInterrupts used for the NVIC's mask functions
/// `next_pending_from_set` and `next_pending_from_set`.
///
/// if let Some(interrupt) =
///     cortexm0p::nvic::next_pending_from_set(interrupts!(interrupts::SIO_IRQ_PROC1))
/// {
///     // ...
/// }
#[macro_export]
macro_rules! interrupts {
    ($($interrupt: expr),+) => {{
        let mut high_interrupt: u128 = 0;
        let mut low_interrupt: u128 = 0;
        $(
            if ($interrupt < 128) {
                low_interrupt = low_interrupt | (1 << $interrupt) as u128
            }
            else
            {
                high_interrupt = high_interrupt | (1 << ($interrupt-128)) as u128
            }
        );+
        CortexMInterrupts(high_interrupt, low_interrupt)
    }};
}

register_structs! {
    /// NVIC Registers.
    ///
    /// Note this generic interface exposes all possible NVICs. Most cores will
    /// not implement all NVIC_XXXX registers. If you need to find the number
    /// of NVICs dynamically, consult `ICTR.INTLINESNUM`.
    NvicRegisters {
        (0x000 => _reserved0),

        /// Interrupt Controller Type Register
        (0x004 => ictr: ReadOnly<u32, InterruptControllerType::Register>),

        (0x008 => _reserved1),

        /// Interrupt Set-Enable Registers
        (0x100 => iser: [ReadWrite<u32, NvicSetClear::Register>; 32]),

        /// Interrupt Clear-Enable Registers
        (0x180 => icer: [ReadWrite<u32, NvicSetClear::Register>; 32]),

        /// Interrupt Set-Pending Registers
        (0x200 => ispr: [ReadWrite<u32, NvicSetClear::Register>; 32]),

        /// Interrupt Clear-Pending Registers
        (0x280 => icpr: [ReadWrite<u32, NvicSetClear::Register>; 32]),

        /// Interrupt Active Bit Registers
        (0x300 => iabr: [ReadWrite<u32, NvicSetClear::Register>; 32]),

        (0x380 => _reserved2),

        /// Interrupt Priority Registers
        (0x400 => ipr: [ReadWrite<u32, NvicInterruptPriority::Register>; 252]),

        (0x7f0 => @END),
    }
}

register_bitfields![u32,
    InterruptControllerType [
        /// Total number of interrupt lines in groups of 32
        INTLINESNUM     OFFSET(0)   NUMBITS(4)
    ],

    NvicSetClear [
        /// For register NVIC_XXXXn, access interrupt (m+(32*n)).
        ///  - m takes the values from 31 to 0, except for NVIC_XXXX15, where:
        ///     - m takes the values from 15 to 0
        ///     - register bits[31:16] are reserved, RAZ/WI
        BITS            OFFSET(0)   NUMBITS(32)
    ],

    NvicInterruptPriority [
        /// For register NVIC_IPRn, priority of interrupt number 4n+3.
        PRI_N3          OFFSET(24)  NUMBITS(8),

        /// For register NVIC_IPRn, priority of interrupt number 4n+2.
        PRI_N2          OFFSET(16)  NUMBITS(8),

        /// For register NVIC_IPRn, priority of interrupt number 4n+1.
        PRI_N1          OFFSET(8)   NUMBITS(8),

        /// For register NVIC_IPRn, priority of interrupt number 4n.
        PRI_N0          OFFSET(0)   NUMBITS(8)
    ]
];

/// The NVIC peripheral in MMIO space.
const NVIC: StaticRef<NvicRegisters> =
    unsafe { StaticRef::new(0xe000e000 as *const NvicRegisters) };

/// Number of valid NVIC_XXXX registers. Note this is a ceiling on the number
/// of available interrupts (as this is the number of banks of 32), but the
/// actual number may be less. See NVIC and ICTR documentation for more detail.
fn number_of_nvic_registers() -> usize {
    (NVIC.ictr.read(InterruptControllerType::INTLINESNUM) + 1) as usize
}

/// Clear all pending interrupts
pub unsafe fn clear_all_pending() {
    for icpr in NVIC.icpr.iter().take(number_of_nvic_registers()) {
        icpr.set(!0)
    }
}

/// Enable all interrupts
pub unsafe fn enable_all() {
    for icer in NVIC.iser.iter().take(number_of_nvic_registers()) {
        icer.set(!0)
    }
}

/// Disable all interrupts
pub unsafe fn disable_all() {
    for icer in NVIC.icer.iter().take(number_of_nvic_registers()) {
        icer.set(!0)
    }
}

/// Get the index (0-240) the lowest number pending interrupt, or `None` if none
/// are pending.
pub unsafe fn next_pending() -> Option<u32> {
    for (block, ispr) in NVIC
        .ispr
        .iter()
        .take(number_of_nvic_registers())
        .enumerate()
    {
        let ispr = ispr.get();

        // If there are any high bits there is a pending interrupt
        if ispr != 0 {
            // trailing_zeros == index of first high bit
            let bit = ispr.trailing_zeros();
            return Some(block as u32 * 32 + bit);
        }
    }
    None
}

/// Get the index (0-240) the lowest number pending interrupt while ignoring the interrupts
/// that correspond to the bits set in mask, or `None` if none
/// are pending.
pub unsafe fn next_pending_from_set(set: CortexMInterrupts) -> Option<u32> {
    for (block, ispr) in NVIC
        .ispr
        .iter()
        .take(number_of_nvic_registers())
        .enumerate()
    {
        let interrupt_set = if block < 4 { set.0[1] } else { set.0[0] };
        let ispr_masked = ispr.get() & !((interrupt_set >> (32 * block % 4)) as u32);

        // If there are any high bits there is a pending interrupt
        if ispr_masked != 0 {
            // trailing_zeros == index of first high bit
            let bit = ispr_masked.trailing_zeros();
            return Some(block as u32 * 32 + bit);
        }
    }
    None
}

pub unsafe fn has_pending() -> CortexMInterrupts {
    NVIC.ispr
        .iter()
        .take(number_of_nvic_registers())
        .enumerate()
        .fold(
            CortexMInterrupts::empty(),
            |mut interrupts, (position, ispr)| {
                interrupts.set_from_register(position, ispr.get());
                interrupts
            },
        )
}

/// Returns whether there are any pending interrupt bits set while ignoring
/// the indices that correspond to the bits set in mask
pub unsafe fn has_pending_from_set(set: CortexMInterrupts) -> CortexMInterrupts {
    NVIC.ispr
        .iter()
        .take(number_of_nvic_registers())
        .enumerate()
        .fold(
            CortexMInterrupts::empty(),
            |mut interrupts, (position, ispr)| {
                interrupts.set_from_register(position, ispr.get() & set.get_register(position));
                interrupts
            },
        )
}

/// An opaque wrapper for a single NVIC interrupt.
///
/// Hand these out to low-level driver to let them control their own interrupts
/// but not others.
pub struct Nvic(u32);

impl Nvic {
    /// Creates a new `Nvic`
    ///
    /// Marked unsafe because only chip/platform configuration code should be
    /// able to create these.
    pub const unsafe fn new(idx: u32) -> Nvic {
        Nvic(idx)
    }

    /// Enable the interrupt
    pub fn enable(&self) {
        let idx = self.0 as usize;

        NVIC.iser[idx / 32].set(1 << (self.0 & 31));
    }

    /// Disable the interrupt
    pub fn disable(&self) {
        let idx = self.0 as usize;

        NVIC.icer[idx / 32].set(1 << (self.0 & 31));
    }

    /// Clear pending state
    pub fn clear_pending(&self) {
        let idx = self.0 as usize;

        NVIC.icpr[idx / 32].set(1 << (self.0 & 31));
    }
}

/// The interrupts set is defined as two u128 fields,
///   mask.0[0] has the bits corresponding to interrupts from 128 to 240
///   mask.0[1] has the bits corresponding to interrupts from 0 to 127
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CortexMInterrupts([u128; 2]);

impl CortexMInterrupts {
    fn set_from_register(&mut self, position: usize, register: u32) {
        if position < 4 {
            self.0[0] = self.0[0] | ((register as u128) << (32 * position))
        } else {
            self.0[1] = self.0[1] | ((register as u128) << (32 * (position - 4)))
        }
    }

    fn get_register(&self, position: usize) -> u32 {
        if position < 4 {
            ((self.0[0] >> (32 * position)) as u32) & 0xFFFF_FFFF
        } else {
            ((self.0[1] >> (32 * position - 4)) as u32) & 0xFFFF_FFFF
        }
    }
}

impl Interrupts for CortexMInterrupts {
    fn full() -> Self {
        CortexMInterrupts([!0x0, !0x0])
    }

    fn empty() -> Self {
        CortexMInterrupts([0, 0])
    }

    fn is_set(&self, interrupt_number: u8) -> bool {
        if interrupt_number < 128 {
            self.0[1] & (1 << interrupt_number) != 0
        } else {
            self.0[0] & (1 << interrupt_number - 128) != 0
        }
    }

    fn is_empty(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    fn set(&mut self, interrupt_number: u8) {
        if interrupt_number < 128 {
            self.0[1] = self.0[1] | (1 << interrupt_number)
        } else {
            self.0[0] = self.0[0] | (1 << interrupt_number - 128)
        }
    }

    fn clear(&mut self, interrupt_number: u8) {
        if interrupt_number < 128 {
            self.0[1] = self.0[1] ^ (1 << interrupt_number)
        } else {
            self.0[0] = self.0[0] ^ (1 << interrupt_number - 128)
        }
    }
}
