// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Platform Level Interrupt Control peripheral driver.

use crate::interrupts;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub IntcRegisters {
        (0x000 => _reserved0),
        (0x040 => gpio_interrupt_pro_map: ReadWrite<u32>),
        (0x044 => gpio_interrupt_pro_nmi_map: ReadWrite<u32>),
        (0x048 => _reserved1),
        (0x054 => uart0_intr_map: ReadWrite<u32>),
        (0x058 => _reserved2),
        (0x080 => timg0_intr_map: ReadWrite<u32>),
        (0x084 => timg1_intr_map: ReadWrite<u32>),
        (0x088 => _reserved3),
        (0x0f8 => status: [ReadWrite<u32>; 2]),
        (0x100 => clk_en: ReadWrite<u32>),
        (0x104 => enable: ReadWrite<u32, INT::Register>),
        (0x108 => type_reg: ReadWrite<u32, INT::Register>),
        (0x10C => clear: ReadWrite<u32, INT::Register>),
        (0x110 => eip: ReadWrite<u32, INT::Register>),
        (0x114 => _reserved4),
        (0x118 => priority: [ReadWrite<u32, PRIORITY::Register>; 31]),
        (0x194 => thresh: ReadWrite<u32, THRESH::Register>),
        (0x198 => @END),
    }
}

register_bitfields![u32,
    INT [
        ONE OFFSET(1) NUMBITS(1) [],
        TWO OFFSET(2) NUMBITS(1) [],
        THREE OFFSET(3) NUMBITS(1) [],
        FOUR OFFSET(4) NUMBITS(1) [],
        FIVE OFFSET(5) NUMBITS(1) [],
        SIX OFFSET(6) NUMBITS(1) [],
        SEVEN OFFSET(7) NUMBITS(1) [],
        EIGHT OFFSET(8) NUMBITS(1) [],
    ],
    PRIORITY [
        PRIORITY OFFSET(0) NUMBITS(4) [],
    ],
    THRESH [
        THRESH OFFSET(0) NUMBITS(4) [],
    ],
];

pub struct Intc {
    registers: StaticRef<IntcRegisters>,
}

impl Intc {
    pub const fn new(base: StaticRef<IntcRegisters>) -> Self {
        Intc { registers: base }
    }

    /// The ESP32C3 is interesting. It allows interrupts to be mapped on the
    /// fly by setting the `intr_map` registers. This feature is completely
    /// undocumented. The ESP32 HAL and projects that use that (like Zephyr)
    /// call into the ROM code to enable interrupts which maps the interrupts.
    /// In Tock we map them ourselves so we don't need to call into the ROM.
    pub fn map_interrupts(&self) {
        self.intc_write_fence(|| self.registers.uart0_intr_map.set(interrupts::IRQ_UART0));
        self.intc_write_fence(|| self.registers.timg0_intr_map.set(interrupts::IRQ_TIMER1));
        self.intc_write_fence(|| self.registers.timg1_intr_map.set(interrupts::IRQ_TIMER2));
        self.intc_write_fence(|| {
            self.registers
                .gpio_interrupt_pro_map
                .set(interrupts::IRQ_GPIO)
        });
        self.intc_write_fence(|| {
            self.registers
                .gpio_interrupt_pro_nmi_map
                .set(interrupts::IRQ_GPIO_NMI)
        });
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        // The ESP32-C3 reference manual requires us to abide by the following
        // order when enabling interrupts:
        //
        // 1. configure type (level vs. edge); we leave all interrupts
        //    level-triggered,
        // 2. configure priority,
        // 3. enable interrupts.

        // Accept all interrupts.
        self.intc_write_fence(|| self.registers.thresh.write(THRESH::THRESH.val(1)));

        // Set some default priority for each interrupt. This is not really used
        // at this point.
        for priority in self.registers.priority.iter() {
            self.intc_write_fence(|| priority.write(PRIORITY::PRIORITY.val(3)));
        }

        self.intc_write_fence(|| self.registers.enable.set(0xFFFF_FFFF));
    }

    /// Clear the "pending" flag of a particular interrupt.
    ///
    /// This should be called before an interrupt is handled in software, to
    /// catch any new interrupts raised while handling it.
    pub fn clear_interrupt(&self, irq: u32) {
        // To clear a pending interrupt, the reference manual requires us to
        // first set the respective bit in the `CLEAR` register, and then clear
        // that bit again.
        self.intc_write_fence(|| {
            self.registers
                .clear
                .set(1_u32.checked_shl(irq).unwrap_or(0))
        });
        self.intc_write_fence(|| self.registers.clear.set(0));
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending.
    ///
    /// RISC-V Intc has a "claim" register which makes it easy to grab the
    /// highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        // The interrupt controller may require up to 4 cycles to settle after
        // clearing an interrupt:
        for _ in 0..4 {
            rv32i::support::nop();
        }

        let eip = self.registers.eip.get();
        if eip == 0 {
            None
        } else {
            Some(eip.trailing_zeros())
        }
    }

    // Run the provided closure, and then emit a `fence` instruction.
    //
    // The ESP32-C3's reference manual wants all writes to the interrupt
    // controller to be followed by a `fence` instruction, and to happen with
    // `mstatus::MIE` cleared.
    //
    // We don't want to do the latter within the Intc driver here, because most
    // often it'd be used in a sequence of atomic operations that have
    // `mstatus::MIE` cleared either way. However, given that a `fence` should
    // happen after _every_ write, we use a helper for that here.
    fn intc_write_fence<R>(&self, fun: impl FnOnce() -> R) -> R {
        let res = fun();

        // The ESP32-C3 reference manual advises us to run a `fence` instruction
        // post any writes to the interrupt controller (while `MIE` is
        // disabled).
        //
        // SAFETY: the fence simply orders memory and I/O accesses, which forces
        // the APB writes to the interrupt controller to complete before setting
        // `MIE`. We don't mark it as `nomem` or `pure` to make sure it's not
        // being re-ordered and guaranteed to be emitted before any other `intc`
        // MMIO write or a CSR setting `mstatus::MIE`.
        unsafe {
            core::arch::asm!("fence");
        }

        res
    }
}
