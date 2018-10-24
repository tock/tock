use kernel::common::registers::{ReadWrite, WriteOnly};
use kernel::common::StaticRef;

use memory_map::AUX_TIMER2_BASE;

pub const REG: StaticRef<Registers> =
    unsafe { StaticRef::new(AUX_TIMER2_BASE as *const Registers) };

// Table 19-128. CC26_AUX_TIMER2_MMAP_AUX_TIMER2 Registers

// 0h       CTL             Timer Control                       Section 19.8.7.1
// 4h       TARGET          Target                              Section 19.8.7.2
// 8h       SHDWTARGET      Shadow Target                       Section 19.8.7.3
// Ch       CNTR            Counter                             Section 19.8.7.4
// 10h      PRECFG          Clock Prescaler Configuration       Section 19.8.7.5
// 14h      EVCTL           Event Control                       Section 19.8.7.6
// 18h      PULSETRIG       Pulse Trigger                       Section 19.8.7.7
// 80h      CH0EVCFG        Channel 0 Event Configuration       Section 19.8.7.8
// 84h      CH0CCFG         Channel 0 Capture Configuration     Section 19.8.7.9
// 88h      CH0PCC          Channel 0 Pipeline Capture Compare  Section 19.8.7.10
// 8Ch      CH0CC           Channel 0 Capture Compare           Section 19.8.7.11
// 90h      CH1EVCFG        Channel 1 Event Configuration       Section 19.8.7.12
// 94h      CH1CCFG         Channel 1 Capture Configuration     Section 19.8.7.13
// 98h      CH1PCC          Channel 1 Pipeline Capture Compare  Section 19.8.7.14
// 9Ch      CH1CC           Channel 1 Capture Compare           Section 19.8.7.15
// A0h      CH2EVCFG        Channel 2 Event Configuration       Section 19.8.7.16
// A4h      CH2CCFG         Channel 2 Capture Configuration     Section 19.8.7.17
// A8h      CH2PCC          Channel 2 Pipeline Capture Compare  Section 19.8.7.18
// ACh      CH2CC           Channel 2 Capture Compare           Section 19.8.7.19
// B0h      CH3EVCFG        Channel 3 Event Configuration       Section 19.8.7.20
// B4h      CH3CCFG         Channel 3 Capture Configuration     Section 19.8.7.21
// B8h      CH3PCC          Channel 3 Pipeline Capture Compare  Section 19.8.7.22
// BCh      CH3CC           Channel 3 Capture Compare           Section 19.8.7.23

#[repr(C)]
pub struct Registers {
    ctrl: ReadWrite<u8, Ctrl::Register>,
    target: ReadWrite<u8, Value16::Register>,
    shadow_target: ReadWrite<u8, Value16::Register>,
    counter: ReadWrite<u8, Value16::Register>,
    precfg: ReadWrite<u8, Value8::Register>,
    evt_ctrl: WriteOnly<u32, EvtCtrl::Register>,
    pulse_trigger: WriteOnly<u32, PulseTrigger::Register>,
}

register_bitfields![
u8,
Ctrl [
    CH3_RESET OFFSET(6) NUMBITS(1) [],
    CH2_RESET OFFSET(5) NUMBITS(1) [],
    CH1_RESET OFFSET(4) NUMBITS(1) [],
    CH0_RESET OFFSET(3) NUMBITS(1) [],
    TARGET_EN OFFSET(2) NUMBITS(1) [],
    MODE OFFSET(2) NUMBITS(1) [
        Disable = 0x0,
        CountUpOnce = 0x1,
        CountUpPeriodically = 0x2,
        CounterUpAndDownPeriodically = 0x3
    ]
],
Value16 [
    VALUE OFFSET(0) NUMBITS(16) []
],
Value8 [
    VALUE OFFSET(0) NUMBITS(8) []
],
EvtCtrl [
    EVT3_SET OFFSET(7) NUMBITS(1) [],
    EVT3_CLR OFFSET(6) NUMBITS(1) [],
    EVT2_SET OFFSET(5) NUMBITS(1) [],
    EVT2_CLR OFFSET(4) NUMBITS(1) [],
    EVT1_SET OFFSET(3) NUMBITS(1) [],
    EVT1_CLR OFFSET(2) NUMBITS(1) [],
    EVT0_SET OFFSET(1) NUMBITS(1) [],
    EVT0_CLR OFFSET(0) NUMBITS(1) []
],
PulseTrigger [
    TRIG OFFSET(0) NUMBITS(1) []
],
ChEvtCfg [
    EV3_GEN OFFSET(7) NUMBITS(1) [],
    EV2_GEN OFFSET(6) NUMBITS(1) [],
    EV1_GEN OFFSET(5) NUMBITS(1) [],
    EV0_GEN OFFSET(4) NUMBITS(1) []
]

];
