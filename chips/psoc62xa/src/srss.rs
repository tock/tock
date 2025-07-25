// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    SrssRegisters {
        (0x000 => pwr_ctl: ReadWrite<u32, PWR_CTL::Register>),
        (0x004 => pwr_hibernate: ReadWrite<u32, PWR_HIBERNATE::Register>),
        (0x008 => pwr_lvd_ctl: ReadWrite<u32, PWR_LVD_CTL::Register>),
        (0x00C => _reserved0),
        (0x014 => pwr_buck_ctl: ReadWrite<u32, PWR_BUCK_CTL::Register>),
        (0x018 => _reserved17),
        (0x01C => pwr_lvd_status: ReadOnly<u32>),
        (0x020 => _reserved1),
        (0x080 => pwr_hib_data_0: ReadWrite<u32>),
        (0x084 => _reserved2),
        (0x180 => wdt_ctl: ReadWrite<u32, WDT_CTL::Register>),
        (0x184 => wdt_cnt: ReadWrite<u32>),
        (0x188 => wdt_match: ReadWrite<u32, WDT_MATCH::Register>),
        (0x18C => _reserved3),
        (0x300 => clk_dsi_select_0: ReadWrite<u32>),
        (0x304 => clk_dsi_select_1: ReadWrite<u32>),
        (0x308 => clk_dsi_select_2: ReadWrite<u32>),
        (0x30C => clk_dsi_select_3: ReadWrite<u32>),
        (0x310 => clk_dsi_select_4: ReadWrite<u32>),
        (0x314 => clk_dsi_select_5: ReadWrite<u32>),
        (0x318 => _reserved18),
        (0x340 => clk_path_select_0: ReadWrite<u32, CLK_PATH_SELECT0::Register>),
        (0x344 => clk_path_select_1: ReadWrite<u32, CLK_PATH_SELECT1::Register>),
        (0x348 => clk_path_select_2: ReadWrite<u32, CLK_PATH_SELECT2::Register>),
        (0x34C => clk_path_select_3: ReadWrite<u32, CLK_PATH_SELECT3::Register>),
        (0x350 => clk_path_select_4: ReadWrite<u32, CLK_PATH_SELECT4::Register>),
        (0x354 => clk_path_select_5: ReadWrite<u32, CLK_PATH_SELECT5::Register>),
        (0x358 => _reserved19),
        (0x380 => clk_root_select_0: ReadWrite<u32, CLK_ROOT_SELECT0::Register>),
        (0x384 => clk_root_select_1: ReadWrite<u32, CLK_ROOT_SELECT1::Register>),
        (0x388 => clk_root_select_2: ReadWrite<u32, CLK_ROOT_SELECT2::Register>),
        (0x38C => clk_root_select_3: ReadWrite<u32, CLK_ROOT_SELECT3::Register>),
        (0x390 => clk_root_select_4: ReadWrite<u32, CLK_ROOT_SELECT4::Register>),
        (0x394 => clk_root_select_5: ReadWrite<u32, CLK_ROOT_SELECT5::Register>),
        (0x398 => _reserved4),
        (0x500 => clk_select: ReadWrite<u32, CLK_SELECT::Register>),
        (0x504 => clk_timer_ctl: ReadWrite<u32, CLK_TIMER_CTL::Register>),
        (0x508 => _reserved5),
        (0x50C => clk_ilo_config: ReadWrite<u32, CLK_ILO_CONFIG::Register>),
        (0x510 => clk_imo_config: ReadWrite<u32>),
        (0x514 => clk_output_fast: ReadWrite<u32, CLK_OUTPUT_FAST::Register>),
        (0x518 => clk_output_slow: ReadWrite<u32, CLK_OUTPUT_SLOW::Register>),
        (0x51C => clk_cal_cnt1: ReadWrite<u32, CLK_CAL_CNT1::Register>),
        (0x520 => clk_cal_cnt2: ReadOnly<u32>),
        (0x524 => _reserved6),
        (0x52C => clk_eco_config: ReadWrite<u32, CLK_ECO_CONFIG::Register>),
        (0x530 => clk_eco_status: ReadOnly<u32, CLK_ECO_STATUS::Register>),
        (0x534 => _reserved7),
        (0x53C => clk_pilo_config: ReadWrite<u32, CLK_PILO_CONFIG::Register>),
        (0x540 => _reserved8),
        (0x580 => clk_fll_config: ReadWrite<u32, CLK_FLL_CONFIG::Register>),
        (0x584 => clk_fll_config2: ReadWrite<u32, CLK_FLL_CONFIG2::Register>),
        (0x588 => clk_fll_config3: ReadWrite<u32, CLK_FLL_CONFIG3::Register>),
        (0x58C => clk_fll_config4: ReadWrite<u32, CLK_FLL_CONFIG4::Register>),
        (0x590 => clk_fll_status: ReadWrite<u32, CLK_FLL_STATUS::Register>),
        (0x594 => _reserved9),
        (0x600 => clk_pll_config_0: ReadWrite<u32, CLK_PLL_CONFIG0::Register>),
        (0x604 => clk_pll_config_1: ReadWrite<u32, CLK_PLL_CONFIG1::Register>),
        (0x608 => _reserved10),
        (0x640 => clk_pll_status_0: ReadWrite<u32, CLK_PLL_STATUS0::Register>),
        (0x644 => clk_pll_status_1: ReadWrite<u32, CLK_PLL_STATUS1::Register>),
        (0x648 => _reserved11),
        (0x700 => srss_intr: ReadWrite<u32, SRSS_INTR::Register>),
        (0x704 => srss_intr_set: ReadWrite<u32, SRSS_INTR_SET::Register>),
        (0x708 => srss_intr_mask: ReadWrite<u32, SRSS_INTR_MASK::Register>),
        (0x70C => srss_intr_masked: ReadOnly<u32, SRSS_INTR_MASKED::Register>),
        (0x710 => srss_intr_cfg: ReadWrite<u32>),
        (0x714 => _reserved12),
        (0x800 => res_cause: ReadWrite<u32, RES_CAUSE::Register>),
        (0x804 => res_cause2: ReadWrite<u32, RES_CAUSE2::Register>),
        (0x808 => _reserved13),
        (0x7F00 => pwr_trim_ref_ctl: ReadWrite<u32, PWR_TRIM_REF_CTL::Register>),
        (0x7F04 => pwr_trim_bodovp_ctl: ReadWrite<u32, PWR_TRIM_BODOVP_CTL::Register>),
        (0x7F08 => clk_trim_cco_ctl: ReadWrite<u32, CLK_TRIM_CCO_CTL::Register>),
        (0x7F0C => clk_trim_cco_ctl2: ReadWrite<u32, CLK_TRIM_CCO_CTL2::Register>),
        (0x7F10 => _reserved14),
        (0x7F30 => pwr_trim_wake_ctl: ReadWrite<u32>),
        (0x7F34 => _reserved15),
        (0xFF10 => pwr_trim_lvd_ctl: ReadWrite<u32, PWR_TRIM_LVD_CTL::Register>),
        (0xFF14 => _reserved16),
        (0xFF18 => clk_trim_ilo_ctl: ReadWrite<u32>),
        (0xFF1C => pwr_trim_pwrsys_ctl: ReadWrite<u32, PWR_TRIM_PWRSYS_CTL::Register>),
        (0xFF20 => clk_trim_eco_ctl: ReadWrite<u32, CLK_TRIM_ECO_CTL::Register>),
        (0xFF24 => clk_trim_pilo_ctl: ReadWrite<u32, CLK_TRIM_PILO_CTL::Register>),
        (0xFF28 => clk_trim_pilo_ctl2: ReadWrite<u32, CLK_TRIM_PILO_CTL2::Register>),
        (0xFF2C => clk_trim_pilo_ctl3: ReadWrite<u32>),
        (0xFF30 => @END),
    }
}
register_bitfields![u32,
PWR_CTL [
    POWER_MODE OFFSET(0) NUMBITS(2) [
        SystemIsResetting = 0,
        AtLeastOneCPUIsRunning = 1,
        NoCPUsAreRunningPeripheralsMayBeRunning = 2,
        DEEPSLEEP = 3
    ],
    DEBUG_SESSION OFFSET(4) NUMBITS(1) [
        NoDebugSessionActive = 0,
        DebugSessionIsActivePowerModesBehaveDifferentlyToKeepTheDebugSessionActive = 1
    ],
    LPM_READY OFFSET(5) NUMBITS(1) [],
    IREF_LPMODE OFFSET(18) NUMBITS(1) [],
    VREFBUF_OK OFFSET(19) NUMBITS(1) [],
    DPSLP_REG_DIS OFFSET(20) NUMBITS(1) [],
    RET_REG_DIS OFFSET(21) NUMBITS(1) [],
    NWELL_REG_DIS OFFSET(22) NUMBITS(1) [],
    LINREG_DIS OFFSET(23) NUMBITS(1) [],
    LINREG_LPMODE OFFSET(24) NUMBITS(1) [],
    PORBOD_LPMODE OFFSET(25) NUMBITS(1) [],
    BGREF_LPMODE OFFSET(26) NUMBITS(1) [],
    PLL_LS_BYPASS OFFSET(27) NUMBITS(1) [],
    VREFBUF_LPMODE OFFSET(28) NUMBITS(1) [],
    VREFBUF_DIS OFFSET(29) NUMBITS(1) [],
    ACT_REF_DIS OFFSET(30) NUMBITS(1) [],
    ACT_REF_OK OFFSET(31) NUMBITS(1) []
],
PWR_HIBERNATE [
    TOKEN OFFSET(0) NUMBITS(8) [],
    UNLOCK OFFSET(8) NUMBITS(8) [],
    FREEZE OFFSET(17) NUMBITS(1) [],
    MASK_HIBALARM OFFSET(18) NUMBITS(1) [],
    MASK_HIBWDT OFFSET(19) NUMBITS(1) [],
    POLARITY_HIBPIN OFFSET(20) NUMBITS(4) [],
    MASK_HIBPIN OFFSET(24) NUMBITS(4) [],
    HIBERNATE_DISABLE OFFSET(30) NUMBITS(1) [],
    HIBERNATE OFFSET(31) NUMBITS(1) []
],
PWR_LVD_CTL [
    HVLVD1_TRIPSEL OFFSET(0) NUMBITS(4) [],
    HVLVD1_SRCSEL OFFSET(4) NUMBITS(3) [
        SelectVDDD = 0,
        SelectAMUXBUSAVDDDBranch = 1,
        NA = 2,
        SelectAMUXBUSBVDDDBranch = 4
    ],
    HVLVD1_EN OFFSET(7) NUMBITS(1) []
],
PWR_BUCK_CTL [
    BUCK_OUT1_SEL OFFSET(0) NUMBITS(3) [],
    BUCK_EN OFFSET(30) NUMBITS(1) [],
    BUCK_OUT1_EN OFFSET(31) NUMBITS(1) []
],
PWR_BUCK_CTL2 [
    BUCK_OUT2_SEL OFFSET(0) NUMBITS(3) [],
    BUCK_OUT2_HW_SEL OFFSET(30) NUMBITS(1) [],
    BUCK_OUT2_EN OFFSET(31) NUMBITS(1) []
],
PWR_LVD_STATUS [
    HVLVD1_OK OFFSET(0) NUMBITS(1) []
],
WDT_CTL [
    WDT_EN OFFSET(0) NUMBITS(1) [],
    WDT_LOCK OFFSET(30) NUMBITS(2) [
        NoEffect = 0,
        ClearsBit0 = 1,
        ClearsBit1 = 2,
        SetsBothBits0And1 = 3
    ]
],
WDT_CNT [
    COUNTER OFFSET(0) NUMBITS(16) []
],
WDT_MATCH [
    MATCH OFFSET(0) NUMBITS(16) [],
    IGNORE_BITS OFFSET(16) NUMBITS(4) []
],
CLK_SELECT [
    LFCLK_SEL OFFSET(0) NUMBITS(2) [
        ILOInternalLowSpeedOscillator = 0,
        WCO = 1,
        ALTLFAlternateLowFrequencyClockCapabilityIsProductSpecific = 2,
        PILO = 3
    ],
    PUMP_SEL OFFSET(8) NUMBITS(4) [],
    PUMP_DIV OFFSET(12) NUMBITS(3) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3,
        DivideSelectedClockSourceBy16 = 4
    ],
    PUMP_ENABLE OFFSET(15) NUMBITS(1) []
],
CLK_TIMER_CTL [
    TIMER_SEL OFFSET(0) NUMBITS(1) [
        IMOInternalMainOscillator = 0,
        SelectTheOutputOfThePredividerConfiguredByTIMER_HF0_DIV = 1
    ],
    TIMER_HF0_DIV OFFSET(8) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividingOrCorrectingDutyCycle = 0,
        DivideHFCLK0By2 = 1,
        DivideHFCLK0By4 = 2,
        DivideHFCLK0By8 = 3
    ],
    TIMER_DIV OFFSET(16) NUMBITS(8) [],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ILO_CONFIG [
    ILO_BACKUP OFFSET(0) NUMBITS(1) [],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_IMO_CONFIG [
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_OUTPUT_FAST [
    FAST_SEL0 OFFSET(0) NUMBITS(4) [
        NC = 0,
        ExternalCrystalOscillatorECO = 1,
        ExternalClockInputEXTCLK = 2,
        AlternateHighFrequencyALTHFClockInputToSRSS = 3,
        TIMERCLK = 4,
        SelectsTheClockPathChosenByPATH_SEL0Field = 5,
        SelectsTheOutputOfTheHFCLK_SEL0Mux = 6,
        SelectsTheOutputOfCLK_OUTPUT_SLOWSLOW_SEL0 = 7
    ],
    PATH_SEL0 OFFSET(4) NUMBITS(4) [],
    HFCLK_SEL0 OFFSET(8) NUMBITS(4) [],
    FAST_SEL1 OFFSET(16) NUMBITS(4) [
        NC = 0,
        ExternalCrystalOscillatorECO = 1,
        ExternalClockInputEXTCLK = 2,
        AlternateHighFrequencyALTHFClockInputToSRSS = 3,
        TIMERCLK = 4,
        SelectsTheClockPathChosenByPATH_SEL1Field = 5,
        SelectsTheOutputOfTheHFCLK_SEL1Mux = 6,
        SelectsTheOutputOfCLK_OUTPUT_SLOWSLOW_SEL1 = 7
    ],
    PATH_SEL1 OFFSET(20) NUMBITS(4) [],
    HFCLK_SEL1 OFFSET(24) NUMBITS(4) []
],
CLK_OUTPUT_SLOW [
    SLOW_SEL0 OFFSET(0) NUMBITS(4) [
        DisabledOutputIs0ForPowerSavingsClocksAreBlockedBeforeEnteringAnyMuxes = 0,
        InternalLowSpeedOscillatorILO = 1,
        WatchCrystalOscillatorWCO = 2,
        RootOfTheBackupDomainClockTreeBAK = 3,
        AlternateLowFrequencyClockInputToSRSSALTLF = 4,
        RootOfTheLowSpeedClockTreeLFCLK = 5,
        IMO = 6,
        SLPCTRL = 7,
        PrecisionInternalLowSpeedOscillatorPILO = 8
    ],
    SLOW_SEL1 OFFSET(4) NUMBITS(4) [
        DisabledOutputIs0ForPowerSavingsClocksAreBlockedBeforeEnteringAnyMuxes = 0,
        InternalLowSpeedOscillatorILO = 1,
        WatchCrystalOscillatorWCO = 2,
        RootOfTheBackupDomainClockTreeBAK = 3,
        AlternateLowFrequencyClockInputToSRSSALTLF = 4,
        RootOfTheLowSpeedClockTreeLFCLK = 5,
        IMO = 6,
        SLPCTRL = 7,
        PrecisionInternalLowSpeedOscillatorPILO = 8
    ]
],
CLK_CAL_CNT1 [
    CAL_COUNTER1 OFFSET(0) NUMBITS(24) [],
    CAL_COUNTER_DONE OFFSET(31) NUMBITS(1) []
],
CLK_CAL_CNT2 [
    CAL_COUNTER2 OFFSET(0) NUMBITS(24) []
],
CLK_ECO_CONFIG [
    AGC_EN OFFSET(1) NUMBITS(1) [],
    ECO_EN OFFSET(31) NUMBITS(1) []
],
CLK_ECO_STATUS [
    ECO_OK OFFSET(0) NUMBITS(1) [],
    ECO_READY OFFSET(1) NUMBITS(1) []
],
CLK_PILO_CONFIG [
    PILO_FFREQ OFFSET(0) NUMBITS(10) [],
    PILO_CLK_EN OFFSET(29) NUMBITS(1) [],
    PILO_RESET_N OFFSET(30) NUMBITS(1) [],
    PILO_EN OFFSET(31) NUMBITS(1) []
],
CLK_FLL_CONFIG [
    FLL_MULT OFFSET(0) NUMBITS(18) [],
    FLL_OUTPUT_DIV OFFSET(24) NUMBITS(1) [],
    FLL_ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_FLL_CONFIG2 [
    FLL_REF_DIV OFFSET(0) NUMBITS(13) [],
    LOCK_TOL OFFSET(16) NUMBITS(9) []
],
CLK_FLL_CONFIG3 [
    FLL_LF_IGAIN OFFSET(0) NUMBITS(4) [],
    FLL_LF_PGAIN OFFSET(4) NUMBITS(4) [],
    SETTLING_COUNT OFFSET(8) NUMBITS(13) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        NA = 0,
        SelectFLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectFLLOutputIgnoresLockIndicator = 3
    ]
],
CLK_FLL_CONFIG4 [
    CCO_LIMIT OFFSET(0) NUMBITS(8) [],
    CCO_RANGE OFFSET(8) NUMBITS(3) [
        TargetFrequencyIsInRange4864MHz = 0,
        TargetFrequencyIsInRange6485MHz = 1,
        TargetFrequencyIsInRange85113MHz = 2,
        TargetFrequencyIsInRange113150MHz = 3,
        TargetFrequencyIsInRange150200MHz = 4
    ],
    CCO_FREQ OFFSET(16) NUMBITS(9) [],
    CCO_HW_UPDATE_DIS OFFSET(30) NUMBITS(1) [],
    CCO_ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_FLL_STATUS [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) [],
    CCO_READY OFFSET(2) NUMBITS(1) []
],
SRSS_INTR [
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_SET [
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_MASK [
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_MASKED [
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_CFG [
    HVLVD1_EDGE_SEL OFFSET(0) NUMBITS(2) [
        Disabled = 0,
        RisingEdge = 1,
        FallingEdge = 2,
        BothRisingAndFallingEdges = 3
    ]
],
RES_CAUSE [
    RESET_WDT OFFSET(0) NUMBITS(1) [],
    RESET_ACT_FAULT OFFSET(1) NUMBITS(1) [],
    RESET_DPSLP_FAULT OFFSET(2) NUMBITS(1) [],
    RESET_CSV_WCO_LOSS OFFSET(3) NUMBITS(1) [],
    RESET_SOFT OFFSET(4) NUMBITS(1) [],
    RESET_MCWDT0 OFFSET(5) NUMBITS(1) [],
    RESET_MCWDT1 OFFSET(6) NUMBITS(1) [],
    RESET_MCWDT2 OFFSET(7) NUMBITS(1) [],
    RESET_MCWDT3 OFFSET(8) NUMBITS(1) []
],
RES_CAUSE2 [
    RESET_CSV_HF_LOSS OFFSET(0) NUMBITS(16) [],
    RESET_CSV_HF_FREQ OFFSET(16) NUMBITS(16) []
],
PWR_TRIM_REF_CTL [
    ACT_REF_TCTRIM OFFSET(0) NUMBITS(4) [],
    ACT_REF_ITRIM OFFSET(4) NUMBITS(4) [],
    ACT_REF_ABSTRIM OFFSET(8) NUMBITS(5) [],
    ACT_REF_IBOOST OFFSET(14) NUMBITS(1) [],
    DPSLP_REF_TCTRIM OFFSET(16) NUMBITS(4) [],
    DPSLP_REF_ABSTRIM OFFSET(20) NUMBITS(5) [],
    DPSLP_REF_ITRIM OFFSET(28) NUMBITS(4) []
],
PWR_TRIM_BODOVP_CTL [
    HVPORBOD_TRIPSEL OFFSET(0) NUMBITS(3) [],
    HVPORBOD_OFSTRIM OFFSET(4) NUMBITS(3) [],
    HVPORBOD_ITRIM OFFSET(7) NUMBITS(3) [],
    LVPORBOD_TRIPSEL OFFSET(10) NUMBITS(3) [],
    LVPORBOD_OFSTRIM OFFSET(14) NUMBITS(3) [],
    LVPORBOD_ITRIM OFFSET(17) NUMBITS(3) []
],
CLK_TRIM_CCO_CTL [
    CCO_RCSTRIM OFFSET(0) NUMBITS(6) [],
    CCO_STABLE_CNT OFFSET(24) NUMBITS(6) [],
    ENABLE_CNT OFFSET(31) NUMBITS(1) []
],
CLK_TRIM_CCO_CTL2 [
    CCO_FCTRIM1 OFFSET(0) NUMBITS(5) [],
    CCO_FCTRIM2 OFFSET(5) NUMBITS(5) [],
    CCO_FCTRIM3 OFFSET(10) NUMBITS(5) [],
    CCO_FCTRIM4 OFFSET(15) NUMBITS(5) [],
    CCO_FCTRIM5 OFFSET(20) NUMBITS(5) []
],
PWR_TRIM_WAKE_CTL [
    WAKE_DELAY OFFSET(0) NUMBITS(8) []
],
PWR_TRIM_LVD_CTL [
    HVLVD1_OFSTRIM OFFSET(0) NUMBITS(3) [],
    HVLVD1_ITRIM OFFSET(4) NUMBITS(3) []
],
CLK_TRIM_ILO_CTL [
    ILO_FTRIM OFFSET(0) NUMBITS(6) []
],
PWR_TRIM_PWRSYS_CTL [
    ACT_REG_TRIM OFFSET(0) NUMBITS(5) [],
    ACT_REG_BOOST OFFSET(30) NUMBITS(2) []
],
CLK_TRIM_ECO_CTL [
    WDTRIM OFFSET(0) NUMBITS(3) [],
    ATRIM OFFSET(4) NUMBITS(4) [],
    FTRIM OFFSET(8) NUMBITS(2) [],
    RTRIM OFFSET(10) NUMBITS(2) [],
    GTRIM OFFSET(12) NUMBITS(2) [],
    ITRIM OFFSET(16) NUMBITS(6) []
],
CLK_TRIM_PILO_CTL [
    PILO_CFREQ OFFSET(0) NUMBITS(6) [],
    PILO_OSC_TRIM OFFSET(12) NUMBITS(3) [],
    PILO_COMP_TRIM OFFSET(16) NUMBITS(2) [],
    PILO_NBIAS_TRIM OFFSET(18) NUMBITS(2) [],
    PILO_RES_TRIM OFFSET(20) NUMBITS(5) [],
    PILO_ISLOPE_TRIM OFFSET(26) NUMBITS(2) [],
    PILO_VTDIFF_TRIM OFFSET(28) NUMBITS(3) []
],
CLK_TRIM_PILO_CTL2 [
    PILO_VREF_TRIM OFFSET(0) NUMBITS(8) [],
    PILO_IREFBM_TRIM OFFSET(8) NUMBITS(5) [],
    PILO_IREF_TRIM OFFSET(16) NUMBITS(8) []
],
CLK_TRIM_PILO_CTL3 [
    PILO_ENGOPT OFFSET(0) NUMBITS(16) []
],
PWR_HIB_DATA0 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA1 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA2 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA3 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA4 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA5 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA6 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA7 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA8 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA9 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA10 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA11 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA12 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA13 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA14 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA15 [
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
CLK_DSI_SELECT0 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT1 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT2 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT3 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT4 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT5 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT6 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT7 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT8 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT9 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT10 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT11 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT12 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT13 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT14 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_DSI_SELECT15 [
    DSI_MUX OFFSET(0) NUMBITS(5) [
        DSI0Dsi_out0 = 0,
        DSI1Dsi_out1 = 1,
        DSI2Dsi_out2 = 2,
        DSI3Dsi_out3 = 3,
        DSI4Dsi_out4 = 4,
        DSI5Dsi_out5 = 5,
        DSI6Dsi_out6 = 6,
        DSI7Dsi_out7 = 7,
        DSI8Dsi_out8 = 8,
        DSI9Dsi_out9 = 9,
        DSI10Dsi_out10 = 10,
        DSI11Dsi_out11 = 11,
        DSI12Dsi_out12 = 12,
        DSI13Dsi_out13 = 13,
        DSI14Dsi_out14 = 14,
        DSI15Dsi_out15 = 15,
        ILOInternalLowSpeedOscillator = 16,
        WCOWatchCrystalOscillator = 17,
        ALTLFAlternateLowFrequencyClock = 18,
        PILOPrecisionInternalLowSpeedOscillator = 19
    ]
],
CLK_PATH_SELECT0 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT1 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT2 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT3 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT4 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT5 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT6 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT7 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT8 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT9 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT10 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT11 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT12 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT13 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT14 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_PATH_SELECT15 [
    PATH_MUX OFFSET(0) NUMBITS(3) [
        IMOInternalRCOscillator = 0,
        EXTCLKExternalClockPin = 1,
        ECOExternalCrystalOscillator = 2,
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        DSI_MUX = 4
    ]
],
CLK_ROOT_SELECT0 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT1 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT2 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT3 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT4 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT5 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT6 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT7 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT8 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT9 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT10 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT11 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT12 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT13 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT14 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ROOT_SELECT15 [
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        SelectPATH0CanBeConfiguredForFLL = 0,
        SelectPATH1CanBeConfiguredForPLL0IfAvailableInTheProduct = 1,
        SelectPATH2CanBeConfiguredForPLL1IfAvailableInTheProduct = 2,
        SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct = 3,
        SelectPATH4CanBeConfiguredForPLL3IfAvailableInTheProduct = 4,
        SelectPATH5CanBeConfiguredForPLL4IfAvailableInTheProduct = 5,
        SelectPATH6CanBeConfiguredForPLL5IfAvailableInTheProduct = 6,
        SelectPATH7CanBeConfiguredForPLL6IfAvailableInTheProduct = 7,
        SelectPATH8CanBeConfiguredForPLL7IfAvailableInTheProduct = 8,
        SelectPATH9CanBeConfiguredForPLL8IfAvailableInTheProduct = 9,
        SelectPATH10CanBeConfiguredForPLL9IfAvailableInTheProduct = 10,
        SelectPATH11CanBeConfiguredForPLL10IfAvailableInTheProduct = 11,
        SelectPATH12CanBeConfiguredForPLL11IfAvailableInTheProduct = 12,
        SelectPATH13CanBeConfiguredForPLL12IfAvailableInTheProduct = 13,
        SelectPATH14CanBeConfiguredForPLL13IfAvailableInTheProduct = 14,
        SelectPATH15CanBeConfiguredForPLL14IfAvailableInTheProduct = 15
    ],
    ROOT_DIV OFFSET(4) NUMBITS(2) [
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        DivideSelectedClockSourceBy2 = 1,
        DivideSelectedClockSourceBy4 = 2,
        DivideSelectedClockSourceBy8 = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG0 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG1 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG2 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG3 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG4 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG5 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG6 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG7 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG8 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG9 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG10 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG11 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG12 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG13 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG14 [
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        AUTO = 0,
        SameAsAUTO = 1,
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        SelectPLLOutputIgnoresLockIndicator = 3
    ],
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_STATUS0 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS1 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS2 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS3 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS4 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS5 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS6 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS7 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS8 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS9 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS10 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS11 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS12 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS13 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CLK_PLL_STATUS14 [
    LOCKED OFFSET(0) NUMBITS(1) [],
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
]
];
const SRSS_BASE: StaticRef<SrssRegisters> =
    unsafe { StaticRef::new(0x40260000 as *const SrssRegisters) };

pub struct Srss {
    registers: StaticRef<SrssRegisters>,
}

impl Srss {
    pub const fn new() -> Srss {
        Srss {
            registers: SRSS_BASE,
        }
    }

    pub fn init_clock(&self) {
        self.registers
            .clk_path_select_3
            .modify(CLK_PATH_SELECT3::PATH_MUX::IMOInternalRCOscillator);

        self.registers.clk_root_select_0.modify(CLK_ROOT_SELECT0::ENABLE::SET + CLK_ROOT_SELECT0::ROOT_MUX::SelectPATH3CanBeConfiguredForPLL2IfAvailableInTheProduct + CLK_ROOT_SELECT0::ROOT_DIV::TransparentModeFeedThroughSelectedClockSourceWODividing);
    }
}
