//! Power Reset Clock Interrupt controller driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

pub static mut CLKGEN: ClkGen = ClkGen::new(CLKGEN_BASE);

const CLKGEN_BASE: StaticRef<ClkGenRegisters> =
    unsafe { StaticRef::new(0x4000_4000 as *const ClkGenRegisters) };

register_structs! {
    pub ClkGenRegisters {
        (0x00 => calxt: ReadWrite<u32>),
        (0x04 => calrc: ReadWrite<u32>),
        (0x08 => acalctr: ReadWrite<u32>),
        (0x0c => octrl: ReadWrite<u32>),
        (0x10 => clkout: ReadWrite<u32>),
        (0x14 => clkkey: ReadWrite<u32>),
        (0x18 => cctrl: ReadWrite<u32>),
        (0x1c => status: ReadWrite<u32>),
        (0x20 => hfadj: ReadWrite<u32>),
        (0x24 => _reserved0),
        (0x28 => clockenstat: ReadWrite<u32>),
        (0x2c => clocken2stat: ReadWrite<u32>),
        (0x30 => clocken3stat: ReadWrite<u32>),
        (0x34 => freqctrl: ReadWrite<u32>),
        (0x38 => _reserved1),
        (0x3c => blebucktonadj: ReadWrite<u32, BLEBUCKTONADJ::Register>),
        (0x40 => _reserved2),
        (0x100 => intrpten: ReadWrite<u32>),
        (0x104 => intrptstat: ReadWrite<u32>),
        (0x108 => intrptclr: ReadWrite<u32>),
        (0x10c => intrptset: ReadWrite<u32>),
        (0x110 => @END),
    }
}

register_bitfields![u32,
    BLEBUCKTONADJ [
        TONLOWTHRESHOLD OFFSET(0) NUMBITS(10) [],
        TONHIGHTHRESHOLD OFFSET(10) NUMBITS(10) [],
        TONADJUSTPERIOD OFFSET(20) NUMBITS(2) [],
        TONADJUSTEN OFFSET(22) NUMBITS(1) [
            DISABLE = 0x0,
            ENALBE = 0x1
        ],
        ZEROLENDETECTTRIM OFFSET(23) NUMBITS(4) [],
        ZEROLENDETECTEN OFFSET(23) NUMBITS(4) []
    ]
];

pub enum ClockFrequency {
    Freq48MHz,
}

pub struct ClkGen {
    registers: StaticRef<ClkGenRegisters>,
}

impl ClkGen {
    pub const fn new(base: StaticRef<ClkGenRegisters>) -> ClkGen {
        ClkGen { registers: base }
    }

    pub fn set_clock_frequency(&self, frequency: ClockFrequency) {
        let regs = self.registers;

        match frequency {
            ClockFrequency::Freq48MHz => {
                // Magic numbers from the HAL
                regs.clkkey.set(71);
                regs.cctrl.set(0);
                regs.clkkey.set(0);
            }
        };
    }

    pub fn enable_ble(&self) {
        let regs = self.registers;

        regs.blebucktonadj
            .modify(BLEBUCKTONADJ::TONADJUSTEN::DISABLE);
    }
}
