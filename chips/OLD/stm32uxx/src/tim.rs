//Basic TIM2 implementation for STM32U5 series microcontrollers.

use cortexm33::support::with_interrupts_disabled;
use kernel::hil::time::{
    Alarm, AlarmClient, Counter, Freq16KHz, Frequency, OverflowClient, Ticks, Ticks32, Time,
};
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::{phclk, Stm32u5Clocks};
use crate::nvic;

#[repr(C)]
struct TimRegisters {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// slave mode control register
    smcr: ReadWrite<u32, SMCR::Register>,
    /// DMA/Interrupt enable register
    dier: ReadWrite<u32, DIER::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    /// event generation register
    egr: WriteOnly<u32, EGR::Register>,
    /// capture/compare mode register 1 (output mode
    ccmr1_output: ReadWrite<u32, CCMR1_Output::Register>,
    /// capture/compare mode register 2 (output mode
    ccmr2_output: ReadWrite<u32, CCMR2_Output::Register>,
    /// capture/compare enable register
    ccer: ReadWrite<u32, CCER::Register>,
    /// counter
    cnt: ReadWrite<u32, CNT::Register>,
    /// prescaler
    psc: ReadWrite<u32>,
    /// auto-reload register
    arr: ReadWrite<u32>,
    /// repetition counter register
    rcr: ReadWrite<u32>,
    /// capture/compare register 1
    ccr1: ReadWrite<u32>,
    /// capture/compare register 2
    ccr2: ReadWrite<u32>,
    /// capture/compare register 3
    ccr3: ReadWrite<u32>,
    /// capture/compare register 4
    ccr4: ReadWrite<u32>,
    /// break and dead-time register
    bdtr: ReadWrite<u32, BDTR::Register>,
    /// alternate function register 2
    ccr5: ReadWrite<u32, CCR5::Register>,
    /// alternate function register 2
    ccr6: ReadWrite<u32>,
    /// capture/compare mode register 3
    ccmr3: ReadWrite<u32, CCMR3::Register>,
    /// deadtime register 2
    dtr2: ReadWrite<u32, DTR2::Register>,
    /// encoder control register
    ecr: ReadWrite<u32, ECR::Register>,
    /// timer input selection register
    tisel: ReadWrite<u32, TISEL::Register>,
    /// alternate function option register 1
    af1: ReadWrite<u32, AF1::Register>,
    /// alternate function register 2
    af2: ReadWrite<u32, AF2::Register>,
    _reserved0: [u8; 0x004],
    /// DMA control register
    dcr: ReadWrite<u32, DCR::Register>,
    /// DMA address for full transfer
    dmar: ReadWrite<u32>,
}

register_bitfields![u32,
CR1 [
    /// Dithering enable
    DITHEN OFFSET(12) NUMBITS(1) [],
    /// UIF status bit remapping
    UIFREMAP OFFSET(11) NUMBITS(1) [],
    /// Clock division
    CKD OFFSET(8) NUMBITS(2) [],
    /// Auto-reload preload enable
    ARPE OFFSET(7) NUMBITS(1) [],
    /// Center-aligned mode
///               selection
    CMS OFFSET(5) NUMBITS(2) [],
    /// Direction
    DIR OFFSET(4) NUMBITS(1) [],
    /// One-pulse mode
    OPM OFFSET(3) NUMBITS(1) [],
    /// Update request source
    URS OFFSET(2) NUMBITS(1) [],
    /// Update disable
    UDIS OFFSET(1) NUMBITS(1) [],
    /// Counter enable
    CEN OFFSET(0) NUMBITS(1) []
],
CR2 [
    /// Master mode selection 2
    MMS_3 OFFSET(25) NUMBITS(1) [],
    /// Master mode selection 2
    MMS2 OFFSET(20) NUMBITS(4) [],
    /// Output Idle state 6
    OIS6 OFFSET(18) NUMBITS(1) [],
    /// Output Idle state 5
    OIS5 OFFSET(16) NUMBITS(1) [],
    /// Output Idle state 4 (OC5
///               output)
    OIS4N OFFSET(15) NUMBITS(1) [],
    /// Output Idle state 4
    OIS4 OFFSET(14) NUMBITS(1) [],
    /// Output Idle state 3
    OIS3N OFFSET(13) NUMBITS(1) [],
    /// Output Idle state 3
    OIS3 OFFSET(12) NUMBITS(1) [],
    /// Output Idle state 2
    OIS2N OFFSET(11) NUMBITS(1) [],
    /// Output Idle state 2
    OIS2 OFFSET(10) NUMBITS(1) [],
    /// Output Idle state 1
    OIS1N OFFSET(9) NUMBITS(1) [],
    /// Output Idle state 1
    OIS1 OFFSET(8) NUMBITS(1) [],
    /// TI1 selection
    TI1S OFFSET(7) NUMBITS(1) [],
    /// Master mode selection
    MMS0_2 OFFSET(4) NUMBITS(3) [],
    /// Capture/compare DMA
///               selection
    CCDS OFFSET(3) NUMBITS(1) [],
    /// Capture/compare control update
///               selection
    CCUS OFFSET(2) NUMBITS(1) [],
    /// Capture/compare preloaded
///               control
    CCPC OFFSET(0) NUMBITS(1) []
],
SMCR [
    /// SMS preload source
    SMSPS OFFSET(25) NUMBITS(1) [],
    /// SMS preload enable
    SMSPE OFFSET(24) NUMBITS(1) [],
    /// Trigger selection
    TS4_3 OFFSET(20) NUMBITS(2) [],
    /// Slave mode selection
    SMS3_0 OFFSET(16) NUMBITS(1) [],
    /// External trigger polarity
    ETP OFFSET(15) NUMBITS(1) [],
    /// External clock enable
    ECE OFFSET(14) NUMBITS(1) [],
    /// External trigger prescaler
    ETPS OFFSET(12) NUMBITS(2) [],
    /// External trigger filter
    ETF OFFSET(8) NUMBITS(4) [],
    /// Master/Slave mode
    MSM OFFSET(7) NUMBITS(1) [],
    /// Trigger selection
    TS OFFSET(4) NUMBITS(3) [],
    /// OCREF clear selection
    OCCS OFFSET(3) NUMBITS(1) [],
    /// Slave mode selection
    SMS OFFSET(0) NUMBITS(3) []
],
DIER [
    /// Transition error interrupt enable
    TERRIE OFFSET(23) NUMBITS(1) [],
    /// Index error interrupt enable
    IERRIE OFFSET(22) NUMBITS(1) [],
    /// Direction change interrupt enable
    DIRIE OFFSET(21) NUMBITS(1) [],
    /// Index interrupt enable
    IDXIE OFFSET(20) NUMBITS(1) [],
    /// Trigger DMA request enable
    TDE OFFSET(14) NUMBITS(1) [],
    /// COM DMA request enable
    COMDE OFFSET(13) NUMBITS(1) [],
    /// Capture/Compare 4 DMA request
///               enable
    CC4DE OFFSET(12) NUMBITS(1) [],
    /// Capture/Compare 3 DMA request
///               enable
    CC3DE OFFSET(11) NUMBITS(1) [],
    /// Capture/Compare 2 DMA request
///               enable
    CC2DE OFFSET(10) NUMBITS(1) [],
    /// Capture/Compare 1 DMA request
///               enable
    CC1DE OFFSET(9) NUMBITS(1) [],
    /// Update DMA request enable
    UDE OFFSET(8) NUMBITS(1) [],
    /// Break interrupt enable
    BIE OFFSET(7) NUMBITS(1) [],
    /// Trigger interrupt enable
    TIE OFFSET(6) NUMBITS(1) [],
    /// COM interrupt enable
    COMIE OFFSET(5) NUMBITS(1) [],
    /// Capture/Compare 4 interrupt
///               enable
    CC4IE OFFSET(4) NUMBITS(1) [],
    /// Capture/Compare 3 interrupt
///               enable
    CC3IE OFFSET(3) NUMBITS(1) [],
    /// Capture/Compare 2 interrupt
///               enable
    CC2IE OFFSET(2) NUMBITS(1) [],
    /// Capture/Compare 1 interrupt
///               enable
    CC1IE OFFSET(1) NUMBITS(1) [],
    /// Update interrupt enable
    UIE OFFSET(0) NUMBITS(1) []
],
SR [
    /// Transition error interrupt flag
    TERRF OFFSET(23) NUMBITS(1) [],
    /// Index error interrupt flag
    IERRF OFFSET(22) NUMBITS(1) [],
    /// Direction change interrupt flag
    DIRF OFFSET(21) NUMBITS(1) [],
    /// Index interrupt flag
    IDXF OFFSET(20) NUMBITS(1) [],
    /// Compare 6 interrupt flag
    CC6IF OFFSET(17) NUMBITS(1) [],
    /// Compare 5 interrupt flag
    CC5IF OFFSET(16) NUMBITS(1) [],
    /// System Break interrupt
///               flag
    SBIF OFFSET(13) NUMBITS(1) [],
    /// Capture/Compare 4 overcapture
///               flag
    CC4OF OFFSET(12) NUMBITS(1) [],
    /// Capture/Compare 3 overcapture
///               flag
    CC3OF OFFSET(11) NUMBITS(1) [],
    /// Capture/compare 2 overcapture
///               flag
    CC2OF OFFSET(10) NUMBITS(1) [],
    /// Capture/Compare 1 overcapture
///               flag
    CC1OF OFFSET(9) NUMBITS(1) [],
    /// Break 2 interrupt flag
    B2IF OFFSET(8) NUMBITS(1) [],
    /// Break interrupt flag
    BIF OFFSET(7) NUMBITS(1) [],
    /// Trigger interrupt flag
    TIF OFFSET(6) NUMBITS(1) [],
    /// COM interrupt flag
    COMIF OFFSET(5) NUMBITS(1) [],
    /// Capture/Compare 4 interrupt
///               flag
    CC4IF OFFSET(4) NUMBITS(1) [],
    /// Capture/Compare 3 interrupt
///               flag
    CC3IF OFFSET(3) NUMBITS(1) [],
    /// Capture/Compare 2 interrupt
///               flag
    CC2IF OFFSET(2) NUMBITS(1) [],
    /// Capture/compare 1 interrupt
///               flag
    CC1IF OFFSET(1) NUMBITS(1) [],
    /// Update interrupt flag
    UIF OFFSET(0) NUMBITS(1) []
],
EGR [
    /// Break 2 generation
    B2G OFFSET(8) NUMBITS(1) [],
    /// Break generation
    BG OFFSET(7) NUMBITS(1) [],
    /// Trigger generation
    TG OFFSET(6) NUMBITS(1) [],
    /// Capture/Compare control update generation
    COMG OFFSET(5) NUMBITS(1) [],
    /// Capture/compare 4
///               generation
    CC4G OFFSET(4) NUMBITS(1) [],
    /// Capture/compare 3
///               generation
    CC3G OFFSET(3) NUMBITS(1) [],
    /// Capture/compare 2
///               generation
    CC2G OFFSET(2) NUMBITS(1) [],
    /// Capture/compare 1
///               generation
    CC1G OFFSET(1) NUMBITS(1) [],
    /// Update generation
    UG OFFSET(0) NUMBITS(1) []
],
CCMR1_Output [
    /// Output Compare 2 mode - bit
///               3
    OC2M_bit3 OFFSET(24) NUMBITS(1) [],
    /// Output Compare 1 mode - bit
///               3
    OC1M_bit3 OFFSET(16) NUMBITS(1) [],
    /// Output Compare 2 clear
///               enable
    OC2CE OFFSET(15) NUMBITS(1) [],
    /// Output Compare 2 mode
    OC2M OFFSET(12) NUMBITS(3) [],
    /// Output Compare 2 preload
///               enable
    OC2PE OFFSET(11) NUMBITS(1) [],
    /// Output Compare 2 fast
///               enable
    OC2FE OFFSET(10) NUMBITS(1) [],
    /// Capture/Compare 2
///               selection
    CC2S OFFSET(8) NUMBITS(2) [],
    /// Output Compare 1 clear
///               enable
    OC1CE OFFSET(7) NUMBITS(1) [],
    /// Output Compare 1 mode
    OC1M OFFSET(4) NUMBITS(3) [],
    /// Output Compare 1 preload
///               enable
    OC1PE OFFSET(3) NUMBITS(1) [],
    /// Output Compare 1 fast
///               enable
    OC1FE OFFSET(2) NUMBITS(1) [],
    /// Capture/Compare 1
///               selection
    CC1S OFFSET(0) NUMBITS(2) []
],
CCMR1_Input [
    /// Input capture 2 filter
    IC2F OFFSET(12) NUMBITS(4) [],
    /// Input capture 2 prescaler
    IC2PCS OFFSET(10) NUMBITS(2) [],
    /// Capture/Compare 2
///               selection
    CC2S OFFSET(8) NUMBITS(2) [],
    /// Input capture 1 filter
    IC1F OFFSET(4) NUMBITS(4) [],
    /// Input capture 1 prescaler
    ICPCS OFFSET(2) NUMBITS(2) [],
    /// Capture/Compare 1
///               selection
    CC1S OFFSET(0) NUMBITS(2) []
],
CCMR2_Output [
    /// Output Compare 4 mode - bit
///               3
    OC4M_bit3 OFFSET(24) NUMBITS(1) [],
    /// Output compare 3 mode
    OC3M_3 OFFSET(16) NUMBITS(1) [],
    /// Output compare 4 clear enable
    OC4CE OFFSET(15) NUMBITS(1) [],
    /// Output compare 4 mode
    OC4M_3_0 OFFSET(12) NUMBITS(3) [],
    /// Output compare 4 preload
///               enable
    OC4PE OFFSET(11) NUMBITS(1) [],
    /// Output compare 4 fast
///               enable
    OC4FE OFFSET(10) NUMBITS(1) [],
    /// Capture/Compare 4
///               selection
    CC4S_1_0 OFFSET(8) NUMBITS(2) [],
    /// Output compare 3 clear
///               enable
    OC3CE OFFSET(7) NUMBITS(1) [],
    /// Output compare 3 mode
    OC3M_2_0 OFFSET(4) NUMBITS(3) [],
    /// Output compare 3 preload
///               enable
    OC3PE OFFSET(3) NUMBITS(1) [],
    /// Output compare 3 fast
///               enable
    OC3FE OFFSET(2) NUMBITS(1) [],
    /// Capture/Compare 3
///               selection
    CC3S_1_0 OFFSET(0) NUMBITS(2) []
],
CCMR2_Input [
    /// Input capture 4 filter
    IC4F OFFSET(12) NUMBITS(4) [],
    /// Input capture 4 prescaler
    IC4PSC OFFSET(10) NUMBITS(2) [],
    /// Capture/Compare 4
///               selection
    CC4S OFFSET(8) NUMBITS(2) [],
    /// Input capture 3 filter
    IC3F OFFSET(4) NUMBITS(4) [],
    /// Input capture 3 prescaler
    IC3PSC OFFSET(2) NUMBITS(2) [],
    /// Capture/compare 3
///               selection
    CC3S OFFSET(0) NUMBITS(2) []
],
CCER [
    /// Capture/Compare 6 output
///               polarity
    CC6P OFFSET(21) NUMBITS(1) [],
    /// Capture/Compare 6 output
///               enable
    CC6E OFFSET(20) NUMBITS(1) [],
    /// Capture/Compare 5 output
///               polarity
    CC5P OFFSET(17) NUMBITS(1) [],
    /// Capture/Compare 5 output
///               enable
    CC5E OFFSET(16) NUMBITS(1) [],
    /// Capture/Compare 4 complementary output
///               polarity
    CC4NP OFFSET(15) NUMBITS(1) [],
    /// Capture/Compare 3 output
///               Polarity
    CC4P OFFSET(13) NUMBITS(1) [],
    /// Capture/Compare 4 output
///               enable
    CC4E OFFSET(12) NUMBITS(1) [],
    /// Capture/Compare 3 output
///               Polarity
    CC3NP OFFSET(11) NUMBITS(1) [],
    /// Capture/Compare 3 complementary output
///               enable
    CC3NE OFFSET(10) NUMBITS(1) [],
    /// Capture/Compare 3 output
///               Polarity
    CC3P OFFSET(9) NUMBITS(1) [],
    /// Capture/Compare 3 output
///               enable
    CC3E OFFSET(8) NUMBITS(1) [],
    /// Capture/Compare 2 output
///               Polarity
    CC2NP OFFSET(7) NUMBITS(1) [],
    /// Capture/Compare 2 complementary output
///               enable
    CC2NE OFFSET(6) NUMBITS(1) [],
    /// Capture/Compare 2 output
///               Polarity
    CC2P OFFSET(5) NUMBITS(1) [],
    /// Capture/Compare 2 output
///               enable
    CC2E OFFSET(4) NUMBITS(1) [],
    /// Capture/Compare 1 output
///               Polarity
    CC1NP OFFSET(3) NUMBITS(1) [],
    /// Capture/Compare 1 complementary output
///               enable
    CC1NE OFFSET(2) NUMBITS(1) [],
    /// Capture/Compare 1 output
///               Polarity
    CC1P OFFSET(1) NUMBITS(1) [],
    /// Capture/Compare 1 output
///               enable
    CC1E OFFSET(0) NUMBITS(1) []
],
CNT [
    /// UIF copy
    UIFCPY OFFSET(31) NUMBITS(1) [],
    /// counter value
    CNT OFFSET(0) NUMBITS(16) []
],
PSC [
    /// Prescaler value
    PSC OFFSET(0) NUMBITS(16) []
],
ARR [
    /// Auto-reload value
    ARR OFFSET(0) NUMBITS(20) []
],
RCR [
    /// Repetition counter value
    REP OFFSET(0) NUMBITS(16) []
],
CCR1 [
    /// Capture/Compare 1 value
    CCR1 OFFSET(0) NUMBITS(20) []
],
CCR2 [
    /// Capture/Compare 2 value
    CCR2 OFFSET(0) NUMBITS(20) []
],
CCR3 [
    /// Capture/Compare value
    CCR3 OFFSET(0) NUMBITS(20) []
],
CCR4 [
    /// Capture/Compare value
    CCR4 OFFSET(0) NUMBITS(20) []
],
BDTR [
    /// Break2 bidirectional
    BK2BID OFFSET(29) NUMBITS(1) [],
    /// Break Bidirectional
    BKBID OFFSET(28) NUMBITS(1) [],
    /// Break2 Disarm
    BK2DSRAM OFFSET(27) NUMBITS(1) [],
    /// Break Disarm
    BKDSRM OFFSET(26) NUMBITS(1) [],
    /// Break 2 polarity
    BK2P OFFSET(25) NUMBITS(1) [],
    /// Break 2 enable
    BK2E OFFSET(24) NUMBITS(1) [],
    /// Break 2 filter
    BK2F OFFSET(20) NUMBITS(4) [],
    /// Break filter
    BKF OFFSET(16) NUMBITS(4) [],
    /// Main output enable
    MOE OFFSET(15) NUMBITS(1) [],
    /// Automatic output enable
    AOE OFFSET(14) NUMBITS(1) [],
    /// Break polarity
    BKP OFFSET(13) NUMBITS(1) [],
    /// Break enable
    BKE OFFSET(12) NUMBITS(1) [],
    /// Off-state selection for Run
///               mode
    OSSR OFFSET(11) NUMBITS(1) [],
    /// Off-state selection for Idle
///               mode
    OSSI OFFSET(10) NUMBITS(1) [],
    /// Lock configuration
    LOCK OFFSET(8) NUMBITS(2) [],
    /// Dead-time generator setup
    DTG OFFSET(0) NUMBITS(8) []
],
CCR5 [
    /// CCR5
    CCR5 OFFSET(0) NUMBITS(20) [],
    /// GC5C1
    GC5C1 OFFSET(29) NUMBITS(1) [],
    /// GC5C2
    GC5C2 OFFSET(30) NUMBITS(1) [],
    /// GC5C3
    GC5C3 OFFSET(31) NUMBITS(1) []
],
CCR6 [
    /// CCR6
    CCR6 OFFSET(0) NUMBITS(20) []
],
CCMR3 [
    /// Output compare 5 fast enable
    OC5FE OFFSET(2) NUMBITS(1) [],
    /// Output compare 5 preload enable
    OC5PE OFFSET(3) NUMBITS(1) [],
    /// Output compare 5 mode
    OC5M1 OFFSET(4) NUMBITS(3) [],
    /// Output compare 5 clear enable
    OC5CE OFFSET(7) NUMBITS(1) [],
    /// Output compare 6 fast enable
    OC6FE OFFSET(10) NUMBITS(1) [],
    /// Output compare 6 preload enable
    OC6PE OFFSET(11) NUMBITS(1) [],
    /// Output compare 6 mode
    OC6M1 OFFSET(12) NUMBITS(3) [],
    /// Output compare 6 clear enable
    OC6CE OFFSET(15) NUMBITS(1) [],
    /// Output compare 5 mode
    OC5M2 OFFSET(16) NUMBITS(1) [],
    /// Output compare 6 mode
    OC6M OFFSET(24) NUMBITS(1) []
],
DTR2 [
    /// Deadtime preload enable
    DTPE OFFSET(17) NUMBITS(1) [],
    /// Deadtime asymmetric enable
    DTAE OFFSET(16) NUMBITS(1) [],
    /// Dead-time falling edge generator setup
    DTGF OFFSET(0) NUMBITS(8) []
],
ECR [
    /// Pulse width prescaler
///
    PWPRSC OFFSET(24) NUMBITS(3) [],
    /// Pulse width
///
    PW OFFSET(16) NUMBITS(8) [],
    /// Index positioning
///
    IPOS OFFSET(6) NUMBITS(2) [],
    /// First index
///
    FIDX OFFSET(5) NUMBITS(1) [],
    /// Index direction
///
    IDIR OFFSET(1) NUMBITS(2) [],
    /// Index enable
///
    IE OFFSET(0) NUMBITS(1) []
],
TISEL [
    /// Selects tim_ti4[0..15] input
    TI4SEL OFFSET(24) NUMBITS(4) [],
    /// Selects tim_ti3[0..15] input
    TI3SEL OFFSET(16) NUMBITS(4) [],
    /// Selects tim_ti3[0..15] input
    TI2SEL OFFSET(8) NUMBITS(4) [],
    /// Selects tim_ti3[0..15] input
    TI1SEL OFFSET(0) NUMBITS(4) []
],
AF1 [
    /// ETR source selection
    ETRSEL OFFSET(14) NUMBITS(4) [],
    /// tim_brk_cmp4 input polarity
    BKCMP4P OFFSET(13) NUMBITS(1) [],
    /// tim_brk_cmp3 input polarity
    BKCMP3P OFFSET(12) NUMBITS(1) [],
    /// BRK COMP2 input polarity
    BKCMP2P OFFSET(11) NUMBITS(1) [],
    /// BRK COMP1 input polarity
    BKCMP1P OFFSET(10) NUMBITS(1) [],
    /// TIMx_BKIN input polarity
    BKINP OFFSET(9) NUMBITS(1) [],
    /// tim_brk_cmp8 enable
    BKCMP8E OFFSET(8) NUMBITS(1) [],
    /// tim_brk_cmp7 enable
    BKCMP7E OFFSET(7) NUMBITS(1) [],
    /// tim_brk_cmp6 enable
    BKCMP6E OFFSET(6) NUMBITS(1) [],
    /// tim_brk_cmp5 enable
    BKCMP5E OFFSET(5) NUMBITS(1) [],
    /// tim_brk_cmp4 enable
    BKCMP4E OFFSET(4) NUMBITS(1) [],
    /// tim_brk_cmp3 enable
    BKCMP3E OFFSET(3) NUMBITS(1) [],
    /// BRK COMP2 enable
    BKCMP2E OFFSET(2) NUMBITS(1) [],
    /// BRK COMP1 enable
    BKCMP1E OFFSET(1) NUMBITS(1) [],
    /// BRK BKIN input enable
    BKINE OFFSET(0) NUMBITS(1) []
],
AF2 [
    /// ocref_clr source selection
    OCRSEL OFFSET(16) NUMBITS(3) [],
    /// tim_brk2_cmp4 input polarity
    BK2CMP4P OFFSET(13) NUMBITS(1) [],
    /// tim_brk2_cmp3 input polarity
    BK2CMP3P OFFSET(12) NUMBITS(1) [],
    /// tim_brk2_cmp2 input polarity
    BK2CMP2P OFFSET(11) NUMBITS(1) [],
    /// tim_brk2_cmp1 input polarity
    BK2CMP1P OFFSET(10) NUMBITS(1) [],
    /// TIMx_BKIN2 input polarity
    BK2INP OFFSET(9) NUMBITS(1) [],
    /// tim_brk2_cmp8 enable
    BK2CMP8E OFFSET(8) NUMBITS(1) [],
    /// tim_brk2_cmp7 enable
    BK2CMP7E OFFSET(7) NUMBITS(1) [],
    /// tim_brk2_cmp6 enable
    BK2CMP6E OFFSET(6) NUMBITS(1) [],
    /// tim_brk2_cmp5 enable
    BK2CMP5E OFFSET(5) NUMBITS(1) [],
    /// tim_brk2_cmp4 enable
    BK2CMP4E OFFSET(4) NUMBITS(1) [],
    /// tim_brk2_cmp3 enable
    BK2CMP3E OFFSET(3) NUMBITS(1) [],
    /// BRK2 COMP2 enable
    BK2CMP2E OFFSET(2) NUMBITS(1) [],
    /// BRK2 COMP1 enable
    BK2CMP1E OFFSET(1) NUMBITS(1) [],
    /// BRK2 BKIN input enable
    BK2INE OFFSET(0) NUMBITS(1) []
],
DCR [
    /// DMA burst source selection
    DBSS OFFSET(16) NUMBITS(4) [],
    /// DMA burst length
    DBL OFFSET(8) NUMBITS(5) [],
    /// DMA base address
    DBA OFFSET(0) NUMBITS(5) []
],
DMAR [
    /// DMA register for burst
///               accesses
    DMAB OFFSET(0) NUMBITS(32) []
]
];
const TIM1_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40012C00 as *const TimRegisters) };

const SEC_TIM1_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50012C00 as *const TimRegisters) };

const TIM2_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const TimRegisters) };

const SEC_TIM2_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000000 as *const TimRegisters) };

const TIM3_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40000400 as *const TimRegisters) };

const SEC_TIM3_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000400 as *const TimRegisters) };

const TIM4_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40000800 as *const TimRegisters) };

const SEC_TIM4_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000800 as *const TimRegisters) };

const TIM5_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40000C00 as *const TimRegisters) };

const SEC_TIM5_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50000C00 as *const TimRegisters) };

const TIM6_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const TimRegisters) };

const SEC_TIM6_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50001000 as *const TimRegisters) };

const TIM7_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40001400 as *const TimRegisters) };

const SEC_TIM7_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50001400 as *const TimRegisters) };

const TIM8_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40013400 as *const TimRegisters) };

const SEC_TIM8_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50013400 as *const TimRegisters) };

const TIM15_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40014000 as *const TimRegisters) };

const SEC_TIM15_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50014000 as *const TimRegisters) };

const TIM16_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40014400 as *const TimRegisters) };

const SEC_TIM16_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50014400 as *const TimRegisters) };

const TIM17_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x40014800 as *const TimRegisters) };

const SEC_TIM17_BASE: StaticRef<TimRegisters> =
    unsafe { StaticRef::new(0x50014800 as *const TimRegisters) };

pub struct Tim2<'a> {
    registers: StaticRef<TimRegisters>,
    clock: Tim2Clock<'a>,
    client: OptionalCell<&'a dyn AlarmClient>,
    irqn: u32,
}

impl<'a> Tim2<'a> {
    pub const fn new(clocks: &'a dyn Stm32u5Clocks) -> Self {
        Self {
            registers: TIM2_BASE,
            clock: Tim2Clock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB1(phclk::PCLK1::TIM2),
                clocks,
            )),
            client: OptionalCell::empty(),
            irqn: nvic::TIM2,
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable()
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn handle_interrupt(&self) {
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        self.client.map(|client| client.alarm());
    }

    // Assume a 16 kHz clock -- Possible to change later
    pub fn start(&self) {
        self.registers.arr.set(0xFFFF_FFFF - 1);
        self.calibrate();
    }

    pub fn calibrate(&self) {
        let clk_freq = self.clock.0.get_frequency();

        let psc = clk_freq / Freq16KHz::frequency();
        self.registers.psc.set(psc - 1);

        self.registers.egr.write(EGR::UG::SET);
        self.registers.cr1.modify(CR1::CEN::SET);
    }

    pub fn get_time_cnt(&self) -> u32 {
        self.registers.cnt.get()
    }

    pub fn set_timer_cnt(&self, value: u32) {
        self.registers.cnt.set(value);
    }
}

impl Time for Tim2<'_> {
    type Frequency = Freq16KHz;
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.get())
    }
}

impl<'a> Counter<'a> for Tim2<'a> {
    fn set_overflow_client(&self, _client: &'a dyn OverflowClient) {}

    fn start(&self) -> Result<(), ErrorCode> {
        self.start();
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.registers.cr1.modify(CR1::CEN::CLEAR);
        self.registers.sr.modify(SR::CC1IF::CLEAR);
        self.registers.dier.modify(DIER::CC1IE::CLEAR);
        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        self.registers.cnt.set(0);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.registers.cr1.is_set(CR1::CEN)
    }
}

impl<'a> Alarm<'a> for Tim2<'a> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();
        if !now.within_range(reference, expire) {
            expire = now;
        }

        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        let _ = self.disarm();
        self.registers.ccr1.set(expire.into_u32());
        self.registers.dier.modify(DIER::CC1IE::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.ccr1.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        unsafe {
            with_interrupts_disabled(|| {
                // Disable counter
                self.registers.dier.modify(DIER::CC1IE::CLEAR);
                self.registers.sr.modify(SR::CC1IF::CLEAR);
                cortexm33::nvic::Nvic::new(self.irqn).clear_pending();
            });
        }
        Ok(())
    }

    fn is_armed(&self) -> bool {
        // If counter is enabled, then CC1IE is set
        self.registers.dier.is_set(DIER::CC1IE)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1)
    }
}

struct Tim2Clock<'a>(phclk::PeripheralClock<'a>);

impl ClockInterface for Tim2Clock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
