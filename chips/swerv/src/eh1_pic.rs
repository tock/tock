//! Platform Level Interrupt Control peripheral driver for SweRV EH1.

use kernel::common::cells::VolatileCell;
use kernel::common::registers::LocalRegisterCopy;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

register_structs! {
    pub PicRegisters {
        /// External Interrupt Priority Level Registers
        (0x000 => _reserved0),
        (0x004 => meipl: [ReadWrite<u32, MEIPL::Register>; 255]),
        (0x400 => _reserved1),
        /// External Interrupt Pending Registers
        (0x1000 => meip: [ReadWrite<u32, MEIP::Register>; 8]),
        (0x1020 => _reserved2),
        /// External Interrupt Enable Registers
        (0x2004 => meie: [ReadWrite<u32, MEIE::Register>; 255]),
        (0x2400 => _reserved3),
        /// PIC Configuration Register
        (0x3000 => mpiccfg: ReadWrite<u32, MPICCFG::Register>),
        (0x3004 => _reserved4),
        /// External Interrupt Gateway Configuration Registers
        (0x4004 => meigwctrl: [ReadWrite<u32, MEIGWCTRL::Register>; 255]),
        (0x4400 => _reserved5),
        /// External Interrupt Gateway Clear Registers
        (0x5004 => meigwclr: [ReadWrite<u32>; 255]),
        (0x5400 => @END),
    }
}

register_bitfields![u32,
    MPICCFG [
        PRIORD OFFSET(0) NUMBITS(1) [
            STANDARD = 0,
            REVERSE = 1,
        ]
    ],
    MEIPL [
        PRIORITY OFFSET(0) NUMBITS(4) []
    ],
    MEIP [
        INTPEND OFFSET(1) NUMBITS(31) []
    ],
    MEIE [
        INTEN OFFSET(0) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0,
        ]
    ],
    MEIGWCTRL [
        POLARITY OFFSET(0) NUMBITS(1) [
            ACTIVE_HIGH = 0,
            ACTIVE_LOW = 1,
        ],
        TYPE OFFSET(1) NUMBITS(1) [
            LEVEL_TRIGGERED = 0,
            EDGE_TRIGGERED = 1,
        ]
    ],
];

pub struct Pic {
    registers: StaticRef<PicRegisters>,
    saved: [VolatileCell<LocalRegisterCopy<u32>>; 3],
}

impl Pic {
    pub const fn new(base: StaticRef<PicRegisters>) -> Self {
        Pic {
            registers: base,
            saved: [
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
                VolatileCell::new(LocalRegisterCopy::new(0)),
            ],
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        for clear in self.registers.meigwclr.iter() {
            clear.set(0);
        }
    }

    /// Enable all interrupts.
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    pub fn enable_all(&self) {
        self.registers.mpiccfg.write(MPICCFG::PRIORD::STANDARD);

        self.disable_all();

        for priority in self.registers.meipl.iter() {
            priority.write(MEIPL::PRIORITY.val(15));
        }

        for property in self.registers.meigwctrl.iter() {
            property.write(MEIGWCTRL::POLARITY::ACTIVE_HIGH + MEIGWCTRL::TYPE::LEVEL_TRIGGERED);
        }

        self.clear_all_pending();

        // Write 0 to meipt, meicidpl and meicurpl
        unsafe {
            let val_to_set: usize = 0;
            asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const 0xBC9);
            asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const 0xBCB);
            asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const 0xBCC);
        }

        // Enable all interrupts
        for enable in self.registers.meie.iter() {
            enable.write(MEIE::INTEN::ENABLE);
        }
    }

    // Mock implementations for tests
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn enable_all(&self) -> Option<u32> {
        unimplemented!()
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        for enable in self.registers.meie.iter() {
            enable.write(MEIE::INTEN::DISABLE);
        }
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V PIC has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    pub fn next_pending(&self) -> Option<u32> {
        let claim: usize;
        let claimid: usize;

        unsafe {
            // Write 0 to meicpct
            let val_to_set: usize = 0;
            asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const 0xBCA);

            // Read interrupt from meihap
            asm!("csrr {rd}, {csr}", rd = out(reg) claim, csr = const 0xFC8);
            claimid = (claim >> 2) & 0xFF;
        }

        if claimid == 0 {
            None
        } else {
            // Clear the interrupt
            self.registers.meigwclr[claimid - 1].set(0);
            // Disable the interrupt, we re-enable it in the complete step
            self.registers.meie[claimid - 1].write(MEIE::INTEN::DISABLE);

            Some(claimid as u32)
        }
    }

    // Mock implementations for tests
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn next_pending(&self) -> Option<u32> {
        unimplemented!()
    }

    /// Save the current interrupt to be handled later
    /// This will save the interrupt at index internally to be handled later.
    /// Interrupts must be disabled before this is called.
    /// Saved interrupts can be retrieved by calling `get_saved_interrupts()`.
    /// Saved interrupts are cleared when `'complete()` is called.
    pub unsafe fn save_interrupt(&self, index: u32) {
        let offset = if index < 32 {
            0
        } else if index < 64 {
            1
        } else {
            2
        };
        let irq = index % 32;

        // OR the current saved state with the new value
        let new_saved = self.saved[offset].get().get() | 1 << irq;

        // Set the new state
        self.saved[offset].set(LocalRegisterCopy::new(new_saved));
    }

    /// The `next_pending()` function will only return enabled interrupts.
    /// This function will return a pending interrupt that has been disabled by
    /// `save_interrupt()`.
    pub fn get_saved_interrupts(&self) -> Option<u32> {
        for (i, pending) in self.saved.iter().enumerate() {
            let saved = pending.get().get();
            if saved != 0 {
                return Some(saved.trailing_zeros() + (i as u32 * 32));
            }
        }

        None
    }

    /// Signal that an interrupt is finished being handled. In Tock, this should be
    /// called from the normal main loop (not the interrupt handler).
    /// Interrupts must be disabled before this is called.
    pub unsafe fn complete(&self, index: u32) {
        // Clear the interrupt
        self.registers.meigwclr[index as usize - 1].set(0);
        // Enable the interrupt
        self.registers.meie[index as usize - 1].write(MEIE::INTEN::ENABLE);

        let offset = if index < 32 {
            0
        } else if index < 64 {
            1
        } else {
            2
        };
        let irq = index % 32;

        // OR the current saved state with the new value
        let new_saved = self.saved[offset].get().get() & !(1 << irq);

        // Set the new state
        self.saved[offset].set(LocalRegisterCopy::new(new_saved));
    }
}
