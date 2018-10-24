use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;

use memory_map::AUX_ANAIF_BASE;

pub const REG: StaticRef<Registers> = unsafe { StaticRef::new(AUX_ANAIF_BASE as *const Registers) };

// TABLE 19-152     CC26_AUX_ANAIF_MMAP_AUX_ANAIF Registers

// OFFSET   Acronym         Register Name
// 10h      ADCCTL          ADC Control                     Section 19.8.8.1
// 14h      ADCFIFOSTAT     ADC FIFO Status                 Section 19.8.8.2
// 18h      ADCFIFO ADC     FIFO                            Section 19.8.8.3
// 1Ch      ADCTRIG ADC     Trigger                         Section 19.8.8.4
// 20h      ISRCCTL         Current Source Control          Section 19.8.8.5
// 30h      DACCTL DAC      Control                         Section 19.8.8.6
// 34h      LPMBIASCTL      Low Power Mode Bias Control     Section 19.8.8.7
// 38h      DACSMPLCTL      DAC Sample Control              Section 19.8.8.8
// 3Ch      DACSMPLCFG0     DAC Sample Configuration 0      Section 19.8.8.9
// 40h      DACSMPLCFG1     DAC Sample Configuration 1      Section 19.8.8.10
// 44h      DACVALUE        DAC Value                       Section 19.8.8.11
// 48h      DACSTAT         DAC Status                      Section 19.8.8.12

#[repr(C)]
pub struct Registers {
    _reserved0: [ReadOnly<u32>; 4],
    pub adc_ctl: ReadWrite<u32, AdcCtl::Register>,
    pub adc_fifo_status: ReadWrite<u32, AdcFifoStatus::Register>,
    pub adc_fifo: ReadOnly<u32, AdcFifo::Register>,
    pub adc_trigger: ReadWrite<u32, AdcTrigger::Register>,
    pub isrc_ctl: ReadWrite<u32, IsrcCtl::Register>,
    _reserved1: [ReadOnly<u32>; 4],
    dac_ctl: ReadWrite<u32, DacCtl::Register>,
    lpmb_ctl: ReadWrite<u32, LpmbCtl::Register>,
    dac_smpl_ctl: ReadWrite<u32, DacSmplCtl::Register>,
    dac_smple_cfg0: ReadWrite<u32, DacSmplCfg0::Register>,
    dac_smple_cfg1: ReadWrite<u32, DacSmplCfg1::Register>,
    dac_value: ReadWrite<u32, DacValue::Register>,
    dac_status: ReadWrite<u32, DacStatus::Register>,
}

register_bitfields! [
    u32,
    AdcCtl [
        START_POL OFFSET(14) NUMBITS(1) [
            RISING = 0x0,
            FALLING = 0b1
        ],
        START_SRC OFFSET(8) NUMBITS(6) [    // Select ADC trigger event source from the async AUX event
            AUXIO0 = 0x0,                   // Set START_SRC to NO_EVENT if you want to tuse ADCTRIG.START
            AUXIO1 = 0x1,
            AUXIO2 = 0x2,
            AUXIO3 = 0x3,
            AUXIO4 = 0x4,
            AUXIO5 = 0x5,
            AUXIO6 = 0x6,
            AUXIO7 = 0x7,
            AUXIO8 = 0x8,
            AUXIO9 = 0x9,
            AUXIO10 = 0xA,
            AUXIO11 = 0xB,
            AUXIO12 = 0xC,
            AUXIO13 = 0xD,
            AUXIO14 = 0xE,
            AUXIO15 = 0xF,
            AUXIO16 = 0x10,
            AUXIO17 = 0x11,
            AUXIO18 = 0x12,
            AUXIO19 = 0x13,
            AUXIO20 = 0x14,
            AUXIO21 = 0x15,
            AUXIO22 = 0x16,
            AUXIO23 = 0x17,
            AUXIO24 = 0x18,
            AUXIO25 = 0x19,
            AUXIO26 = 0x1A,
            AUXIO27 = 0x1B,
            AUXIO28 = 0x1C,
            AUXIO29 = 0x1D,
            AUXIO30 = 0x1E,
            AUXIO31 = 0x1F,
            MANUAL_EV = 0x20,
            AON_RTC_CH2 = 0x21,
            AON_RTC_CH2_DLY = 0x22,
            AON_RTC_4KHZ = 0x23,
            AON_BATMON_BAT_UPD = 0x24,
            AON_BATMON_TEMP_UPD = 0x25,
            SCLK_LF = 0x26,
            PWR_DWN = 0x27,
            MCU_ACTIVE = 0x28,
            VDDR_RECHARGE = 0x29,
            ACLK_REF = 0x2A,
            MCU_EV = 0x2B,
            AUX_COMPA = 0x2E,
            AUX_COMPB = 0x2F,
            AUX_TIMER2_EV0 = 0x30,
            AUX_TIMER2_EV1 = 0x31,
            AUX_TIMER2_EV2 = 0x32,
            AUX_TIMER2_EV3 = 0x33,
            AUX_TIMER2_PULSE = 0x34,
            AUX_TIMER1_EV = 0x35,
            AUX_TIMER0_EV = 0x36,
            AUX_TDC_DONE = 0x37,
            AUX_ISRC_RESET_N = 0x38,
            AUX_SMPH_AUTOTAKE_DONE = 0x3D,
            NO_EVENT = 0x3F
        ],
        CMD OFFSET(0) NUMBITS(2) [
            Disable = 0,
            Enable = 0x1,
            FlushFifo = 0x3 // you must send CMD EN or DIS after flush
        ]
    ],
    AdcFifoStatus [
        OVERFLOW    OFFSET(4) NUMBITS(1) [],
        UNDERFLOW   OFFSET(3) NUMBITS(1) [],
        FULL        OFFSET(2) NUMBITS(1) [],
        ALMOST_FULL OFFSET(1) NUMBITS(1) [], //3 samples or more = almost full
        EMPTY       OFFSET(0) NUMBITS(1) []
    ],
    AdcFifo [
        DATA OFFSET(0) NUMBITS(12) []
    ],
    AdcTrigger [
        START OFFSET(0) NUMBITS(1) [] // triggers a one-shot ADC
    ],
    IsrcCtl [
         RESET_N OFFSET(0) NUMBITS(1) []
    ],
    DacCtl [
        DAC_EN OFFSET(5) NUMBITS(1) [],
        DAC_BUFFER_EN OFFSET(4) NUMBITS(1) [],
        DAC_PRECHARGE_EN OFFSET(3) NUMBITS(1) [],
        DAC_VOUT_SEL OFFSET(0) NUMBITS(3) [
            None = 0x0,
            CompB_Ref = 0x1,
            CompA_Ref = 0x2,
            CompA_In = 0x4
        ]
    ],
    LpmbCtl [
        EN OFFSET(0) NUMBITS(1) []
    ],
    DacSmplCtl [
        EN OFFSET(0) NUMBITS(1)
    ],
    DacSmplCfg0 [
        CLKDIV OFFSET(0) NUMBITS(6) []
    ],
    DacSmplCfg1 [
        HIGH_PERIODS OFFSET(14) NUMBITS(1) [    // sample clock period is high for this many base periods
            Two = 0x0,
            Four = 0x1
        ],
        LOW_PERIODS OFFSET(12) NUMBITS(2) [    // sample clock period is low for this many base periods
            One = 0x0,
            Two = 0x1,
            Three = 0x2,
            Four = 0x3
        ],
        SETUP_COUNT OFFSET(8) NUMBITS(4) [      // setup count
            One = 0x0,
            Two = 0x1,
            Three = 0x2,
            Four = 0x3,
            Five = 0x4,
            Six = 0x5,
            Seven = 0x6,
            Eight = 0x7,
            Nine = 0x8,
            Ten = 0x9,
            Eleven = 0xA,
            Twelve = 0xB,
            Thirteen = 0xC,
            Fourteen = 0xD,
            Fifteen = 0xE,
            Sixteen = 0xF
        ],
        HOLD_INTERVAL OFFSET(0) NUMBITS(8) []   // hold interval
    ],
    DacValue [
        VAL OFFSET(0) NUMBITS(8) []
    ],
    DacStatus [
        SETUP_ACTIVE OFFSET(1) NUMBITS(1) [],
        HOLD_ACTIVE OFFSET(0) NUMBITS(1) []
    ]

];
