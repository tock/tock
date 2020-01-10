// GENERAL PURPOSE TIMER
// Table 15-7. CC26_GPT_MAP1 Registers

// Offset   Acronym     Register Name                   Section
// 0h       CFG         Configuration                   Section 15.5.1.1.1
// 4h       TAMR        Timer A Mode                    Section 15.5.1.1.2
// 8h       TBMR        Timer B Mode                    Section 15.5.1.1.3
// Ch       CTL         Control                         Section 15.5.1.1.4
// 10h      SYNC        Synch Register                  Section 15.5.1.1.5
// ..
// 18h      IMR         Interrupt Mask                  Section 15.5.1.1.6
// 1Ch      RIS         Raw Interrupt Status            Section 15.5.1.1.7
// 20h      MIS         Masked Interrupt Status         Section 15.5.1.1.8
// 24h      ICLR        Interrupt Clear                 Section 15.5.1.1.9
// 28h      TAILR       Timer A Interval Load Register  Section 15.5.1.1.10
// 2Ch      TBILR       Timer B Interval Load Register  Section 15.5.1.1.11
// 30h      TAMATCHR    Timer A Match Register          Section 15.5.1.1.12
// 34h      TBMATCHR    Timer B Match Register          Section 15.5.1.1.13
// 38h      TAPR        Timer A Pre-scale               Section 15.5.1.1.14
// 3Ch      TBPR        Timer B Pre-scale               Section 15.5.1.1.15
// 40h      TAPMR       Timer A Pre-scale Match         Section 15.5.1.1.16
// 44h      TBPMR       Timer B Pre-scale Match         Section 15.5.1.1.17
// 48h      TAR         Timer A Register                Section 15.5.1.1.18
// 4Ch      TBR         Timer B Register                Section 15.5.1.1.19
// 50h      TAV         Timer A Value                   Section 15.5.1.1.20
// 54h      TBV         Timer B Value                   Section 15.5.1.1.21
// ..
// 5Ch      TAPS        Timer A Pre-scale Snap-shot     Section 15.5.1.1.22
// 60h      TBPS        Timer B Pre-scale Snap-shot     Section 15.5.1.1.23
// 64h      TAPV        Timer A Pre-scale Value         Section 15.5.1.1.24
// 68h      TBPV        Timer B Pre-scale Value         Section 15.5.1.1.25
// 6Ch      DMAEV       DMA Event                       Section 15.5.1.1.26
// FB0h     VERSION     Peripheral Version              Section 15.5.1.1.27
// FB4h     ANDCCP      Combined CCP Output             Section 15.5.1.1.28

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

use crate::memory_map::{GPT0_BASE, GPT1_BASE, GPT2_BASE, GPT3_BASE};

pub const GPT: [StaticRef<Registers>; 4] = unsafe {
    [
        StaticRef::new(GPT0_BASE as *const Registers),
        StaticRef::new(GPT1_BASE as *const Registers),
        StaticRef::new(GPT2_BASE as *const Registers),
        StaticRef::new(GPT3_BASE as *const Registers),
    ]
};

#[repr(C)]
pub struct Registers {
    pub cfg: ReadWrite<u32, Cfg::Register>,
    pub timer_a_mode: ReadWrite<u32, Mode::Register>, //TAMR
    pub timer_b_mode: ReadWrite<u32, Mode::Register>, //TBMR
    pub ctl: ReadWrite<u32, Ctl::Register>,
    sync: ReadWrite<u32, Sync::Register>,
    _offset0: ReadOnly<u32>,
    int_mask: ReadWrite<u32, Interrupt::Register>,
    int_raw: ReadWrite<u32, Interrupt::Register>,
    mask_int_stat: ReadWrite<u32, Interrupt::Register>,
    int_clr: ReadWrite<u32, Interrupt::Register>,
    pub timer_a_load: ReadWrite<u32, Value32::Register>,
    pub timer_b_load: ReadWrite<u32, Value32::Register>,
    pub timer_a_match: ReadWrite<u32, Value32::Register>,
    pub timer_b_match: ReadWrite<u32, Value32::Register>,
    pub timer_a_prescale: ReadWrite<u32, Prescale::Register>,
    pub timer_b_prescale: ReadWrite<u32, Prescale::Register>,
    pub timer_a_prescale_match: ReadWrite<u32, Prescale::Register>,
    pub timer_b_prescale_match: ReadWrite<u32, Prescale::Register>,
    timer_a: ReadWrite<u32>,
    timer_b: ReadWrite<u32>,
    timer_a_value: ReadOnly<u32>,
    timer_b_value: ReadOnly<u32>,
    _offset1: ReadOnly<u32>,
    timer_a_ps_ss: ReadOnly<u32, Prescale::Register>,
    timer_b_ps_ss: ReadOnly<u32, Prescale::Register>,
    timer_a_ps_val: ReadOnly<u32, Prescale::Register>,
    timer_b_ps_val: ReadOnly<u32, Prescale::Register>,
}

register_bitfields![
    u32,
    pub Cfg [
        BITS  OFFSET(0) NUMBITS(3) [
            _32 = 0x0,
            _16 = 0x4
        ]
    ],
    pub Mode [
        ACTION_ON_TIMOUT OFFSET(13) NUMBITS(3) [
            DISABLE = 0,
            TOGGLE = 0x1,
            CLEAR_CCP = 0x2,
            SET_CCP = 0x3,
            SET_CCP_AND_TOGGLE = 0x4,
            CLEAR_CCP_AND_TOGGLE = 0x5,
            SET_CCP_AND_CLEAR = 0x6,
            CLEAR_CCP_AND_SET = 0x7
        ],
        INT OFFSET(12) NUMBITS(1) [
            DISABLE = 0x1,
            ENABLE = 0x0
        ],
        LEGACY_OP OFFSET(11) NUMBITS(1) [
            ENABLE = 0x0,
            DISABLE = 0x1 //CCP output pin is set to 1 on timeout
        ],
        REG_UPDATE_MODE OFFSET(10) NUMBITS(1) [
            CYCLE = 0x0,
            TIMEOUT = 0x1
        ],
        PWM_INT OFFSET(9) NUMBITS(1) [
            DISABLED = 0x0,
            ENABLE = 0x1
        ],
        PWM_LOAD_WR OFFSET(8) NUMBITS(1) [
            CYCLE = 0x0,
            TIMEOUT = 0x1
        ],
        SNAPSHOT_MODE OFFSET(7) NUMBITS(1) [
            DISABLED = 0x0,
            ENABLED = 0x1
        ],
        WAIT_ON_TRIGGER OFFSET(6) NUMBITS(1) [
            DISABLED = 0x0,
            ENABLED = 0x1
        ],
        MATCH_INTERRUPT OFFSET(5) NUMBITS(1) [
            DISABLED = 0x0,
            ENABLED = 0x1
        ],
        COUNT_DIRECTION OFFSET(4) NUMBITS(1) [
            DOWN = 0x0,
            UP = 0x1
        ],
        ALT_MODE OFFSET(3) NUMBITS(1) [
            CAPTURE_COMPARE = 0x0,
            PWM = 0x1
        ],
        CAPTURE_MODE OFFSET(2) NUMBITS(1) [
            EDGE_COUNT = 0,
            EDGE_TIME = 1
        ],
        MODE OFFSET(0) NUMBITS(2) [
            ONE_SHOT = 0x1,
            PERIODIC = 0x2,
            CAPTURE = 0x3
        ]
    ],
    pub Ctl [
        TIMER_B_PWM_OUTPUT_INVERT OFFSET(14) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TIMER_B_EVENT OFFSET(10) NUMBITS (2) [
            POSITIVE_EDGE = 0x0,
            NEGATIVE_EDGE = 0x1,
            BOTH_EDGES = 0x3
        ],
        // timer counts when processor halted by debugger
        TIMER_B_STALL OFFSET(9) NUMBITS(1) [
            DISABLE = 0x0,
            ENBABLE = 0x1
        ],
        TIMER_B_EN OFFSET(8) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TIMER_A_PWM_OUTPUT_INVERT OFFSET(6) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TIMER_A_EVENT OFFSET(2) NUMBITS (2) [
            POSITIVE_EDGE = 0x0,
            NEGATIVE_EDGE = 0x1,
            BOTH_EDGES = 0x3
        ],
        // timer counts when processor halted by debugger
        TIMER_A_STALL OFFSET(1) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TIMER_A_EN OFFSET(0) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ]
    ],
    pub Sync [
        _3 OFFSET(6) NUMBITS(2) [
            NONE = 0x0,
            TIMEOUT_TIMER_A = 0x1,
            TIMEOUT_TIMER_B = 0x2,
            TIMEOUT_BOTH_TIMERS = 0x3
        ],
        _2 OFFSET(4) NUMBITS(2) [
            NONE = 0x0,
            TIMEOUT_TIMER_A = 0x1,
            TIMEOUT_TIMER_B = 0x2,
            TIMEOUT_BOTH_TIMERS = 0x3
        ],
        _1 OFFSET(2) NUMBITS(2) [
            NONE = 0x0,
            TIMEOUT_TIMER_A = 0x1,
            TIMEOUT_TIMER_B = 0x2,
            TIMEOUT_BOTH_TIMERS = 0x3
        ],
        _0 OFFSET(2) NUMBITS(2) [
            NONE = 0x0,
            TIMEOUT_TIMER_A = 0x1,
            TIMEOUT_TIMER_B = 0x2,
            TIMEOUT_BOTH_TIMERS = 0x3
        ]
    ],
    pub Interrupt [
        DMAB OFFSET(13) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TBM OFFSET(11) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        CBE OFFSET(10) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        CBM OFFSET(9) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TBT OFFSET(8) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        DMAA OFFSET(5) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TAM OFFSET(4) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        CAE OFFSET(2) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        CAM OFFSET(1) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        TAT OFFSET(0) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ]
    ],
    pub Prescale [
        // ratio is value written here plus one
        RATIO OFFSET(0) NUMBITS(8) []
    ],
    pub PrescaleMSB [
        // "In 16-bit mode, this register holds bits 23 to 16" What?
        RATIO OFFSET(0) NUMBITS(8) []
    ],
    pub Value32 [
        // "In 16-bit mode, this register holds bits 23 to 16" What?
        SET OFFSET(0) NUMBITS(32) []
    ]
];
