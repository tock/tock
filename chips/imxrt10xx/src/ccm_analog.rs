//! CCM Analog peripheral

// Generated following
//
// 1. Use svd2regs.py to import the memory layout and all fields
// 2. For each reg, set, clear, toggle, grouping, replace it with
//    a Groups struct.
// 3. Remove unused

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadWrite, RegisterLongName, WriteOnly,
};
use kernel::utilities::StaticRef;

/// Many CCM_ANALOG registers are laid out with additional set, clear, and toggle
/// registers. This groups them together into an extended register.
///
/// Selecting suffix 'SCT' given the mention in the reference manual:
///
/// > A set of SCT registeres is offered for registers in many modules [...]
///
/// Although the CCM_ANALOG memory map indicates the SCT registers are R/W,
/// the reference manual front-matter (section 1.5.2) indicates that they should
/// be treated as as write only:
///
/// > The SCT registers always read back 0, and should be considered write-only.
#[repr(C)]
struct RegisterSCT<R: RegisterLongName = ()> {
    /// The normal register
    reg: ReadWrite<u32, R>,
    /// Write 1 sets bits in reg
    set: WriteOnly<u32, R>,
    /// Write 1 clears bits in reg
    clear: WriteOnly<u32, R>,
    /// Write 1 toggles bits in reg
    toggle: WriteOnly<u32, R>,
}

register_structs! {
    /// CCM_ANALOG
    CcmAnalogRegisters {
        /// Analog ARM PLL control Register
        (0x000 => pll_arm: RegisterSCT<PLL_ARM::Register>),
        /// Analog USB1 480MHz PLL Control Register
        (0x010 => pll_usb1: RegisterSCT<PLL_USB1::Register>),
        /// Analog USB2 480MHz PLL Control Register
        (0x020 => pll_usb2: RegisterSCT<PLL_USB2::Register>),
        /// Analog System PLL Control Register
        (0x030 => pll_sys: RegisterSCT<PLL_SYS::Register>),
        /// 528MHz System PLL Spread Spectrum Register
        (0x040 => pll_sys_ss: ReadWrite<u32, PLL_SYS_SS::Register>),
        (0x044 => _reserved0),
        /// Numerator of 528MHz System PLL Fractional Loop Divider Register
        (0x050 => pll_sys_num: ReadWrite<u32>),
        (0x054 => _reserved1),
        /// Denominator of 528MHz System PLL Fractional Loop Divider Register
        (0x060 => pll_sys_denom: ReadWrite<u32>),
        (0x064 => _reserved2),
        /// Analog Audio PLL control Register
        (0x070 => pll_audio: RegisterSCT<PLL_AUDIO::Register>),
        /// Numerator of Audio PLL Fractional Loop Divider Register
        (0x080 => pll_audio_num: ReadWrite<u32>),
        (0x084 => _reserved3),
        /// Denominator of Audio PLL Fractional Loop Divider Register
        (0x090 => pll_audio_denom: ReadWrite<u32>),
        (0x094 => _reserved4),
        /// Analog Video PLL control Register
        (0x0A0 => pll_video: RegisterSCT<PLL_VIDEO::Register>),
        /// Numerator of Video PLL Fractional Loop Divider Register
        (0x0B0 => pll_video_num: ReadWrite<u32>),
        (0x0B4 => _reserved5),
        /// Denominator of Video PLL Fractional Loop Divider Register
        (0x0C0 => pll_video_denom: ReadWrite<u32>),
        (0x0C4 => _reserved6),
        /// Analog ENET PLL Control Register
        (0x0E0 => pll_enet: RegisterSCT<PLL_ENET::Register>),
        /// 480MHz Clock (PLL3) Phase Fractional Divider Control Register
        (0x0F0 => pfd_480: RegisterSCT<PFD_480::Register>),
        /// 528MHz Clock (PLL2) Phase Fractional Divider Control Register
        (0x100 => pfd_528: RegisterSCT<PFD_528::Register>),
        (0x110 => _reserved7),
        /// Miscellaneous Register 0
        (0x150 => misc0: RegisterSCT<MISC0::Register>),
        /// Miscellaneous Register 1
        (0x160 => misc1: RegisterSCT<MISC1::Register>),
        /// Miscellaneous Register 2
        (0x170 => misc2: RegisterSCT<MISC2::Register>),
        (0x180 => @END),
    }
}

register_bitfields![u32,
    PLL_ARM [
        /// This field controls the PLL loop divider
        DIV_SELECT OFFSET(0) NUMBITS(7) [],
        /// Powers down the PLL.
        POWERDOWN OFFSET(12) NUMBITS(1) [],
        /// Enable the clock output.
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// Reserved
        PLL_SEL OFFSET(19) NUMBITS(1) [],
        /// 1 - PLL is currently locked. 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_USB1 [
        /// This field controls the PLL loop divider. 0 - Fout=Fref*20; 1 - Fout=Fref*22.
        DIV_SELECT OFFSET(1) NUMBITS(1) [],
        /// Powers the 9-phase PLL outputs for USBPHYn
        EN_USB_CLKS OFFSET(6) NUMBITS(1) [
            /// PLL outputs for USBPHYn off.
            PLLOutputsForUSBPHYnOff = 0,
            /// PLL outputs for USBPHYn on.
            PLLOutputsForUSBPHYnOn = 1
        ],
        /// Powers up the PLL. This bit will be set automatically when USBPHY0 remote wakeup
        POWER OFFSET(12) NUMBITS(1) [],
        /// Enable the PLL clock output.
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// 1 - PLL is currently locked. 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_USB2 [
        /// This field controls the PLL loop divider. 0 - Fout=Fref*20; 1 - Fout=Fref*22.
        DIV_SELECT OFFSET(1) NUMBITS(1) [],
        /// 0: 8-phase PLL outputs for USBPHY1 are powered down
        EN_USB_CLKS OFFSET(6) NUMBITS(1) [],
        /// Powers up the PLL. This bit will be set automatically when USBPHY1 remote wakeup
        POWER OFFSET(12) NUMBITS(1) [],
        /// Enable the PLL clock output.
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// 1 - PLL is currently locked. 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_SYS [
        /// This field controls the PLL loop divider. 0 - Fout=Fref*20; 1 - Fout=Fref*22.
        DIV_SELECT OFFSET(0) NUMBITS(1) [],
        /// Powers down the PLL.
        POWERDOWN OFFSET(12) NUMBITS(1) [],
        /// Enable PLL output
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// 1 - PLL is currently locked; 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_AUDIO [
        /// This field controls the PLL loop divider. Valid range for DIV_SELECT divider val
        DIV_SELECT OFFSET(0) NUMBITS(7) [],
        /// Powers down the PLL.
        POWERDOWN OFFSET(12) NUMBITS(1) [],
        /// Enable PLL output
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// These bits implement a divider after the PLL, but before the enable and bypass m
        POST_DIV_SELECT OFFSET(19) NUMBITS(2) [
            /// Divide by 4.
            DivideBy4 = 0,
            /// Divide by 2.
            DivideBy2 = 1,
            /// Divide by 1.
            DivideBy1 = 2
        ],
        /// 1 - PLL is currently locked. 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_SYS_SS [
        /// Frequency change step = step/CCM_ANALOG_PLL_SYS_DENOM[B]*24MHz.
        STEP OFFSET(0) NUMBITS(15) [],
        /// Enable bit
        ENABLE OFFSET(15) NUMBITS(1) [
            /// Spread spectrum modulation disabled
            SpreadSpectrumModulationDisabled = 0,
            /// Soread spectrum modulation enabled
            SoreadSpectrumModulationEnabled = 1
        ],
        /// Frequency change = stop/CCM_ANALOG_PLL_SYS_DENOM[B]*24MHz.
        STOP OFFSET(16) NUMBITS(16) []
    ],
    PLL_VIDEO [
        /// This field controls the PLL loop divider. Valid range for DIV_SELECT divider val
        DIV_SELECT OFFSET(0) NUMBITS(7) [],
        /// Powers down the PLL.
        POWERDOWN OFFSET(12) NUMBITS(1) [],
        /// Enalbe PLL output
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// These bits implement a divider after the PLL, but before the enable and bypass m
        POST_DIV_SELECT OFFSET(19) NUMBITS(2) [
            /// Divide by 4.
            DivideBy4 = 0,
            /// Divide by 2.
            DivideBy2 = 1,
            /// Divide by 1.
            DivideBy1 = 2
        ],
        /// 1 - PLL is currently locked; 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PLL_ENET [
        /// Controls the frequency of the ethernet reference clock
        DIV_SELECT OFFSET(0) NUMBITS(2) [],
        /// Controls the frequency of the ENET2 reference clock.
        ENET2_DIV_SELECT OFFSET(2) NUMBITS(2) [
            /// 25MHz
            _25MHz = 0,
            /// 50MHz
            _50MHz = 1,
            /// 100MHz (not 50% duty cycle)
            _100MHzNot50DutyCycle = 2,
            /// 125MHz
            _125MHz = 3
        ],
        /// Powers down the PLL.
        POWERDOWN OFFSET(12) NUMBITS(1) [],
        /// Enable the PLL providing the ENET reference clock.
        ENABLE OFFSET(13) NUMBITS(1) [],
        /// Determines the bypass source.
        BYPASS_CLK_SRC OFFSET(14) NUMBITS(2) [
            /// Select the 24MHz oscillator as source.
            SelectThe24MHzOscillatorAsSource = 0,
            /// Select the CLK1_N / CLK1_P as source.
            SelectTheCLK1_NCLK1_PAsSource = 1
        ],
        /// Bypass the PLL.
        BYPASS OFFSET(16) NUMBITS(1) [],
        /// Enable the PLL providing the ENET2 reference clock
        ENET2_REF_EN OFFSET(20) NUMBITS(1) [],
        /// Enable the PLL providing ENET 25 MHz reference clock
        ENET_25M_REF_EN OFFSET(21) NUMBITS(1) [],
        /// 1 - PLL is currently locked; 0 - PLL is not currently locked.
        LOCK OFFSET(31) NUMBITS(1) []
    ],
    PFD_480 [
        /// This field controls the fractional divide value
        PFD0_FRAC OFFSET(0) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD0_STABLE OFFSET(6) NUMBITS(1) [],
        /// If set to 1, the IO fractional divider clock (reference ref_pfd0) is off (power
        PFD0_CLKGATE OFFSET(7) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD1_FRAC OFFSET(8) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD1_STABLE OFFSET(14) NUMBITS(1) [],
        /// IO Clock Gate
        PFD1_CLKGATE OFFSET(15) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD2_FRAC OFFSET(16) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD2_STABLE OFFSET(22) NUMBITS(1) [],
        /// IO Clock Gate
        PFD2_CLKGATE OFFSET(23) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD3_FRAC OFFSET(24) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD3_STABLE OFFSET(30) NUMBITS(1) [],
        /// IO Clock Gate
        PFD3_CLKGATE OFFSET(31) NUMBITS(1) []
    ],
    PFD_528 [
        /// This field controls the fractional divide value
        PFD0_FRAC OFFSET(0) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD0_STABLE OFFSET(6) NUMBITS(1) [],
        /// If set to 1, the IO fractional divider clock (reference ref_pfd0) is off (power
        PFD0_CLKGATE OFFSET(7) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD1_FRAC OFFSET(8) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD1_STABLE OFFSET(14) NUMBITS(1) [],
        /// IO Clock Gate
        PFD1_CLKGATE OFFSET(15) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD2_FRAC OFFSET(16) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD2_STABLE OFFSET(22) NUMBITS(1) [],
        /// IO Clock Gate
        PFD2_CLKGATE OFFSET(23) NUMBITS(1) [],
        /// This field controls the fractional divide value
        PFD3_FRAC OFFSET(24) NUMBITS(6) [],
        /// This read-only bitfield is for DIAGNOSTIC PURPOSES ONLY since the fractional div
        PFD3_STABLE OFFSET(30) NUMBITS(1) [],
        /// IO Clock Gate
        PFD3_CLKGATE OFFSET(31) NUMBITS(1) []
    ],
    MISC0 [
        /// Control bit to power-down the analog bandgap reference circuitry
        REFTOP_PWD OFFSET(0) NUMBITS(1) [],
        /// Control bit to disable the self-bias circuit in the analog bandgap
        REFTOP_SELFBIASOFF OFFSET(3) NUMBITS(1) [
            /// Uses coarse bias currents for startup
            UsesCoarseBiasCurrentsForStartup = 0,
            /// Uses bandgap-based bias currents for best performance.
            UsesBandgapBasedBiasCurrentsForBestPerformance = 1
        ],
        /// Not related to CCM. See Power Management Unit (PMU)
        REFTOP_VBGADJ OFFSET(4) NUMBITS(3) [],
        /// Status bit that signals the analog bandgap voltage is up and stable
        REFTOP_VBGUP OFFSET(7) NUMBITS(1) [],
        /// Configure the analog behavior in stop mode.
        STOP_MODE_CONFIG OFFSET(10) NUMBITS(2) [
            /// All analog except RTC powered down on stop mode assertion.
            AllAnalogExceptRTCPoweredDownOnStopModeAssertion = 0,
            /// Beside RTC, analog bandgap, 1p1 and 2p5 regulators are also on.
            BesideRTCAnalogBandgap1p1And2p5RegulatorsAreAlsoOn = 1,
            /// Beside RTC, 1p1 and 2p5 regulators are also on, low-power bandgap is selected so
            STOP_MODE_CONFIG_2 = 2,
            /// Beside RTC, low-power bandgap is selected and the rest analog is powered down.
            BesideRTCLowPowerBandgapIsSelectedAndTheRestAnalogIsPoweredDown = 3
        ],
        /// This bit controls a switch from VDD_HIGH_IN to VDD_SNVS_IN.
        DISCON_HIGH_SNVS OFFSET(12) NUMBITS(1) [
            /// Turn on the switch
            TurnOnTheSwitch = 0,
            /// Turn off the switch
            TurnOffTheSwitch = 1
        ],
        /// This field determines the bias current in the 24MHz oscillator
        OSC_I OFFSET(13) NUMBITS(2) [
            /// Nominal
            Nominal = 0,
            /// Decrease current by 12.5%
            DecreaseCurrentBy125 = 1,
            /// Decrease current by 25.0%
            DecreaseCurrentBy250 = 2,
            /// Decrease current by 37.5%
            DecreaseCurrentBy375 = 3
        ],
        /// Status bit that signals that the output of the 24-MHz crystal oscillator is stab
        OSC_XTALOK OFFSET(15) NUMBITS(1) [],
        /// This bit enables the detector that signals when the 24MHz crystal oscillator is
        OSC_XTALOK_EN OFFSET(16) NUMBITS(1) [],
        /// This bit allows disabling the clock gate (always ungated) for the xtal 24MHz clo
        CLKGATE_CTRL OFFSET(25) NUMBITS(1) [
            /// Allow the logic to automatically gate the clock when the XTAL is powered down.
            AllowTheLogicToAutomaticallyGateTheClockWhenTheXTALIsPoweredDown = 0,
            /// Prevent the logic from ever gating off the clock.
            PreventTheLogicFromEverGatingOffTheClock = 1
        ],
        /// This field specifies the delay between powering up the XTAL 24MHz clock and rele
        CLKGATE_DELAY OFFSET(26) NUMBITS(3) [
            /// 0.5ms
            _05ms = 0,
            /// 1.0ms
            _10ms = 1,
            /// 2.0ms
            _20ms = 2,
            /// 3.0ms
            _30ms = 3,
            /// 4.0ms
            _40ms = 4,
            /// 5.0ms
            _50ms = 5,
            /// 6.0ms
            _60ms = 6,
            /// 7.0ms
            _70ms = 7
        ],
        /// This field indicates which chip source is being used for the rtc clock
        RTC_XTAL_SOURCE OFFSET(29) NUMBITS(1) [
            /// Internal ring oscillator
            InternalRingOscillator = 0,
            /// RTC_XTAL
            RTC_XTAL = 1
        ],
        /// This field powers down the 24M crystal oscillator if set true
        XTAL_24M_PWD OFFSET(30) NUMBITS(1) []
    ],
    MISC1 [
        /// This field selects the clk to be routed to anaclk1/1b.
        LVDS1_CLK_SEL OFFSET(0) NUMBITS(5) [
            /// Arm PLL
            ArmPLL = 0,
            /// System PLL
            SystemPLL = 1,
            /// ref_pfd4_clk == pll2_pfd0_clk
            Ref_pfd4_clkPll2_pfd0_clk = 2,
            /// ref_pfd5_clk == pll2_pfd1_clk
            Ref_pfd5_clkPll2_pfd1_clk = 3,
            /// ref_pfd6_clk == pll2_pfd2_clk
            Ref_pfd6_clkPll2_pfd2_clk = 4,
            /// ref_pfd7_clk == pll2_pfd3_clk
            Ref_pfd7_clkPll2_pfd3_clk = 5,
            /// Audio PLL
            AudioPLL = 6,
            /// Video PLL
            VideoPLL = 7,
            /// ethernet ref clock (ENET_PLL)
            EthernetRefClockENET_PLL = 9,
            /// USB1 PLL clock
            USB1PLLClock = 12,
            /// USB2 PLL clock
            USB2PLLClock = 13,
            /// ref_pfd0_clk == pll3_pfd0_clk
            Ref_pfd0_clkPll3_pfd0_clk = 14,
            /// ref_pfd1_clk == pll3_pfd1_clk
            Ref_pfd1_clkPll3_pfd1_clk = 15,
            /// ref_pfd2_clk == pll3_pfd2_clk
            Ref_pfd2_clkPll3_pfd2_clk = 16,
            /// ref_pfd3_clk == pll3_pfd3_clk
            Ref_pfd3_clkPll3_pfd3_clk = 17,
            /// xtal (24M)
            Xtal24M = 18
        ],
        /// This enables the LVDS output buffer for anaclk1/1b
        LVDSCLK1_OBEN OFFSET(10) NUMBITS(1) [],
        /// This enables the LVDS input buffer for anaclk1/1b
        LVDSCLK1_IBEN OFFSET(12) NUMBITS(1) [],
        /// This enables a feature that will clkgate (reset) all PFD_480 clocks anytime the
        PFD_480_AUTOGATE_EN OFFSET(16) NUMBITS(1) [],
        /// This enables a feature that will clkgate (reset) all PFD_528 clocks anytime the
        PFD_528_AUTOGATE_EN OFFSET(17) NUMBITS(1) [],
        /// This status bit is set to one when the temperature sensor panic interrupt assert
        IRQ_TEMPPANIC OFFSET(27) NUMBITS(1) [],
        /// This status bit is set to one when the temperature sensor low interrupt asserts
        IRQ_TEMPLOW OFFSET(28) NUMBITS(1) [],
        /// This status bit is set to one when the temperature sensor high interrupt asserts
        IRQ_TEMPHIGH OFFSET(29) NUMBITS(1) [],
        /// This status bit is set to one when when any of the analog regulator brownout int
        IRQ_ANA_BO OFFSET(30) NUMBITS(1) [],
        /// This status bit is set to one when when any of the digital regulator brownout in
        IRQ_DIG_BO OFFSET(31) NUMBITS(1) []
    ],
    MISC2 [
        /// This field defines the brown out voltage offset for the CORE power domain
        REG0_BO_OFFSET OFFSET(0) NUMBITS(3) [
            /// Brownout offset = 0.100V
            BrownoutOffset0100V = 4,
            /// Brownout offset = 0.175V
            BrownoutOffset0175V = 7
        ],
        /// Reg0 brownout status bit.Not related to CCM. See Power Management Unit (PMU)
        REG0_BO_STATUS OFFSET(3) NUMBITS(1) [
            /// Brownout, supply is below target minus brownout offset.
            BrownoutSupplyIsBelowTargetMinusBrownoutOffset = 1
        ],
        /// Enables the brownout detection.Not related to CCM. See Power Management Unit (PM
        REG0_ENABLE_BO OFFSET(5) NUMBITS(1) [],
        /// ARM supply Not related to CCM. See Power Management Unit (PMU)
        REG0_OK OFFSET(6) NUMBITS(1) [],
        /// When USB is in low power suspend mode this Control bit is used to indicate if ot
        PLL3_DISABLE OFFSET(7) NUMBITS(1) [
            /// PLL3 is being used by peripherals and is enabled when SoC is not in any low powe
            PLL3IsBeingUsedByPeripheralsAndIsEnabledWhenSoCIsNotInAnyLowPowerMode = 0,
            /// PLL3 can be disabled when the SoC is not in any low power mode
            PLL3CanBeDisabledWhenTheSoCIsNotInAnyLowPowerMode = 1
        ],
        /// This field defines the brown out voltage offset for the xPU power domain
        REG1_BO_OFFSET OFFSET(8) NUMBITS(3) [
            /// Brownout offset = 0.100V
            BrownoutOffset0100V = 4,
            /// Brownout offset = 0.175V
            BrownoutOffset0175V = 7
        ],
        /// Reg1 brownout status bit. Not related to CCM. See Power Management Unit (PMU)
        REG1_BO_STATUS OFFSET(11) NUMBITS(1) [
            /// Brownout, supply is below target minus brownout offset.
            BrownoutSupplyIsBelowTargetMinusBrownoutOffset = 1
        ],
        /// Enables the brownout detection.Not related to CCM. See Power Management Unit (PM
        REG1_ENABLE_BO OFFSET(13) NUMBITS(1) [],
        /// GPU/VPU supply Not related to CCM. See Power Management Unit (PMU)
        REG1_OK OFFSET(14) NUMBITS(1) [],
        /// LSB of Post-divider for Audio PLL
        AUDIO_DIV_LSB OFFSET(15) NUMBITS(1) [
            /// divide by 1 (Default)
            DivideBy1Default = 0,
            /// divide by 2
            DivideBy2 = 1
        ],
        /// This field defines the brown out voltage offset for the xPU power domain
        REG2_BO_OFFSET OFFSET(16) NUMBITS(3) [
            /// Brownout offset = 0.100V
            BrownoutOffset0100V = 4,
            /// Brownout offset = 0.175V
            BrownoutOffset0175V = 7
        ],
        /// Reg2 brownout status bit.Not related to CCM. See Power Management Unit (PMU)
        REG2_BO_STATUS OFFSET(19) NUMBITS(1) [],
        /// Enables the brownout detection.Not related to CCM. See Power Management Unit (PM
        REG2_ENABLE_BO OFFSET(21) NUMBITS(1) [],
        /// Signals that the voltage is above the brownout level for the SOC supply
        REG2_OK OFFSET(22) NUMBITS(1) [],
        /// MSB of Post-divider for Audio PLL
        AUDIO_DIV_MSB OFFSET(23) NUMBITS(1) [
            /// divide by 1 (Default)
            DivideBy1Default = 0,
            /// divide by 2
            DivideBy2 = 1
        ],
        /// Number of clock periods (24MHz clock).Not related to CCM. See Power Management U
        REG0_STEP_TIME OFFSET(24) NUMBITS(2) [
            /// 64
            _64 = 0,
            /// 128
            _128 = 1,
            /// 256
            _256 = 2,
            /// 512
            _512 = 3
        ],
        /// Number of clock periods (24MHz clock).Not related to CCM. See Power Management U
        REG1_STEP_TIME OFFSET(26) NUMBITS(2) [
            /// 64
            _64 = 0,
            /// 128
            _128 = 1,
            /// 256
            _256 = 2,
            /// 512
            _512 = 3
        ],
        /// Number of clock periods (24MHz clock).Not related to CCM. See Power Management U
        REG2_STEP_TIME OFFSET(28) NUMBITS(2) [
            /// 64
            _64 = 0,
            /// 128
            _128 = 1,
            /// 256
            _256 = 2,
            /// 512
            _512 = 3
        ],
        /// Post-divider for video
        VIDEO_DIV OFFSET(30) NUMBITS(2) [
            /// divide by 1 (Default)
            DivideBy1Default = 0,
            /// divide by 2
            DivideBy2 = 1,
            /// divide by 1
            DivideBy1 = 2,
            /// divide by 4
            DivideBy4 = 3
        ]
    ]
];

const CCM_ANALOG_BASE: StaticRef<CcmAnalogRegisters> =
    unsafe { StaticRef::new(0x400D8000 as *const CcmAnalogRegisters) };

pub struct CcmAnalog {
    registers: StaticRef<CcmAnalogRegisters>,
}

impl CcmAnalog {
    /// Creates a new `CcmAnalog` peripheral
    pub const fn new() -> Self {
        Self {
            registers: CCM_ANALOG_BASE,
        }
    }

    /// Returns the PLL1 `DIV_SEL` value
    pub fn pll1_div_sel(&self) -> u32 {
        self.registers.pll_arm.reg.read(PLL_ARM::DIV_SELECT)
    }

    /// Restart PLL1 using the new `div_sel`
    ///
    /// Clamps `div_sel` to [54, 108].
    pub fn restart_pll1(&self, div_sel: u32) {
        let div_sel = div_sel.min(108).max(54);

        // Clear all bits except powerdown
        self.registers.pll_arm.reg.write(PLL_ARM::POWERDOWN::SET);
        // Clear powerdown write above
        self.registers
            .pll_arm
            .reg
            .write(PLL_ARM::DIV_SELECT.val(div_sel));
        // Enable the PLL
        self.registers.pll_arm.set.write(PLL_ARM::ENABLE::SET);
        // Wait for lock
        while self.registers.pll_arm.reg.read(PLL_ARM::LOCK) == 0 {}
    }
}
