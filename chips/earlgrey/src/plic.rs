//! Platform Level Interrupt Control peripheral driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
//use kernel::debug;

pub const PLIC_BASE: StaticRef<PlicRegisters> =
    unsafe { StaticRef::new(0x4009_0000 as *const PlicRegisters) };

register_structs! {
    pub PlicRegisters {
        /// Interrupt Pending Register
        (0x000 => pending: [ReadOnly<u32>; 3]),
        /// Interrupt Source Register
        (0x00C => source: [ReadWrite<u32>; 3]),
        /// Interrupt Priority Registers
        (0x018 => priority: [ReadWrite<u32, priority::Register>; 79]),
        (0x154 => _reserved0: [ReadWrite<u32>; 43]),
        /// Interrupt Enable Register
        (0x200 => enable: [ReadWrite<u32>; 3]),
        /// Priority Threshold Register
        (0x20C => threshold: ReadWrite<u32, priority::Register>),
        /// Claim/Complete Register
        (0x210 => claim: ReadWrite<u32>),
        /// MSIP Register
        (0x214 => msip: ReadWrite<u32>),
        (0x218 => @END),
    }
}

register_bitfields![u32,
    priority [
        Priority OFFSET(0) NUMBITS(3) []
    ]
];

pub struct Plic {
    registers: StaticRef<PlicRegisters>,
}

impl Plic {
    pub const fn new(base: StaticRef<PlicRegisters>) -> Self {
        Plic { registers: base }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        unimplemented!()
    }

    /// Enable all interrupts.
    pub fn enable_all(&self) {
        // USB hardware on current OT master branch seems to have
        // interrupt bugs: running Alarms causes persistent USB
        // CONNECTED interrupts that can't be masked from USBDEV and
        // cause the system to hang. So enable all interrupts except
        // for the USB ones. Some open PRs on OT fix this, we'll re-enable
        // USB interrurupts.
        //
        // https://github.com/lowRISC/opentitan/issues/3388
        self.registers.enable[0].set(0xFFFF_FFFF);
        self.registers.enable[1].set(0xFFFF_FFFF);
        self.registers.enable[2].set(0xFFFF_0000); // USB are 64-79

        // Set the max priority for each interrupt. This is not really used
        // at this point.
        for priority in self.registers.priority.iter() {
            priority.write(priority::Priority.val(3));
        }

        // Accept all interrupts.
        self.registers.threshold.write(priority::Priority.val(1));
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        for enable in self.registers.enable.iter() {
            enable.set(0);
        }
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V PLIC has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        let claim = self.registers.claim.get();
        if claim == 0 {
            None
        } else {
            Some(claim)
        }
    }

    /// Signal that an interrupt is finished being handled. In Tock, this should be
    /// called from the normal main loop (not the interrupt handler).
    pub fn complete(&self, index: u32) {
        self.registers.claim.set(index);
    }

    /// Return `true` if there are any pending interrupts in the PLIC, `false`
    /// otherwise.
    pub fn has_pending(&self) -> bool {
        self.registers
            .pending
            .iter()
            .fold(0, |i, pending| pending.get() | i)
            != 0
    }
}
