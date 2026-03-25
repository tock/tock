use kernel::utilities::registers::{
    interfaces::ReadWriteable, interfaces::Readable, register_bitfields, register_structs,
    ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    SrssRegisters {
        (0x000 => _reserved0),
        /// Clock DSI Select Register
        (0x100 => clk_dsi_select_0: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x104 => clk_dsi_select_1: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x108 => clk_dsi_select_2: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x10C => clk_dsi_select_3: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x110 => clk_dsi_select_4: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x114 => clk_dsi_select_5: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        /// Clock DSI Select Register
        (0x118 => clk_dsi_select_6: ReadWrite<u32, CLK_DSI_SELECT::Register>),
        (0x11C => _reserved1),
        /// Fast Clock Output Select Register
        (0x140 => clk_output_fast: ReadWrite<u32, CLK_OUTPUT_FAST::Register>),
        /// Slow Clock Output Select Register
        (0x144 => clk_output_slow: ReadWrite<u32, CLK_OUTPUT_SLOW::Register>),
        /// Clock Calibration Counter 1
        (0x148 => clk_cal_cnt1: ReadWrite<u32, CLK_CAL_CNT1::Register>),
        /// Clock Calibration Counter 2
        (0x14C => clk_cal_cnt2: ReadWrite<u32, CLK_CAL_CNT2::Register>),
        (0x150 => _reserved2),
        /// SRSS Interrupt Register
        (0x200 => srss_intr: ReadWrite<u32, SRSS_INTR::Register>),
        /// SRSS Interrupt Set Register
        (0x204 => srss_intr_set: ReadWrite<u32, SRSS_INTR_SET::Register>),
        /// SRSS Interrupt Mask Register
        (0x208 => srss_intr_mask: ReadWrite<u32, SRSS_INTR_MASK::Register>),
        /// SRSS Interrupt Masked Register
        (0x20C => srss_intr_masked: ReadOnly<u32>),
        (0x210 => _reserved3),
        /// SRSS Additional Interrupt Register
        (0x300 => srss_aintr: ReadWrite<u32, SRSS_AINTR::Register>),
        /// SRSS Additional Interrupt Set Register
        (0x304 => srss_aintr_set: ReadWrite<u32, SRSS_AINTR_SET::Register>),
        /// SRSS Additional Interrupt Mask Register
        (0x308 => srss_aintr_mask: ReadWrite<u32, SRSS_AINTR_MASK::Register>),
        /// SRSS Additional Interrupt Masked Register
        (0x30C => srss_aintr_masked: ReadOnly<u32>),
        (0x310 => _reserved4),
        /// Debug Control Register
        (0x404 => boot_dlm_ctl: ReadWrite<u32, BOOT_DLM_CTL::Register>),
        /// Debug Control Register 2
        (0x408 => boot_dlm_ctl2: ReadWrite<u32, BOOT_DLM_CTL2::Register>),
        /// Debug Status Register
        (0x40C => boot_dlm_status: ReadOnly<u32, BOOT_DLM_STATUS::Register>),
        /// Soft Reset Trigger Register
        (0x410 => res_soft_ctl: ReadWrite<u32, RES_SOFT_CTL::Register>),
        (0x414 => _reserved5),
        /// Boot Execution Status Register
        (0x418 => boot_status: ReadOnly<u32, BOOT_STATUS::Register>),
        (0x41C => _reserved6),
        /// Warm Boot Entry Address
        (0x430 => boot_entry: ReadWrite<u32, BOOT_ENTRY::Register>),
        (0x434 => _reserved7),
        /// Hibernate Wakeup Mask Register
        (0x8A0 => pwr_hib_wake_ctl: ReadWrite<u32, PWR_HIB_WAKE_CTL::Register>),
        /// Hibernate Wakeup Polarity Register
        (0x8A4 => pwr_hib_wake_ctl2: ReadWrite<u32, PWR_HIB_WAKE_CTL2::Register>),
        (0x8A8 => _reserved8),
        /// Hibernate Wakeup Cause Register
        (0x8AC => pwr_hib_wake_cause: ReadOnly<u32, PWR_HIB_WAKE_CAUSE::Register>),
        (0x8B0 => _reserved9),
        /// Power Mode Control
        (0x1000 => pwr_ctl: ReadWrite<u32, PWR_CTL::Register>),
        /// Power Mode Control 2
        (0x1004 => pwr_ctl2: ReadWrite<u32, PWR_CTL2::Register>),
        /// HIBERNATE Mode Register
        (0x1008 => pwr_hibernate: ReadWrite<u32, PWR_HIBERNATE::Register>),
        (0x100C => _reserved10),
        /// High Voltage / Low Voltage Detector (HVLVD) Configuration Register
        (0x1020 => pwr_lvd_ctl: ReadWrite<u32, PWR_LVD_CTL::Register>),
        (0x1024 => _reserved11),
        // Clock Path Select Registers
        (0x1200 => clk_path_select0: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x1204 => clk_path_select1: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x1208 => clk_path_select2: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x120C => clk_path_select3: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x1210 => clk_path_select4: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x1214 => clk_path_select5: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x1218 => clk_path_select6: ReadWrite<u32, CLK_PATH_SELECT::Register>),

        (0x121C => _reserved12),

        // Clock Root Select Registers
        (0x1240 => clk_root_select0: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x1244 => clk_root_select1: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x1248 => clk_root_select2: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x124C => clk_root_select3: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x1250 => clk_root_select4: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x1254 => clk_root_select5: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x1258 => clk_root_select6: ReadWrite<u32, CLK_ROOT_SELECT::Register>),

        (0x125C => _reserved13),

        // Clock Root Direct Select Registers
        (0x1280 => clk_direct_select0: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x1284 => clk_direct_select1: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x1288 => clk_direct_select2: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x128C => clk_direct_select3: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x1290 => clk_direct_select4: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x1294 => clk_direct_select5: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),
        (0x1298 => clk_direct_select6: ReadWrite<u32, CLK_DIRECT_SELECT::Register>),

        (0x129C => _reserved14),

        // Clock Supervision (CSV) HF Registers
        (0x1400 => csv_hf_csv0_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1404 => csv_hf_csv0_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1408 => csv_hf_csv0_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x140C => _reserved15),
        (0x1410 => csv_hf_csv1_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1414 => csv_hf_csv1_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1418 => csv_hf_csv1_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x141C => _reserved16),
        (0x1420 => csv_hf_csv2_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1424 => csv_hf_csv2_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1428 => csv_hf_csv2_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x142C => _reserved17),
        (0x1430 => csv_hf_csv3_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1434 => csv_hf_csv3_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1438 => csv_hf_csv3_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x143C => _reserved18),
        (0x1440 => csv_hf_csv4_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1444 => csv_hf_csv4_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1448 => csv_hf_csv4_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x144C => _reserved19),
        (0x1450 => csv_hf_csv5_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1454 => csv_hf_csv5_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1458 => csv_hf_csv5_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x145C => _reserved20),
        (0x1460 => csv_hf_csv6_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1464 => csv_hf_csv6_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1468 => csv_hf_csv6_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),

        (0x146C => _reserved21),

        // General Clock Config
        (0x1500 => clk_select: ReadWrite<u32, CLK_SELECT::Register>),
        (0x1504 => _reserved22),
        (0x1518 => clk_imo_config: ReadWrite<u32, CLK_IMO_CONFIG::Register>),
        (0x151C => clk_eco_config: ReadWrite<u32, CLK_ECO_CONFIG::Register>),
        (0x1520 => clk_eco_prescale: ReadWrite<u32, CLK_ECO_PRESCALE::Register>),
        (0x1524 => clk_eco_status: ReadOnly<u32, CLK_ECO_STATUS::Register>),
        (0x1528 => _reserved23),
        (0x1530 => clk_fll_config: ReadWrite<u32, CLK_FLL_CONFIG::Register>),
        (0x1534 => clk_fll_config2: ReadWrite<u32, CLK_FLL_CONFIG2::Register>),
        (0x1538 => clk_fll_config3: ReadWrite<u32, CLK_FLL_CONFIG3::Register>),
        (0x153C => clk_fll_config4: ReadWrite<u32, CLK_FLL_CONFIG4::Register>),
        (0x1540 => clk_fll_status: ReadOnly<u32, CLK_FLL_STATUS::Register>),
        (0x1544 => clk_eco_config2: ReadWrite<u32, CLK_ECO_CONFIG2::Register>),
        (0x1548 => clk_ilo_config: ReadWrite<u32, CLK_ILO_CONFIG::Register>),
        (0x154C => clk_trim_ilo_ctl: ReadWrite<u32, CLK_TRIM_ILO_CTL::Register>),
        (0x1550 => _reserved24),
        (0x1554 => clk_mf_select: ReadWrite<u32, CLK_MF_SELECT::Register>),
        (0x1558 => clk_mfo_config: ReadWrite<u32, CLK_MFO_CONFIG::Register>),
        (0x155C => _reserved25),
        (0x1560 => clk_iho_config: ReadWrite<u32, CLK_IHO_CONFIG::Register>),

        (0x1564 => _reserved26),

        // CSV Reference Control
        (0x1700 => csv_ref_sel: ReadWrite<u32, CSV_REF_SEL::Register>),
        (0x1704 => _reserved27),
        (0x1710 => csv_ref_csv_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1714 => csv_ref_csv_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1718 => csv_ref_csv_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x171C => _reserved28),
        (0x1720 => csv_lf_csv_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1724 => csv_lf_csv_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1728 => csv_lf_csv_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),
        (0x172C => _reserved29),
        (0x1730 => csv_ilo_csv_ref_ctl: ReadWrite<u32, CSV_REF_CTL::Register>),
        (0x1734 => csv_ilo_csv_ref_limit: ReadWrite<u32, CSV_REF_LIMIT::Register>),
        (0x1738 => csv_ilo_csv_mon_ctl: ReadWrite<u32, CSV_MON_CTL::Register>),

        (0x173C => _reserved30),

        // Resets
        (0x1800 => res_cause: ReadOnly<u32, RES_CAUSE::Register>),
        (0x1804 => res_cause2: ReadOnly<u32, RES_CAUSE2::Register>),
        (0x1808 => _reserved31),
        (0x1814 => res_pxres_ctl: ReadWrite<u32, RES_PXRES_CTL::Register>),

        (0x1818 => _reserved32),

        // DPLL LP0
        (0x1A00 => clk_dpll_lp0_config: ReadWrite<u32, CLK_DPLL_LP_CONFIG::Register>),
        (0x1A04 => clk_dpll_lp0_config2: ReadWrite<u32, CLK_DPLL_LP_CONFIG2::Register>),
        (0x1A08 => clk_dpll_lp0_config3: ReadWrite<u32, CLK_DPLL_LP_CONFIG3::Register>),
        (0x1A0C => clk_dpll_lp0_config4: ReadWrite<u32, CLK_DPLL_LP_CONFIG4::Register>),
        (0x1A10 => clk_dpll_lp0_config5: ReadWrite<u32, CLK_DPLL_LP_CONFIG5::Register>),
        (0x1A14 => clk_dpll_lp0_config6: ReadWrite<u32, CLK_DPLL_LP_CONFIG6::Register>),
        (0x1A18 => clk_dpll_lp0_config7: ReadWrite<u32, CLK_DPLL_LP_CONFIG7::Register>),
        (0x1A1C => clk_dpll_lp0_status: ReadOnly<u32, CLK_DPLL_LP_STATUS::Register>),

        // DPLL LP1
        (0x1A20 => clk_dpll_lp1_config: ReadWrite<u32, CLK_DPLL_LP_CONFIG::Register>),
        (0x1A24 => clk_dpll_lp1_config2: ReadWrite<u32, CLK_DPLL_LP_CONFIG2::Register>),
        (0x1A28 => clk_dpll_lp1_config3: ReadWrite<u32, CLK_DPLL_LP_CONFIG3::Register>),
        (0x1A2C => clk_dpll_lp1_config4: ReadWrite<u32, CLK_DPLL_LP_CONFIG4::Register>),
        (0x1A30 => clk_dpll_lp1_config5: ReadWrite<u32, CLK_DPLL_LP_CONFIG5::Register>),
        (0x1A34 => clk_dpll_lp1_config6: ReadWrite<u32, CLK_DPLL_LP_CONFIG6::Register>),
        (0x1A38 => clk_dpll_lp1_config7: ReadWrite<u32, CLK_DPLL_LP_CONFIG7::Register>),
        (0x1A3C => clk_dpll_lp1_status: ReadOnly<u32, CLK_DPLL_LP_STATUS::Register>),

        (0x1A40 => _reserved33),

        // Trims and Security
        (0x2054 => tst_xres_secure: ReadWrite<u32, TST_XRES_SECURE::Register>),
        (0x2058 => _reserved34),
        (0x20E0 => pwr_trim_pwrsys_ctl: ReadWrite<u32, PWR_TRIM_PWRSYS_CTL::Register>),
        (0x20E4 => pwr_trim_pwrsys_ctl2: ReadWrite<u32, PWR_TRIM_PWRSYS_CTL2::Register>),

        (0x20E8 => _reserved35),

        (0x301C => clk_trim_eco_ctl: ReadWrite<u32, CLK_TRIM_ECO_CTL::Register>),

        (0x3020 => _reserved36),

        (0x4000 => ram_trim_trim_ram_ctl: ReadWrite<u32, RAM_TRIM_TRIM_RAM_CTL::Register>),
        (0x4004 => ram_trim_trim_rom_ctl: ReadWrite<u32, RAM_TRIM_TRIM_ROM_CTL::Register>),

        (0x4008 => _reserved37),

        // DPLL Trims
        (0x4200 => clk_trim_dpll_lp0_dpll_lp_ctl: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL::Register>),
        (0x4204 => _reserved38),
        (0x4208 => clk_trim_dpll_lp0_dpll_lp_ctl3: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL3::Register>),
        (0x420C => clk_trim_dpll_lp0_dpll_lp_ctl4: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL4::Register>),
        (0x4210 => _reserved39),
        (0x421C => clk_trim_dpll_lp0_dpll_lp_test4: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_TEST4::Register>),
        (0x4220 => clk_trim_dpll_lp1_dpll_lp_ctl: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL::Register>),
        (0x4224 => _reserved40),
        (0x4228 => clk_trim_dpll_lp1_dpll_lp_ctl3: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL3::Register>),
        (0x422C => clk_trim_dpll_lp1_dpll_lp_ctl4: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_CTL4::Register>),
        (0x4230 => _reserved41),
        (0x423C => clk_trim_dpll_lp1_dpll_lp_test4: ReadWrite<u32, CLK_TRIM_DPLL_LP_DPLL_LP_TEST4::Register>),

        (0x4240 => _reserved42),

        // Watchdog (WDT Type A)
        (0xC000 => wdt_ctl: ReadWrite<u32, WDT_CTL::Register>),
        (0xC004 => wdt_cnt: ReadOnly<u32, WDT_CNT::Register>),
        (0xC008 => wdt_match: ReadWrite<u32, WDT_MATCH::Register>),
        (0xC00C => wdt_match2: ReadWrite<u32, WDT_MATCH2::Register>),

        (0xC010 => _reserved43),

        // Multi-Counter Watchdog (MCWDT)
        (0xD004 => mcwdt_cntlow0: ReadOnly<u32, MCWDT_CNTLOW::Register>),
        (0xD008 => mcwdt_cnthigh0: ReadOnly<u32, MCWDT_CNTHIGH::Register>),
        (0xD00C => mcwdt_match0: ReadWrite<u32, MCWDT_MATCH::Register>),
        (0xD010 => mcwdt_config0: ReadWrite<u32, MCWDT_CONFIG::Register>),
        (0xD014 => mcwdt_ctl0: ReadWrite<u32, MCWDT_CTL::Register>),
        (0xD018 => mcwdt_intr0: ReadWrite<u32, MCWDT_INTR::Register>),
        (0xD01C => mcwdt_intr_set0: ReadWrite<u32, MCWDT_INTR_SET::Register>),
        (0xD020 => mcwdt_intr_mask0: ReadWrite<u32, MCWDT_INTR_MASK::Register>),
        (0xD024 => mcwdt_intr_masked0: ReadOnly<u32, MCWDT_INTR_MASKED::Register>),
        (0xD028 => mcwdt_lock0: ReadWrite<u32, MCWDT_LOCK::Register>),
        (0xD02C => mcwdt_lower_limit0: ReadWrite<u32, MCWDT_LOWER_LIMIT::Register>),
        (0xD030 => @END),
    }
}
register_bitfields![u32,
PWR_LVD_STATUS [
    /// HVLVD1 output.
/// 0: below voltage threshold
/// 1: above voltage threshold
    HVLVD1_OK OFFSET(0) NUMBITS(1) []
],
PWR_LVD_STATUS2 [
    /// HVLVD2 output.
/// 0: below voltage threshold
/// 1: above voltage threshold
    HVLVD2_OUT OFFSET(0) NUMBITS(1) []
],
CLK_DSI_SELECT [
    /// Selects a DSI source or low frequency clock for use in a clock path.  The output of this mux can be selected for clock PATH<i> using CLK_SELECT_PATH register.  Using the output of this mux as HFCLK source will result in undefined behavior.  It can be used to clocks to DSI or as reference inputs for the FLL/PLL, subject to the frequency limits of those circuits.  This mux is not glitch free, so do not change the selection while it is an actively selected clock.
    DSI_MUX OFFSET(0) NUMBITS(5) [
        /// DSI0 - dsi_out[0]
        DSI0Dsi_out0 = 0, // CY_SYSCLK_CLKPATH_IN_IMO
        /// DSI1 - dsi_out[1]
        DSI1Dsi_out1 = 1,
        /// DSI2 - dsi_out[2]
        DSI2Dsi_out2 = 2,
        /// DSI3 - dsi_out[3]
        DSI3Dsi_out3 = 3,
        /// DSI4 - dsi_out[4]
        DSI4Dsi_out4 = 4,
        /// DSI5 - dsi_out[5]
        DSI5Dsi_out5 = 5,
        /// DSI6 - dsi_out[6]
        DSI6Dsi_out6 = 6, // CY_SYSCLK_CLKPATH_IN_IHO
        /// DSI7 - dsi_out[7]
        DSI7Dsi_out7 = 7,
        /// DSI8 - dsi_out[8]
        DSI8Dsi_out8 = 8,
        /// DSI9 - dsi_out[9]
        DSI9Dsi_out9 = 9,
        /// DSI10 - dsi_out[10]
        DSI10Dsi_out10 = 10,
        /// DSI11 - dsi_out[11]
        DSI11Dsi_out11 = 11,
        /// DSI12 - dsi_out[12]
        DSI12Dsi_out12 = 12,
        /// DSI13 - dsi_out[13]
        DSI13Dsi_out13 = 13,
        /// DSI14 - dsi_out[14]
        DSI14Dsi_out14 = 14,
        /// DSI15 - dsi_out[15]
        DSI15Dsi_out15 = 15,
        /// ILO - Internal Low-speed Oscillator #0
        ILOInternalLowSpeedOscillator0 = 16,
        /// WCO - Watch-Crystal Oscillator
        WCOWatchCrystalOscillator = 17,
        /// ALTLF - Alternate Low-Frequency Clock
        ALTLFAlternateLowFrequencyClock = 18,
        /// PILO - Precision Internal Low-speed Oscillator
        PILOPrecisionInternalLowSpeedOscillator = 19,
        /// ILO1 - Internal Low-speed Oscillator #1, if present.
        ILO1InternalLowSpeedOscillator1IfPresent = 20,
    ]
],
CLK_OUTPUT_FAST [
    /// Select signal for fast clock output #0
    FAST_SEL0 OFFSET(0) NUMBITS(4) [
        /// Disabled - output is 0.  For power savings, clocks are blocked before entering any muxes, including PATH_SEL0 and HFCLK_SEL0.
        NC = 0,
        /// External Crystal Oscillator (ECO)
        ExternalCrystalOscillatorECO = 1,
        /// External clock input (EXTCLK)
        ExternalClockInputEXTCLK = 2,
        /// Alternate High-Frequency (ALTHF) clock input to SRSS
        AlternateHighFrequencyALTHFClockInputToSRSS = 3,
        /// Timer clock.  It is grouped with the fast clocks because it may be a gated version of a fast clock, and therefore may have a short high pulse.
        TIMERCLK = 4,
        /// Selects the clock path chosen by PATH_SEL0 field
        SelectsTheClockPathChosenByPATH_SEL0Field = 5,
        /// Selects the output of the HFCLK_SEL0 mux
        SelectsTheOutputOfTheHFCLK_SEL0Mux = 6,
        /// Selects the output of CLK_OUTPUT_SLOW.SLOW_SEL0
        SelectsTheOutputOfCLK_OUTPUT_SLOWSLOW_SEL0 = 7,
        /// Internal High-speed Oscillator (IHO).
        InternalHighSpeedOscillatorIHO = 8,
        /// clk_pwr: used for PPU and related components
        Clk_pwrUsedForPPUAndRelatedComponents = 9
    ],
    /// Selects a clock path to use in fast clock output #0 logic.
    PATH_SEL0 OFFSET(4) NUMBITS(4) [],
    /// Selects a HFCLK tree for use in fast clock output #0
    HFCLK_SEL0 OFFSET(8) NUMBITS(4) [],
    /// Select signal for fast clock output #1
    FAST_SEL1 OFFSET(16) NUMBITS(4) [
        /// Disabled - output is 0.  For power savings, clocks are blocked before entering any muxes, including PATH_SEL1 and HFCLK_SEL1.
        NC = 0,
        /// External Crystal Oscillator (ECO)
        ExternalCrystalOscillatorECO = 1,
        /// External clock input (EXTCLK)
        ExternalClockInputEXTCLK = 2,
        /// Alternate High-Frequency (ALTHF) clock input to SRSS
        AlternateHighFrequencyALTHFClockInputToSRSS = 3,
        /// Timer clock.  It is grouped with the fast clocks because it may be a gated version of a fast clock, and therefore may have a short high pulse.
        TIMERCLK = 4,
        /// Selects the clock path chosen by PATH_SEL1 field
        SelectsTheClockPathChosenByPATH_SEL1Field = 5,
        /// Selects the output of the HFCLK_SEL1 mux
        SelectsTheOutputOfTheHFCLK_SEL1Mux = 6,
        /// Selects the output of CLK_OUTPUT_SLOW.SLOW_SEL1
        SelectsTheOutputOfCLK_OUTPUT_SLOWSLOW_SEL1 = 7,
        /// Internal High-speed Oscillator (IHO).
        InternalHighSpeedOscillatorIHO = 8,
        /// clk_pwr: used for PPU and related components
        Clk_pwrUsedForPPUAndRelatedComponents = 9
    ],
    /// Selects a clock path to use in fast clock output #1 logic.
    PATH_SEL1 OFFSET(20) NUMBITS(4) [],
    /// Selects a HFCLK tree for use in fast clock output #1 logic
    HFCLK_SEL1 OFFSET(24) NUMBITS(4) []
],
CLK_OUTPUT_SLOW [
    /// Select signal for slow clock output #0
    SLOW_SEL0 OFFSET(0) NUMBITS(4) [
        /// Disabled - output is 0.  For power savings, clocks are blocked before entering any muxes.
        DisabledOutputIs0ForPowerSavingsClocksAreBlockedBeforeEnteringAnyMuxes = 0,
        /// Internal Low Speed Oscillator (ILO)
        InternalLowSpeedOscillatorILO = 1,
        /// Watch-Crystal Oscillator (WCO)
        WatchCrystalOscillatorWCO = 2,
        /// Root of the Backup domain clock tree (BAK)
        RootOfTheBackupDomainClockTreeBAK = 3,
        /// Alternate low-frequency clock input to SRSS (ALTLF)
        AlternateLowFrequencyClockInputToSRSSALTLF = 4,
        /// Root of the low-speed clock tree (LFCLK)
        RootOfTheLowSpeedClockTreeLFCLK = 5,
        /// Internal Main Oscillator (IMO).  This is grouped with the slow clocks so it can be observed during DEEPSLEEP entry/exit.
        IMO = 6,
        /// Sleep Controller clock (SLPCTRL).  This is grouped with the slow clocks so it can be observed during DEEPSLEEP entry/exit.
        SLPCTRL = 7,
        /// Precision Internal Low Speed Oscillator (PILO)
        PrecisionInternalLowSpeedOscillatorPILO = 8,
        /// Internal Low Speed Oscillator (ILO1), if present on the product.
        InternalLowSpeedOscillatorILO1IfPresentOnTheProduct = 9,
        /// ECO Prescaler (ECO_PRESCALER)
        ECOPrescalerECO_PRESCALER = 10,
        /// LPECO
        LPECO = 11,
        /// LPECO Prescaler (LPECO_PRESCALER)
        LPECOPrescalerLPECO_PRESCALER = 12,
        /// Medium Frequency Oscillator (MFO)
        MediumFrequencyOscillatorMFO = 13
    ],
    /// Select signal for slow clock output #1
    SLOW_SEL1 OFFSET(4) NUMBITS(4) [
        /// Disabled - output is 0.  For power savings, clocks are blocked before entering any muxes.
        DisabledOutputIs0ForPowerSavingsClocksAreBlockedBeforeEnteringAnyMuxes = 0,
        /// Internal Low Speed Oscillator (ILO)
        InternalLowSpeedOscillatorILO = 1,
        /// Watch-Crystal Oscillator (WCO)
        WatchCrystalOscillatorWCO = 2,
        /// Root of the Backup domain clock tree (BAK)
        RootOfTheBackupDomainClockTreeBAK = 3,
        /// Alternate low-frequency clock input to SRSS (ALTLF)
        AlternateLowFrequencyClockInputToSRSSALTLF = 4,
        /// Root of the low-speed clock tree (LFCLK)
        RootOfTheLowSpeedClockTreeLFCLK = 5,
        /// Internal Main Oscillator (IMO).  This is grouped with the slow clocks so it can be observed during DEEPSLEEP entry/exit.
        IMO = 6,
        /// Sleep Controller clock (SLPCTRL).  This is grouped with the slow clocks so it can be observed during DEEPSLEEP entry/exit.
        SLPCTRL = 7,
        /// Precision Internal Low Speed Oscillator (PILO)
        PrecisionInternalLowSpeedOscillatorPILO = 8,
        /// Internal Low Speed Oscillator (ILO1), if present on the product.
        InternalLowSpeedOscillatorILO1IfPresentOnTheProduct = 9,
        /// ECO Prescaler (ECO_PRESCALER)
        ECOPrescalerECO_PRESCALER = 10,
        /// LPECO
        LPECO = 11,
        /// LPECO Prescaler (LPECO_PRESCALER)
        LPECOPrescalerLPECO_PRESCALER = 12,
        /// Medium Frequency Oscillator (MFO)
        MediumFrequencyOscillatorMFO = 13
    ]
],
CLK_CAL_CNT1 [
    /// Down-counter clocked on fast clock output #0 (see CLK_OUTPUT_FAST). This register always reads as zero.  Counting starts internally when this register is written with a nonzero value.  CAL_COUNTER_DONE goes immediately low to indicate that the counter has started and will be asserted when the counters are done.  Do not write this field unless CAL_COUNTER_DONE==1.  Both clocks must be running or the measurement will not complete, and this case can be recovered using CAL_RESET.
    CAL_COUNTER1 OFFSET(0) NUMBITS(24) [],
    /// Reset clock calibration logic for window mode.  This can be used to recover from unexpected conditions, such as no clock present on counter #1.
/// Set this bit only when CLK_CAL_TEST.CAL_WINDOW_SEL=1 (window mode).  It takes 3 clock cycles for reset to propagate.
    CAL_RESET OFFSET(29) NUMBITS(1) [],
    /// Status bit indicating that a posedge was detected by counter #1.  If this bit never asserts, there is no clock on counter #1 and CAL_COUNTER_DONE will stay low indefinitely.  This can be recovered with CAL_RESET.
    CAL_CLK1_PRESENT OFFSET(30) NUMBITS(1) [],
    /// Status bit indicating that the internal counter #1 is finished counting and CLK_CAL_CNT2.COUNTER stopped counting up
    CAL_COUNTER_DONE OFFSET(31) NUMBITS(1) []
],
CLK_CAL_CNT2 [
    /// Up-counter clocked on fast clock output  #1 (see CLK_OUTPUT_FAST). When CLK_CAL_CNT1.CAL_COUNTER_DONE==1, the counter is stopped and can be read by SW.  Do not read this value unless CAL_COUNTER_DONE==1.  The expected final value is related to the ratio of clock frequencies used for the two counters and the value loaded into counter 1: CLK_CAL_CNT2.COUNTER=(F_cnt2/F_cnt1)*(CLK_CAL_CNT1.COUNTER)
    CAL_COUNTER2 OFFSET(0) NUMBITS(24) []
],
SRSS_INTR [
    /// WDT Interrupt Request.  This bit is set each time WDT_COUNTR==WDT_MATCH.  W1C also feeds the watch dog.  Missing 2 interrupts in a row will generate a reset.  Due to internal synchronization, it takes 2 SYSCLK cycles to update after a W1C.
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    /// Clock calibration counter is done.  This field is reset during DEEPSLEEP mode.
    CLK_CAL OFFSET(5) NUMBITS(1) [],
    /// See additional interrupts in SRSS_AINTR.
    AINTR OFFSET(31) NUMBITS(1) []
],
SRSS_INTR_SET [
    /// Set interrupt for low voltage detector WDT_MATCH
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    /// Set interrupt for clock calibration counter done.  This field is reset during DEEPSLEEP mode.
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_MASK [
    /// Mask for watchdog timer.  Clearing this bit will not forward the interrupt to the CPU.  It will not, however, disable the WDT reset generation on 2 missed interrupts.  When WDT resets the chip, it also internally pends an interrupt that survives the reset.  To prevent unintended ISR execution, clear SRSS_INTR.WDT_MATCH before setting this bit.
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    /// Mask for clock calibration done
    CLK_CAL OFFSET(5) NUMBITS(1) []
],
SRSS_INTR_MASKED [
    /// Logical and of corresponding request and mask bits.
    WDT_MATCH OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    CLK_CAL OFFSET(5) NUMBITS(1) [],
    /// See additional MASKED bits in SRSS_AINTR_MASKED.ADDITIONAL
    AINTR OFFSET(31) NUMBITS(1) []
],
SRSS_AINTR [
    /// Interrupt for low voltage detector HVLVD1
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    /// Interrupt for low voltage detector HVLVD2
    HVLVD2 OFFSET(2) NUMBITS(1) []
],
SRSS_AINTR_SET [
    /// Set interrupt for low voltage detector HVLVD1
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    /// Set interrupt for low voltage detector HVLVD2
    HVLVD2 OFFSET(2) NUMBITS(1) []
],
SRSS_AINTR_MASK [
    /// Mask for low voltage detector HVLVD1
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    /// Mask for low voltage detector HVLVD2
    HVLVD2 OFFSET(2) NUMBITS(1) []
],
SRSS_AINTR_MASKED [
    /// Logical and of corresponding request and mask bits.
    HVLVD1 OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    HVLVD2 OFFSET(2) NUMBITS(1) []
],
BOOT_DLM_CTL [
    /// A request to ROM_BOOT FW to execute particular code.
/// This field survives some resets, including a system reset:
/// * 0 - No request (default).
/// * 1 - ROM_BOOT to wait for DLM app (PC=0).
/// * 2 - ROM_BOOT to wait for an OEM debug token.
/// * 3 - ROM_BOOT to wait for a PROT_FW debug token.
/// * 4, 5 - ignored by ROM_BOOT.
/// * 6 - ROM_BOOT to launch DLM app which has been downloaded by DFU.
/// * 7 - ROM_BOOT to respond to DFU with DLM app status.
/// Other: ignored by ROM_BOOT.
    REQUEST OFFSET(0) NUMBITS(4) [],
    /// Notify RAM application that input parameters are valid for processing:
///        0 - the input parameters are not available.
///        1 - the input parameters are available (ready) for processing.
    INPUT_AVAIL OFFSET(29) NUMBITS(1) [],
    /// Request the device reset after RAM application complete input parameters processing. This bit is analyzed with APP_INPUT_AVAIL so to take effect write both.
///        0 - No action, the device waits for input parameters.
///        1 - Reset the device after input parameters processing complete.
    RESET OFFSET(30) NUMBITS(1) [],
    /// Wait for Action.  Set by BootROM when it waits for application or debug certificate to be loaded into the RAM. The bit must be cleared to continue BootROM operation. It is used by the Sys-AP.
    WFA OFFSET(31) NUMBITS(1) []
],
BOOT_DLM_CTL2 [
    /// Address of application descriptor or debug certificate depends on DEBUG_TST_CTL.REQUEST. The application descriptor provides info about RAM application and its parameters layout in the staging area to BootROM and RAM application itself
    APP_CTL OFFSET(0) NUMBITS(32) []
],
BOOT_DLM_STATUS [
    /// RAM application execution status. This status can be read by the debugger using Sys-AP or user application when RAM application completes with system reset. This field survives some resets, including a system reset.
    DEBUG_STATUS OFFSET(0) NUMBITS(32) []
],
RES_SOFT_CTL [
    /// Triggers a soft reset.  The reset clears this bit.
    TRIGGER_SOFT OFFSET(0) NUMBITS(1) []
],
BOOT_STATUS [
    /// Boot execution status. This status register can be used for communication between ROM_BOOT and User Application. This status can be read by the debugger using Sys-AP or User Application. This field survives some resets, including a system reset.
    DEBUG_STATUS OFFSET(0) NUMBITS(32) []
],
BOOT_ENTRY [
    /// Warm boot entry point. This status register can be used for communication between 2 software application before/after DS-RAM or soft reset. This field survives low voltage resets, including a system reset.
    WARM_BOOT_ENTRY OFFSET(0) NUMBITS(32) []
],
PWR_HIB_DATA [
    /// Additional data that is retained through a HIBERNATE/WAKEUP sequence that can be used by firmware for any application-specific purpose.  Note that waking up from HIBERNATE using XRES will reset this register.
    HIB_DATA OFFSET(0) NUMBITS(32) []
],
PWR_HIB_WAKE_CTL [
    /// When set, HIBERNATE will wakeup for the assigned source.
    HIB_WAKE_SRC OFFSET(0) NUMBITS(24) [],
    /// When set, HIBERNATE will wakeup for CSV_BAK detection.
    HIB_WAKE_CSV_BAK OFFSET(29) NUMBITS(1) [],
    /// When set, HIBERNATE will wakeup for a pending RTC interrupt.
    HIB_WAKE_RTC OFFSET(30) NUMBITS(1) [],
    /// When set, HIBERNATE will wakeup for a pending WDT interrupt.
    HIB_WAKE_WDT OFFSET(31) NUMBITS(1) []
],
PWR_HIB_WAKE_CTL2 [
    /// Each bit selects the polarity for the corresponding HIBERNATE wakeup source.  0: Wakes when unmasked input is 0.
/// 1: Wakes when unmasked input is 1.
    HIB_WAKE_SRC OFFSET(0) NUMBITS(24) []
],
PWR_HIB_WAKE_CAUSE [
    /// Each bit indicates a HIBERNATE wakeup cause.  For each bit, writing a 1 clears the cause flag.
    HIB_WAKE_SRC OFFSET(0) NUMBITS(24) [],
    /// Indicates CSV_BAK wakeup cause.  The related fault must be handled before this bit can be cleared.
    HIB_WAKE_CSV_BAK OFFSET(29) NUMBITS(1) [],
    /// Indicates RTC wakeup cause.  The RTC interrupt must be cleared before this bit can be cleared.
    HIB_WAKE_RTC OFFSET(30) NUMBITS(1) [],
    /// Indicates WDT wakeup cause.  The WDT interrupt must be cleared before this bit can be cleared.
    HIB_WAKE_WDT OFFSET(31) NUMBITS(1) []
],
PWR_CTL [
    /// Current power mode of the device.  Note that this field cannot be read in all power modes on actual silicon.
    POWER_MODE OFFSET(0) NUMBITS(2) [
        /// System is resetting.
        SystemIsResetting = 0,
        /// At least one CPU is running.
        AtLeastOneCPUIsRunning = 1,
        /// No CPUs are running.  Peripherals may be running.
        NoCPUsAreRunningPeripheralsMayBeRunning = 2,
        /// Main high-frequency clock is off; low speed clocks are available.  Communication interface clocks may be present.
        DEEPSLEEP = 3
    ],
    /// Indicates whether a debug session is active (CDBGPWRUPREQ signal is 1)
    DEBUG_SESSION OFFSET(4) NUMBITS(1) [
        /// No debug session active
        NoDebugSessionActive = 0,
        /// Debug session is active.  Power modes behave differently to keep the debug session active, and current consumption may be higher than datasheet specification.
        SESSION_ACTIVE = 1
    ],
    /// Indicates whether certain low power functions are ready.  The low current circuits take longer to startup after XRES, HIBERNATE wakeup, or supply supervision reset wakeup than the normal mode circuits.  HIBERNATE mode may be entered regardless of this bit.
/// 0: If a low power circuit operation is requested, it will stay in its normal operating mode until it is ready.  If DEEPSLEEP is requested by all processors WFI/WFE, the device will instead enter SLEEP.  When low power circuits are ready, device will automatically enter the originally requested mode.
/// 1: Normal operation.  DEEPSLEEP and low power circuits operate as requested in other registers.
    LPM_READY OFFSET(5) NUMBITS(1) []
],
PWR_CTL2 [
    /// Explicitly disable the linear Core Regulator.  Write zero for Traveo II devices.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Linear Core Regulator is not explicitly disabled.  Hardware disables it automatically for internal sequences, including for DEEPSLEEP, HIBERNATE, and XRES low power modes.
/// 1: Linear Core Regulator is explicitly disabled.  Only use this for special cases when another source supplies vccd during ACTIVE and SLEEP modes.  This setting is only legal when another source supplies vccd, but there is no special hardware protection for this case.
    LINREG_DIS OFFSET(0) NUMBITS(1) [],
    /// Status of the linear Core Regulator.
    LINREG_OK OFFSET(1) NUMBITS(1) [],
    /// Control the power mode of the Linear Regulator.  The value in this register is ignored and normal mode is used until LPM_READY==1.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Linear Regulator operates in normal mode.
/// 1: Linear Regulator operates in low power mode.  Load current capability is reduced, and firmware must ensure the current is kept within the limit.
    LINREG_LPMODE OFFSET(2) NUMBITS(1) [],
    /// Explicity disable the DeepSleep regulator, including circuits shared with the Active Regulator.  This register must not be set except as part of an Infineon-provided sequence or API.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: DeepSleep Regulator is not explicitly disabled.  This is the normal setting, and hardware automatically controls the DeepSleep regulator for most sequences, including for HIBERNATE and XRES low power modes.  This setting must be used if the Active Linear Regulator is used, because some circuitry is shared.
/// 1: DeepSleep Regulator is explicitly disabled.  Only use this for special cases as part of an Infineon-provided handoff to another supply source.  For example, this setting may be used when another source supplies vccdpslp during DEEPSLEEP mode and the Active Linear Regulator is not usedfor ACTIVE/SLEEP modes.
    DPSLP_REG_DIS OFFSET(4) NUMBITS(1) [],
    /// Explicitly disable the Retention regulator.  This field should normally be zero, except for special sequences provided by Infineon to use a different regulator.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Retention Regulator is not explicitly disabled.  Hardware disables it automatically for internal sequences, including for HIBERNATE and XRES low power modes.  Hardware keeps the Retention Regulator enabled during ACTIVE/SLEEP modes, so it is ready to enter DEEPSLEEP at any time.
/// 1: Retention Regulator is explicitly disabled.  Only use this for special cases when another source supplies vccret during DEEPSLEEP mode.  This setting is only legal when another source supplies vccret, but there is no special hardware protection for this case.
    RET_REG_DIS OFFSET(8) NUMBITS(1) [],
    /// Explicitly disable the Nwell regulator.  This register should normally be zero, except for special sequences provided by Infineon to use a different regulator.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Nwell Regulator is on.  Hardware disables it automatically for internal sequences, including for HIBERNATE and XRES low power modes.  Hardware keeps the Nwell Regulator enabled during ACTIVE/SLEEP modes, so it is ready to enter DEEPSLEEP at any time.
/// 1: Nwell Regulator is explicitly disabled.  Only use this for special cases when another source supplies vnwell during DEEPSLEEP mode.  This setting is only legal when another source supplies vnwell, but there is no special hardware protection for this case.
    NWELL_REG_DIS OFFSET(12) NUMBITS(1) [],
    /// N/A
    REFV_DIS OFFSET(16) NUMBITS(1) [],
    /// Indicates that the normal mode of the voltage reference is ready.
    REFV_OK OFFSET(17) NUMBITS(1) [],
    /// Disable the voltage reference buffer.  Firmware should only disable the buffer when there is no connected circuit that is using it.  SRSS circuits that require it are the PLL and ECO.  A particular product may have circuits outside the SRSS that use the buffer.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    REFVBUF_DIS OFFSET(20) NUMBITS(1) [],
    /// Indicates that the voltage reference buffer is ready.  Due to synchronization delays, it may take two IMO clock cycles for hardware to clear this bit after asserting REFVBUF_DIS=1.
    REFVBUF_OK OFFSET(21) NUMBITS(1) [],
    /// N/A
    REFI_DIS OFFSET(24) NUMBITS(1) [],
    /// Indicates that the current reference is ready.  Due to synchronization delays, it may take two IMO clock cycles for hardware to clear this bit after asserting REFI_DIS=1.
    REFI_OK OFFSET(25) NUMBITS(1) [],
    /// Control the power mode of the reference current generator.  The value in this register is ignored and normal mode is used until LPM_READY==1.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Current reference generator operates in normal mode.
/// 1: Current reference generator operates in low power mode.  Response time is reduced to save current.
    REFI_LPMODE OFFSET(26) NUMBITS(1) [],
    /// Control the power mode of the POR/BOD circuits.  The value in this register is ignored and normal mode is used until LPM_READY==1.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: POR/BOD circuits operate in normal mode.
/// 1: POR/BOD circuits operate in low power mode.  Response time is reduced to save current.
    PORBOD_LPMODE OFFSET(27) NUMBITS(1) [],
    /// Control the circuit-level power mode of the Bandgap Reference circuits for higher operating modes than DEEPSLEEP.   This selects a second set of bandgap voltage and current generation circuits that are optimized for low current consumption.  The low current circuits are automatically used in DEEPSLEEP mode regardless of this bit.  The value in this register is ignored and higher-current mode is used until LPM_READY==1.   After this bit is set, the Active Reference circuit can be disabled to reduce current (ACT_REF_DIS=0).    Firmware is responsible to enable the Active Reference and ensure ACT_REF_OK==1 before changing back to higher current mode.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: Bandgap Reference uses the normal settings.
/// 1: Bandgap Reference uses the low power DeepSleep circuits.  Power supply rejection is reduced to save current.
    BGREF_LPMODE OFFSET(28) NUMBITS(1) [],
    /// Controls whether mode and state of GPIOs and SIOs in the system are frozen.  This is intended to be used as part of the DEEPSLEEP-RAM and DEEPSLEEP-OFF entry and exit sequences.  It is set by HW while entering DEEPSLEEP-RAM and DEEPSLEEP-OFF modes.   Writing a 1 clears freeze and GPIOs and SIOs resume normal operation.
    FREEZE_DPSLP OFFSET(30) NUMBITS(1) [],
    /// Bypass level shifter inside the PLL.  Unused, if no PLL is present in the product. Note that this only applies to PLL200M.
/// 0: Do not bypass the level shifter.  This setting is ok for all operational modes and vccd target voltage.
/// 1: Bypass the level shifter.  This may reduce jitter on the PLL output clock, but can only be used when vccd is targeted to 1.1V nominal.  Otherwise, it can result in clock degradation and static current.
    PLL_LS_BYPASS OFFSET(31) NUMBITS(1) []
],
PWR_HIBERNATE [
    /// Contains a 8-bit token that is retained through a HIBERNATE/WAKEUP sequence that can be used by firmware to differentiate WAKEUP from a general RESET event.  Note that waking up from HIBERNATE using XRES will reset this register.
    TOKEN OFFSET(0) NUMBITS(8) [],
    /// This byte must be set to 0x3A for FREEZE or HIBERNATE fields to operate.  Any other value in this register will cause FREEZE/HIBERNATE to have no effect, except as noted in the FREEZE description.
    UNLOCK OFFSET(8) NUMBITS(8) [],
    /// Controls whether mode and state of GPIOs and SIOs in the system are frozen.  This is intended to be used as part of the HIBERNATE entry and exit sequences.  When entering HIBERNATE mode, the first write instructs DEEPSLEEP peripherals that they cannot ignore the upcoming freeze command.  This occurs even in the illegal condition where UNLOCK is not set.  If UNLOCK and HIBERNATE are properly set, the IOs actually freeze on the second write.  Supply supervision is disabled during HIBERNATE mode.  HIBERNATE peripherals ignore resets (excluding XRES) while FREEZE==1.
    FREEZE OFFSET(17) NUMBITS(1) [],
    /// N/A
    MASK_HIBALARM OFFSET(18) NUMBITS(1) [],
    /// N/A
    MASK_HIBWDT OFFSET(19) NUMBITS(1) [],
    /// N/A
    POLARITY_HIBPIN OFFSET(20) NUMBITS(4) [],
    /// N/A
    MASK_HIBPIN OFFSET(24) NUMBITS(4) [],
    /// Power mode when wakeups are sensitive.  The default of this field is 0 for software compatibility with other products.  It is recommended to set this field to 1 for new/updated software.
/// 0: Wakeups are sensitive only during HIBERNATE mode.  A wakeup pulse that comes just before HIBERNATE entry may be missed.  Backward compatible.
/// 1: Wakeups are sensitive in HIBERNATE and higher modes.  Before entering HIBERNATE, software must clear all unmasked, pending wakeups in PWR_HIB_WAKE_CAUSE register.  An unmasked, pending wakeup causes HIBERNATE wakeup, even if it was pending from before HIBERNATE entry.  This prevents missed wakeups.
    SENSE_MODE OFFSET(29) NUMBITS(1) [],
    /// Hibernate disable bit.
/// 0: Normal operation, HIBERNATE works as described
/// 1: Further writes to this register are ignored
/// Note: This bit is a write-once bit until the next reset.  Avoid changing any other bits in this register while disabling HIBERNATE mode.  Also, it is recommended to clear the UNLOCK code, if it was previously written..
    HIBERNATE_DISABLE OFFSET(30) NUMBITS(1) [],
    /// Firmware sets this bit to enter HIBERNATE mode.  The system will enter HIBERNATE mode immediately after writing to this bit and will wakeup only in response to XRES or WAKEUP.  Both UNLOCK and FREEZE must have been set correctly in a previous write operations.  Otherwise, it will not enter HIBERNATE.  External supplies must have been stable for 250us before entering HIBERNATE mode.
    HIBERNATE OFFSET(31) NUMBITS(1) []
],
PWR_BUCK_CTL [
    /// Voltage output selection for vccbuck1 output.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.  When increasing the voltage, it can take up to 200us for the output voltage to settle.  When decreasing the voltage, the settling time depends on the load current.
/// 0: 0.85V
/// 1: 0.875V
/// 2: 0.90V
/// 3: 1.0V (SISO-MC), 0.95V (SISO-LC, SIMO-LC)
/// 4: 1.05V
/// 5: 1.10V
/// 6: 1.15V
/// 7: 1.20V
    BUCK_OUT1_SEL OFFSET(0) NUMBITS(3) [],
    /// Master enable for buck converter.    This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    BUCK_EN OFFSET(30) NUMBITS(1) [],
    /// Enable for vccbuck1 output.  The value in this register is ignored unless PWR_BUCK_CTL.BUCK_EN==1.    This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.  The regulator takes up to 600us to charge the external capacitor.  If there is additional load current while charging, this will increase the startup time.  The TRM specifies the required sequence when transitioning vccd from the LDO to SIMO Buck output #1.
    BUCK_OUT1_EN OFFSET(31) NUMBITS(1) []
],
PWR_BUCK_CTL2 [
    /// Voltage output selection for vccbuck2 output.  When increasing the voltage, it can take up to 200us for the output voltage to settle.  When decreasing the voltage, the settling time depends on the load current.
/// 0: 1.15V
/// 1: 1.20V
/// 2: 1.25V
/// 3: 1.30V
/// 4: 1.35V
/// 5: 1.40V
/// 6: 1.45V
/// 7: 1.50V
    BUCK_OUT2_SEL OFFSET(0) NUMBITS(3) [],
    /// Hardware control for vccbuck2 output.  When this bit is set, the value in BUCK_OUT2_EN is ignored and a hardware signal is used instead.  If the product has supporting hardware, it can directly control the enable signal for vccbuck2.  The same charging time in BUCK_OUT2_EN applies.
    BUCK_OUT2_HW_SEL OFFSET(30) NUMBITS(1) [],
    /// Enable for vccbuck2 output.  The value in this register is ignored unless PWR_BUCK_CTL.BUCK_EN==1.  The regulator takes up to 600us to charge the external capacitor.  If there is additional load current while charging, this will increase the startup time.
    BUCK_OUT2_EN OFFSET(31) NUMBITS(1) []
],
PWR_SSV_CTL [
    /// Selects the voltage threshold for BOD on vddd.  The BOD does not reliably monitor the supply during the transition.
/// 0: vddd<2.7V
/// 1: vddd<3.0V
    BODVDDD_VSEL OFFSET(0) NUMBITS(1) [],
    /// Enable for BOD on vddd.  This cannot be disabled during normal operation.
    BODVDDD_ENABLE OFFSET(3) NUMBITS(1) [],
    /// Selects the voltage threshold for BOD on vdda.  Ensure BODVDDA_ENABLE==0 before changing this setting to prevent false triggers.
/// 0: vdda<2.7V
/// 1: vdda<3.0V
    BODVDDA_VSEL OFFSET(4) NUMBITS(1) [],
    /// Action taken when the BOD on vdda triggers.
    BODVDDA_ACTION OFFSET(6) NUMBITS(2) [
        /// No action
        NoAction = 0,
        /// Generate a fault
        GenerateAFault = 1,
        /// Reset the chip
        ResetTheChip = 2
    ],
    /// Enable for BOD on vdda.  BODVDDA_ACTION will be triggered when the BOD is disabled.  If no action is desired when disabling, firmware must first write BODVDDA_ACTION=NOTHING in a separate write cycle.
    BODVDDA_ENABLE OFFSET(8) NUMBITS(1) [],
    /// Enable for BOD on vccd.  This cannot be disabled during normal operation.
    BODVCCD_ENABLE OFFSET(11) NUMBITS(1) [],
    /// Selects the voltage threshold for OVD on vddd.  The OVD does not reliably monitor the supply during the transition.
/// 0: vddd>5.5V
/// 1: vddd>5.0V
    OVDVDDD_VSEL OFFSET(16) NUMBITS(1) [],
    /// Enable for OVD on vddd.  This cannot be disabled during normal operation.
    OVDVDDD_ENABLE OFFSET(19) NUMBITS(1) [],
    /// Selects the voltage threshold for OVD on vdda.  Ensure OVDVDDA_ENABLE==0 before changing this setting to prevent false triggers
/// 0: vddd>5.5V
/// 1: vddd>5.0V
    OVDVDDA_VSEL OFFSET(20) NUMBITS(1) [],
    /// Action taken when the OVD on vdda triggers.
    OVDVDDA_ACTION OFFSET(22) NUMBITS(2) [
        /// No action
        NoAction = 0,
        /// Generate a fault
        GenerateAFault = 1,
        /// Reset the chip
        ResetTheChip = 2
    ],
    /// Enable for OVD on vdda.
    OVDVDDA_ENABLE OFFSET(24) NUMBITS(1) [],
    /// Enable for OVD on vccd.  This cannot be disabled during normal operation.
    OVDVCCD_ENABLE OFFSET(27) NUMBITS(1) []
],
PWR_SSV_STATUS [
    /// BOD indicates vddd is ok.  This will always read 1, because a detected brownout will reset the chip.
    BODVDDD_OK OFFSET(0) NUMBITS(1) [],
    /// BOD indicates vdda is ok.
    BODVDDA_OK OFFSET(1) NUMBITS(1) [],
    /// BOD indicates vccd is ok.  This will always read 1, because a detected brownout will reset the chip.
    BODVCCD_OK OFFSET(2) NUMBITS(1) [],
    /// OVD indicates vddd is ok.  This will always read 1, because a detected over-voltage condition will reset the chip.
    OVDVDDD_OK OFFSET(8) NUMBITS(1) [],
    /// OVD indicates vdda is ok.
    OVDVDDA_OK OFFSET(9) NUMBITS(1) [],
    /// OVD indicates vccd is ok.    This will always read 1, because a detected over-over-voltage condition will reset the chip.
    OVDVCCD_OK OFFSET(10) NUMBITS(1) [],
    /// OCD indicates the current drawn from the linear Active Regulator is ok.  This will always read 1, because a detected over-current condition will reset the chip.
    OCD_ACT_LINREG_OK OFFSET(16) NUMBITS(1) [],
    /// OCD indicates the current drawn from the linear DeepSleep Regulator is ok.    This will always read 1, because a detected over-current condition will reset the chip.
    OCD_DPSLP_REG_OK OFFSET(17) NUMBITS(1) []
],
PWR_LVD_CTL [
    /// Threshold selection for HVLVD1.  Disable the detector (HVLVD1_EN=0) before changing the threshold.
/// 0: rise=1.225V (nom), fall=1.2V (nom)
/// 1: rise=1.425V (nom), fall=1.4V (nom)
/// 2: rise=1.625V (nom), fall=1.6V (nom)
/// 3: rise=1.825V (nom), fall=1.8V (nom)
/// 4: rise=2.025V (nom), fall=2V (nom)
/// 5: rise=2.125V (nom), fall=2.1V (nom)
/// 6: rise=2.225V (nom), fall=2.2V (nom)
/// 7: rise=2.325V (nom), fall=2.3V (nom)
/// 8: rise=2.425V (nom), fall=2.4V (nom)
/// 9: rise=2.525V (nom), fall=2.5V (nom)
/// 10: rise=2.625V (nom), fall=2.6V (nom)
/// 11: rise=2.725V (nom), fall=2.7V (nom)
/// 12: rise=2.825V (nom), fall=2.8V (nom)
/// 13: rise=2.925V (nom), fall=2.9V (nom)
/// 14: rise=3.025V (nom), fall=3.0V (nom)
/// 15: rise=3.125V (nom), fall=3.1V (nom)
    HVLVD1_TRIPSEL OFFSET(0) NUMBITS(4) [],
    /// Source selection for HVLVD1
    HVLVD1_SRCSEL OFFSET(4) NUMBITS(3) [
        /// Select VDDD
        SelectVDDD = 0,
        /// Select AMUXBUSA (VDDD branch)
        SelectAMUXBUSAVDDDBranch = 1,
        /// N/A
        NA = 2,
        /// Select AMUXBUSB (VDDD branch)
        SelectAMUXBUSBVDDDBranch = 4
    ],
    /// Enable HVLVD1 voltage monitor.  HVLVD1 does not function during DEEPSLEEP, but it automatically returns to its configured setting after DEEPSLEEP wakeup.  Do not change other HVLVD1 settings when enabled.
    HVLVD1_EN OFFSET(7) NUMBITS(1) [],
    /// N/A
    HVLVD1_TRIPSEL_HT OFFSET(8) NUMBITS(5) [],
    /// Keep HVLVD1 voltage monitor enabled during DEEPSLEEP mode.  This field is only used when HVLVD1_EN_HT==1.
    HVLVD1_DPSLP_EN_HT OFFSET(14) NUMBITS(1) [],
    /// Enable HVLVD1 voltage monitor.  This detector monitors vddd only.  Do not change other HVLVD1 settings when enabled.
    HVLVD1_EN_HT OFFSET(15) NUMBITS(1) [],
    /// Sets which edge(s) will trigger an action when the threshold is crossed.
    HVLVD1_EDGE_SEL OFFSET(16) NUMBITS(2) [
        /// Disabled
        Disabled = 0,
        /// Rising edge
        RisingEdge = 1,
        /// Falling edge
        FallingEdge = 2,
        /// Both rising and falling edges
        BothRisingAndFallingEdges = 3
    ],
    /// Action taken when the threshold is crossed in the programmed directions(s)
    HVLVD1_ACTION OFFSET(18) NUMBITS(1) [
        /// Generate an interrupt
        GenerateAnInterrupt = 0,
        /// Generate a fault
        GenerateAFault = 1
    ]
],
PWR_LVD_CTL2 [
    /// N/A
    HVLVD2_TRIPSEL_HT OFFSET(8) NUMBITS(5) [],
    /// Keep HVLVD2 voltage monitor enabled during DEEPSLEEP mode.  This field is only used when HVLVD1_EN_HT==1.
    HVLVD2_DPSLP_EN_HT OFFSET(14) NUMBITS(1) [],
    /// Enable HVLVD2 voltage monitor.  This detector monitors vddd only.  Do not change other HVLVD2 settings when enabled.
    HVLVD2_EN_HT OFFSET(15) NUMBITS(1) [],
    /// Sets which edge(s) will trigger an action when the threshold is crossed.
    HVLVD2_EDGE_SEL OFFSET(16) NUMBITS(2) [
        /// Disabled
        Disabled = 0,
        /// Rising edge
        RisingEdge = 1,
        /// Falling edge
        FallingEdge = 2,
        /// Both rising and falling edges
        BothRisingAndFallingEdges = 3
    ],
    /// Action taken when the threshold is crossed in the programmed directions(s)
    HVLVD2_ACTION OFFSET(18) NUMBITS(1) [
        /// Generate an interrupt
        GenerateAnInterrupt = 0,
        /// Generate a fault
        GenerateAFault = 1
    ]
],
PWR_REGHC_CTL [
    /// REGHC control mode:
/// 0: external transistor connected,
/// 1: external PMIC connected
    REGHC_MODE OFFSET(0) NUMBITS(1) [],
    /// Setting for DRV_VOUT pin for PMIC mode.  See REGHC_VADJ for calculation of vadj.
/// 2'b00: DRV_VOUT=vccd*0.9/vadj;
/// 2'b01: DRV_VOUT=vccd*0.8/vadj;
/// 2'b10: DRV_VOUT=vccd*0.6/vadj;
/// 2'b11: DRV_VOUT=vccd
    REGHC_PMIC_DRV_VOUT OFFSET(2) NUMBITS(2) [],
    /// Regulator output trim according to the formula vadj=(1.020V + REGHC_VADJ*0.005V).  The default is 1.1V.  For transistor mode, REGHC will dynamically adjust DRV_VOUT so the supply targets the vadj voltage.  For PMIC mode, see REGHC_PMIC_DRV_VOUT.
    REGHC_VADJ OFFSET(4) NUMBITS(5) [],
    /// For REGHC external PMIC mode, controls whether hardware sequencer keeps the internal Active Linear Regulator enabled to improve supply supervision of vccd.  When using this feature, if the PMIC fails to keep vccd above the internal regulator target, then the internal regulator will attempt to recover vccd.  If the regulator current is too high, the regulator triggers an over-current detector (OCD) reset.
/// 0: Internal Active Linear Regulator disabled after PMIC enabled.  OCD is disabled.;
/// 1: Internal Active Linear Regulator kept enabled.  See datasheet for minimum PMIC vccd input to prevent OCD.
    REGHC_PMIC_USE_LINREG OFFSET(10) NUMBITS(1) [],
    /// Controls whether hardware sequencer enables reset voltage adjustment circuit when enabling a PMIC.
    REGHC_PMIC_USE_RADJ OFFSET(11) NUMBITS(1) [],
    /// Reset voltage adjustment for PMIC as a factor (Vfbk/Vref) where Vfbk is the feedback voltage and Vref is the PMIC internal reference.  The reset voltage adjustment circuit is enabled by the hardware sequencer if REGHC_PMIC_USE_RADJ=1.  PMIC have Vref of 0.8V or 0.9V, and the resulting reset voltage (Vreset) are precalculated in the table below:
/// 3'b000: Vfbk/Vref=1.0000, Vreset=.800V@(Vref=0.8V), .900V@(Vref=0.9V);
/// 3'b001: Vfbk/Vref=1.0556, Vreset=.844V@(Vref=0.8V), .950V@(Vref=0.9V);
/// 3'b010: Vfbk/Vref=1.1111, Vreset=.889V@(Vref=0.8V), 1.000V@(Vref=0.9V);
/// 3'b011: Vfbk/Vref=1.1250, Vreset=.900V@(Vref=0.8V), 1.013V@(Vref=0.9V);
/// 3'b100: Vfbk/Vref=1.1667, Vreset=.933V@(Vref=0.8V), 1.050V@(Vref=0.9V);
/// 3'b101: Vfbk/Vref=1.1875, Vreset=.950V@(Vref=0.8V), 1.069V@(Vref=0.9V);
/// 3'b110: Vfbk/Vref=1.2500, Vreset=1.000V@(Vref=0.8V), 1.125V@(Vref=0.9V);
/// 3'b111: Vfbk/Vref=1.3125, Vreset=1.050V@(Vref=0.8V), 1.181V@(Vref=0.9V);
    REGHC_PMIC_RADJ OFFSET(12) NUMBITS(3) [],
    /// Output enable for PMIC enable pin.  Set this bit high to enable the driver on this pin.
    REGHC_PMIC_CTL_OUTEN OFFSET(16) NUMBITS(1) [],
    /// Polarity used to enable the PMIC.  The sequencer uses REGHC_PMIC_CTL_POLARITY to enable the PMIC, and it uses the complement to disable the PMIC.
    REGHC_PMIC_CTL_POLARITY OFFSET(17) NUMBITS(1) [],
    /// Input buffer enable for PMIC status input.  Set this bit high to enable the input receiver.
    REGHC_PMIC_STATUS_INEN OFFSET(18) NUMBITS(1) [],
    /// The polarity used to trigger a reset action based on the PMIC status input.  The reset system triggers a reset when the unmasked PMIC status matches this value.
    REGHC_PMIC_STATUS_POLARITY OFFSET(19) NUMBITS(1) [],
    /// Wait count in 4us steps after PMIC status ok.  This is used by the hardware sequencer to allow additional settling time before disabling the internal regulator.  The LSB is 32 IMO periods which results in a nominal LSB step of 4us.
    REGHC_PMIC_STATUS_WAIT OFFSET(20) NUMBITS(10) [],
    /// N/A
    REGHC_TRANS_USE_OCD OFFSET(30) NUMBITS(1) [],
    /// Indicates the REGHC has been configured.  This is used to know if REGHC should be enabled in response to a debug power up request.  Do not change REGHC settings after this bit is set high.
    REGHC_CONFIGURED OFFSET(31) NUMBITS(1) []
],
PWR_REGHC_STATUS [
    /// Indicates the state of the REGHC enable/disable sequencer.  This bit is only valid when REGHC_SEQ_BUSY==0.
/// 0: REGHC sequencer indicates REGHC is disabled.
/// 1: REGHC sequencer indicates REGHC is enabled.
    REGHC_ENABLED OFFSET(0) NUMBITS(1) [],
    /// Indicates the over-current detector is operating and the current drawn from REGHC is within limits.  OCD is only a choice for transistor mode, and it is disabled for PMIC mode.
/// 0: Current measurement exceeds limit or detector is OFF,
/// 1: Current measurement within limit
    REGHC_OCD_OK OFFSET(1) NUMBITS(1) [],
    /// Indicates the REGHC circuit is enabled and operating.  It does not indicate that the voltage and current are within required limits for robust operation.
/// 0: REGHC circuit is not ready.  This can occur if the REGHC circuit is disabled or if it was recently enabled.
/// 1: REGHC circuit is enabled and operating.
    REGHC_CKT_OK OFFSET(2) NUMBITS(1) [],
    /// N/A
    REGHC_UV_OUT OFFSET(8) NUMBITS(1) [],
    /// N/A
    REGHC_OV_OUT OFFSET(9) NUMBITS(1) [],
    /// Indicates the PMIC status is ok.  This includes polarity adjustment according to REGHC_PMIC_STATUS_POLARITY.
/// 0: PMIC status is not ok or PMIC input buffer is disabled (REGHC_PMIC_STATUS_INEN==0);
/// 1: PMIC status input buffer is enabled and indicates ok
    REGHC_PMIC_STATUS_OK OFFSET(12) NUMBITS(1) [],
    /// Indicates the REGHC enable/disable sequencer is busy transitioning to/from REGHC.
/// 0: Sequencer is not busy;
/// 1: Sequencer is busy either enabling or disabling REGHC.
    REGHC_SEQ_BUSY OFFSET(31) NUMBITS(1) []
],
PWR_REGHC_CTL2 [
    /// Timeout while waiting for REGHC_PMIC_STATUS_OK==1 when switching to PMIC.
/// 0: disables timeout.
/// >0: enables timeout of REGHC_PMIC_STATUS_TIMEOUT*128us (nominal, clocked by IMO).  Timeout expiration triggers reset.
    REGHC_PMIC_STATUS_TIMEOUT OFFSET(0) NUMBITS(8) [],
    /// Enable REGHC.  This bit will not set if REGHC_CONFIGURED==0.  Use PWR_REGHC_STATUS.ENABLED to know the actual status of REGHC.  It will differ from this bit in the following cases:
/// A) Do not enter DEEPSLEEP while the sequencer is busy (see PWR_REGHC_STATUS.REGHC_SEQ_BUSY).  The hardware sequencer disables REGHC during DEEPSLEEP entry and enables it upon wakeup.
/// B) The debugger requests the chip remain powered up.  Hardware prevents REGHC from disabling when this bit is cleared.  Hardware does not automatically enable REGHC in response to debugger power up request.  If this bit is low when the debugger deasserts the power up request, the hardware sequencer will disable REGHC.
    REGHC_EN OFFSET(31) NUMBITS(1) []
],
PWR_PMIC_CTL [
    /// PMIC reference voltage setting.  This selects the scaling factor used to generate the output voltage (vout) given the feedback voltage (vfb) for the chosen PMIC.  For a PMIC that compares vfb to an internal reference voltage (vref) according to the formula vout=vref/vfb, select that vref below.  For a PMIC that contains an internal resistor divider and expects an unscaled feedback voltage, use the 'No scaling' choice.
/// 2'b00: Scale for vref=0.9V, use PMIC_VADJ to set the vccd target;
/// 2'b01: Scale for vref=0.8V, use PMIC_VADJ to set the vccd target;
/// 2'b10: Scale for vref=0.6V, use PMIC_VADJ to set the vccd target;
/// 2'b11: No scaling, PMIC_VADJ has no effect
    PMIC_VREF OFFSET(2) NUMBITS(2) [],
    /// Voltage adjustment output setting.  The lookup table in this field requires the proper setting in PMIC_VREF for the chosen PMIC.  This field has no effect when PMIC_VREF selects no scaling.  The feedback tap point is at a vccd pad inside the chip, so the voltage may be a little higher at the PMIC output.
/// 0x03: 1.040V, 0x04: 1.049V,
/// 0x05: 1.057V, 0x06: 1.066V,
/// 0x07: 1.074V, 0x08: 1.083V,
/// 0x09: 1.091V, 0x0A: 1.099V,
/// 0x0B: 1.108V, 0x0C: 1.116V,
/// 0x0D: 1.125V, 0x0E: 1.133V,
/// 0x0F: 1.142V, 0x10: 1.150V,
/// 0x11: 1.158V, 0x12: 1.167V,
/// 0x13: 1.175V, 0x14: 1.184V,
/// 0x15: 1.192V, 0x16: 1.201V,
/// 0x17: 1.209V, 0x18: 1.218V,
/// 0x19: 1.226V, 0x1A: 1.234V,
/// 0x1B: 1.243V, 0x1C: 1.251V,
/// others: Illegal.  Behavior is undefined.
    PMIC_VADJ OFFSET(4) NUMBITS(5) [],
    /// Controls whether hardware sequencer keeps the internal Active Linear Regulator enabled to improve supply supervision of vccd.  When using this feature, if the PMIC fails to keep vccd above the internal regulator target, then the internal regulator will attempt to recover vccd.  If the regulator current is too high, the regulator triggers an over-current detector (OCD) reset.
/// 0: Internal Active Linear Regulator disabled after PMIC enabled.  OCD is disabled.;
/// 1: Internal Active Linear Regulator kept enabled.  See datasheet for minimum PMIC vccd input to prevent OCD.
    PMIC_USE_LINREG OFFSET(10) NUMBITS(1) [],
    /// Analog buffer enable on voltage adjust output.  Write this bit depending on the type of PMIC connected:
/// 0: Bypass buffer.  This connects the resistor divider directly to the output pin.  Use this setting for a PMIC with a high-impedance feedback input, such as those that support a resistor divider on the PCB.  This setting can also be used with a low-impedance PMIC with PMIC_VREF=2'b11 (no scaling).
/// 1: Use analog buffer.  This enables an analog buffer between the resistor divider output and the pin.  The buffer can drive a resistor divider on the PCB that feeds into the PMIC feedback input.  This allows targeting a different PMIC reference voltage from PMIC_VREF choices, while still supporting voltage adjustment using the internal divider.
    PMIC_VADJ_BUF_EN OFFSET(15) NUMBITS(1) [],
    /// Output enable for PMIC enable pin.  Set this bit high to enable the driver on this pin.
    PMIC_CTL_OUTEN OFFSET(16) NUMBITS(1) [],
    /// Polarity used to enable the PMIC.  The sequencer uses PMIC_CTL_POLARITY to enable the PMIC, and it uses the complement to disable the PMIC.
    PMIC_CTL_POLARITY OFFSET(17) NUMBITS(1) [],
    /// Input buffer enable for PMIC status input.  Set this bit high to enable the input receiver.
    PMIC_STATUS_INEN OFFSET(18) NUMBITS(1) [],
    /// The polarity used to trigger a reset action based on the PMIC status input.  The reset system triggers a reset when the unmasked PMIC status matches this value.
    PMIC_STATUS_POLARITY OFFSET(19) NUMBITS(1) [],
    /// Wait count in 4us steps after PMIC status ok.  This is used by the hardware sequencer to allow additional settling time before disabling the internal regulator.  The LSB is 32 IMO periods which results in a nominal LSB step of 4us.
    PMIC_STATUS_WAIT OFFSET(20) NUMBITS(10) [],
    /// Indicates the PMIC has been configured.  This is used to know if PMIC should be enabled in response to a debug power up request.  Do not change PMIC settings after this bit is set high.
    PMIC_CONFIGURED OFFSET(31) NUMBITS(1) []
],
PWR_PMIC_STATUS [
    /// Indicates the state of the PMIC enable/disable sequencer.  This bit is only valid when PMIC_SEQ_BUSY==0.
/// 0: PMIC sequencer indicates PMIC is disabled.
/// 1: PMIC sequencer indicates PMIC is enabled.
    PMIC_ENABLED OFFSET(0) NUMBITS(1) [],
    /// Indicates the PMIC status is ok.  This includes polarity adjustment according to PMIC_STATUS_POLARITY.
/// 0: PMIC status is not ok or PMIC input buffer is disabled (PMIC_STATUS_INEN==0);
/// 1: PMIC status input buffer is enabled and indicates ok
    PMIC_STATUS_OK OFFSET(12) NUMBITS(1) [],
    /// Indicates the PMIC enable/disable sequencer is busy transitioning to/from PMIC.
/// 0: Sequencer is not busy;
/// 1: Sequencer is busy either enabling or disabling PMIC.
    PMIC_SEQ_BUSY OFFSET(31) NUMBITS(1) []
],
PWR_PMIC_CTL2 [
    /// Timeout while waiting for PMIC_STATUS_OK==1 when switching to PMIC.
/// 0: disables timeout.  Do not change this register after setting PWR_PMIC_CTL.PMIC_CONFIGURED.
/// >0: enables timeout of PMIC_STATUS_TIMEOUT*128us (nominal, clocked by IMO).  Timeout expiration triggers reset.
    PMIC_STATUS_TIMEOUT OFFSET(0) NUMBITS(8) [],
    /// Enable PMIC.  This bit will not set if PMIC_CONFIGURED==0.  Use PWR_PMIC_STATUS.ENABLED to know the actual status of PMIC.  It will differ from this bit in the following cases:
/// A) Do not enter DEEPSLEEP while the sequencer is busy (see PWR_PMIC_STATUS.PMIC_SEQ_BUSY).  The hardware sequencer disables PMIC during DEEPSLEEP entry and enables it upon wakeup.
/// B) The debugger requests the chip remain powered up.  Hardware prevents PMIC from disabling when this bit is cleared.  Hardware does not automatically enable PMIC in response to debugger power up request.  If this bit is low when the debugger deasserts the power up request, the hardware sequencer will disable PMIC.
    PMIC_EN OFFSET(31) NUMBITS(1) []
],
PWR_PMIC_CTL4 [
    /// Disables the VADJ circuitry.  This can be used to decrease current consumption if the entire feedback network is outside the device.
/// 0: Device generates VADJ when PMIC is enabled.  This allows the feedback loop to compensate for voltage drops in the PCB and package.
/// 1: Device does not generate VADJ, and it must not be part of the PMIC feedback loop.  This reduces current by turning off the internal resistor divider that generates VADJ.
    PMIC_VADJ_DIS OFFSET(30) NUMBITS(1) [],
    /// Configures PMIC behavior during DEEPSLEEP.
/// 0: Device operates from internal regulators during DEEPSLEEP.  If PMIC is enabled at the beginning of the DEEPSLEEP transition, hardware changes to the internal regulators and disables the PMIC.
/// 1: DEEPSLEEP transition does not change PMIC enable.
    PMIC_DPSLP OFFSET(31) NUMBITS(1) []
],
CLK_PATH_SELECT [
    /// Selects a source for clock PATH<i>.  Note that not all products support all clock sources.  Selecting a clock source that is not supported will result in undefined behavior.  It takes four cycles of the originally selected clock to switch away from it.  Do not disable the original clock during this time.
    PATH_MUX OFFSET(0) NUMBITS(3) [
        /// IMO - Internal R/C Oscillator
        IMOInternalRCOscillator = 0,
        /// EXTCLK - External Clock Pin
        EXTCLKExternalClockPin = 1,
        /// ECO - External-Crystal Oscillator
        ECOExternalCrystalOscillator = 2,
        /// ALTHF - Alternate High-Frequency clock input (product-specific clock)
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        /// DSI_MUX - Output of DSI mux for this path.  Using a DSI source directly as root of HFCLK will result in undefined behavior.
        DSI_MUX = 4,
        /// LPECO - Low-Power External-Crystal Oscillator
        LPECOLowPowerExternalCrystalOscillator = 5,
        /// IHO - Internal High-speed Oscillator
        IHOInternalHighSpeedOscillator = 6
    ]
],
CLK_ROOT_SELECT [
    /// Selects a clock path for HFCLK<k> and SRSS DSI input <k>.  The output of this mux goes to the direct mux (see CLK_DIRECT_SELECT).  Use CLK_SELECT_PATH[i] to configure the desired path.  The number of clock paths is product-specific, and selecting an unimplemented path is not supported.  Some paths may have FLL or PLL available (product-specific), and the control and bypass mux selections of these are in other registers.  Configure the FLL using CLK_FLL_CONFIG register.  Configure a PLL using the related CLK_PLL_CONFIG[k] register.  Note that not all products support all clock sources.  Selecting a clock source that is not supported will result in undefined behavior.  It takes four cycles of the originally selected clock to switch away from it.  Do not disable the original clock during this time.
    ROOT_MUX OFFSET(0) NUMBITS(4) [
        /// Select PATH0
        SelectPATH0 = 0,
        /// Select PATH1
        SelectPATH1 = 1,
        /// Select PATH2
        SelectPATH2 = 2,
        /// Select PATH3
        SelectPATH3 = 3,
        /// Select PATH4
        SelectPATH4 = 4,
        /// Select PATH5
        SelectPATH5 = 5,
        /// Select PATH6
        SelectPATH6 = 6,
        /// Select PATH7
        SelectPATH7 = 7,
        /// Select PATH8
        SelectPATH8 = 8,
        /// Select PATH9
        SelectPATH9 = 9,
        /// Select PATH10
        SelectPATH10 = 10,
        /// Select PATH11
        SelectPATH11 = 11,
        /// Select PATH12
        SelectPATH12 = 12,
        /// Select PATH13
        SelectPATH13 = 13,
        /// Select PATH14
        SelectPATH14 = 14,
        /// Select PATH15
        SelectPATH15 = 15
    ],
    /// Obsolete.  Do not use in new designs.
    ROOT_DIV OFFSET(4) NUMBITS(2) [],
    /// Selects predivider value for this clock root and DSI input.  This divider is after DIRECT_MUX.  For products with DSI, the output of this mux is routed to DSI for use as a signal.  For products with clock supervision, the output of this mux is the monitored clock for CSV_HF<k>.
    ROOT_DIV_INT OFFSET(8) NUMBITS(4) [
        /// Transparent mode, feed through selected clock source w/o dividing.
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        /// Divide selected clock source by 2
        DivideSelectedClockSourceBy2 = 1,
        /// Divide selected clock source by 3
        DivideSelectedClockSourceBy3 = 2,
        /// Divide selected clock source by 4
        DivideSelectedClockSourceBy4 = 3,
        /// Divide selected clock source by 5
        DivideSelectedClockSourceBy5 = 4,
        /// Divide selected clock source by 6
        DivideSelectedClockSourceBy6 = 5,
        /// Divide selected clock source by 7
        DivideSelectedClockSourceBy7 = 6,
        /// Divide selected clock source by 8
        DivideSelectedClockSourceBy8 = 7,
        /// Divide selected clock source by 9
        DivideSelectedClockSourceBy9 = 8,
        /// Divide selected clock source by 10
        DivideSelectedClockSourceBy10 = 9,
        /// Divide selected clock source by 11
        DivideSelectedClockSourceBy11 = 10,
        /// Divide selected clock source by 12
        DivideSelectedClockSourceBy12 = 11,
        /// Divide selected clock source by 13
        DivideSelectedClockSourceBy13 = 12,
        /// Divide selected clock source by 14
        DivideSelectedClockSourceBy14 = 13,
        /// Divide selected clock source by 15
        DivideSelectedClockSourceBy15 = 14,
        /// Divide selected clock source by 16
        DivideSelectedClockSourceBy16 = 15
    ],
    /// Enable for this clock root.  All clock roots default to disabled (ENABLE==0) except HFCLK0, which cannot be disabled.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_DIRECT_SELECT [
    /// Direct selection mux that allows IMO to bypass most of the clock mux structure.    For products with multiple regulators, this mux can be used to reduce current without requiring significant reconfiguration of the clocking network.  The default value of HFCLK<0>==ROOT_MUX, and the default value for other clock trees is product-specific.
    DIRECT_MUX OFFSET(8) NUMBITS(1) [
        /// Select IMO
        SelectIMO = 0,
        /// Select ROOT_MUX selection
        SelectROOT_MUXSelection = 1
    ]
],
CLK_SELECT [
    /// Select source for LFCLK.  Note that not all products support all clock sources.  Selecting a clock source that is not supported will result in undefined behavior.  Writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.  It takes four cycles of the originally selected clock to switch away from it.  Do not disable the original clock during this time.
    LFCLK_SEL OFFSET(0) NUMBITS(3) [
        /// ILO - Internal Low-speed Oscillator
        ILOInternalLowSpeedOscillator = 0,
        /// WCO - Watch-Crystal Oscillator.  Requires Backup domain to be present and properly configured (including external watch crystal, if used).
        WCO = 1,
        /// ALTLF - Alternate Low-Frequency Clock.  Capability is product-specific
        ALTLFAlternateLowFrequencyClockCapabilityIsProductSpecific = 2,
        /// PILO - Precision ILO, if present.
        PILOPrecisionILOIfPresent = 3,
        /// ILO1 - Internal Low-speed Oscillator #1, if present.
        ILO1InternalLowSpeedOscillator1IfPresent = 4,
        /// ECO_PRESCALER - External-Crystal Oscillator after prescaling, if present.  Does not work in DEEPSLEEP or HIBERNATE modes.  Intended for applications that operate in ACTIVE/SLEEP modes only.  This option is only valid when ECO is present in the product. Not compatible with an clk_sys frequency <48MHz.
        ECO_PRESCALER = 5,
        /// LPECO_PRESCALER - Low-Power External-Crystal Oscillator after prescaling, if present.  This choice works in ACTIVE/SLEEP/DEEPSLEEP modes.  This option is only valid when LPECO is present in the product.
        LPECO_PRESCALER = 6
    ],
    /// Selects clock PATH<k>, where k=PUMP_SEL.  The output of this mux goes to the PUMP_DIV to make PUMPCLK  Each product has a specific number of available clock paths.  Selecting a path that is not implemented on a product will result in undefined behavior.  Note that this is not a glitch free mux.
    PUMP_SEL OFFSET(8) NUMBITS(4) [],
    /// Division ratio for PUMPCLK.  Uses selected PUMP_SEL clock as the source.
    PUMP_DIV OFFSET(12) NUMBITS(3) [
        /// Transparent mode, feed through selected clock source w/o dividing.
        TransparentModeFeedThroughSelectedClockSourceWODividing = 0,
        /// Divide selected clock source by 2
        DivideSelectedClockSourceBy2 = 1,
        /// Divide selected clock source by 4
        DivideSelectedClockSourceBy4 = 2,
        /// Divide selected clock source by 8
        DivideSelectedClockSourceBy8 = 3,
        /// Divide selected clock source by 16
        DivideSelectedClockSourceBy16 = 4
    ],
    /// Enable the pump clock.  PUMP_ENABLE and the PUMP_SEL mux are not glitch-free to minimize side-effects, avoid changing the PUMP_SEL and PUMP_DIV while changing PUMP_ENABLE.  To change the settings, do the following:
/// 1) If the pump clock is enabled, write PUMP_ENABLE=0 without changing PUMP_SEL and PUMP_DIV.
/// 2) Change PUMP_SEL and PUMP_DIV to desired settings with PUMP_ENABLE=0.
/// 3) Write PUMP_ENABLE=1 without changing PUMP_SEL and PUMP_DIV.
    PUMP_ENABLE OFFSET(15) NUMBITS(1) []
],
CLK_ILO0_CONFIG [
    /// This register indicates that ILO0 should stay enabled during XRES and HIBERNATE modes.  If backup voltage domain is implemented on the product, this bit also indicates if ILO0 should stay enabled through power-related resets on other supplies, e.g.. BOD on VDDD/VCCD.  Writes to this field are ignored unless the WDT is unlocked.  This register is reset when the backup logic resets.
/// 0: ILO0 turns off during XRES, HIBERNATE, and power-related resets.  ILO0 configuration and trims are reset by these events.
/// 1: ILO0 stays enabled, as described above.  ILO0 configuration and trims are not reset by these events.
    ILO0_BACKUP OFFSET(0) NUMBITS(1) [],
    /// N/A
    ILO0_MON_ENABLE OFFSET(30) NUMBITS(1) [],
    /// Master enable for ILO.  Writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.
///
/// HT-variant: This register will not clear unless PWR_CTL2.BGREF_LPMODE==0. After enabling, the first ILO0 cycle occurs within 12us and is +/-10 percent accuracy.  Thereafter, ILO0 is +/-5 percent accurate.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ILO1_CONFIG [
    /// N/A
    ILO1_MON_ENABLE OFFSET(30) NUMBITS(1) [],
    /// Master enable for ILO1.
///
/// HT-variant: After enabling, the first ILO1 cycle occurs within 12us and is +/-10 percent accuracy.  Thereafter, ILO1 is +/-5 percent accurate.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_IMO_CONFIG [
    /// Enable for IMO during DEEPSLEEP.  This bit configures IMO behavior during DEEPSLEEP:
/// 0: IMO is automatically disabled during DEEPSLEEP and enables upon wakeup;
/// 1: IMO is kept enabled throughout DEEPSLEEP
    DPSLP_ENABLE OFFSET(30) NUMBITS(1) [],
    /// Master enable for IMO oscillator.  This bit must be high at all times for all functions to work properly.  Hardware will automatically disable the IMO during HIBERNATE and XRES.  It will automatically disable during DEEPSLEEP if DPSLP_ENABLE==0.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ECO_CONFIG [
    /// Automatic Gain Control (AGC) enable.  When set, the oscillation amplitude is controlled to the level selected by CLK_ECO_CONFIG2.ATRIM.  When low, the amplitude is not explicitly controlled and can be as high as the vddd supply.  WARNING: use care when disabling AGC because driving a crystal beyond its rated limit can permanently damage the crystal.
    AGC_EN OFFSET(1) NUMBITS(1) [],
    /// ECO prescaler disable command (mutually exclusive with ECO_DIV_ENABLE). SW sets this field to '1' and HW sets this field to '0'.
///
/// HW sets ECO_DIV_DISABLE field to '0' immediately and HW sets CLK_ECO_PRESCALE.ECO_DIV_EN field to '0' immediately.
    ECO_DIV_DISABLE OFFSET(27) NUMBITS(1) [],
    /// ECO prescaler enable command (mutually exclusive with ECO_DIV_DISABLE). ECO Prescaler only works in ACTIVE and SLEEP modes.  SW sets this field to '1' to enable the divider and HW sets this field to '0' to indicate that divider enabling has completed. When the divider is enabled, its integer and fractional counters are initialized to '0'. If a divider is to be re-enabled using different integer and fractional divider values, the SW should follow these steps:
/// 0: Disable the divider using the ECO_DIV_DISABLE field.
/// 1: Configure CLK_ECO_PRESCALE registers.
/// 2: Enable the divider using the ECO_DIV_ENABLE field.
///
/// HW sets the ECO_DIV_ENABLE field to '0' when the enabling is performed and HW set CLK_ECO_PRESCALER.ENABLED to '1' when the enabling is performed.
    ECO_DIV_ENABLE OFFSET(28) NUMBITS(1) [],
    /// Master enable for ECO oscillator.  Configure the settings in CLK_ECO_CONFIG2 to work with the selected crystal, before enabling ECO.
    ECO_EN OFFSET(31) NUMBITS(1) []
],
CLK_ECO_PRESCALE [
    /// ECO prescaler enabled. HW sets this field to '1' as a result of an CLK_ECO_CONFIG.ECO_DIV_ENABLE command. HW sets this field to '0' as a result on a CLK_ECO_CONFIG.ECO_DIV_DISABLE command.  ECO prescaler is incompatible with DEEPSLEEP modes, and firmware must disable it before entering DEEPSLEEP.
    ECO_DIV_ENABLED OFFSET(0) NUMBITS(1) [],
    /// 8-bit fractional value, sufficient to get prescaler output within the +/-65ppm calibration range.  Do not change this setting when ECO Prescaler is enabled.
    ECO_FRAC_DIV OFFSET(8) NUMBITS(8) [],
    /// 10-bit integer value allows for ECO frequencies up to 33.55MHz.  Subtract one from the desired divide value when writing this field.  For example, to divide by 1, write ECO_INT_DIV=0.  Do not change this setting when ECO Prescaler is enabled.
    ECO_INT_DIV OFFSET(16) NUMBITS(10) []
],
CLK_ECO_STATUS [
    /// Indicates the ECO internal oscillator circuit has sufficient amplitude.  It may not meet the PPM accuracy or duty cycle spec.
    ECO_OK OFFSET(0) NUMBITS(1) [],
    /// Indicates the ECO internal oscillator circuit has had enough time to fully stabilize.  This is the output of a counter since ECO was enabled, and it does not check the ECO output.  It is recommended to also confirm ECO_OK==1.
    ECO_READY OFFSET(1) NUMBITS(1) []
],
CLK_PILO_CONFIG [
    /// If backup domain is present on this product, this register indicates that PILO should stay enabled for use by backup domain during XRES, and HIBERNATE mode.    If backup voltage domain is implemented on the product, PILO should stay enabled through power-related resets on other supplies, e.g.. BOD on VDDD/VCCD.   If the PILO is the selected source for WDT, writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.
/// 0: PILO turns off at XRES/BOD events.  (unless backup voltage domain is implemented on the product)
/// 1: PILO remains on if backup domain is present and powered even for XRES/BOD or HIBERNATE entry.
    PILO_BACKUP OFFSET(0) NUMBITS(1) [],
    /// PILO second order temperature curvature correction enable.  If the PILO is the selected source for WDT, writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.
/// 0: Disable second order temperature curvature correction.
/// 1: Enable second order temperature curvature correction.
    PILO_TCSC_EN OFFSET(16) NUMBITS(1) [],
    /// Enable PILO.  If the PILO is the selected source for WDT, writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.
    PILO_EN OFFSET(31) NUMBITS(1) []
],
CLK_FLL_CONFIG [
    /// Multiplier to determine CCO frequency in multiples of the frequency of the selected reference clock (Fref).
///
/// Ffll = (FLL_MULT)  * (Fref / REFERENCE_DIV) / (OUTPUT_DIV+1)
    FLL_MULT OFFSET(0) NUMBITS(18) [],
    /// Control bits for Output divider.  Set the divide value before enabling the FLL, and do not change it while FLL is enabled.
/// 0: no division
/// 1: divide by 2
    FLL_OUTPUT_DIV OFFSET(24) NUMBITS(1) [],
    /// Master enable for FLL.  The FLL requires firmware sequencing when enabling and disabling.  Hardware handles sequencing automatically when entering/exiting DEEPSLEEP.
///
/// To enable the FLL, use the following sequence:
/// 1) Configure FLL and CCO settings.  Do not modify CLK_FLL_CONFIG3.BYPASS_SEL (must be AUTO) or CLK_FLL_CONFIG.FLL_ENABLE (must be 0).
/// 2) Enable the CCO by writing CLK_FLL_CONFIG4.CCO_ENABLE=1
/// 3) Wait until CLK_FLL_STATUS.CCO_READY==1.
/// 4) Ensure the reference clock has stabilized.
/// 5) Write FLL_ENABLE=1.
/// 6) Optionally wait until CLK_FLL_STATUS.LOCKED==1.  The hardware automatically changes to the FLL output when LOCKED==1.
///
/// To disable the FLL, use the following sequence:
/// 1) Write CLK_FLL_CONFIG3.BYPASS_SEL=FLL_REF.
/// 2) Read CLK_FLL_CONFIG3.BYPASS_SEL to ensure the write completes (read is not optional).
/// 3) Wait at least ten cycles of either FLL reference clock or FLL output clock, whichever is slower.
/// 4) Disable FLL with FLL_ENABLE=0.
/// 5) Disable the CCO by writing CLK_FLL_CONFIG4.CCO_ENABLE=0.
/// 6) Write CLK_FLL_CONFIG3.BYPASS_SEL=AUTO.
/// 7) Read CLK_FLL_CONFIG3.BYPASS_SEL to ensure the write completes (read is not optional).
/// 8) Wait three cycles of FLL reference clock.
///
/// 0: Block is powered off
/// 1: Block is powered on
    FLL_ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_FLL_CONFIG2 [
    /// Control bits for reference divider.  Set the divide value before enabling the FLL, and do not change it while FLL is enabled.
/// 0: illegal (undefined behavior)
/// 1: divide by 1
/// ...
/// 8191: divide by 8191
    FLL_REF_DIV OFFSET(0) NUMBITS(13) [],
    /// Lock tolerance sets the error threshold for when the FLL output is considered locked to the reference input.  A high tolerance can be used to lock more quickly or allow less accuracy.  The tolerance is the allowed difference between the count value for the ideal formula and the measured value.
/// 0: tolerate error of 1 count value
/// 1: tolerate error of 2 count values
/// ...
/// 255: tolerate error of 256 count values
    LOCK_TOL OFFSET(16) NUMBITS(8) [],
    /// Update tolerance sets the error threshold for when the FLL will update the CCO frequency settings.  The update tolerance is the allowed difference between the count value for the ideal formula and the measured value. UPDATE_TOL should be less than LOCK_TOL.
    UPDATE_TOL OFFSET(24) NUMBITS(8) []
],
CLK_FLL_CONFIG3 [
    /// FLL Loop Filter Gain Setting #1.  The proportional gain is the sum of FLL_LF_IGAIN and FLL_LF_PGAIN.
/// 0: 1/256
/// 1: 1/128
/// 2: 1/64
/// 3: 1/32
/// 4: 1/16
/// 5: 1/8
/// 6: 1/4
/// 7: 1/2
/// 8: 1.0
/// 9: 2.0
/// 10: 4.0
/// 11: 8.0
/// >=12: illegal
    FLL_LF_IGAIN OFFSET(0) NUMBITS(4) [],
    /// FLL Loop Filter Gain Setting #2.  The proportional gain is the sum of FLL_LF_IGAIN and FLL_LF_PGAIN.
/// 0: 1/256
/// 1: 1/128
/// 2: 1/64
/// 3: 1/32
/// 4: 1/16
/// 5: 1/8
/// 6: 1/4
/// 7: 1/2
/// 8: 1.0
/// 9: 2.0
/// 10: 4.0
/// 11: 8.0
/// >=12: illegal
    FLL_LF_PGAIN OFFSET(4) NUMBITS(4) [],
    /// Number of undivided reference clock cycles to wait after changing the CCO trim until the loop measurement restarts.  A delay allows the CCO output to settle and gives a more accurate measurement.  The default is tuned to an 8MHz reference clock since the IMO is expected to be the most common use case.
/// 0: no settling time
/// 1: wait one reference clock cycle
/// ...
/// 8191: wait 8191 reference clock cycles
    SETTLING_COUNT OFFSET(8) NUMBITS(13) [],
    /// Bypass mux located just after FLL output.  This register can be written while the FLL is enabled.  When changing BYPASS_SEL, do not turn off the reference clock or CCO clock for five cycles (whichever is slower).  Whenever BYPASS_SEL is changed, it is required to read CLK_FLL_CONFIG3 to ensure the change takes effect.
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        /// Automatic using lock indicator.  When unlocked, automatically selects FLL reference input (bypass mode).  When locked, automatically selects FLL output.  This can allow some processing to occur while the FLL is locking, such as after DEEPSLEEP wakeup.  It is incompatible with clock supervision, because the frequency changes based on the lock signal.
        AUTO = 0,
        /// Similar to AUTO, except the clock is gated off when unlocked.  This is compatible with clock supervision, because the supervisors allow no clock during startup (until a timeout occurs), and the clock targets the proper frequency whenever it is running.
        LOCKED_OR_NOTHING = 1,
        /// Select FLL reference input (bypass mode).  Ignores lock indicator
        SelectFLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        /// Select FLL output.  Ignores lock indicator.
        SelectFLLOutputIgnoresLockIndicator = 3
    ]
],
CLK_FLL_CONFIG4 [
    /// Maximum CCO offset allowed (used to prevent FLL dynamics from selecting an CCO frequency that the logic cannot support)
    CCO_LIMIT OFFSET(0) NUMBITS(8) [],
    /// Frequency range of CCO
    CCO_RANGE OFFSET(8) NUMBITS(3) [
        /// Target frequency is in range [48, 64) MHz
        TargetFrequencyIsInRange4864MHz = 0,
        /// Target frequency is in range [64, 85) MHz
        TargetFrequencyIsInRange6485MHz = 1,
        /// Target frequency is in range [85, 113) MHz
        TargetFrequencyIsInRange85113MHz = 2,
        /// Target frequency is in range [113, 150) MHz
        TargetFrequencyIsInRange113150MHz = 3,
        /// Target frequency is in range [150, 200] MHz
        TargetFrequencyIsInRange150200MHz = 4
    ],
    /// CCO frequency code.  This is updated by HW when the FLL is enabled.  It can be manually updated to use the CCO in an open loop configuration.  The meaning of each frequency code depends on the range.
    CCO_FREQ OFFSET(16) NUMBITS(9) [],
    /// Disable CCO frequency update by FLL hardware
/// 0: Hardware update of CCO settings is allowed.  Use this setting for normal FLL operation.
/// 1: Hardware update of CCO settings is disabled.  Use this setting for open-loop FLL operation.
    CCO_HW_UPDATE_DIS OFFSET(30) NUMBITS(1) [],
    /// Enable the CCO.  It is required to enable the CCO before using the FLL.
/// 0: Block is powered off
/// 1: Block is powered on
    CCO_ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_FLL_STATUS [
    /// FLL Lock Indicator
    LOCKED OFFSET(0) NUMBITS(1) [],
    /// This bit sets whenever the FLL is enabled and goes out of lock.  This bit stays set until cleared by firmware.
/// Note: When exiting Deep Sleep with FLL enabled, UNLOCK_OCCURRED will set. Therefore, after FLL successfully locks, FW should clear UNLOCK_OCCURRED flag to prevent a false positive that would indicate that FLL erroneously unlocked.
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) [],
    /// This indicates that the CCO is internally settled and ready to use.
    CCO_READY OFFSET(2) NUMBITS(1) []
],
CLK_ECO_CONFIG2 [
    /// Watch Dog Trim -  Delta voltage below steady state level
/// 0x0 - 50mV
/// 0x1 - 75mV
/// 0x2 - 100mV
/// 0x3 - 125mV
/// 0x4 - 150mV
/// 0x5 - 175mV
/// 0x6 - 200mV
/// 0x7 - 225mV
    WDTRIM OFFSET(0) NUMBITS(3) [],
    /// Amplitude trim to set the crystal drive level when ECO_CONFIG.AGC_EN=1.  WARNING: use care when setting this field because driving a crystal beyond its rated limit can permanently damage the crystal.
/// 0x0 - 150mV
/// 0x1 - 175mV
/// 0x2 - 200mV
/// 0x3 - 225mV
/// 0x4 - 250mV
/// 0x5 - 275mV
/// 0x6 - 300mV
/// 0x7 - 325mV
/// 0x8 - 350mV
/// 0x9 - 375mV
/// 0xA - 400mV
/// 0xB - 425mV
/// 0xC - 450mV
/// 0xD - 475mV
/// 0xE - 500mV
/// 0xF - 525mV
    ATRIM OFFSET(4) NUMBITS(4) [],
    /// Filter Trim - 3rd harmonic oscillation
    FTRIM OFFSET(8) NUMBITS(2) [],
    /// Feedback resistor Trim
    RTRIM OFFSET(10) NUMBITS(2) [],
    /// Gain Trim - Startup time.
    GTRIM OFFSET(12) NUMBITS(3) []
],
CLK_ILO_CONFIG [
    /// If backup domain is present on this product, this register indicates that ILO should stay enabled for use by backup domain during XRES, HIBERNATE mode, and through power-related resets like BOD on VDDD/VCCD.  Writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.
/// 0: ILO turns off at XRES/BOD event or HIBERNATE entry.
/// 1: ILO remains on if backup domain is present and powered even for XRES/BOD or HIBERNATE entry.
    ILO_BACKUP OFFSET(0) NUMBITS(1) [],
    /// Master enable for ILO.  Writes to this field are ignored unless the WDT is unlocked using WDT_LOCK register.  After enabling, it takes at most two cycles to reach the accuracy spec.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_TRIM_ILO_CTL [
    /// IL0 frequency trims.  LSB step size is 1.5 percent (typical) of the frequency.
    ILO_FTRIM OFFSET(0) NUMBITS(6) []
],
CLK_TRIM_ILO0_CTL [
    /// ILO0 frequency trims.  LSB step size is 1.5 percent (typical) of the frequency.
    ILO0_FTRIM OFFSET(0) NUMBITS(6) [],
    /// ILO0 internal monitor trim.
    ILO0_MONTRIM OFFSET(8) NUMBITS(4) []
],
CLK_MF_SELECT [
    /// Select source for MFCLK (clk_mf).  Note that not all products support all clock sources.  Selecting a clock source that is not supported results in undefined behavior.
    MFCLK_SEL OFFSET(0) NUMBITS(3) [
        /// MFO - Medium Frequency Oscillator.  DEEPSLEEP compatibility is product-specific.  See CLK_MFO_CONFIG for capability of this product.
        MFO = 0,
        /// ILO - Internal Low-speed Oscillator.
        ILOInternalLowSpeedOscillator = 1,
        /// WCO - Watch-Crystal Oscillator, if present.
        WCOWatchCrystalOscillatorIfPresent = 2,
        /// ALTLF - Alternate Low-Frequency Clock.  Capability is product-specific
        ALTLFAlternateLowFrequencyClockCapabilityIsProductSpecific = 3,
        /// PILO - Precision ILO, if present.
        PILOPrecisionILOIfPresent = 4,
        /// ILO1 - Internal Low-speed Oscillator #1, if present.
        ILO1InternalLowSpeedOscillator1IfPresent = 5,
        /// ECO_PRESCALER - External-Crystal Oscillator, if present, after prescaling in CLK_ECO_PRESCALE.  Intended for applications that operate in ACTIVE/SLEEP modes only.  Does not work in DEEPSLEEP mode.
        ECO_PRESCALER = 6,
        /// LPECO - Low Power External Crystal Oscillator, if present.
        LPECOLowPowerExternalCrystalOscillatorIfPresent = 7
    ],
    /// Divide selected clock source by (1+MFCLK_DIV).  The output of this divider is MFCLK (clk_mf).  Allows for integer divisions in the range [1, 256].  Do not change this setting while ENABLE==1.
    MFCLK_DIV OFFSET(8) NUMBITS(8) [],
    /// Enable for MFCLK (clk_mf).  When disabling clk_mf, do not disable the source until after 5 clk_mf periods.  clk_mf continues to operate in DEEPSLEEP for compatible sources.  Firmware must disable clk_mf before entering DEEPSLEEP if the source is not compatible with DEEPSLEEP mode.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_MFO_CONFIG [
    /// Enable for MFO during DEEPSLEEP.  This bit is ignored when ENABLE==0.  When ENABLE==1:
/// 0: MFO is automatically disabled during DEEPSLEEP and enables upon wakeup;
/// 1: MFO is kept enabled throughout DEEPSLEEP
    DPSLP_ENABLE OFFSET(30) NUMBITS(1) [],
    /// Enable for Medium Frequency Oscillator (MFO) to generate clk_mf.  It is product-specific whether this is a separate component or implemented as a divided version of another clock (eg. IMO).
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_IHO_CONFIG [
    /// Enable for Internal High-speed Oscillator (IHO) to generate clk_iho.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_ALTHF_CTL [
    /// Indicates that ALTHF is actually enabled.  The delay between a transition on ALTHF_ENABLE and ALTHF_ENABLED is product specific.
    ALTHF_ENABLED OFFSET(0) NUMBITS(1) [],
    /// Enable for ALTHF clock when used by SRSS.  There may be independent control of ALTHF by another subsystem, and this bit prevents ALTHF from being disabled when SRSS needs it.  SRSS automatically removes its enable request during DEEPSLEEP and lower modes.
    ALTHF_ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_CONFIG [
    /// Control bits for feedback divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0-21: illegal (undefined behavior)
/// 22: divide by 22
/// ...
/// 112: divide by 112
/// >112: illegal (undefined behavior)
    FEEDBACK_DIV OFFSET(0) NUMBITS(7) [],
    /// Control bits for reference divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0: illegal (undefined behavior)
/// 1: divide by 1
/// ...
/// 20: divide by 20
/// others: illegal (undefined behavior)
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    /// Control bits for Output divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0: illegal (undefined behavior)
/// 1: illegal (undefined behavior)
/// 2: divide by 2.  Suitable for direct usage as HFCLK source.
/// ...
/// 16: divide by 16.  Suitable for direct usage as HFCLK source.
/// >16: illegal (undefined behavior)
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    /// N/A
    LOCK_DELAY OFFSET(25) NUMBITS(2) [],
    /// VCO frequency range selection.  Configure this bit according to the targeted VCO frequency.  Do not change this setting while the PLL is enabled.
/// 0: VCO frequency is [200MHz, 400MHz]
/// 1: VCO frequency is [170MHz, 200MHz)
    PLL_LF_MODE OFFSET(27) NUMBITS(1) [],
    /// Bypass mux located just after PLL output.  This selection is glitch-free and can be changed while the PLL is running.  When changing BYPASS_SEL, do not turn off the reference clock or PLL clock for five cycles (whichever is slower).
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        /// Automatic using lock indicator.  When unlocked, automatically selects PLL reference input (bypass mode).  When locked, automatically selects PLL output.  If ENABLE=0, automatically selects PLL reference input.
        AUTO = 0,
        /// Similar to AUTO, except the clock is gated off when unlocked.  This is compatible with clock supervision, because the supervisors allow no clock during startup (until a timeout occurs), and the clock targets the proper frequency whenever it is running.  If ENABLE=0, no clock is output.
        LOCKED_OR_NOTHING = 1,
        /// Select PLL reference input (bypass mode).  Ignores lock indicator
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        /// Select PLL output.  Ignores lock indicator.  If ENABLE=0, no clock is output.
        SelectPLLOutputIgnoresLockIndicatorIfENABLE0NoClockIsOutput = 3
    ],
    /// Master enable for PLL.  Setup FEEDBACK_DIV, REFERENCE_DIV, and OUTPUT_DIV at least one cycle before setting ENABLE=1.
///
/// Fpll = (FEEDBACK_DIV)  * (Fref / REFERENCE_DIV) / (OUTPUT_DIV)
///
/// 0: Block is disabled.  When the PLL disables, hardware controls the bypass mux as described in BYPASS_SEL, before disabling the PLL circuit.
/// 1: Block is enabled
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_PLL_STATUS [
    /// PLL Lock Indicator
    LOCKED OFFSET(0) NUMBITS(1) [],
    /// This bit sets whenever the PLL Lock bit goes low, and stays set until cleared by firmware.
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
CSV_REF_SEL [
    /// Selects a source for clock clk_ref_hf.  Note that not all products support all clock sources.  Selecting a clock source that is not supported will result in undefined behavior.   It takes four cycles of the originally selected clock to switch away from it.  Do not disable the original clock during this time.
    REF_MUX OFFSET(0) NUMBITS(3) [
        /// IMO - Internal R/C Oscillator
        IMOInternalRCOscillator = 0,
        /// EXTCLK - External Clock Pin
        EXTCLKExternalClockPin = 1,
        /// ECO - External-Crystal Oscillator
        ECOExternalCrystalOscillator = 2,
        /// ALTHF - Alternate High-Frequency clock input (product-specific clock)
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        /// IHO - Internal High-speed Oscillator
        IHOInternalHighSpeedOscillator = 4
    ]
],
RES_CAUSE [
    /// A basic WatchDog Timer (WDT) reset has occurred since last power cycle.  ULP products: This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above).
///
/// For products that support high-voltage cause detection, this bit blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.  This bit is not blocked by other HV cause bits.
    RESET_WDT OFFSET(0) NUMBITS(1) [],
    /// N/A
    RESET_ACT_FAULT OFFSET(1) NUMBITS(1) [],
    /// N/A
    RESET_DPSLP_FAULT OFFSET(2) NUMBITS(1) [],
    /// Test controller or debugger asserted reset. Only resets debug domain.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above).
    RESET_TC_DBGRESET OFFSET(3) NUMBITS(1) [],
    /// A CPU requested a system reset through it's SYSRESETREQ.  This can be done via a debugger probe or in firmware.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above).
    RESET_SOFT OFFSET(4) NUMBITS(1) [],
    /// Multi-Counter Watchdog timer reset #0.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above). This bit is only valid when parameter NUM_MCWDT>0
    RESET_MCWDT0 OFFSET(5) NUMBITS(1) [],
    /// Multi-Counter Watchdog timer reset #1.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above). This bit is only valid when parameter NUM_MCWDT>1
    RESET_MCWDT1 OFFSET(6) NUMBITS(1) [],
    /// Multi-Counter Watchdog timer reset #2.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above). This bit is only valid when parameter NUM_MCWDT>2
    RESET_MCWDT2 OFFSET(7) NUMBITS(1) [],
    /// Multi-Counter Watchdog timer reset #3.  This is a low-voltage cause bit that hardware clears when the low-voltage supply is initialized (see comments above). This bit is only valid when parameter NUM_MCWDT>3
    RESET_MCWDT3 OFFSET(8) NUMBITS(1) []
],
RES_CAUSE2 [
    /// Clock supervision logic requested a reset due to loss or frequency violation of a high-frequency clock.  Each bit index K corresponds to a HFCLK<K>.  Unimplemented clock bits return zero.  Each bit is only valid when the corresponding bit in parameter MASK_HFCSV is 1 and CSV_PRESENT is set.
    RESET_CSV_HF OFFSET(0) NUMBITS(16) [],
    /// Clock supervision logic requested a reset due to loss or frequency violation of the reference clock source that is used to monitor the other HF clock sources.  This bit is only valid when parameter CSV_PRESENT is set.
    RESET_CSV_REF OFFSET(16) NUMBITS(1) []
],
RES_CAUSE_EXTEND [
    /// External XRES pin was asserted.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.  This bit is not blocked by other HV cause bits.
    RESET_XRES OFFSET(16) NUMBITS(1) [],
    /// External VDDD supply crossed brown-out limit.  Note that this cause will only be observable as long as the VDDD supply does not go below the POR (power on reset) detection limit.  Below this limit it is not possible to reliably retain information in the device.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_BODVDDD OFFSET(17) NUMBITS(1) [],
    /// External VDDA supply crossed the brown-out limit.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_BODVDDA OFFSET(18) NUMBITS(1) [],
    /// Internal VCCD core supply crossed the brown-out limit.  Note that this detector will detect gross issues with the internal core supply, but may not catch all brown-out conditions.  Functional and timing supervision (CSV, WDT) is provided to create fully failsafe internal crash detection.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_BODVCCD OFFSET(19) NUMBITS(1) [],
    /// Overvoltage detection on the external VDDD supply.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OVDVDDD OFFSET(20) NUMBITS(1) [],
    /// Overvoltage detection on the external VDDA supply.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OVDVDDA OFFSET(21) NUMBITS(1) [],
    /// Overvoltage detection on the internal core VCCD supply.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OVDVCCD OFFSET(22) NUMBITS(1) [],
    /// Overcurrent detection on the internal VCCD supply when supplied by the ACTIVE power mode linear regulator.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OCD_ACT_LINREG OFFSET(23) NUMBITS(1) [],
    /// Overcurrent detection on the internal VCCD supply when supplied by the DEEPSLEEP power mode linear regulator.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OCD_DPSLP_LINREG OFFSET(24) NUMBITS(1) [],
    /// Overcurrent detection from REGHC (if present).  If REGHC is not present, hardware will never set this bit.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_OCD_REGHC OFFSET(25) NUMBITS(1) [],
    /// PMIC status triggered a reset.  If PMIC control is not present, hardware will never set this bit.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.
    RESET_PMIC OFFSET(26) NUMBITS(1) [],
    /// PXRES triggered.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.  This bit is not blocked by other HV cause bits.
    RESET_PXRES OFFSET(28) NUMBITS(1) [],
    /// Structural reset was asserted.  This is a high-voltage cause bit that blocks recording of other high-voltage cause bits, except RESET_PORVDDD.  Hardware clears this bit during POR.  This bit is not blocked by other HV cause bits.
    RESET_STRUCT_XRES OFFSET(29) NUMBITS(1) [],
    /// Indicator that a POR occurred.  This is a high-voltage cause bit, and hardware clears the other bits when this one is set.  It does not block further recording of other high-voltage causes.
    RESET_PORVDDD OFFSET(30) NUMBITS(1) []
],
RES_PXRES_CTL [
    /// Triggers PXRES.  This causes a full-scope reset and reboot.
    PXRES_TRIGGER OFFSET(0) NUMBITS(1) []
],
PWR_CBUCK_CTL [
    /// Voltage output selection.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.   The actual CBUCK voltage is the maximum of this setting, the settings for all enabled step-down regulators (see PWR_SDR*_CTL), and the minimum DEEPSLEEP setting (which is not user configurable).  These settings follow the formula (0.76+0.02*CBUCK_VSEL).
/// 0: 0.76V, 1: 0.78V, 2: 0.80V, 3: 0.82V, 4: 0.84V, 5: 0.86V, 6: 0.88V, 7: 0.90V, 8: 0.92V, 9: 0.94V, 10: 0.96V, 11: 0.98V, 12: 1.00V, 13: 1.02V, 14: 1.04V, 15: 1.06V, 16: 1.08V, 17: 1.10V, 18: 1.12V, 19: 1.14V, 20: 1.16V, 21: 1.18V, 22: 1.20V, 23: 1.22V, 24: 1.24V, 25: 1.26V, 26: 1.28V, 27: 1.30V, 28: 1.32V, 29: 1.34V, 30: 1.36V, 31: 1.38V.
    CBUCK_VSEL OFFSET(0) NUMBITS(5) [],
    /// CBUCK mode.  Low ripple (high power) modes are intended for analog that needs low ripple.  Low power mode is suitable for digital processing.
/// The CBUCK mode is defined as = {mode*_sr_mode, mode*_sr_hp_submode[1:0], mode*_sr_lp_submode[1:0]}
/// The actual CBUCK mode is the maximum of this setting and the settings of all enabled step-down regulators.
/// 0x11: HP, PFM Auto, High-Low (default Active)
/// 0x01: LP, PFM Auto, High-Low (DeepSleep)
/// See s40power BROS for other settings
    CBUCK_MODE OFFSET(8) NUMBITS(5) []
],
PWR_CBUCK_CTL2 [
    /// Forces the CBUCK to use the settings in PWR_CBUCK_CTL register, ignoring the other hardware requests.  This can be used as part of a firmware algorithm to change the voltage of an enabled stepdown regulator.  This bit is cleared by any reset.
    CBUCK_OVERRIDE OFFSET(28) NUMBITS(1) [],
    /// Pauses new dynamic CBUCK transitions.  An already started transition will complete, but new dynamic transitions are paused.  This can be used as part of a firmware sequence to change the voltage setting of an enabled stepdown regulator.
    CBUCK_PAUSE OFFSET(29) NUMBITS(1) [],
    /// Copies the current CBUCK composite state to the fields in  PWR_CBUCK_CTL register (CBUCK_VSEL and CBUCK_MODE).  It is recommended to pause transitions using CBUCK_PAUSE to ensure the state does not change near the copy.  After it is copied, the CBUCK_OVERRIDE bit can be used to hold the CBUCK in the current state.  Note, reading this bit always returns 0.
/// 0: no change
/// 1: copy settings.
    CBUCK_COPY_SETTINGS OFFSET(30) NUMBITS(1) [],
    /// Causes the settings in  PWR_CBUCK_CTL register to be included in the CBUCK setting decision.  Can be used to override the normal hardware voltage behavior.  Regardless of this bit, the extra settings in  PWR_CBUCK_CTL register are not used during DEEPSLEEP.
    CBUCK_USE_SETTINGS OFFSET(31) NUMBITS(1) []
],
PWR_CBUCK_CTL3 [
    /// CBUCK inrush limit selection.
/// 0: 10mA limit.
/// 1: 100mA limit.
    CBUCK_INRUSH_SEL OFFSET(31) NUMBITS(1) []
],
PWR_CBUCK_STATUS [
    /// Indicates the power management unit is finished with a transition.
/// 0: PMU busy
/// 1: PMU done
    PMU_DONE OFFSET(31) NUMBITS(1) []
],
PWR_SDR0_CTL [
    /// Minimum voltage selection of CBUCK when using this SDR0 (see PWR_CBUCK_CTL for voltage table).  The voltage must be 60mV higher than the SDR output or the regulator output may bypass.
    SDR0_CBUCK_VSEL OFFSET(0) NUMBITS(5) [],
    /// Minimum CBUCK mode when using SDR0 (see PWR_CBUCK_CTL for mode table).
/// Default Active
    SDR0_CBUCK_MODE OFFSET(5) NUMBITS(5) [],
    /// DeepSleep voltage selection of CBUCK (see PWR_CBUCK_CTL for voltage table).  The voltage must be 60mV higher than the SDR output or the regulator output may bypass.
    SDR0_CBUCK_DPSLP_VSEL OFFSET(10) NUMBITS(5) [],
    /// DeepSleep CBUCK mode when using SDR0 (see PWR_CBUCK_CTL for mode table).
/// Default DeepSleep
    SDR0_CBUCK_DPSLP_MODE OFFSET(15) NUMBITS(5) [],
    /// SDR0 output voltage.
/// 0: 0.850V, 1: 0.875V, 2: 0.900V, 3: 0.925V, 4: 0.950V, 5: 0.975V, 6: 1.000V, 7: 1.025V, 8: 1.050V, 9: 1.075V, 10: 1.100V, 11: 1.125V, 12: 1.150V, 13: 1.175V, 14: 1.200V, 15: 1.225V
    SDR0_VSEL OFFSET(20) NUMBITS(4) [],
    /// SDR0 output voltage during DeepSleep.  (See SDR0_VSEL for voltage table).
    SDR0_DPSLP_VSEL OFFSET(26) NUMBITS(4) [],
    /// SDR0 bypass control.
/// 0: Force SDR0 to regulate.
/// 1: Allow SDR0 to bypass if the actual CBUCK voltage matches SDR0_CBUCK_VSEL.
    SDR0_ALLOW_BYPASS OFFSET(31) NUMBITS(1) []
],
PWR_SDR1_CTL [
    /// Minimum voltage selection of CBUCK when using this SDR1 (see PWR_CBUCK_CTL for voltage table).  The voltage must be 60mV higher than the SDR output or the regulator output may bypass.
    SDR1_CBUCK_VSEL OFFSET(0) NUMBITS(5) [],
    /// Minimum CBUCK mode when using SDR1 (see PWR_CBUCK_CTL for mode table).
    SDR1_CBUCK_MODE OFFSET(8) NUMBITS(5) [],
    /// SDR1 output voltage.
/// 0: 0.850V, 1: 0.875V, 2: 0.900V, 3: 0.925V, 4: 0.950V, 5: 0.975V, 6: 1.000V, 7: 1.025V, 8: 1.050V, 9: 1.075V, 10: 1.100V, 11: 1.125V, 12: 1.150V, 13: 1.175V, 14: 1.200V, 15: 1.225V
    SDR1_VSEL OFFSET(16) NUMBITS(4) [],
    /// Selects hardware control for SDR1.
/// 0: SDR1_ENABLE controls SDR1.  Hardware controls are ignored.
/// 1: SDR1_ENABLE is ignored and a hardware signal is used instead.  Selecting this on products that don't have supporting hardware will disable SDR1.
    SDR1_HW_SEL OFFSET(30) NUMBITS(1) [],
    /// Enable for SDR1.
    SDR1_ENABLE OFFSET(31) NUMBITS(1) []
],
PWR_HVLDO0_CTL [
    /// HVLDO0 output voltage.
/// 0: 1.8V, 1: 1.9V, 2: 2.0V, 3: 2.1V, 4: 2.2V, 5: 2.3V, 6: 2.4V, 7: 2.5V, 8: 2.6V, 9: 2.7V, 10: 2.8V, 11: 2.9V, 12: 3.0V, 13: 3.1V, 14: 3.2V, 15: 3.3V
    HVLDO0_VSEL OFFSET(0) NUMBITS(4) [],
    /// Selects hardware control for HVLDO0.
/// 0: HVLDO0_ENABLE controls SDR1.  Hardware controls are ignored.
/// 1: HVLDO0_ENABLE is ignored and a hardware signal is used instead.  Selecting this on products that don't have supporting hardware will disable HVLDO0.
    HVLDO0_HW_SEL OFFSET(30) NUMBITS(1) [],
    /// HVLDO0 enable
    HVLDO0_ENABLE OFFSET(31) NUMBITS(1) []
],
TST_XRES_SECURE [
    /// Data byte to be set into either SECURE TEST or FIRMWARE TEST key.  Must not be changed in the same write that is toggling any of the *_WR bits below,
    DATA8 OFFSET(0) NUMBITS(8) [],
    /// Latch enables for each of the 4 bytes in the 32-bit FIRMWARE TEST key.  Must be toggled high and then low while keeping DATA8 to the correct value.
    FW_WR OFFSET(8) NUMBITS(4) [],
    /// Latch enables for each of the 4 bytes in the 32-bit SECURE TEST key.  Must be toggled high and then low while keeping DATA8 to the correct value.
    SECURE_WR OFFSET(16) NUMBITS(4) [],
    /// Indicates that the 32-bit FIRMWARE TEST key is observing the correct key.  Firmware key is reset by (A)XRES and STRUCT_XRES.
    FW_KEY_OK OFFSET(29) NUMBITS(1) [],
    /// Indicates that the 32-bit SECURE TEST key is observing the correct key.  Secure key is not reset, but it will establish low after a deep power cycle that causes it to lose its written state.
    SECURE_KEY_OK OFFSET(30) NUMBITS(1) [],
    /// Disables the SECURE TEST key entry capability until next reset.   Must not be set in the same write when any of the above *_WR bits are set or toggling.
    SECURE_DISABLE OFFSET(31) NUMBITS(1) []
],
PWR_TRIM_CBUCK_CTL [
    /// The CBUCK voltage setting to use during DEEPSLEEP.
    CBUCK_DPSLP_VSEL OFFSET(0) NUMBITS(5) [],
    /// The CBUCK mode setting to use during DEEPSLEEP.
    CBUCK_DPSLP_MODE OFFSET(8) NUMBITS(5) []
],
PWR_TRIM_PWRSYS_CTL [
    /// Trim for the Active-Regulator.  This sets the output voltage level.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.  The nominal output voltage is vccd=850mV + ACT_REG_TRIM*12.5mV.  The actual output voltage will vary depending on conditions and load.  The following settings are explicitly shown for convenience, and other values may be calculated using the formula:
/// 5'h04: 900mV (nominal)
/// 5'h0C: 1000mV (nominal)
/// 5'h14: 1100mV (nominal)
/// 5'h1C: 1200mV (nominal)
    ACT_REG_TRIM OFFSET(0) NUMBITS(5) [],
    /// Controls the tradeoff between output current and internal operating current for the Active Regulator.  The maximum output current depends on the silicon implementation, but an application may limit its maximum current to less than that.  This may allow a reduction in the internal operating current of the regulator.  The regulator internal operating current depends on the boost setting:
/// 2'b00: 50uA
/// 2'b01: 100uA
/// 2'b10: 150uA
/// 2'b11: 200uA
///
/// The allowed setting is a lookup table based on the chip-specific maximum (set in factory) and an application-specific maximum (set by customer).  The defaults are set assuming the application consumes the maximum allowed by the chip.
/// 50mA chip: 2'b00 (default);
/// 100mA chip: 2'b00 (default);
/// 150mA chip: 50..100mA app => 2'b00, 150mA app => 2'b01 (default);
/// 200mA chip: 50mA app => 2'b00, 100..150mA app => 2'b01,  200mA app => 2'b10 (default);
/// 250mA chip: 50mA app => 2'b00, 100..150mA app => 2'b01,  200..250mA app => 2'b10 (default);
/// 300mA chip: 50mA app => 2'b00, 100..150mA app => 2'b01, 200..250mA app => 2'b10, 300mA app => 2'b11 (default);
///
/// This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    ACT_REG_BOOST OFFSET(30) NUMBITS(2) []
],
PWR_TRIM_PWRSYS_CTL2 [
    /// Trim for the DeepSleep-Regulator applied during DEEPSLEEP mode.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
/// 0: 0.825V
/// 1: 0.850V
/// 2: 0.875V
/// 3: 0.900V
/// 4: 0.925V
/// 5: 1.050V
/// 6: 1.100V
/// 7: 1.150V
    DPSLP_REG_TRIM OFFSET(8) NUMBITS(3) [],
    /// Trim for the Retention-Regulator (if present) applied during DEEPSLEEP mode.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    RET_REG_TRIM OFFSET(12) NUMBITS(3) [],
    /// Trim for the Nwell-Regulator (if present) applied during DEEPSLEEP mode.  Nwell trim is always forced to zero during (LP)ACTIVE/(LP)SLEEP modes.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    NWELL_REG_TRIM OFFSET(16) NUMBITS(3) [],
    /// Trim for the DeepSleep-Regulator applied during (LP)ACTIVE/(LP)SLEEP modes.  These are expected to be constant but provided as registers for risk mitigation.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    DPSLP_REG_ACT_TRIM OFFSET(20) NUMBITS(3) [],
    /// Trim for the Retention-Regulator (if present) applied during (LP)ACTIVE/(LP)SLEEP modes.  These are expected to be constant but provided as registers for risk mitigation.  This register is only reset by XRES, HIBERNATE wakeup, or supply supervision reset.
    RET_REG_ACT_TRIM OFFSET(24) NUMBITS(3) []
],
CLK_TRIM_ECO_CTL [
    /// Current Trim
    ITRIM OFFSET(16) NUMBITS(6) []
],
CLK_TRIM_ILO1_CTL [
    /// ILO1 frequency trims.  LSB step size is 1.5 percent (typical) of the frequency.
    ILO1_FTRIM OFFSET(0) NUMBITS(6) [],
    /// ILO1 internal monitor trim.
    ILO1_MONTRIM OFFSET(8) NUMBITS(4) []
],
WDT_CTL [
    /// Enable this watchdog timer.  This field is retained during DEEPSLEEP and HIBERNATE modes.
    WDT_EN OFFSET(0) NUMBITS(1) [],
    /// Select source for WDT.  Not all products support all clock sources.  Selecting a clock source that is not supported will result in undefined behavior.  Writes to this field are ignored unless the WDT is unlock using WDT_LOCK register.  It takes four cycles of the originally selected clock to switch away from it.  Do not disable the original clock during this time.
    WDT_CLK_SEL OFFSET(4) NUMBITS(2) [
        /// ILO - Internal Low-speed Oscillator
        ILOInternalLowSpeedOscillator = 0,
        /// PILO - Precision ILO. If present, if present
        PILOPrecisionILOIfPresentIfPresent = 1,
        /// BAK - Selected clk_bak source, if present.  See BACKUP_CTL.  This choice is not recommended for applications that rely upon the watchdog timer for safety or security, unless the product supports clock supervision of clk_bak (CSV_BAK).  Generation of clk_bak is not protected by WDT_LOCK and is in a different memory region with potentially different security attributes.
        BAK = 2
    ],
    /// Prohibits writing to WDT_*, CLK_ILO_CONFIG, CLK_SELECT.LFCLK_SEL, and CLK_TRIM_ILO_CTL registers when not equal 0.  Requires at least two different writes to unlock.  A change in WDT_LOCK takes effect beginning with the next write cycle.
/// Note that this field is 2 bits to force multiple writes only.  It represents only a single write protect signal protecting all those registers at the same time.  WDT will lock on any reset.  This field is not retained during DEEPSLEEP or HIBERNATE mode, so the WDT will be locked after wakeup from these modes.
    WDT_LOCK OFFSET(30) NUMBITS(2) [
        /// No effect
        NoEffect = 0,
        /// Clears bit 0
        ClearsBit0 = 1,
        /// Clears bit 1
        ClearsBit1 = 2,
        /// Sets both bits 0 and 1
        SetsBothBits0And1 = 3
    ]
],
WDT_CNT [
    /// Current value of WDT Counter.  The write feature of this register is for engineering use (DfV), have no synchronization, and can only be applied when the WDT is fully off.  When writing, the value is updated immediately in the WDT counter, but it will read back as the old value until this register resynchronizes just after the negedge of ILO.  Writes will be ignored if they occur when the WDT is enabled.
    COUNTER OFFSET(0) NUMBITS(32) []
],
WDT_MATCH [
    /// Match value for Watchdog counter.  Every time WDT_COUNTER reaches MATCH an interrupt is generated.  Two unserviced interrupts will lead to a system reset (i.e. at the third match).
    MATCH OFFSET(0) NUMBITS(32) []
],
WDT_MATCH2 [
    /// The bit index to be considered the MSB for matching.  Bit indices above this setting are NOT checked against MATCH.  This value provides control over the time-to-reset of the watchdog (which happens after 3 successive matches).  The four LSBs cannot be ignored for matching.  Settings <3 behave like a setting of 3.  If the setting is higher than the number of bits in the WDT counter, all actual bits in the counter are matched.
    IGNORE_BITS_ABOVE OFFSET(0) NUMBITS(5) []
],
CSV_REF_LIMIT [
    /// Cycle time lower limit.  Set the lower limit -1, in reference clock cycles, before the next monitored clock event is allowed to happen.  If a monitored clock event happens before this limit is reached a CSV error is detected.
/// LOWER must be at least 1 less than UPPER. In case the clocks are asynchronous LOWER must be at least 3 less than UPPER.
    LOWER OFFSET(0) NUMBITS(16) [],
    /// Cycle time upper limit.  Set the upper limit -1, in reference clock cycles, before (or same time) the next monitored clock event must happen.  If a monitored clock event does not happen before this limit is reached, or does not happen at all (clock loss), a CSV error is detected.
    UPPER OFFSET(16) NUMBITS(16) []
],
CSV_MON_CTL [
    /// Period time.  Set the Period -1, in monitored clock cycles, before the next monitored clock event happens.
/// PERIOD <=  (UPPER+1) / FREQ_RATIO -1, with FREQ_RATIO = (Reference frequency / Monitored frequency)
/// In case the clocks are asynchronous: PERIOD <=  UPPER / FREQ_RATIO -1
/// Additionally margin must be added for accuracy of both clocks.
    PERIOD OFFSET(0) NUMBITS(16) []
],
CSV_REF_CTL [
    /// Startup delay time -1 (in reference clock cycles), after enable or DeepSleep wakeup, from reference clock start to monitored clock start.
/// At a minimum (both clocks running): STARTUP >= (PERIOD +3) * FREQ_RATIO - UPPER, with FREQ_RATIO = (Reference frequency / Monitored frequency)
/// On top of that the actual clock startup delay and the margin for accuracy of both clocks must be added.
    STARTUP OFFSET(0) NUMBITS(16) [],
    /// Specifies the action taken when an anomaly is detected on the monitored clock.  CSV in DeepSleep domain always do a Fault report (which also wakes up the system).
    CSV_ACTION OFFSET(30) NUMBITS(1) [
        /// Generate a fault
        GenerateAFault = 0,
        /// Cause a power reset. This should only be used for clk_hf0.
        CauseAPowerResetThisShouldOnlyBeUsedForClk_hf0 = 1
    ],
    /// Enables clock supervision, both frequency and loss.
/// CSV in Active domain: Clock supervision is reset during DeepSleep and Hibernate modes.  When enabled it begins operating automatically after a DeepSleep wakeup, but it must be reconfigured after Hibernate wakeup.
/// CSV in DeepSleep domain: Clock supervision is reset during Hibernate mode.  It must be reconfigured after Hibernate wakeup.
///
/// A CSV error detection is reported to the Fault structure, or instead it can generate a power reset.
    CSV_EN OFFSET(31) NUMBITS(1) []
],
CLK_DPLL_LP_CONFIG [
    /// Control bits for feedback divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0-15: illegal (undefined behavior)
/// 16: divide by 16
/// ...
/// 125: divide by 125
/// >125: illegal (undefined behavior)
    FEEDBACK_DIV OFFSET(0) NUMBITS(8) [],
    /// Control bits for reference divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0: illegal (undefined behavior)
/// 1: divide by 1
/// ...
/// 16: divide by 16
/// others: illegal (undefined behavior)
    REFERENCE_DIV OFFSET(8) NUMBITS(5) [],
    /// Control bits for Output divider.  Set the divide value before enabling the PLL, and do not change it while PLL is enabled.
/// 0: illegal (undefined behavior)
/// 1: divide by 1.  Suitable for direct usage as HFCLK source.
/// 2: divide by 2.  Suitable for direct usage as HFCLK source.
/// ...
/// 16: divide by 16.  Suitable for direct usage as HFCLK source.
/// >16: illegal (undefined behavior)
    OUTPUT_DIV OFFSET(16) NUMBITS(5) [],
    /// DCO code coefficient during SAR operation; this trim bit is Fpfd frequency dependent
/// 0: multilply by 16, Fpfd <= 8MHZ
/// 1: mulitply by 28, Fpfd > 8MHz
    PLL_DCO_CODE_MULT OFFSET(27) NUMBITS(1) [],
    /// Bypass mux located just after PLL output.  This selection is glitch-free and can be changed while the PLL is running.  When changing BYPASS_SEL, do not turn off the reference clock or PLL clock for five cycles (whichever is slower).
    BYPASS_SEL OFFSET(28) NUMBITS(2) [
        /// Automatic using lock indicator.  When unlocked, automatically selects PLL reference input (bypass mode).  When locked, automatically selects PLL output.  If ENABLE=0, automatically selects PLL reference input.
        AUTO = 0,
        /// Similar to AUTO, except the clock is gated off when unlocked.  This is compatible with clock supervision, because the supervisors allow no clock during startup (until a timeout occurs), and the clock targets the proper frequency whenever it is running.  If ENABLE=0, no clock is output.
        LOCKED_OR_NOTHING = 1,
        /// Select PLL reference input (bypass mode).  Ignores lock indicator
        SelectPLLReferenceInputBypassModeIgnoresLockIndicator = 2,
        /// Select PLL output.  Ignores lock indicator.  If ENABLE=0, no clock is output.
        SelectPLLOutputIgnoresLockIndicatorIfENABLE0NoClockIsOutput = 3
    ],
    /// Master enable for PLL.  Setup FEEDBACK_DIV, REFERENCE_DIV, and OUTPUT_DIV at least one cycle before setting ENABLE=1.
///
/// fOUT = (FEEDBACK_DIV + FRAC_EN*FRAC_DIV/2^24)  * (fREF / REFERENCE_DIV) / (OUTPUT_DIV)
///
/// 0: Block is disabled.  When the PLL disables, hardware controls the bypass mux as described in BYPASS_SEL, before disabling the PLL circuit.
/// 1: Block is enabled
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_DPLL_LP_CONFIG2 [
    /// Control bits for fractional divider.  This value is interpreted as a fraction of the PFD frequency, i.e. fPFD * (FRAC_DIV/2^24).  This field can be dynamically updated within the 1000ppm control limit. It takes up to 115 AHB cycles to transfer the setting to the PLL, and writes that occur faster may be silently ignored and require the application to write again after the previous update has finished.  Reading the register returns the accepted value.  The PLL will start targeting the new value, but it may take significant time (milliseconds) to stabilize at the new average value.  Do not change the FRAC_DIV setting while the PLL is initially locking.
    FRAC_DIV OFFSET(0) NUMBITS(24) [],
    /// N/A
    FRAC_DITHER_EN OFFSET(28) NUMBITS(3) [],
    /// Enables fractional division mode.
    FRAC_EN OFFSET(31) NUMBITS(1) []
],
CLK_DPLL_LP_CONFIG3 [
    /// N/A
    SSCG_DEPTH OFFSET(0) NUMBITS(10) [],
    /// N/A
    SSCG_RATE OFFSET(16) NUMBITS(3) [],
    /// N/A
    SSCG_DITHER_EN OFFSET(24) NUMBITS(1) [],
    /// N/A
    SSCG_MODE OFFSET(28) NUMBITS(1) [],
    /// Enables spreading mode.
/// When SSCG mode is enabled, DPLL_LP_TEST4.PLL_DIS_FAST_LOCK should be set to 1 to disable fast re-lock.
    SSCG_EN OFFSET(31) NUMBITS(1) []
],
CLK_DPLL_LP_CONFIG4 [
    /// Initial DCO code.  It is recommended to leave this at the default setting. This setting only has effect in open loop mode.  See DPLL BROS regarding test modes.
    DCO_CODE OFFSET(0) NUMBITS(11) [],
    /// N/A
    ACC_MODE OFFSET(16) NUMBITS(2) [],
    /// N/A
    TDC_MODE OFFSET(18) NUMBITS(2) [],
    /// This value is a number of PFD clocks in relation to the DCO count.  Can change the number of counts, set by p_div in integer mode, by the values of -1, +1, or +2.
/// 0: 0
/// 1: -1
/// 2: +1
/// 3: +2
///
/// In integer and SSCG mode, PLL_TG must be set to 0.
/// If in fractional mode, PLL_TG must be 0 if frac_ratio <= 0.5 (pll_frac <= 2^23), PLL_TG must be 2 if frac_ratio > 0.5 (pll_frac > 2^23).
    PLL_TG OFFSET(20) NUMBITS(2) [],
    /// Control signal for switching to stable filter coefficients (PLL_KP_TRIM/PLL_KI_TRIM)
/// 0: PLL_ACC_PHASE_CNT_DONE
/// 1: PLL_LOCK
    ACC_CNT_LOCK OFFSET(24) NUMBITS(1) []
],
CLK_DPLL_LP_CONFIG5 [
    /// Gain of P/I loop filter integrator path for INT operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_INT OFFSET(0) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path for INT operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_INT OFFSET(8) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for INT operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_ACC_INT OFFSET(16) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for INT operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_ACC_INT OFFSET(24) NUMBITS(7) []
],
CLK_DPLL_LP_CONFIG6 [
    /// Gain of P/I loop filter proportional path for FRACT operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_FRACT OFFSET(0) NUMBITS(7) [],
    /// Gain of P/I loop filter proportional path for FRACT operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_FRACT OFFSET(8) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for FRACT operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_ACC_FRACT OFFSET(16) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for FRACT operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_ACC_FRACT OFFSET(24) NUMBITS(7) []
],
CLK_DPLL_LP_CONFIG7 [
    /// Gain of P/I loop filter proportional path for SSCG operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_SSCG OFFSET(0) NUMBITS(7) [],
    /// Gain of P/I loop filter proportional path for SSCG operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_SSCG OFFSET(8) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for SSCG operation.  Gain coefficient is 2^KI, eg. 0=>1, 15=>32768.
    KI_ACC_SSCG OFFSET(16) NUMBITS(7) [],
    /// Gain of P/I loop filter integrator path during cold start for SSCG operation.  Gain coefficient is 2^KP, eg. 0=>1, 15=>32768.
    KP_ACC_SSCG OFFSET(24) NUMBITS(7) []
],
CLK_DPLL_LP_STATUS [
    /// PLL Lock Indicator
    LOCKED OFFSET(0) NUMBITS(1) [],
    /// This bit sets whenever the PLL Lock bit goes low, and stays set until cleared by firmware.
/// Note: When disabling DPLL via register write, UNLOCK_OCCURRED will set. Therefore, after enabling DPLL and DPLL successfully locks, FW should clear UNLOCK_OCCURRED flag to prevent a false positive that would indicate that DPLL erroneously unlocked.
    UNLOCK_OCCURRED OFFSET(1) NUMBITS(1) []
],
RAM_TRIM_TRIM_RAM_CTL [
    /// N/A
    TRIM OFFSET(0) NUMBITS(32) []
],
RAM_TRIM_TRIM_ROM_CTL [
    /// N/A
    TRIM OFFSET(0) NUMBITS(32) []
],
CLK_TRIM_DPLL_LP_DPLL_LP_CTL [
    /// Successive Approximation Register (SAR) configuration.  Ignored when SAR_DIS==1.
/// 0x1...0xB: Number of cycles until SAR stops.
/// others: illegal
/// Note: For initial lock, default should be left at 0x3. After successful lock with code write = 0, FW should set to 0x1 to improve recovery time for successive locks.
    SAR_CYCLE_STOP OFFSET(4) NUMBITS(4) [],
    /// Disable Successive Approximation Register during locking.
/// 0: Use SAR.
/// 1: Disable SAR.
    SAR_DIS OFFSET(8) NUMBITS(1) [],
    /// SAR FSM enable. Always set to 1 for functional mode. This register was kept in the design (and moved to hidden trim register) to increase coverage on the hard IP, and could be removed once the hard IP input is removed.
    PLL_SAR_FSM_EN OFFSET(9) NUMBITS(1) [],
    /// Trim for PLL local regulator for DCO
/// 0 0.906V
/// 1 0.786V
/// 2 0.846V
/// 3 0.876V
/// 4 0.936V
/// 5 0.966V
    LDO_DCO_TRIM OFFSET(12) NUMBITS(3) [],
    /// 00': disable, '01' first order SigmaDelta enabled;'10' third order SigmaDelta enabled;'11' RSVD
    PLL_DCO_SD_SEL OFFSET(16) NUMBITS(2) [],
    /// pll ldo peripheri voltage trim (scan mode - can be '0' or '1' - not High Z)
    LDO_PERI_TRIM OFFSET(19) NUMBITS(3) [],
    /// Order of the delta-sigma modulator used for fractional mode.
/// If DPLL_LP_STRUCT_Regs.CONFIG4.ACC_MODE = 00b:
/// 0: 4th order
/// 1: 3rd order
/// If DPLL_LP_STRUCT_Regs.CONFIG4.ACC_MODE = 01b:
/// 0: 2nd order
/// 1: 1st order
    PLL_FRAC_ORDER OFFSET(22) NUMBITS(1) [],
    /// Manual isolation control for PLL .  This field is ignored when ENABLE_CNT==1.  When controlling manually, de-assert >= 3us after ENABLE=1.  Assertion can happen in cycle just before ENABLE=0.
/// 0: Isolate outputs
/// 1: Do not isolate outputs
    ISOLATE_N OFFSET(23) NUMBITS(1) [],
    /// Terminal count for the stabilization counter handles PLL_ISOLATE_N
    ISOLATE_CNT OFFSET(24) NUMBITS(6) [],
    /// Enable for the PLL hardware sequencer.
/// 0: Disables the hardware sequencer.  Before enabling the DPLL manually make sure to have DPLL_LP_STRUCT_Regs->CONFIG->BYPASS_SEL at default value to avoid unwanted glitch on clock HF root.  When disabling the PLL, first deselect it using .BYPASS_SEL=PLL_REF, wait at least six PLL clock cycles, and then disable it with .ENABLE=0.  Before entering DEEPSLEEP, firmware must switch to another clock source and disable the PLL.
/// 1: Enables the hardware sequencer.  The sequencer handles all PLL enable/disable transitions, including around DEEPSLEEP entry/exit.
    ENABLE_CNT OFFSET(31) NUMBITS(1) []
],
CLK_TRIM_DPLL_LP_DPLL_LP_CTL3 [
    /// counter for phase accelerate cycles during PLL wakeup for INT and FRAC modes
/// Optimal settings:
/// 4 --> 5 MHz: 127
/// 5-->6 MHz: 157
/// 6-->7 MHz: 187
/// 7-->8 MHz: 217
/// 8-->9 MHz: 255
/// 9-->10 MHz: 286
/// 10-->11 MHz: 317
/// 11-->12 MHz: 348
/// 12-->13 MHz: 379
/// 13-->14 MHz: 410
/// 14-->15 MHz: 441
/// 15-->16 MHz: 472
/// 16 MHz: 	511
    PHASE_ACC_CNT OFFSET(0) NUMBITS(10) [],
    /// counter for phase accelerate cycles during PLL wakeup SSCG mode
/// Optimal settings:
/// 4 --> 5 MHz: 127
/// 5-->6 MHz: 157
/// 6-->7 MHz: 187
/// 7-->8 MHz: 217
/// 8-->9 MHz: 255
/// 9-->10 MHz: 286
/// 10-->11 MHz: 317
/// 11-->12 MHz: 348
/// 12-->13 MHz: 379
/// 13-->14 MHz: 410
/// 14-->15 MHz: 441
/// 15-->16 MHz: 472
/// 16 MHz: 	511
    PHASE_ACC_CNT_SSCG OFFSET(16) NUMBITS(10) []
],
CLK_TRIM_DPLL_LP_DPLL_LP_CTL4 [
    /// Wait time from when static phase error is within the lock window until lock signal asserts.
/// 0: 0 PFD clocks
/// 1: 2 PFD clocks
/// 2: 3 PFD clocks
/// 3: 4 PFD clocks
    LOCK_WAIT_FALL OFFSET(0) NUMBITS(2) [],
    /// Wait time from when static phase error is within the lock window until lock signal asserts.
/// 0: Illegal
/// 1: 1 PFD clocks
/// 2: 2 PFD clocks
/// ...
/// 1023: 1023 PFD clocks
/// Optimal settings
/// f, MHz         With Code Write                    Without Code Write
/// ---------      ------------------------------------      ------------------------------------
///  4-->5                      6                                         33
///  5-->6                      6                                         40
///  6-->7                      6                                         49
///  7-->8                      8                                         60
///  8-->9                      8                                         85
///  9-->10                   11                                        95
/// 10-->11                  12                                       105
/// 11-->12                  14                                       115
/// 12-->13                  15                                       125
/// 13-->14                  17                                       135
/// 14-->15                  18                                       145
/// 15-->16                  20                                       155
///         16                  21                                       165
/// Notes:
/// 1. For initial DPLL locking or when DPLL Fast Lock is disabled (PLL_DIS_FAST_LOCK=1), user must program DPLL_CTL4.LOCK_WAIT_RISE using appropriate value for 'Without Code Write' for corresponding frequency range.
/// 2. After initial DPLL locking, when DPLL Fast Lock is enabled (PLL_DIS_FAST_LOCK=0), before DEEPSLEEP entry, FW must poll the DPLL_LP.STATUS.LOCKED bit to acknowledge successful DPLL lock, then must program DPLL_LP_CTL4.LOCK_WAIT_RISE using appropriate value for 'With Code Write' for corresponding frequency range.
    LOCK_WAIT_RISE OFFSET(4) NUMBITS(10) []
],
CLK_TRIM_DPLL_LP_DPLL_LP_TEST4 [
    /// FW can read the code saved by the DPLL before deepsleep.Use PLL_READ_EN to get updated DCO_CODE from the DPLL: To read properly please refer to PLL_USER_DCO_CODE_RD_EN/PLL_READ_EN.
/// Before any read, FW should check first that PLL_READ_EN is 0.
/// To account for Clock Domain Crossings, FW should wait at least 3 CLK_IMO cycles (3*125ns=375ns) between SW write and SW read operation.
    PLL_USER_DCO_CODE OFFSET(0) NUMBITS(14) [],
    /// 0: Enables DPLL Fast Lock
/// 1: Disables DPLL Fast Lock
///
/// When PLL_DIS_FAST_LOCK=0, if DPLL successfully locks prior to a DEEPSLEEP entry:
/// - HW will read the DCO code before DEEPSLEEP entry, then write the DCO code upon DEEPSLEEP exit, to reduce DPLL lock time.
/// - FW must follow the procedure described under DPLL_LP_CTL4.LOCK_WAIT_RISE Notes: 2
    PLL_DIS_FAST_LOCK OFFSET(14) NUMBITS(1) [],
    /// This bit is a self clear bit. The FW writes 1 to get an updated value in PLL_USER_DCO_CODE.HW will self clear this bit to indicate that the PLL_USER_DCO_CODE be read.
    PLL_READ_EN OFFSET(15) NUMBITS(1) [],
    /// Reduce KI,KP coefficient during PLL deepsleep wakeup integer.
    PHASE_ACC_USER_WRITE_INT OFFSET(16) NUMBITS(7) [],
    /// reduce KI,KP coefficient during PLL deepsleep wakeup fract.
    PHASE_ACC_USER_WRITE_FRACT OFFSET(23) NUMBITS(7) []
],
MCWDT_CNTLOW [
    /// Current value of sub-counter 0 for this MCWDT.  Software writes are ignored when the sub-counter is enabled.
    WDT_CTR0 OFFSET(0) NUMBITS(16) [],
    /// Current value of sub-counter 1 for this MCWDT.  Software writes are ignored when the sub-counter is enabled
    WDT_CTR1 OFFSET(16) NUMBITS(16) []
],
MCWDT_CNTHIGH [
    /// Current value of sub-counter 2 for this MCWDT.  Software writes are ignored when the sub-counter is enabled
    WDT_CTR2 OFFSET(0) NUMBITS(32) []
],
MCWDT_MATCH [
    /// Match value for sub-counter 0 of this MCWDT
    WDT_MATCH0 OFFSET(0) NUMBITS(16) [],
    /// Match value for sub-counter 1 of this MCWDT
    WDT_MATCH1 OFFSET(16) NUMBITS(16) []
],
MCWDT_CONFIG [
    /// Watchdog Counter Action on Match.  Action is taken on the next increment after the values match (WDT_CTR0=WDT_MATCH0).
    WDT_MODE0 OFFSET(0) NUMBITS(2) [
        /// Do nothing
        DoNothing = 0,
        /// Assert WDT_INTx
        AssertWDT_INTx = 1,
        /// Assert WDT Reset
        AssertWDTReset = 2,
        /// Assert WDT_INTx, assert WDT Reset after 3rd unhandled interrupt
        AssertWDT_INTxAssertWDTResetAfter3rdUnhandledInterrupt = 3
    ],
    /// Clear Watchdog Counter when WDT_CTR0=WDT_MATCH0. In other words WDT_CTR0 divides LFCLK by (WDT_MATCH0+1).
/// 0: Free running counter
/// 1: Clear on match.  In this mode, the minimum legal setting of WDT_MATCH0 is 1.
    WDT_CLEAR0 OFFSET(2) NUMBITS(1) [],
    /// Cascade Watchdog Counters 0,1.  Counter 1 increments the cycle after WDT_CTR0=WDT_MATCH0.
/// 0: Independent counters
/// 1: Cascaded counters
    WDT_CASCADE0_1 OFFSET(3) NUMBITS(1) [],
    /// Watchdog Counter Action on service before lower limit.
    WDT_LOWER_MODE0 OFFSET(4) NUMBITS(2) [
        /// Do nothing
        DoNothing = 0,
        /// Assert WDT_INTx
        AssertWDT_INTx = 1,
        /// Assert WDT Reset
        AssertWDTReset = 2
    ],
    /// Carry out behavior that applies when WDT_CASCADE0_1==1.  This bit is not used when WDT_CASCADE0_1==0.
/// 0: carry out on counter 0 match.
/// 1: carry out on counter 0 roll-over.
    WDT_CARRY0_1 OFFSET(6) NUMBITS(1) [],
    /// Specifies matching behavior when WDT_CASCADE0_1==1.  When WDT_CASCADE0_1==0, this bit is not used and match is based on counter 1 alone.
/// 0: Match based on counter 1 alone.
/// 1: Match based on counter 1 and counter 0 matching simultaneously.
    WDT_MATCH0_1 OFFSET(7) NUMBITS(1) [],
    /// Watchdog Counter Action on Match.  Action is taken on the next increment after the values match (WDT_CTR1=WDT_MATCH1).
    WDT_MODE1 OFFSET(8) NUMBITS(2) [
        /// Do nothing
        DoNothing = 0,
        /// Assert WDT_INTx
        AssertWDT_INTx = 1,
        /// Assert WDT Reset
        AssertWDTReset = 2,
        /// Assert WDT_INTx, assert WDT Reset after 3rd unhandled interrupt
        AssertWDT_INTxAssertWDTResetAfter3rdUnhandledInterrupt = 3
    ],
    /// Clear Watchdog Counter when WDT_CTR1==WDT_MATCH1. In other words WDT_CTR1 divides LFCLK by (WDT_MATCH1+1).
/// 0: Free running counter
/// 1: Clear on match.  In this mode, the minimum legal setting of WDT_MATCH1 is 1.
    WDT_CLEAR1 OFFSET(10) NUMBITS(1) [],
    /// Cascade Watchdog Counters 1,2.  Counter 2 increments the cycle after WDT_CTR1=WDT_MATCH1.  It is allowed to cascade all three WDT counters.
/// 0: Independent counters
/// 1: Cascaded counters.  When cascading all three counters, WDT_CLEAR1 must be 1.
    WDT_CASCADE1_2 OFFSET(11) NUMBITS(1) [],
    /// Watchdog Counter Action on service before lower limit.
    WDT_LOWER_MODE1 OFFSET(12) NUMBITS(2) [
        /// Do nothing
        DoNothing = 0,
        /// Assert WDT_INTx
        AssertWDT_INTx = 1,
        /// Assert WDT Reset
        AssertWDTReset = 2
    ],
    /// Carry out behavior that applies when WDT_CASCADE1_2==1.  This bit is not used when WDT_CASCADE1_2==0.
/// 0: carry out on counter 1 match.
/// 1: carry out on counter 1 roll-over.
    WDT_CARRY1_2 OFFSET(14) NUMBITS(1) [],
    /// Specifies matching behavior when WDT_CASCADE1_2==1.  When WDT_CASCADE1_2==0, this bit is not used and match is based on counter 2 alone.
/// 0: Match based on counter 2 alone.
/// 1: Match based on counter 2 and counter 1 matching simultaneously.
    WDT_MATCH1_2 OFFSET(15) NUMBITS(1) [],
    /// Watchdog Counter 2 Mode.
    WDT_MODE2 OFFSET(16) NUMBITS(1) [
        /// Free running counter with no interrupt requests
        FreeRunningCounterWithNoInterruptRequests = 0,
        /// Free running counter with interrupt request that occurs one LFCLK cycle after the specified bit in CTR2 toggles (see WDT_BITS2).
        INT = 1
    ],
    /// Bit to observe for WDT_INT2:
/// 0: Assert after bit0 of WDT_CTR2 toggles (one int every tick)
/// ...
/// 31: Assert after bit31 of WDT_CTR2 toggles (one int every 2^31 ticks)
    WDT_BITS2 OFFSET(24) NUMBITS(5) []
],
MCWDT_CTL [
    /// Enable subcounter 0.  May take up to 2 LFCLK cycles to take effect.
/// 0: Counter is disabled (not clocked)
/// 1: Counter is enabled (counting up)
    WDT_ENABLE0 OFFSET(0) NUMBITS(1) [],
    /// Indicates actual state of counter.  May lag WDT_ENABLE0 by up to two LFCLK cycles.
    WDT_ENABLED0 OFFSET(1) NUMBITS(1) [],
    /// Resets counter 0 back to 0000.  Hardware will reset this bit after counter was reset.  This will take up to one LFCLK cycle to take effect.
    WDT_RESET0 OFFSET(3) NUMBITS(1) [],
    /// Enable subcounter 1.  May take up to 2 LFCLK cycles to take effect.
/// 0: Counter is disabled (not clocked)
/// 1: Counter is enabled (counting up)
    WDT_ENABLE1 OFFSET(8) NUMBITS(1) [],
    /// Indicates actual state of counter.  May lag WDT_ENABLE1 by up to two LFCLK cycles.
    WDT_ENABLED1 OFFSET(9) NUMBITS(1) [],
    /// Resets counter 1 back to 0000.  Hardware will reset this bit after counter was reset.  This will take up to one LFCLK cycle to take effect.
    WDT_RESET1 OFFSET(11) NUMBITS(1) [],
    /// Enable subcounter 2.  May take up to 2 LFCLK cycles to take effect.
/// 0: Counter is disabled (not clocked)
/// 1: Counter is enabled (counting up)
    WDT_ENABLE2 OFFSET(16) NUMBITS(1) [],
    /// Indicates actual state of counter.  May lag WDT_ENABLE2 by up to two LFCLK cycles.
    WDT_ENABLED2 OFFSET(17) NUMBITS(1) [],
    /// Resets counter 2 back to 0000.  Hardware will reset this bit after counter was reset.  This will take up to one LFCLK cycle to take effect.
    WDT_RESET2 OFFSET(19) NUMBITS(1) []
],
MCWDT_INTR [
    /// MCWDT Interrupt Request for sub-counter 0.  This bit is set by hardware as configured by this registers.  This bit must be cleared by firmware.  Clearing this bit also prevents Reset from happening when WDT_MODE0=3.
    MCWDT_INT0 OFFSET(0) NUMBITS(1) [],
    /// MCWDT Interrupt Request for sub-counter 1.  This bit is set by hardware as configured by this registers.  This bit must be cleared by firmware.  Clearing this bit also prevents Reset from happening when WDT_MODE1=3.
    MCWDT_INT1 OFFSET(1) NUMBITS(1) [],
    /// MCWDT Interrupt Request for sub-counter 2.  This bit is set by hardware as configured by this registers.  This bit must be cleared by firmware.  Clearing this bit also prevents Reset from happening when WDT_MODE2=3.
    MCWDT_INT2 OFFSET(2) NUMBITS(1) []
],
MCWDT_INTR_SET [
    /// Set interrupt for MCWDT_INT0
    MCWDT_INT0 OFFSET(0) NUMBITS(1) [],
    /// Set interrupt for MCWDT_INT1
    MCWDT_INT1 OFFSET(1) NUMBITS(1) [],
    /// Set interrupt for MCWDT_INT2
    MCWDT_INT2 OFFSET(2) NUMBITS(1) []
],
MCWDT_INTR_MASK [
    /// Mask for sub-counter 0. This controls if the interrupt is forwarded to the CPU.
/// 0: Interrupt is masked (not forwarded).
/// 1: Interrupt is forwarded.
    MCWDT_INT0 OFFSET(0) NUMBITS(1) [],
    /// Mask for sub-counter 1. This controls if the interrupt is forwarded to the CPU.
/// 0: Interrupt is masked (not forwarded).
/// 1: Interrupt is forwarded.
    MCWDT_INT1 OFFSET(1) NUMBITS(1) [],
    /// Mask for sub-counter 2. This controls if the interrupt is forwarded to the CPU.
/// 0: Interrupt is masked (not forwarded).
/// 1: Interrupt is forwarded.
    MCWDT_INT2 OFFSET(2) NUMBITS(1) []
],
MCWDT_INTR_MASKED [
    /// Logical and of corresponding request and mask bits.
    MCWDT_INT0 OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    MCWDT_INT1 OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    MCWDT_INT2 OFFSET(2) NUMBITS(1) []
],
MCWDT_LOCK [
    /// Prohibits writing control and configuration registers related to this MCWDT when not equal 0 (as specified in the other register descriptions).  Requires at least two different writes to unlock.
    /// Note that this field is 2 bits to force multiple writes only.  Each MCWDT has a separate local lock.  LFCLK settings are locked by the global WDT_LOCK register, and this register has no effect on that.
    MCWDT_LOCK OFFSET(30) NUMBITS(2) [
        /// No effect
        NoEffect = 0,
        /// Clears bit 0
        ClearsBit0 = 1,
        /// Clears bit 1
        ClearsBit1 = 2,
        /// Sets both bits 0 and 1
        SetsBothBits0And1 = 3
    ]
],
MCWDT_LOWER_LIMIT [
    /// Lower limit for sub-counter 0 of this MCWDT
    WDT_LOWER_LIMIT0 OFFSET(0) NUMBITS(16) [],
    /// Lower limit for sub-counter 1 of this MCWDT
    WDT_LOWER_LIMIT1 OFFSET(16) NUMBITS(16) []
]
];
const SRSS_BASE: StaticRef<SrssRegisters> =
    unsafe { StaticRef::new(0x42200000 as *const SrssRegisters) };

pub struct Srss {
    registers: StaticRef<SrssRegisters>,
}

impl Srss {
    pub const fn new() -> Srss {
        Srss {
            registers: SRSS_BASE,
        }
    }

    pub fn wdt_unlock(&self) {
        self.registers.wdt_ctl.modify(WDT_CTL::WDT_LOCK::ClearsBit1);
        self.registers
            .wdt_ctl
            .modify(WDT_CTL::WDT_LOCK::SetsBothBits0And1);
    }

    pub fn init_clock(&self) {
        self.registers
            .clk_path_select3
            .modify(CLK_PATH_SELECT::PATH_MUX::IMOInternalRCOscillator);

        self.registers
            .clk_root_select0
            .modify(CLK_ROOT_SELECT::ENABLE::SET
                + CLK_ROOT_SELECT::ROOT_MUX::SelectPATH3
                + CLK_ROOT_SELECT::ROOT_DIV_INT::TransparentModeFeedThroughSelectedClockSourceWODividing);
    }

    pub fn init_clock_paths(&self) {
        self.registers
            .clk_dsi_select_0
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select0
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        self.registers
            .clk_dsi_select_1
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select2
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        self.registers
            .clk_dsi_select_2
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select2
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        self.registers
            .clk_dsi_select_3
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select3
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        self.registers
            .clk_dsi_select_4
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select4
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        self.registers
            .clk_dsi_select_5
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI6Dsi_out6);
        self.registers
            .clk_path_select5
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);

        // 6 to 0/IMO
        self.registers
            .clk_dsi_select_6
            .modify(CLK_DSI_SELECT::DSI_MUX::DSI0Dsi_out0);
        self.registers
            .clk_path_select6
            .modify(CLK_PATH_SELECT::PATH_MUX::DSI_MUX);
    }

    pub fn sys_init_enable_clocks(&self) {
        // set source
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ROOT_MUX::SelectPATH0);
        // set divider
        self.registers.clk_root_select2.modify(
            CLK_ROOT_SELECT::ROOT_DIV_INT::TransparentModeFeedThroughSelectedClockSourceWODividing,
        );
        // enable
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ROOT_MUX::SelectPATH0);
        self.registers.clk_root_select3.modify(
            CLK_ROOT_SELECT::ROOT_DIV_INT::TransparentModeFeedThroughSelectedClockSourceWODividing,
        );
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ROOT_MUX::SelectPATH0);
        self.registers.clk_root_select4.modify(
            CLK_ROOT_SELECT::ROOT_DIV_INT::TransparentModeFeedThroughSelectedClockSourceWODividing,
        );
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ENABLE::SET);
    }

    pub fn init_dpll(&self) {
        // self.registers.clk_dpll_lp0_config.modify(CLK_DPLL_LP_CONFIG::)
    }

    fn is_voltage_change_possible(&self) -> bool {
        const SRSS_TRIM_RAM_CTL_WC_MASK: u32 = 0x3 << 10;

        let trim_ram_check_val = self
            .registers
            .ram_trim_trim_ram_ctl
            .read(RAM_TRIM_TRIM_RAM_CTL::TRIM)
            & SRSS_TRIM_RAM_CTL_WC_MASK;

        self.registers
            .ram_trim_trim_ram_ctl
            .modify(RAM_TRIM_TRIM_RAM_CTL::TRIM.val(!SRSS_TRIM_RAM_CTL_WC_MASK));
        self.registers.ram_trim_trim_ram_ctl.modify(
            RAM_TRIM_TRIM_RAM_CTL::TRIM.val(
                self.registers
                    .ram_trim_trim_ram_ctl
                    .read(RAM_TRIM_TRIM_RAM_CTL::TRIM)
                    | ((!trim_ram_check_val) & SRSS_TRIM_RAM_CTL_WC_MASK),
            ),
        );

        return trim_ram_check_val
            != (self
                .registers
                .ram_trim_trim_ram_ctl
                .read(RAM_TRIM_TRIM_RAM_CTL::TRIM)
                & SRSS_TRIM_RAM_CTL_WC_MASK);
    }
}
