//! Power Control driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub static mut PWRCTRL: PwrCtrl = PwrCtrl::new(PWRCTRL_BASE);

const PWRCTRL_BASE: StaticRef<PwrCtrlRegisters> =
    unsafe { StaticRef::new(0x4002_1000 as *const PwrCtrlRegisters) };

register_structs! {
    pub PwrCtrlRegisters {
        (0x000 => supplysrc: ReadWrite<u32, SUPPLYSRC::Register>),
        (0x004 => supplystatus: ReadWrite<u32, SUPPLYSTATUS::Register>),
        (0x008 => devpwren: ReadWrite<u32, DEVPWREN::Register>),
        (0x00c => mempwdinsleep: ReadWrite<u32>),
        (0x010 => mempwren: ReadWrite<u32>),
        (0x014 => mempwrstatus: ReadWrite<u32>),
        (0x018 => devpwrstatus: ReadOnly<u32, DEVPWRSTATUS::Register>),
        (0x01c => sramctrl: ReadWrite<u32>),
        (0x020 => adcstatus: ReadWrite<u32>),
        (0x024 => misc: ReadWrite<u32>),
        (0x028 => devpwreventen: ReadWrite<u32>),
        (0x02c => mempwreventen: ReadWrite<u32>),
        (0x030 => @END),
    }
}

register_bitfields![u32,
    SUPPLYSRC [
        BLEBUCKEN OFFSET(0) NUMBITS(8) []
    ],
    SUPPLYSTATUS [
        SIMOBUCKON OFFSET(0) NUMBITS(1) [],
        BLEBUCKON OFFSET(1) NUMBITS(1) []
    ],
    DEVPWREN [
        PWRIOS OFFSET(0) NUMBITS(1) [],
        PWRIOM0 OFFSET(1) NUMBITS(1) [],
        PWRIOM1 OFFSET(2) NUMBITS(1) [],
        PWRIOM2 OFFSET(3) NUMBITS(1) [],
        PWRIOM3 OFFSET(4) NUMBITS(1) [],
        PWRIOM4 OFFSET(5) NUMBITS(1) [],
        PWRIOM5 OFFSET(6) NUMBITS(1) [],
        PWRUART0 OFFSET(7) NUMBITS(1) [],
        PWRUART1 OFFSET(8) NUMBITS(1) [],
        PWRADC OFFSET(9) NUMBITS(1) [],
        PWRSCARD OFFSET(10) NUMBITS(1) [],
        PWRMSPI OFFSET(11) NUMBITS(1) [],
        PWRPDM OFFSET(12) NUMBITS(1) [],
        PWRBLEL OFFSET(13) NUMBITS(1) []
    ],
    DEVPWRSTATUS [
        MCUL OFFSET(0) NUMBITS(1) [],
        MCUH OFFSET(1) NUMBITS(1) [],
        HCPA OFFSET(2) NUMBITS(1) [],
        HCPB OFFSET(3) NUMBITS(1) [],
        HCPC OFFSET(4) NUMBITS(1) [],
        PWRADC OFFSET(5) NUMBITS(1) [],
        PWRMSPI OFFSET(6) NUMBITS(1) [],
        PWRPDM OFFSET(7) NUMBITS(1) [],
        BLEL OFFSET(8) NUMBITS(1) [],
        BLEH OFFSET(9) NUMBITS(1) [],
        CORESLEEP OFFSET(29) NUMBITS(1) [],
        COREDEEPSLEEP OFFSET(30) NUMBITS(1) [],
        SYSDEEPSLEEP OFFSET(31) NUMBITS(1) []
    ]
];

pub struct PwrCtrl {
    registers: StaticRef<PwrCtrlRegisters>,
}

impl PwrCtrl {
    pub const fn new(base: StaticRef<PwrCtrlRegisters>) -> PwrCtrl {
        PwrCtrl { registers: base }
    }

    pub fn enable_uart0(&self) {
        let regs = self.registers;

        regs.devpwren.modify(DEVPWREN::PWRUART0::SET);
    }

    pub fn enable_iom2(&self) {
        let regs = self.registers;

        regs.devpwren.modify(DEVPWREN::PWRIOM2::SET);
    }

    pub fn enable_ble(&self) {
        let regs = self.registers;

        regs.devpwren.modify(DEVPWREN::PWRBLEL::SET);

        while !regs.devpwrstatus.is_set(DEVPWRSTATUS::BLEL) {}
    }
}
