//! STimer driver for the Apollo3

use kernel::common::cells::OptionalCell;

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

pub static mut STIMER: STimer = STimer::new(STIMER_BASE);

const STIMER_BASE: StaticRef<STimerRegisters> =
    unsafe { StaticRef::new(0x4000_8000 as *const STimerRegisters) };

register_structs! {
    pub STimerRegisters {
        (0x000 => _reserved0),
        (0x140 => stcfg: ReadWrite<u32, STCFG::Register>),
        (0x144 => sttmr: ReadWrite<u32, STTMR::Register>),
        (0x148 => capturecontrol: ReadWrite<u32, CAPTURECONTROL::Register>),
        (0x14C => _reserved1),
        (0x150 => scmpr: [ReadWrite<u32, SCMPR::Register>; 8]),
        (0x170 => _reserved2),
        (0x1E0 => scapt: [ReadWrite<u32, SCAPT::Register>; 4]),
        (0x1F0 => snvr: [ReadWrite<u32, SNVR::Register>; 4]),
        (0x200 => _reserved3),
        (0x300 => stminten: ReadWrite<u32, STMINT::Register>),
        (0x304 => stmintstat: ReadWrite<u32, STMINT::Register>),
        (0x308 => stmintclr: ReadWrite<u32, STMINT::Register>),
        (0x30C => stmintset: ReadWrite<u32, STMINT::Register>),
        (0x310 => @END),
    }
}

register_bitfields![u32,
    STCFG [
        CLKSEL OFFSET(0) NUMBITS(4) [
            NOCLK = 0x0,
            HRFC_DIV16 = 0x1,
            HRFC_DIV256 = 0x2,
            XTAL_DIV1 = 0x3,
            XTAL_DIV2 = 0x4,
            XTAL_DIV32 = 0x5,
            LFRC_DIV1 = 0x6,
            CTIMER0A = 0x7,
            CTIMER0B = 0x8
        ],
        COMPARE_A_EN OFFSET(8) NUMBITS(1) [],
        COMPARE_B_EN OFFSET(9) NUMBITS(1) [],
        COMPARE_C_EN OFFSET(10) NUMBITS(1) [],
        COMPARE_D_EN OFFSET(11) NUMBITS(1) [],
        COMPARE_E_EN OFFSET(12) NUMBITS(1) [],
        COMPARE_F_EN OFFSET(13) NUMBITS(1) [],
        COMPARE_G_EN OFFSET(14) NUMBITS(1) [],
        COMPARE_H_EN OFFSET(15) NUMBITS(1) [],
        CLEAR OFFSET(30) NUMBITS(1) [],
        FREEZE OFFSET(31) NUMBITS(1) []
    ],
    STTMR [
        STTMR OFFSET(0) NUMBITS(31) []
    ],
    CAPTURECONTROL [
        CAPTURE0 OFFSET(0) NUMBITS(1) [],
        CAPTURE1 OFFSET(1) NUMBITS(1) [],
        CAPTURE2 OFFSET(2) NUMBITS(1) [],
        CAPTURE3 OFFSET(3) NUMBITS(1) []
    ],
    SCMPR [
        SCMPR OFFSET(0) NUMBITS(31) []
    ],
    SCAPT [
        SCATP OFFSET(0) NUMBITS(31) []
    ],
    SNVR [
        SNVR OFFSET(0) NUMBITS(31) []
    ],
    STMINT [
        COMPAREA OFFSET(0) NUMBITS(1) [],
        COMPAREB OFFSET(1) NUMBITS(1) [],
        COMPAREC OFFSET(2) NUMBITS(1) [],
        COMPARED OFFSET(3) NUMBITS(1) [],
        COMPAREE OFFSET(4) NUMBITS(1) [],
        COMPAREF OFFSET(5) NUMBITS(1) [],
        COMPAREG OFFSET(6) NUMBITS(1) [],
        COMPAREH OFFSET(7) NUMBITS(1) [],
        OVERFLOW OFFSET(8) NUMBITS(1) [],
        CAPTUREA OFFSET(9) NUMBITS(1) [],
        CAPTUREB OFFSET(10) NUMBITS(1) [],
        CAPTUREC OFFSET(11) NUMBITS(1) [],
        CAPTURED OFFSET(12) NUMBITS(1) []
    ]
];

pub struct STimer<'a> {
    registers: StaticRef<STimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> STimer<'_> {
    const fn new(base: StaticRef<STimerRegisters>) -> STimer<'a> {
        STimer {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Disable timer
        regs.stcfg.modify(STCFG::COMPARE_A_EN::CLEAR);

        // Disable interrupt
        regs.stminten.modify(STMINT::COMPAREA::CLEAR);

        // Clear interrupt
        regs.stmintclr.modify(STMINT::COMPAREA::SET);

        self.client.map(|client| client.fired());
    }
}

impl<'a> hil::time::Alarm<'a> for STimer<'a> {
    fn set_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        let regs = self.registers;

        // Set the clock source
        regs.stcfg.write(STCFG::CLKSEL::XTAL_DIV2);

        // Enable interrupts
        regs.stminten.modify(STMINT::COMPAREA::SET);

        // Set the delta, this can take a few goes
        // See Errata 4.14 at at https://ambiqmicro.com/static/mcu/files/Apollo3_Blue_MCU_Errata_List_v2_0.pdf
        let timer_delta = tics - regs.sttmr.get();
        let mut tries = 0;

        while regs.scmpr[0].get() != tics && tries < 5 {
            regs.scmpr[0].set(timer_delta);
            tries = tries + 1;
        }

        // Enable the compare
        regs.stcfg.modify(STCFG::COMPARE_A_EN::SET);
    }

    fn get_alarm(&self) -> u32 {
        let regs = self.registers;

        regs.scmpr[0].get()
    }

    fn disable(&self) {
        let regs = self.registers;

        regs.stcfg.modify(
            STCFG::COMPARE_A_EN::CLEAR
                + STCFG::COMPARE_B_EN::CLEAR
                + STCFG::COMPARE_C_EN::CLEAR
                + STCFG::COMPARE_D_EN::CLEAR
                + STCFG::COMPARE_E_EN::CLEAR
                + STCFG::COMPARE_F_EN::CLEAR
                + STCFG::COMPARE_G_EN::CLEAR
                + STCFG::COMPARE_H_EN::CLEAR,
        );
    }

    fn is_enabled(&self) -> bool {
        let regs = self.registers;

        regs.stcfg.read(STCFG::COMPARE_A_EN) != 0
    }
}

impl<'a> hil::time::Time for STimer<'_> {
    type Frequency = hil::time::Freq16KHz;

    fn now(&self) -> u32 {
        let regs = self.registers;

        regs.sttmr.get()
    }

    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}
