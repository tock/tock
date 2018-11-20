//! Power Reset Clock Interrupts

use kernel::common::StaticRef;
use kernel::common::registers::ReadWrite;

#[repr(C)]
struct PrciRegisters {
    /// Clock Configuration Register
    hfrosccfg: ReadWrite<u32, hfrosccfg::Register>,
    /// Clock Configuration Register
    hfxosccfg: ReadWrite<u32, hfxosccfg::Register>,
    /// PLL Configuration Register
    pllcfg: ReadWrite<u32, pllcfg::Register>,
    /// PLL Divider Register
    plloutdiv: ReadWrite<u32, plloutdiv::Register>,
    /// Clock Configuration Register
    coreclkcfg: ReadWrite<u32>,
}

register_bitfields![u32,
    hfrosccfg [
        ready OFFSET(31) NUMBITS(1) [],
        enable OFFSET(30) NUMBITS(1) [],
        trim OFFSET(16) NUMBITS(5) [],
        div OFFSET(0) NUMBITS(6) []
    ],
    hfxosccfg [
        ready OFFSET(31) NUMBITS(1) [],
        enable OFFSET(30) NUMBITS(1) []
    ],
    pllcfg [
        lock OFFSET(31) NUMBITS(1) [],
        bypass OFFSET(18) NUMBITS(1) [],
        refsel OFFSET(17) NUMBITS(1) [],
        sel OFFSET(16) NUMBITS(1) [],
        pllq OFFSET(10) NUMBITS(2) [],
        pllf OFFSET(4) NUMBITS(6) [],
        pllr OFFSET(0) NUMBITS(3) [
            R1 = 0
        ]
    ],
    plloutdiv [
        divby1 OFFSET(8) NUMBITS(1) [],
        div OFFSET(0) NUMBITS(6) []
    ]
];

pub enum ClockFrequency {
    Freq18Mhz,
    Freq384Mhz,
}

pub struct Prci {
    registers: StaticRef<PrciRegisters>,
}

impl Prci {
    const fn new(base: StaticRef<PrciRegisters>) -> Prci {
        Prci {
            registers: base,
        }
    }

    pub fn set_clock_frequency(&self, frequency: ClockFrequency) {
        let regs = self.registers;

        // debug!("reg {:#x}", regs.hfrosccfg.get());

        // Assume a 72 MHz clock, then `div` is (72/frequency) - 1.
        let div = match frequency {
            ClockFrequency::Freq18Mhz => {
                // 4, // this seems wrong, but it works??
                regs.hfrosccfg.modify(hfrosccfg::div.val(4));
            }
            ClockFrequency::Freq384Mhz => {

            }

        };


    }
}
