//! Platform Level Interrupt Control peripheral driver for SweRV EH1.

use kernel::utilities::cells::VolatileCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, LocalRegisterCopy, ReadWrite,
};
use kernel::utilities::StaticRef;
use riscv_csr::csr::ReadWriteRiscvCsr;

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
register_bitfields![usize,
    MEIVT [
        BASE OFFSET(10) NUMBITS(22) []
    ],
    MEIPT [
        PRITHRESH OFFSET(0) NUMBITS(4) []
    ],
    MEICIDPL [
        CLIDPRI OFFSET(0) NUMBITS(4) []
    ],
    MEICURPL [
        CURRPRI OFFSET(0) NUMBITS(4) []
    ],
    MEICPCT [
        RESERVED OFFSET(0) NUMBITS(32) []
    ],
    MEIHAP [
        ZERO OFFSET(0) NUMBITS(2) [],
        CLAIMID OFFSET(2) NUMBITS(8) [],
        BASE OFFSET(10) NUMBITS(22) [],
    ],
];

#[allow(dead_code)]
pub struct Pic {
    registers: StaticRef<PicRegisters>,
    saved: [VolatileCell<LocalRegisterCopy<u32>>; 3],
    meivt: ReadWriteRiscvCsr<usize, MEIVT::Register, 0xBC8>,
    meipt: ReadWriteRiscvCsr<usize, MEIPT::Register, 0xBC9>,
    meicpct: ReadWriteRiscvCsr<usize, MEICPCT::Register, 0xBCA>,
    meicidpl: ReadWriteRiscvCsr<usize, MEICIDPL::Register, 0xBCB>,
    meicurpl: ReadWriteRiscvCsr<usize, MEICURPL::Register, 0xBCC>,
    meihap: ReadWriteRiscvCsr<usize, MEIHAP::Register, 0xFC8>,
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
            meivt: ReadWriteRiscvCsr::new(),
            meipt: ReadWriteRiscvCsr::new(),
            meicpct: ReadWriteRiscvCsr::new(),
            meicidpl: ReadWriteRiscvCsr::new(),
            meicurpl: ReadWriteRiscvCsr::new(),
            meihap: ReadWriteRiscvCsr::new(),
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        for clear in self.registers.meigwclr.iter() {
            clear.set(0);
        }
    }

    /// Enable all interrupts.
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

        self.meipt.set(0);
        self.meicidpl.set(0);
        self.meicurpl.set(0);

        // Enable all interrupts
        for enable in self.registers.meie.iter() {
            enable.write(MEIE::INTEN::ENABLE);
        }
    }
    /// Disable all interrupts.
    pub fn disable_all(&self) {
        for enable in self.registers.meie.iter() {
            enable.write(MEIE::INTEN::DISABLE);
        }
    }

    /// Get the index (0-96) of the lowest number pending interrupt, or `None` if
    /// none is pending. RISC-V PIC has a "claim" register which makes it easy
    /// to grab the highest priority pending interrupt.
    pub fn next_pending(&self) -> Option<u32> {
        self.meicpct.set(0);
        let claimid = self.meihap.read(MEIHAP::CLAIMID);

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
        } else if index < 96 {
            2
        } else {
            panic!("Unsupported index {}", index);
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
