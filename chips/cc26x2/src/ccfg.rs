use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

use crate::memory_map::CCFG_BASE;

pub const OFFSET: usize = 0x1FA8;

pub const REG: StaticRef<Registers> =
    unsafe { StaticRef::new((CCFG_BASE + OFFSET) as *const Registers) };

//Table 11-1. CC26_CCFG_MMAP1 Registers

// Offset  Acronym                 Register Name                   Section
// 1FA8h   EXT_LF_CLK              Extern LF clock configuration   Section 11.2.1.1
// 1FACh   MODE_CONF_1             Mode Configuration 1            Section 11.2.1.2
// 1FB0h   SIZE_AND_DIS_FLAGS      CCFG Size and Disable Flags     Section 11.2.1.3
// 1FB4h   MODE_CONF               Mode Configuration 0            Section 11.2.1.4
// 1FB8h   VOLT_LOAD_0             Voltage Load 0                  Section 11.2.1.5
// 1FBCh   VOLT_LOAD_1             Voltage Load 1                  Section 11.2.1.6
// 1FC0h   RTC_OFFSET              Real Time Clock Offset          Section 11.2.1.7
// 1FC4h   FREQ_OFFSET             Frequency Offset                Section 11.2.1.8
// 1FC8h   IEEE_MAC_0              IEEE MAC Address 0              Section 11.2.1.9
// 1FCCh   IEEE_MAC_1              IEEE MAC Address 1              Section 11.2.1.10
// 1FD0h   IEEE_BLE_0              IEEE BLE Address 0              Section 11.2.1.11
// 1FD4h   IEEE_BLE_1              IEEE BLE Address 1              Section 11.2.1.12
// 1FD8h   BL_CONFIG               Bootloader Configuration        Section 11.2.1.13
// 1FDCh   ERASE_CONF              Erase Configuration             Section 11.2.1.14
// 1FE0h   CCFG_TI_OPTIONS         TI Options                      Section 11.2.1.15
// 1FE4h   CCFG_TAP_DAP_0          Test Access Points Enable 0     Section 11.2.1.16
// 1FE8h   CCFG_TAP_DAP_1          Test Access Points Enable 1     Section 11.2.1.17
// 1FECh   IMAGE_VALID_CONF        Image Valid                     Section 11.2.1.18
// 1FF0h   CCFG_PROT_31_0          Protect Sectors 0-31            Section 11.2.1.19
// 1FF4h   CCFG_PROT_63_32         Protect Sectors 32-63           Section 11.2.1.20
// 1FF8h   CCFG_PROT_95_64         Protect Sectors 64-95           Section 11.2.1.21
// 1FFCh   CCFG_PROT_127_96        Protect Sectors 96-127          Section 11.2.1.22

#[repr(C)]
pub struct Registers {
    //_offset: [ReadOnly<u8>; 0x1FA8],
    pub ext_lf_clk: ReadWrite<u32, ExtLfClk::Register>,
    mode_conf1: ReadWrite<u32, ModeConf1::Register>,
    size_and_dis_flags: ReadWrite<u32, SizeAndDisFlags::Register>,
    mode_conf0: ReadWrite<u32, ModeConf0::Register>,
    _volt_load0: ReadOnly<u32>,  //unimplemented by TI
    _volt_load1: ReadOnly<u32>,  //unimplemented by TI
    _rtc_offset: ReadOnly<u32>,  //unimplemented by TI
    _freq_offset: ReadOnly<u32>, //unimplemented by TI
    iee_mac0: ReadWrite<u32>,
    iee_mac1: ReadWrite<u32>,
    iee_ble0: ReadWrite<u32>,
    iee_ble1: ReadWrite<u32>,
    bl_config: ReadWrite<u32, BlConfig::Register>,
    erase_config: ReadWrite<u32, EraseConfig::Register>,
    ti_options: ReadOnly<u32, TiOptions::Register>,
    tap_dap0: ReadOnly<u32, TapDap0::Register>,
    tap_dap1: ReadOnly<u32, TapDap1::Register>,
    image_valid: ReadOnly<u32, ImageValid::Register>,
    ccfg_prot_31_0: ReadWrite<u32>,
    ccfg_prot_63_32: ReadWrite<u32>,
    ccfg_prot_95_64: ReadWrite<u32>,
    ccfg_prot_127_96: ReadWrite<u32>,
}

// a reduced version of Registers for constructing
pub struct RegisterInitializer {
    pub ext_lf_clk: ReadWrite<u32, ExtLfClk::Register>,
    pub mode_conf0: ReadWrite<u32, ModeConf0::Register>,
    pub mode_conf1: ReadWrite<u32, ModeConf1::Register>,
    pub bl_config: ReadWrite<u32, BlConfig::Register>,
}

register_bitfields![
    u8,
    ExtLfClk [
        // Unsigned value pin selection
        DIO OFFSET(24) NUMBITS(8) [],
        // Unsignd integer, defines input freq of ext_clk
        // EXT_LF_CLK.RTC_INCREMENT = 2^38/InputClockFrequency in Hertz
        // is written to AON_RTC:SUBSECINC.VALUEINC
        // e.g.: RTC_INCREMENT=0x800000 for InputClockFrequency=32768 Hz)
        RTC_INCREMENT OFFSET(0) NUMBITS(24) []
    ],
    ModeConf1 [
        // "The DriverLib function SysCtrl_DCDC_VoltageConditionalControl()
        // must be called regularly to apply this field
        ALT_DCDC_VMIN OFFSET(20) NUMBITS(4) [ // Voltage = 28 + ALT_DCDC_VMIN/16
            _1p75v = 0 // the zero example
        ],
        ALT_DCDC_DITHER_EN OFFSET(19) NUMBITS(1) [],
        // Assumes 10uH ext inductor
        // Peak current = 31 + (4 * ALT_DCDC_IPEAK)
        ALT_DCDC_IPEAK OFFSET(16) NUMBITS (3) [
            _31mA = 0, //min
            _47mA = 4,
            _59mA = 7  //max
        ],
        // Signed value for IBIAS_INIT
        // only applies if SIZE_AND_DIS_FLAGS.DIS_XOSC_OVR=0
        DELTA_IBIAS_INIT OFFSET(12) NUMBITS(4) [],
        // Signed value for IBIAS_OFFSET
        // only applies if SIZE_AND_DIS_FLAGS.DIS_XOSC_OVR=0
        DELTA_IBIAS_OFFSET OFFSET(8) NUMBITS(4) [],
        // Unsigned value of maximum XOSC startup time in 100us units
        XOSC_MAX_START OFFSET(0) NUMBITS(8) []
    ],
    SizeAndDisFlags [
        // in bytes
        SIZE_OF_CCFG OFFSET(16) NUMBITS(16) [],
        DIS_TCXO OFFSET(3) NUMBITS(1) [],
        DIS_GPRAM OFFSET(2) NUMBITS(1) [],
        DIS_ALT_DCDC_SETING OFFSET(1) NUMBITS(1) [],
        DIS_XOSC_OVR OFFSET(0) NUMBITS(1) []
    ],
    ModeConf0 [
        // signed dela value to apply to VDDR_TRIM_SLEEP target, minus one
        VDDR_TRIM_SLEEP_DELTA OFFSET(28) NUMBITS(4) [
            minus7 = 0x8,
            zero = 0xF,
            plus1 = 0x0,
            plus7 = 0x7
        ],
        // DC/DC during recharge in powerdown
        // "The DriverLib function SysCtrl_DCDC_VoltageConditionalControl()
        // must be called regularly to apply this field"
        DCDC_RECHARGE OFFSET(27) NUMBITS(1) [],
        // DC/DC in active mode
        // "The DriverLib function SysCtrl_DCDC_VoltageConditionalControl()
        // must be called regularly to apply this field"
        DCDC_ACTIVE OFFSET(26) NUMBITS(1) [],
        VDDS_BOD_LEVEL  OFFSET(24) NUMBITS(1) [
            _2p0v = 0,      // necessary for maximum PA output on 13xx
            _1p8or1p65v = 1 // 1.65V for external regulator mode
        ],
        SCLK_LF_OPTION OFFSET(23) NUMBITS(2) [
            HF_XOSC = 0x0,      // derived from 24MHz XOSC
            EXT_LF_CLK = 0x1,   // DIO inut defined in EXT_LF_CLK
            EXT_LF_XOSC = 0x2,
            RCOSC = 0x3
        ],
        VDDR_TRIM_SLEEP_TC OFFSET(21) NUMBITS(1) [
            DISABLE = 0x1,
            ENABLED = 0x0 // must be done every time standby mode is entered
        ],
        XOSC_CAP_MOD OFFSET(17) NUMBITS(1) [],
        XOSC_CAP_ARRAY_DELTA OFFSET(8) NUMBITS(8) [],
        VDDR_CAP OFFSET(0) NUMBITS(8) []
    ],
    BlConfig [
        BOOTLOADER   OFFSET(24) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A //could be any value
        ],
        // level tha pin must be held for BL backdoor
        BL_LEVEL OFFSET(16) NUMBITS(1) [
            LOW = 0x0,
            HIGH = 0x1
        ],
        // pin number for BL backdoor
        BL_PIN_NUMBER OFFSET(8) NUMBITS(8) [],
        BL_BACKDOOR OFFSET(0) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A //could be any value
        ]
    ],
    EraseConfig[
        CHIP_ERASE OFFSET(8) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ],
        BANK_ERASE OFFSET(0) NUMBITS(1) [
            DISABLE = 0x0,
            ENABLE = 0x1
        ]
    ],
    TiOptions [
        FA OFFSET(0) NUMBITS(8)[
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ]
    ],
    TapDap0 [
        CPU OFFSET(16) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ],
        PWR_PROF OFFSET(8) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ],
        TEST OFFSET(8) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ]
    ],
    TapDap1 [
        PBIST2 OFFSET(16) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ],
        PBIST1 OFFSET(8) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ],
        AON OFFSET(8) NUMBITS(8) [
            ENABLE = 0xC5,
            DISABLE = 0x3A
        ]
    ],
    ImageValid [
        // provides the boot address
        START OFFSET(0) NUMBITS(32) []
    ]

];
