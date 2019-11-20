//! Core Local Interrupt Control peripheral driver.

use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

/// CLIC Hart Specific Region
#[repr(C)]
struct ClicRegisters {
    /// CLIC Interrupt Pending Registers
    clicintip: IntPendRegisters,
    /// CLIC Interrupt Enable Registers
    clicintie: IntEnableRegisters,
    /// CLIC Interrupt Configuration Registers
    clicintcfg: IntConfigRegisters,
    /// CLIC Configuration Registers
    cliccfg: ConfigRegisters,
}

/// Interrupt Pending Registers
#[repr(C)]
struct IntPendRegisters {
    _reserved0: [u8; 3],
    /// Machine Software Interrupt
    msip: ReadWrite<u8, intpend::Register>,
    _reserved1: [u8; 3],
    /// Machine Timer Interrupt
    mtip: ReadWrite<u8, intpend::Register>,
    _reserved2: [u8; 3],
    /// Machine External Interrupt
    meip: ReadWrite<u8, intpend::Register>,
    /// CLIC Software Interrupt
    csip: ReadWrite<u8, intpend::Register>,
    _reserved3: [u8; 3],
    /// Local Interrupt 0-127
    localintpend: [ReadWrite<u8, intpend::Register>; 128],
    _reserved4: [u8; 880],
}

/// Interrupt Enable Registers
#[repr(C)]
struct IntEnableRegisters {
    _reserved0: [u8; 3],
    /// Machine Software Interrupt
    msip: ReadWrite<u8, inten::Register>,
    _reserved1: [u8; 3],
    /// Machine Timer Interrupt
    mtip: ReadWrite<u8, inten::Register>,
    _reserved2: [u8; 3],
    /// Machine External Interrupt
    meip: ReadWrite<u8, inten::Register>,
    /// CLIC Software Interrupt
    csip: ReadWrite<u8, inten::Register>,
    _reserved3: [u8; 3],
    /// Local Interrupt 0-127
    localint: [ReadWrite<u8, inten::Register>; 128],
    _reserved4: [u8; 880],
}

/// Interrupt Configuration Registers
#[repr(C)]
struct IntConfigRegisters {
    _reserved0: [u8; 3],
    /// Machine Software Interrupt
    msip: ReadWrite<u8, intcon::Register>,
    _reserved1: [u8; 3],
    /// Machine Timer Interrupt
    mtip: ReadWrite<u8, intcon::Register>,
    _reserved2: [u8; 3],
    /// Machine External Interrupt
    meip: ReadWrite<u8, intcon::Register>,
    /// CLIC Software Interrupt
    csip: ReadWrite<u8, intcon::Register>,
    _reserved3: [u8; 3],
    /// Local Interrupt 0-127
    localint: [ReadWrite<u8, intcon::Register>; 128],
    _reserved4: [u8; 880],
}

/// Configuration Register
#[repr(C)]
struct ConfigRegisters {
    cliccfg: ReadWrite<u8, conreg::Register>,
}

register_bitfields![u8,
      intpend [
          IntPend OFFSET(0) NUMBITS(1) []
      ]
  ];

register_bitfields![u8,
      inten [
          IntEn OFFSET(0) NUMBITS(1) []
      ]
  ];

// The data sheet isn't completely clear on this field, but it looks like there
// are four bits for priority and level, and the lowest for bits of the register
// are reserved.
register_bitfields![u8,
      intcon [
          IntCon OFFSET(4) NUMBITS(4) []
      ]
  ];

register_bitfields![u8,
      conreg [
          nvbits OFFSET(0) NUMBITS(1) [],
          nlbits OFFSET(1) NUMBITS(4) [],
          nmbits OFFSET(5) NUMBITS(2) []
      ]
  ];

const CLIC_BASE: StaticRef<ClicRegisters> =
    unsafe { StaticRef::new(0x0280_0000 as *const ClicRegisters) };

pub struct Clic {
    registers: StaticRef<ClicRegisters>,

    /// A 32 bit long bit-vector of interrupts that are actually used on this
    /// platform. This is needed because disabled interrupts can still set their
    /// pending bits, but since they are disabled they do not actually cause the
    /// trap handler to execute. Also, there are interrupts that can fire
    /// without any way in software to disable them (for example with physical
    /// switches wired directly to in-chip interrupt lines). This means that we
    /// cannot rely on just using the pending interrupt bits to determine which
    /// interrupts have fired. However, if we also keep track of which
    /// interrupts this chip actually wants to use, then we can ignore the
    /// pending bits that we cannot control and have no use for.
    in_use_interrupts: u64,
}

impl Clic {
    pub const fn new(in_use_interrupts: u64) -> Clic {
        Clic {
            registers: CLIC_BASE,
            in_use_interrupts,
        }
    }

    /// Clear all pending interrupts.
    pub fn clear_all_pending(&self) {
        self.registers.clicintip.msip.write(intpend::IntPend::CLEAR);
        self.registers.clicintip.mtip.write(intpend::IntPend::CLEAR);
        self.registers.clicintip.meip.write(intpend::IntPend::CLEAR);
        self.registers.clicintip.csip.write(intpend::IntPend::CLEAR);

        for pending in self.registers.clicintip.localintpend.iter() {
            pending.write(intpend::IntPend::CLEAR);
        }
    }

    /// Enable ONLY the interrupts we actually want to use.
    ///
    /// The CLIC allows disabled interrupts to still set the pending bit. Therefore
    /// we have to be very careful about which interrupts we check.
    pub fn enable_all(&self) {
        if self.in_use_interrupts & (1 << 3) > 0 {
            self.registers.clicintie.msip.write(inten::IntEn::SET);
        } else if self.in_use_interrupts & (1 << 7) > 0 {
            self.registers.clicintie.mtip.write(inten::IntEn::SET);
        } else if self.in_use_interrupts & (1 << 11) > 0 {
            self.registers.clicintie.meip.write(inten::IntEn::SET);
        } else if self.in_use_interrupts & (1 << 12) > 0 {
            self.registers.clicintie.csip.write(inten::IntEn::SET);
        }

        for (i, enable) in self.registers.clicintie.localint.iter().enumerate() {
            if self.in_use_interrupts & (1 << (i + 16)) > 0 {
                enable.write(inten::IntEn::SET);
            }
        }
    }

    // Disable any interrupt that has its pending bit set. Since the pending bit
    // is how we check which interrupts need to be serviced, this just prevents
    // the interrupt from re-firing until the kernel is able to service it.
    pub fn disable_pending(&self) {
        // Do all of the non-local interrupts.
        if self.registers.clicintip.msip.is_set(intpend::IntPend) {
            self.registers.clicintie.msip.write(inten::IntEn::CLEAR);
        } else if self.registers.clicintip.mtip.is_set(intpend::IntPend) {
            self.registers.clicintie.mtip.write(inten::IntEn::CLEAR);
        } else if self.registers.clicintip.meip.is_set(intpend::IntPend) {
            self.registers.clicintie.meip.write(inten::IntEn::CLEAR);
        } else if self.registers.clicintip.csip.is_set(intpend::IntPend) {
            self.registers.clicintie.csip.write(inten::IntEn::CLEAR);
        }

        // Iterate through all interrupts. If the interrupt is enabled and it
        // is pending then disable the interrupt.
        for (i, pending) in self.registers.clicintip.localintpend.iter().enumerate() {
            if pending.is_set(intpend::IntPend)
                && self.registers.clicintie.localint[i].is_set(inten::IntEn)
            {
                self.registers.clicintie.localint[i].write(inten::IntEn::CLEAR);
            }
        }
    }

    /// Disable all interrupts.
    pub fn disable_all(&self) {
        self.registers.clicintie.msip.write(inten::IntEn::CLEAR);
        self.registers.clicintie.mtip.write(inten::IntEn::CLEAR);
        self.registers.clicintie.meip.write(inten::IntEn::CLEAR);
        self.registers.clicintie.csip.write(inten::IntEn::CLEAR);

        for enable in self.registers.clicintie.localint.iter() {
            enable.write(inten::IntEn::CLEAR);
        }
    }

    /// Get the index (0-256) of the lowest number pending interrupt, or `None` if
    /// none is pending.
    pub fn next_pending(&self) -> Option<u32> {
        if self.in_use_interrupts & (1 << 3) > 0
            && self.registers.clicintip.msip.is_set(intpend::IntPend)
        {
            return Some(3);
        } else if self.in_use_interrupts & (1 << 7) > 0
            && self.registers.clicintip.mtip.is_set(intpend::IntPend)
        {
            return Some(7);
        } else if self.in_use_interrupts & (1 << 11) > 0
            && self.registers.clicintip.meip.is_set(intpend::IntPend)
        {
            return Some(11);
        } else if self.in_use_interrupts & (1 << 12) > 0
            && self.registers.clicintip.csip.is_set(intpend::IntPend)
        {
            return Some(12);
        }

        for (i, pending) in self.registers.clicintip.localintpend.iter().enumerate() {
            if self.in_use_interrupts & (1 << (i + 16)) > 0 && pending.is_set(intpend::IntPend) {
                return Some((i + 16) as u32);
            }
        }
        None
    }

    /// Signal that an interrupt is finished being handled. In Tock, this should
    /// be called from the normal main loop (not the interrupt handler). This
    /// marks the interrupt as no longer pending and re-enables it.
    pub fn complete(&self, index: u32) {
        match index {
            3 => {
                self.registers.clicintip.msip.write(intpend::IntPend::CLEAR);
                self.registers.clicintie.msip.write(inten::IntEn::SET);
            }
            7 => {
                self.registers.clicintip.mtip.write(intpend::IntPend::CLEAR);
                self.registers.clicintie.mtip.write(inten::IntEn::SET);
            }
            11 => {
                self.registers.clicintip.meip.write(intpend::IntPend::CLEAR);
                self.registers.clicintie.meip.write(inten::IntEn::SET);
            }
            12 => {
                self.registers.clicintip.csip.write(intpend::IntPend::CLEAR);
                self.registers.clicintie.csip.write(inten::IntEn::SET);
            }
            16..=144 => {
                self.registers.clicintip.localintpend[(index as usize) - 16]
                    .write(intpend::IntPend::CLEAR);
                self.registers.clicintie.localint[(index as usize) - 16].write(inten::IntEn::SET);
            }
            _ => {}
        }
    }

    /// Return `true` if there are any pending interrupts in the CLIC, `false`
    /// otherwise.
    pub fn has_pending(&self) -> bool {
        self.next_pending().is_some()
    }
}

/// Helper function to disable a specific interrupt.
///
/// This is outside of the `Clic` struct because it has to be called from the
/// trap handler which does not have a reference to the CLIC object.
pub unsafe fn disable_interrupt(index: u32) {
    let regs: &ClicRegisters = &*CLIC_BASE;

    match index {
        3 => regs.clicintie.msip.write(inten::IntEn::CLEAR),
        7 => regs.clicintie.mtip.write(inten::IntEn::CLEAR),
        11 => regs.clicintie.meip.write(inten::IntEn::CLEAR),
        12 => regs.clicintie.csip.write(inten::IntEn::CLEAR),
        16..=144 => regs.clicintie.localint[(index as usize) - 16].write(inten::IntEn::CLEAR),
        _ => {}
    }
}
