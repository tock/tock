use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub struct AuxSysIfRegisters {
    op_mode_req: ReadWrite<u32, Req::Register>,
    op_mode_ack: ReadOnly<u32, Ack::Register>,
    _prog_wu0_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu1_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu2_cfg: ReadWrite<u32, WUCfg::Register>,
    _prog_wu3_cfg: ReadWrite<u32, WUCfg::Register>,
    _wu_flags: ReadOnly<u32, WUFlags::Register>,
    _wu_flags_clr: ReadWrite<u32, WUFlags::Register>,
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
        EN          OFFSET(6) NUMBITS(1) [],
        WU_SRC      OFFSET(0) NUMBITS(5) []
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

pub const WUMODE_A: u8 = 0;
pub const WUMODE_LP: u8 = 1;
pub const WUMODE_PDA: u8 = 2;
pub const WUMODE_PDLP: u8 = 3;

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
