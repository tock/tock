//! Cache Control driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

pub static mut CACHECTRL: CacheCtrl = CacheCtrl::new(CACHECTRL_BASE);

const CACHECTRL_BASE: StaticRef<CacheCtrlRegisters> =
    unsafe { StaticRef::new(0x4001_8000 as *const CacheCtrlRegisters) };

register_structs! {
    pub CacheCtrlRegisters {
        (0x00 => cachecfg: ReadWrite<u32, CACHECFG::Register>),
        (0x04 => flashcfg: ReadWrite<u32>),
        (0x08 => ctrl: ReadWrite<u32>),
        (0x0C => _reserved0),
        (0x10 => ncr0start: ReadWrite<u32>),
        (0x14 => ncr0end: ReadWrite<u32>),
        (0x18 => ncr1start: ReadWrite<u32>),
        (0x1C => ncr1end: ReadWrite<u32>),
        (0x20 => _reserved1),
        (0x40 => dmon0: ReadWrite<u32>),
        (0x44 => dmon1: ReadWrite<u32>),
        (0x48 => dmon2: ReadWrite<u32>),
        (0x4C => dmon3: ReadWrite<u32>),
        (0x50 => imon0: ReadWrite<u32>),
        (0x54 => imon1: ReadWrite<u32>),
        (0x58 => imon2: ReadWrite<u32>),
        (0x5C => imon3: ReadWrite<u32>),
        (0x60 => @END),
    }
}

register_bitfields![u32,
    CACHECFG [
        ENABLE OFFSET(0) NUMBITS(1) [],
        LRU OFFSET(1) NUMBITS(1) [],
        ENABLE_NC0 OFFSET(2) NUMBITS(1) [],
        ENABLE_NC1 OFFSET(3) NUMBITS(1) [],
        CONFIG OFFSET(4) NUMBITS(4) [],
        ICACHE_ENABLE OFFSET(8) NUMBITS(1) [],
        DCACHE_ENABLE OFFSET(9) NUMBITS(1) [],
        CACHE_CLKGATE OFFSET(10) NUMBITS(1) [],
        CACHE_LS OFFSET(11) NUMBITS(1) [],
        DATA_CLK_GATE OFFSET(20) NUMBITS(1) [],
        ENABLE_MONITOR OFFSET(24) NUMBITS(1) []
    ]
];

pub struct CacheCtrl {
    registers: StaticRef<CacheCtrlRegisters>,
}

impl CacheCtrl {
    pub const fn new(base: StaticRef<CacheCtrlRegisters>) -> CacheCtrl {
        CacheCtrl { registers: base }
    }

    pub fn enable_cache(&self) {
        self.registers.cachecfg.write(
            CACHECFG::ENABLE::SET
                + CACHECFG::CACHE_CLKGATE::SET
                + CACHECFG::DATA_CLK_GATE::SET
                + CACHECFG::ICACHE_ENABLE::SET
                + CACHECFG::DCACHE_ENABLE::SET,
        );
    }
}
