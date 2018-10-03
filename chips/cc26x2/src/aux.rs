use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub struct AuxSysIfRegisters {
    op_mode_req: ReadWrite<u32, Req::Register>,
    op_mode_ack: ReadOnly<u32, Ack::Register>,
    prog_wu0_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu1_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu2_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu3_cfg: ReadWrite<u32, WUCfg::Register>,
    _wu_flags: ReadOnly<u32, WUFlags::Register>,
    wu_flags_clr: ReadWrite<u32, WUFlags::Register>,
    wu_gate: ReadWrite<u32, WUGate::Register>,
    // remainder unimplemented
    _vec_cfg: [ReadOnly<u8>; 32],
    _evsyncrate: [ReadOnly<u8>; 4],
    _peroprate: [ReadOnly<u8>; 4],
    _adc_clk_ctl: ReadOnly<u32>,
    _tdc_clk_ctl: ReadOnly<u32>,
    _tdc_ref_clk_ctl: ReadOnly<u32>,
    _timer2: [ReadOnly<u32>; 4],
    _reserved: ReadOnly<u32>,
    _clk_shift_det: ReadOnly<u32>,
    _recharge: [ReadOnly<u32>; 2],
    _rtc_subsec_inc: [ReadOnly<u32>; 6],
    _batmon_bat: ReadOnly<u32>,
    _reserved2: ReadOnly<u32>,
    _batmon_temp: ReadOnly<u32>,
    _timer_halt: ReadOnly<u32>,
    _reserved3: [ReadOnly<u32>; 3],
    _timer2_bridge: ReadOnly<u32>,
    _sw_pwr_prof: ReadOnly<u32>,
}

register_bitfields! [
    u32,
    Req [
        REQ         OFFSET(0) NUMBITS(2) [
            Active = 0x0,
            LowPower = 0x1,
            PowerDownActive = 0x2,
            PowerDownLowPower = 0x3
        ]
    ],
    Ack [
        ACK         OFFSET(0) NUMBITS(2) []
    ],
    WUCfg [
        POL         OFFSET(7) NUMBITS(1) [],
        EN          OFFSET(6) NUMBITS(1) [],
        WU_SRC      OFFSET(0) NUMBITS(6) [
            AuxIO0              = 0b000000,
            AuxIO1              = 0b000001,
            AuxIO2              = 0b000010,
            AuxIO3              = 0b000011,
            AuxIO4              = 0b000100,
            AuxIO5              = 0b000101,
            AuxIO6              = 0b000110,
            AuxIO7              = 0b000111,
            AuxIO8              = 0b001000,
            AuxIO9              = 0b001001,
            AuxIO10             = 0b001010,
            AuxIO11             = 0b001011,
            AuxIO12             = 0b001100,
            AuxIO13             = 0b001101,
            AuxIO14             = 0b001110,
            AuxIO15             = 0b001111,
            AuxIO16             = 0b010000,
            AuxIO17             = 0b010001,
            AuxIO18             = 0b010010,
            AuxIO19             = 0b010011,
            AuxIO20             = 0b010100,
            AuxIO21             = 0b010101,
            AuxIO22             = 0b010110,
            AuxIO23             = 0b010111,
            AuxIO24             = 0b011000,
            AuxIO25             = 0b011001,
            AuxIO26             = 0b011010,
            AuxIO27             = 0b011011,
            AuxIO28             = 0b011100,
            AuxIO29             = 0b011101,
            AuxIO30             = 0b011110,
            AuxIO31             = 0b011111,
            ManuelEv            = 0b100000,
            AonRtcCh2           = 0b100001,
            AonRtcCh2Dly        = 0b100010,
            AonRtc4khz          = 0b100011,
            AonBatBatUpd        = 0b100100,
            AonBatTempUpd       = 0b100101,
            SclkLf              = 0b100110,
            PwrDwn              = 0b100111,
            McuActive           = 0b101000,
            VddrRecharge        = 0b101001,
            AclkRef             = 0b101010,
            McuEv               = 0b101011,
            McuObsMux0          = 0b101100,
            McuObsMux1          = 0b101101,
            AuxCompA            = 0b101110,
            AuxCompB            = 0b101111,
            AuxTimer2Ev0        = 0b110000,
            AuxTimer2Ev1        = 0b110001,
            AuxTimer2Ev2        = 0b110010,
            AuxTimer2Ev3        = 0b110011,
            AuxTimer2Pulse      = 0b110100,
            AuxTimer1Ev         = 0b110101,
            AuxTimer0Ev         = 0b110110,
            AuxTdcDone          = 0b110111,
            AuxIsrcReset        = 0b111000,
            AuxAdcDone          = 0b111001,
            AuxAdcIrq           = 0b111010,
            AuxAdcFifoFull      = 0b111011,
            AuxAdcFifoNotEmpty  = 0b111100,
            AuxSmphAutoTakeDone = 0b111101,
            NoEvent             = 0b111110,
            NoEvent2            = 0b111111
        ]
    ],
    WUFlags [
        SW_WU3      OFFSET(7) NUMBITS(1) [],
        SW_WU2      OFFSET(6) NUMBITS(1) [],
        SW_WU1      OFFSET(5) NUMBITS(1) [],
        SW_WU0      OFFSET(4) NUMBITS(1) [],
        PROG_WU3    OFFSET(3) NUMBITS(1) [],
        PROG_WU2    OFFSET(2) NUMBITS(1) [],
        PROG_WU1    OFFSET(1) NUMBITS(1) [],
        PROG_WU0    OFFSET(0) NUMBITS(1) []
    ],
    WUGate [
        EN          OFFSET(0) NUMBITS(1) []
    ]
];

pub enum WakeUpSource {
    AuxIO0 = 0b000000,
    AuxIO1 = 0b000001,
    AuxIO2 = 0b000010,
    AuxIO3 = 0b000011,
    AuxIO4 = 0b000100,
    AuxIO5 = 0b000101,
    AuxIO6 = 0b000110,
    AuxIO7 = 0b000111,
    AuxIO8 = 0b001000,
    AuxIO9 = 0b001001,
    AuxIO10 = 0b001010,
    AuxIO11 = 0b001011,
    AuxIO12 = 0b001100,
    AuxIO13 = 0b001101,
    AuxIO14 = 0b001110,
    AuxIO15 = 0b001111,
    AuxIO16 = 0b010000,
    AuxIO17 = 0b010001,
    AuxIO18 = 0b010010,
    AuxIO19 = 0b010011,
    AuxIO20 = 0b010100,
    AuxIO21 = 0b010101,
    AuxIO22 = 0b010110,
    AuxIO23 = 0b010111,
    AuxIO24 = 0b011000,
    AuxIO25 = 0b011001,
    AuxIO26 = 0b011010,
    AuxIO27 = 0b011011,
    AuxIO28 = 0b011100,
    AuxIO29 = 0b011101,
    AuxIO30 = 0b011110,
    AuxIO31 = 0b011111,
    ManuelEv = 0b100000,
    AonRtcCh2 = 0b100001,
    AonRtcCh2Dly = 0b100010,
    AonRtc4khz = 0b100011,
    AonBatBatUpd = 0b100100,
    AonBatTempUpd = 0b100101,
    SclkLf = 0b100110,
    PwrDwn = 0b100111,
    McuActive = 0b101000,
    VddrRecharge = 0b101001,
    AclkRef = 0b101010,
    McuEv = 0b101011,
    McuObsMux0 = 0b101100,
    McuObsMux1 = 0b101101,
    AuxCompA = 0b101110,
    AuxCompB = 0b101111,
    AuxTimer2Ev0 = 0b110000,
    AuxTimer2Ev1 = 0b110001,
    AuxTimer2Ev2 = 0b110010,
    AuxTimer2Ev3 = 0b110011,
    AuxTimer2Pulse = 0b110100,
    AuxTimer1Ev = 0b110101,
    AuxTimer0Ev = 0b110110,
    AuxTdcDone = 0b110111,
    AuxIsrcReset = 0b111000,
    AuxAdcDone = 0b111001,
    AuxAdcIrq = 0b111010,
    AuxAdcFifoFull = 0b111011,
    AuxAdcFifoNotEmpty = 0b111100,
    AuxSmphAutoTakeDone = 0b111101,
    NoEvent = 0b111110,
    NoEvent2 = 0b111111,
}

pub const WUMODE_A: u8 = 0;
pub const WUMODE_LP: u8 = 1;
pub const WUMODE_PDA: u8 = 2;
pub const WUMODE_PDLP: u8 = 3;

pub enum Polarity {
    High,
    Low,
}

const AUX_SYSIF_BASE: StaticRef<AuxSysIfRegisters> =
    unsafe { StaticRef::new(0x400C_6000 as *const AuxSysIfRegisters) };

pub const AUX_CTL: Aux = Aux::new();

pub struct Aux {
    sysif_regs: StaticRef<AuxSysIfRegisters>,
}

impl Aux {
    const fn new() -> Aux {
        Aux {
            sysif_regs: AUX_SYSIF_BASE,
        }
    }

    pub fn aux_prog_wu_cfg0(&self, src: WakeUpSource, pol: Polarity, en: bool) {
        let regs = &*self.sysif_regs;
        match pol {
            Polarity::High => regs.prog_wu0_cfg.modify(WUCfg::POL::CLEAR),
            Polarity::Low => regs.prog_wu0_cfg.modify(WUCfg::POL::SET),
        }
        match en {
            true => regs.prog_wu0_cfg.modify(WUCfg::EN::SET),
            false => regs.prog_wu0_cfg.modify(WUCfg::EN::CLEAR),
        }
        match src {
            WakeUpSource::NoEvent => regs.prog_wu0_cfg.modify(WUCfg::WU_SRC::NoEvent),
            WakeUpSource::McuActive => regs.prog_wu0_cfg.modify(WUCfg::WU_SRC::McuActive),
            _ => regs.prog_wu0_cfg.modify(WUCfg::WU_SRC::NoEvent),
        }
    }

    pub fn setup(&self) {
        self.operation_mode_request(WUMODE_A);
        while self.operation_mode_ack() != WUMODE_A {}

        // self.aux_wu_enable(false);
        self.aux_prog_wu_cfg0(WakeUpSource::NoEvent, Polarity::High, true);
        self.aux_wu_enable(true);
    }

    pub fn operation_mode_request(&self, new_mode: u8) {
        let regs = &*self.sysif_regs;
        match new_mode {
            WUMODE_A => {
                regs.op_mode_req.modify(Req::REQ::Active);
            }
            WUMODE_LP => {
                regs.op_mode_req.modify(Req::REQ::LowPower);
            }
            WUMODE_PDA => {
                regs.op_mode_req.modify(Req::REQ::PowerDownActive);
            }
            WUMODE_PDLP => {
                regs.op_mode_req.modify(Req::REQ::PowerDownLowPower);
            }
            _ => panic!("Not a valid op mode"),
        }
    }

    pub fn operation_mode_ack(&self) -> u8 {
        let regs = &*self.sysif_regs;
        regs.op_mode_ack.read(Ack::ACK) as u8
    }

    pub fn aux_wu_enable(&self, enable: bool) {
        let regs = &*self.sysif_regs;
        if enable {
            regs.wu_gate.modify(WUGate::EN::SET);
        } else {
            regs.wu_gate.modify(WUGate::EN::CLEAR);
        }
    }
}
