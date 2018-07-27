use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use oscfh;

pub struct DdiRegisters {
    ctl0: ReadWrite<u32, Ctl0::Register>,
    _ctl1: ReadOnly<u32>,

    _radc_ext_cfg: ReadOnly<u32>,
    _amp_comp_ctl: ReadOnly<u32>,
    _amp_comp_th1: ReadOnly<u32>,
    _amp_comp_th2: ReadOnly<u32>,

    _ana_bypass_val1: ReadOnly<u32>,
    _ana_bypass_val2: ReadOnly<u32>,

    _analog_test_ctl: ReadOnly<u32>,
    _adc_doubler_nanoamp_ctl: ReadOnly<u32>,

    _xosc_hf_ctl: ReadOnly<u32>,
    _lf_osc_ctl: ReadOnly<u32>,
    _rco_sc_hf_ctl: ReadOnly<u32>,

    stat0: ReadOnly<u32, Stat0::Register>,
    _stat1: ReadOnly<u32>,
    _stat2: ReadOnly<u32>,
}

register_bitfields! [
    u32,
    Ctl0 [
        XTAL_IS_24M              OFFSET(31) NUMBITS(1) [],
        // RESERVED 30
        BYPASS_XOSC_LF_CLK_QUAL  OFFSET(29) NUMBITS(1) [],
        BYPASS_RCOSC_LF_CLK_QUAL OFFSET(28) NUMBITS(1) [],
        DOUBLER_START_DURATION   OFFSET(26) NUMBITS(2) [],
        DOUBLER_RESET_DURATION   OFFSET(25) NUMBITS(1) [],
        CLK_DCDC_SRC_SEL         OFFSET(24) NUMBITS(1) [],
        // RESERVED 15-23
        HPOSC_MODE_ON            OFFSET(14) NUMBITS(1) [],
        // RESERVED 14
        RCOSC_LF_TRIMMED         OFFSET(12) NUMBITS(1) [],
        XOSC_HF_POWER_MODE       OFFSET(11) NUMBITS(1) [],
        XOSC_LF_DIG_BYPASS       OFFSET(10) NUMBITS(1) [],

        CLK_LOSS_EN              OFFSET(9) NUMBITS(1) [],
        ACLK_TDC_SRC_SEL         OFFSET(7) NUMBITS(2) [],
        ACLK_REF_SRC_SEL         OFFSET(5) NUMBITS(2) [],

        SCLK_LF_SRC_SEL          OFFSET(2) NUMBITS(2) [
            RCOSC_HF_DERIVED = 0b00,
            XOSC_HF_DERIVED  = 0b01,
            RCOSC_LF         = 0b10,
            XOSC_LF          = 0b11
        ],
        // RESERVED 1
        SCLK_HF_SRC_SEL OFFSET(0) NUMBITS(1) [
            RCOSC_HF = 0b00,
            XOSC_HF  = 0b01
        ]
    ],
    Stat0 [
        // RESERVED 31
        SCLK_LF_SRC     OFFSET(29) NUMBITS(2) [
            RCOSC_HF_DERIVED = 0b00,
            XOSC_HF_DERIVED  = 0b01,
            RCOSC_LF         = 0b10,
            XOSC_LF          = 0b11
        ],
        SCLK_HF_SRC     OFFSET(28) NUMBITS(1) [
            RCOSC_HF = 0b00,
            XOSC_HF  = 0b01
        ],
        // RESERVED 23-27
        RCOSC_HF_EN      OFFSET(22) NUMBITS(1) [],
        RCOSC_LF_EN      OFFSET(21) NUMBITS(1) [],
        XOSC_LF_EN       OFFSET(20) NUMBITS(1) [],
        CLK_DCDC_RDY     OFFSET(19) NUMBITS(1) [],
        CLK_DCDC_RDY_ACK OFFSET(18) NUMBITS(1) [],

        SCLK_HF_LOSS     OFFSET(17) NUMBITS(1) [],
        SCLK_LF_LOSS     OFFSET(16) NUMBITS(1) [],
        XOSC_HF_EN       OFFSET(15) NUMBITS(1) [],
        // RESERVED 14
        // Indicates the 48MHz clock from the DOUBLER enabled
        XB_48M_CLK_EN    OFFSET(13) NUMBITS(1) [],
        // RESERVED 12
        XOSC_HF_LP_BUF_EN OFFSET(11) NUMBITS(1) [],
        XOSC_HF_HP_BUF_EN OFFSET(10) NUMBITS(1) [],
        // RESERVED 9
        ADC_THMET       OFFSET(8) NUMBITS(1) [],
        ADC_DATA_READY  OFFSET(7) NUMBITS(1) [],
        ADC_DATA        OFFSET(1) NUMBITS(6) [],
        PENDING_SCLK_HF_SWITCHING OFFSET(0) NUMBITS(1) []
    ]
];

pub enum ClockType {
    LF,
    HF,
}

#[allow(non_camel_case_types)]
pub enum SCLKLFSRC {
    RCOSC_HF_DERIVED,
    XOSC_HF_DERIVED,
    RCOSC_LF,
    XOSC_LF,
}

#[allow(non_camel_case_types)]
pub enum SCLKHFSRC {
    RCOSC_HF,
    XOSC_HF,
}

const DDI0_BASE: StaticRef<DdiRegisters> =
    unsafe { StaticRef::new(0x400C_A000 as *const DdiRegisters) };

pub const OSC: Oscillator = Oscillator::new();

pub struct Oscillator {
    regs: StaticRef<DdiRegisters>,
}

impl Oscillator {
    pub const fn new() -> Oscillator {
        Oscillator { regs: DDI0_BASE }
    }

    pub fn lfosc_config(&self, lf_clk: SCLKLFSRC) {
        let regs = DDI0_BASE;

        match lf_clk {
            SCLKLFSRC::RCOSC_HF_DERIVED => {
                self.regs
                    .ctl0
                    .modify(Ctl0::SCLK_LF_SRC_SEL::RCOSC_HF_DERIVED);
            }
            SCLKLFSRC::RCOSC_LF => {
                self.regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL::RCOSC_LF);
            }
            SCLKLFSRC::XOSC_HF_DERIVED => {
                self.regs
                    .ctl0
                    .modify(Ctl0::SCLK_LF_SRC_SEL::XOSC_HF_DERIVED);
            }
            SCLKLFSRC::XOSC_LF => {
                self.regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL::XOSC_LF);
            }
        }

        // Switch for LF source unimplemented
    }

    pub fn hfosc_config(&self, hf_clk: SCLKHFSRC) {
        let regs = DDI0_BASE;

        match hf_clk {
            SCLKHFSRC::RCOSC_HF => {
                self.regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL::RCOSC_HF);
            }
            SCLKHFSRC::XOSC_HF => {
                self.regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL::XOSC_HF);
            }
        }

        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
    }

    pub fn enable_hfosc(&self, hf_clk: SCLKHFSRC) {
        let regs = DDI0_BASE;

        let clock = self.get_clock_source(ClockType::HF);

        match hf_clk {
            SCLKHFSRC::RCOSC_HF => {
                if clock != 0x00 {
                    while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
                    self.switch_osc();
                }
            }
            SCLKHFSRC::XOSC_HF => {
                if clock != 0x00 {
                    while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
                    self.switch_osc();
                }
            }
        }
    }

    pub fn disable_lfclk_qualifier(&self) {
        while self.get_clock_source(ClockType::LF) != 0x02 {}

        let regs = DDI0_BASE;

        regs.ctl0
            .modify(Ctl0::BYPASS_XOSC_LF_CLK_QUAL::SET + Ctl0::BYPASS_RCOSC_LF_CLK_QUAL::SET);
    }

    pub fn get_clock_source(&self, source: ClockType) -> u8 {
        let regs = DDI0_BASE;
        match source {
            ClockType::LF => regs.stat0.read(Stat0::SCLK_LF_SRC) as u8,
            ClockType::HF => regs.stat0.read(Stat0::SCLK_HF_SRC) as u8,
        }
    }

    pub fn switch_osc(&self) {
        unsafe { oscfh::source_switch() };
    }
}
