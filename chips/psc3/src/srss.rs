use kernel::utilities::registers::{
    interfaces::ReadWriteable, register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    /// SRSS Core Registers
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
        (0x140 => clk_output_fast: ReadWrite<u32>),
        /// Slow Clock Output Select Register
        (0x144 => clk_output_slow: ReadWrite<u32>),
        /// Clock Calibration Counter 1
        (0x148 => clk_cal_cnt1: ReadWrite<u32>),
        /// Clock Calibration Counter 2
        (0x14C => clk_cal_cnt2: ReadWrite<u32>),
        (0x150 => _reserved2),
        /// SRSS Interrupt Register
        (0x200 => srss_intr: ReadWrite<u32>),
        /// SRSS Interrupt Set Register
        (0x204 => srss_intr_set: ReadWrite<u32>),
        /// SRSS Interrupt Mask Register
        (0x208 => srss_intr_mask: ReadWrite<u32>),
        /// SRSS Interrupt Masked Register
        (0x20C => srss_intr_masked: ReadOnly<u32>),
        (0x210 => _reserved3),
        /// SRSS Additional Interrupt Register
        (0x300 => srss_aintr: ReadWrite<u32>),
        /// SRSS Additional Interrupt Set Register
        (0x304 => srss_aintr_set: ReadWrite<u32>),
        /// SRSS Additional Interrupt Mask Register
        (0x308 => srss_aintr_mask: ReadWrite<u32>),
        /// SRSS Additional Interrupt Masked Register
        (0x30C => srss_aintr_masked: ReadOnly<u32>),
        (0x310 => _reserved4),
        /// Debug Control Register
        (0x404 => boot_dlm_ctl: ReadWrite<u32>),
        /// Debug Control Register 2
        (0x408 => boot_dlm_ctl2: ReadWrite<u32>),
        /// Debug Status Register
        (0x40C => boot_dlm_status: ReadOnly<u32>),
        /// Soft Reset Trigger Register
        (0x410 => res_soft_ctl: ReadWrite<u32>),
        (0x414 => _reserved5),
        /// Boot Execution Status Register
        (0x418 => boot_status: ReadOnly<u32>),
        (0x41C => _reserved6),
        /// Warm Boot Entry Address
        (0x430 => boot_entry: ReadWrite<u32>),
        (0x434 => _reserved7),
        /// Hibernate Wakeup Mask Register
        (0x8A0 => pwr_hib_wake_ctl: ReadWrite<u32>),
        /// Hibernate Wakeup Polarity Register
        (0x8A4 => pwr_hib_wake_ctl2: ReadWrite<u32>),
        (0x8A8 => _reserved8),
        /// Hibernate Wakeup Cause Register
        (0x8AC => pwr_hib_wake_cause: ReadOnly<u32>),
        (0x8B0 => _reserved9),
        /// Power Mode Control
        (0x1000 => pwr_ctl: ReadWrite<u32>),
        /// Power Mode Control 2
        (0x1004 => pwr_ctl2: ReadWrite<u32>),
        /// HIBERNATE Mode Register
        (0x1008 => pwr_hibernate: ReadWrite<u32>),
        (0x100C => _reserved10),
        /// High Voltage / Low Voltage Detector (HVLVD) Configuration Register
        (0x1020 => pwr_lvd_ctl: ReadWrite<u32>),
        (0x1024 => _reserved11),
        /// Clock Path Select Register
        (0x1200 => clk_path_select_0: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x1204 => clk_path_select_1: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x1208 => clk_path_select_2: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x120C => clk_path_select_3: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x1210 => clk_path_select_4: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x1214 => clk_path_select_5: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        /// Clock Path Select Register
        (0x1218 => clk_path_select_6: ReadWrite<u32, CLK_PATH_SELECT::Register>),
        (0x121C => _reserved12),
        /// Clock Root Select Register
        (0x1240 => clk_root_select_0: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x1244 => clk_root_select_1: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x1248 => clk_root_select_2: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x124C => clk_root_select_3: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x1250 => clk_root_select_4: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x1254 => clk_root_select_5: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        /// Clock Root Select Register
        (0x1258 => clk_root_select_6: ReadWrite<u32, CLK_ROOT_SELECT::Register>),
        (0x125C => _reserved13),
        /// Clock Root Direct Select Register
        (0x1280 => clk_direct_select_0: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x1284 => clk_direct_select_1: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x1288 => clk_direct_select_2: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x128C => clk_direct_select_3: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x1290 => clk_direct_select_4: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x1294 => clk_direct_select_5: ReadWrite<u32>),
        /// Clock Root Direct Select Register
        (0x1298 => clk_direct_select_6: ReadWrite<u32>),
        // MISSING CSV, FLL, DPLL, TRIM, etc., can be added later if needed
        (0x129C => @END),
    }
}
register_bitfields![u32,
CLK_DSI_SELECT [
    /// Selects a DSI source or low frequency clock for use in a clock path.
    /// The output of this mux can be selected for clock PATH<i> using
    /// CLK_SELECT_PATH register. Using the output of this mux as HFCLK source
    /// will result in undefined behavior. It can be used to clocks to DSI or
    /// as reference inputs for the FLL/PLL, subject to the frequency limits
    /// of those circuits. This mux is not glitch free, so do not change the
    /// selection while it is an actively selected clock.
    DSI_MUX OFFSET(0) NUMBITS(5) [
        /// DSI0 - dsi_out[0]
        DSI0Dsi_out0 = 0,
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
        DSI6Dsi_out6 = 6,
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
        ILO1InternalLowSpeedOscillator1IfPresent = 20
    ]
],
CLK_PATH_SELECT [
    /// Selects a source for clock PATH<i>. Note that not all products support
    /// all clock sources. Selecting a clock source that is not supported will
    /// result in undefined behavior. It takes four cycles of the originally
    /// selected clock to switch away from it. Do not disable the original
    /// clock during this time.
    PATH_MUX OFFSET(0) NUMBITS(3) [
        /// IMO - Internal R/C Oscillator
        IMOInternalRCOscillator = 0,
        /// EXTCLK - External Clock Pin
        EXTCLKExternalClockPin = 1,
        /// ECO - External-Crystal Oscillator
        ECOExternalCrystalOscillator = 2,
        /// ALTHF - Alternate High-Frequency clock input (product-specific clock)
        ALTHFAlternateHighFrequencyClockInputProductSpecificClock = 3,
        /// DSI_MUX - Output of DSI mux for this path. Using a DSI source
        /// directly as root of HFCLK will result in undefined behavior.
        DSI_MUX = 4,
        /// LPECO - Low-Power External-Crystal Oscillator
        LPECOLowPowerExternalCrystalOscillator = 5,
        /// IHO - Internal High-speed Oscillator
        IHOInternalHighSpeedOscillator = 6
    ]
],
CLK_ROOT_SELECT [
    /// Selects a clock path for HFCLK<k> and SRSS DSI input <k>.
    /// The output of this mux goes to the direct mux (see CLK_DIRECT_SELECT).
    /// Use CLK_SELECT_PATH[i] to configure the desired path. The number of
    /// clock paths is product-specific, and selecting an unimplemented path is
    /// not supported. Some paths may have FLL or PLL available
    /// (product-specific), and the control and bypass mux selections of these
    /// are in other registers. Configure the FLL using CLK_FLL_CONFIG register.
    /// Configure a PLL using the related CLK_PLL_CONFIG[k] register. Note that
    /// not all products support all clock sources. Selecting a clock source
    /// that is not supported will result in undefined behavior. It takes four
    /// cycles of the originally selected clock to switch away from it. Do not
    /// disable the original clock during this time.
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
    /// Selects predivider value for this clock root and DSI input.
    /// This divider is after DIRECT_MUX. For products with DSI, the output of
    /// this mux is routed to DSI for use as a signal. For products with clock
    /// supervision, the output of this mux is the monitored clock for CSV_HF<k>.
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
    /// Enable for this clock root. All clock roots default to disabled
    /// (ENABLE==0) except HFCLK0, which cannot be disabled.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLK_DIRECT_SELECT [
    /// Direct selection mux that allows IMO to bypass most of the clock mux
    /// structure. For products with multiple regulators, this mux can be used
    /// to reduce current without requiring significant reconfiguration of the
    /// clocking network. The default value of HFCLK<0>==ROOT_MUX, and the
    /// default value for other clock trees is product-specific.
    DIRECT_MUX OFFSET(8) NUMBITS(1) [
        /// Select IMO
        SelectIMO = 0,
        /// Select ROOT_MUX selection
        SelectROOT_MUXSelection = 1
    ]
],
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

    pub fn init_clock(&self) {
        self.registers
            .clk_path_select_3
            .modify(CLK_PATH_SELECT::PATH_MUX::IMOInternalRCOscillator);

        self.registers
            .clk_root_select_0
            .modify(CLK_ROOT_SELECT::ENABLE::SET
                + CLK_ROOT_SELECT::ROOT_MUX::SelectPATH3
                + CLK_ROOT_SELECT::ROOT_DIV_INT::TransparentModeFeedThroughSelectedClockSourceWODividing);
    }
}
