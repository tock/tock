//! MCU Control driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

pub static mut MCUCTRL: McuCtrl = McuCtrl::new(MCUCTRL_BASE);

const MCUCTRL_BASE: StaticRef<McuCtrlRegisters> =
    unsafe { StaticRef::new(0x4002_0000 as *const McuCtrlRegisters) };

register_structs! {
    pub McuCtrlRegisters {
        (0x000 => chippn: ReadWrite<u32>),
        (0x004 => chipid0: ReadWrite<u32>),
        (0x008 => chipid1: ReadWrite<u32>),
        (0x00c => chiprev: ReadWrite<u32>),
        (0x010 => vendorid: ReadWrite<u32>),
        (0x014 => sku: ReadWrite<u32>),
        (0x018 => featureenable: ReadWrite<u32, FEATUREENABLE::Register>),
        (0x01C => _reserved0),
        (0x020 => debugger: ReadWrite<u32>),
        (0x024 => _reserved1),
        (0x104 => adcpwrdly: ReadWrite<u32>),
        (0x108 => _reserved2),
        (0x10C => adccal: ReadWrite<u32>),
        (0x110 => adcbattload: ReadWrite<u32>),
        (0x114 => _reserved3),
        (0x118 => adctrim: ReadWrite<u32>),
        (0x11C => adcrefcomp: ReadWrite<u32>),
        (0x120 => xtalctrl: ReadWrite<u32>),
        (0x124 => xtalgenctrl: ReadWrite<u32>),
        (0x128 => _reserved4),
        (0x198 => miscctrl: ReadWrite<u32, MISCCTRL::Register>),
        (0x19C => _reserved5),
        (0x1A0 => bootloader: ReadWrite<u32>),
        (0x1A4 => shadowvalid: ReadWrite<u32>),
        (0x1A8 => _reserved6),
        (0x1B0 => scratch0: ReadWrite<u32>),
        (0x1B4 => scratch1: ReadWrite<u32>),
        (0x1B8 => _reserved7),
        (0x1C0 => icodefaultaddr: ReadWrite<u32>),
        (0x1C4 => dcodefaultaddr: ReadWrite<u32>),
        (0x1C8 => sysfaultaddr: ReadWrite<u32>),
        (0x1CC => faultstatus: ReadWrite<u32>),
        (0x1D0 => faultcaptureen: ReadWrite<u32>),
        (0x1D4 => _reserved8),
        (0x200 => dbgr1: ReadWrite<u32>),
        (0x204 => dbgr2: ReadWrite<u32>),
        (0x208 => _reserved9),
        (0x220 => pmuenable: ReadWrite<u32>),
        (0x224 => _reserved10),
        (0x250 => tpiuctrl: ReadWrite<u32>),
        (0x254 => _reserved11),
        (0x264 => otapointer: ReadWrite<u32>),
        (0x268 => _reserved12),
        (0x284 => srammode: ReadWrite<u32>),
        (0x288 => _reserved13),
        (0x348 => kextclksel: ReadWrite<u32>),
        (0x34C => _reserved14),
        (0x358 => simobuck3: ReadWrite<u32>),
        (0x35C => simobuck4: ReadWrite<u32>),
        (0x360 => _reserved15),
        (0x368 => blebuck2: ReadWrite<u32, BLEBUCK2::Register>),
        (0x36C => _reserved16),
        (0x3A0 => flashwprot0: ReadWrite<u32>),
        (0x3A4 => flashwprot1: ReadWrite<u32>),
        (0x3A8 => _reserved17),
        (0x3B0 => flashrprot0: ReadWrite<u32>),
        (0x3B4 => flashrprot1: ReadWrite<u32>),
        (0x3B8 => _reserved18),
        (0x3C0 => dmasramwriteprotect0: ReadWrite<u32>),
        (0x3C4 => dmasramwriteprotect1: ReadWrite<u32>),
        (0x3C8 => _reserved19),
        (0x3D0 => dmasramreadprotect0: ReadWrite<u32>),
        (0x3D4 => dmasramreadprotect1: ReadWrite<u32>),
        (0x3D8 => @END),
    }
}

register_bitfields![u32,
    FEATUREENABLE [
        BLEREQ OFFSET(0) NUMBITS(1) [],
        BLEACK OFFSET(1) NUMBITS(1) [],
        BLEAVAIL OFFSET(2) NUMBITS(1) [],
        BURSTREQ OFFSET(4) NUMBITS(1) [],
        BURSTSTACK OFFSET(5) NUMBITS(1) [],
        BURSTAVAIL OFFSET(6) NUMBITS(1) []
    ],
    MISCCTRL [
        BLE_RESETN OFFSET(5) NUMBITS(1) []
    ],
    BLEBUCK2 [
        BLEBUCKTONLOWTRIM OFFSET(0) NUMBITS(6) [],
        BLEBUCKTONHITRIM OFFSET(6) NUMBITS(6) [],
        BLEBUCKTOND2ATRIM OFFSET(12) NUMBITS(6) []
    ]
];

pub struct McuCtrl {
    registers: StaticRef<McuCtrlRegisters>,
}

impl McuCtrl {
    pub const fn new(base: StaticRef<McuCtrlRegisters>) -> McuCtrl {
        McuCtrl { registers: base }
    }
}
