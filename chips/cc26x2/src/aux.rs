use kernel::common::registers::{ReadWrite, ReadOnly};
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
    _wu_gate: ReadWrite<u32, WUGate::Register>,
    // remainder unimplemented
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

pub enum WUMode {
    Active = 0,
    LowPower = 1,
    PowerDownActive = 2,
    PowerDownLowPower = 3,
}

const AUX_SYSIF_BASE: StaticRef<AuxSysIfRegisters> = 
    unsafe { StaticRef::new(0x400C_6000 as *const AuxSysIfRegisters) };

pub const AUX_CTL: Aux = Aux::new();

pub struct Aux {
    sysif_regs: StaticRef<AuxSysIfRegisters>,
}

impl Aux {
    pub const fn new() -> Aux {
        Aux {
            sysif_regs: AUX_SYSIF_BASE,
        }
    }

    pub fn operation_mode_request(&self, new_mode: WUMode) {
        let regs = AUX_SYSIF_BASE;
        match new_mode {
            WUMode::Active => {
                regs.op_mode_req.modify(Req::REQ::Active);
            }
            WUMode::LowPower => {
                regs.op_mode_req.modify(Req::REQ::LowPower);
            }
            WUMode::PowerDownActive => {
                regs.op_mode_req.modify(Req::REQ::PowerDownActive);
            }
            WUMode::PowerDownLowPower => {
                regs.op_mode_req.modify(Req::REQ::PowerDownLowPower);
            }
        }
    }

    pub fn operation_mode_ack(&self) -> u8 {
        let regs = AUX_SYSIF_BASE;
        regs.op_mode_ack.read(Ack::ACK) as u8
    }

}






