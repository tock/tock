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
        // RESERVED 13
        RCOSC_LF_TRIMMED         OFFSET(12) NUMBITS(1) [],
        XOSC_HF_POWER_MODE       OFFSET(11) NUMBITS(1) [],
        XOSC_LF_DIG_BYPASS       OFFSET(10) NUMBITS(1) [],

        CLK_LOSS_EN              OFFSET(9) NUMBITS(1) [],
        ACLK_TDC_SRC_SEL         OFFSET(7) NUMBITS(2) [],
        ACLK_REF_SRC_SEL         OFFSET(4) NUMBITS(3) [],

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

pub const HF_RCOSC: u8 = 0x00;
pub const HF_XOSC: u8 = 0x01;

pub const LF_DERIVED_RCOSC: u8 = 0x00;
pub const LF_DERIVED_XOSC: u8 = 0x01;
pub const LF_RCOSC: u8 = 0x02;
pub const LF_XOSC: u8 = 0x03;

const DDI0_BASE: StaticRef<DdiRegisters> =
    unsafe { StaticRef::new(0x400C_A000 as *const DdiRegisters) };

pub const OSC: Oscillator = Oscillator::new();

pub struct Oscillator {
    regs: StaticRef<DdiRegisters>,
}

pub fn set_24_mhz_clk() {
    let regs = DDI0_BASE;

    regs.ctl0.modify(Ctl0::XTAL_IS_24M::SET);
}

impl Oscillator {
    pub const fn new() -> Oscillator {
        Oscillator { regs: DDI0_BASE }
    }

    pub fn config_lf_osc(&self, lf_clk: u8) {
        match lf_clk {
            LF_DERIVED_RCOSC => {
                self.regs
                    .ctl0
                    .modify(Ctl0::SCLK_LF_SRC_SEL::RCOSC_HF_DERIVED);
            }
            LF_RCOSC => {
                self.regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL::RCOSC_LF);
            }
            LF_DERIVED_XOSC => {
                self.regs
                    .ctl0
                    .modify(Ctl0::SCLK_LF_SRC_SEL::XOSC_HF_DERIVED);
            }
            LF_XOSC => {
                self.regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL::XOSC_LF);
            }
            _ => panic!("Undefined LF OSC"),
        }
    }

    pub fn config_hf_osc(&self, hf_clk: u8) {
        let regs = DDI0_BASE;

        match hf_clk {
            HF_RCOSC => {
                regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL::RCOSC_HF);
            }
            HF_XOSC => {
                regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL::XOSC_HF);
            }
            _ => panic!("Undefined HF OSC"),
        }

        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
    }

    pub fn switch_to_rc_osc(&self) {
        let regs = DDI0_BASE;

        if self.clock_source_get(ClockType::HF) != HF_RCOSC {
            self.clock_source_set(ClockType::HF, HF_RCOSC);
        }
        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}

        self.clock_source_set(ClockType::LF, LF_RCOSC);
        self.disable_lfclk_qualifier();
    }

    // Check if the current clock source is HF_XOSC. If not, set it.
    pub fn request_switch_to_hf_xosc(&self) {
        // self.configure();

        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            self.clock_source_set(ClockType::HF, HF_XOSC);
        }
    }

    // Check if current clock source is HF_XOSC. If not, wait until request is done, then set it in
    // ddi
    pub fn switch_to_hf_xosc(&self) {
        let regs = DDI0_BASE;

        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
            self.switch_osc();
        }
    }

    pub fn switch_to_hf_rcosc(&self) {
        let regs = DDI0_BASE;

        self.clock_source_set(ClockType::HF, HF_RCOSC);
        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
        if self.clock_source_get(ClockType::HF) != HF_RCOSC {
            self.switch_osc();
        }
    }

    pub fn disable_lfclk_qualifier(&self) {
        let regs = DDI0_BASE;

        while self.clock_source_get(ClockType::LF) != LF_RCOSC {}

        regs.ctl0.modify(Ctl0::BYPASS_XOSC_LF_CLK_QUAL::SET + Ctl0::BYPASS_RCOSC_LF_CLK_QUAL::SET);
    }

    // Get the current clock source of either LF or HF sources
    pub fn clock_source_get(&self, source: ClockType) -> u8 {
        match source {
            ClockType::LF => self.regs.stat0.read(Stat0::SCLK_LF_SRC) as u8,
            ClockType::HF => self.regs.stat0.read(Stat0::SCLK_HF_SRC) as u8,
        }
    }

    // Set the clock source in DDI_0_OSC
    pub fn clock_source_set(&self, clock: ClockType, src: u8) {
        match clock {
            ClockType::LF => {
                self.regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL.val(src as u32));
            }
            ClockType::HF => {
                self.regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL.val(src as u32));
            }
        }
    }

    // Switch the source OSC in DDI0
    pub fn switch_osc(&self) {
        unsafe { oscfh::source_switch() };
    }
}
