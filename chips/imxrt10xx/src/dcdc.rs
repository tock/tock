//! DCDC Converter

use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{self, ReadWrite};
use kernel::utilities::StaticRef;

use crate::ccm;

registers::register_structs! {
    /// DCDC
    DcdcRegisters {
        /// DCDC Register 0
        (0x000 => reg0: ReadWrite<u32, REG0::Register>),
        /// DCDC Register 1
        (0x004 => reg1: ReadWrite<u32, REG1::Register>),
        /// DCDC Register 2
        (0x008 => reg2: ReadWrite<u32, REG2::Register>),
        /// DCDC Register 3
        (0x00C => reg3: ReadWrite<u32, REG3::Register>),
        (0x010 => @END),
    }
}

registers::register_bitfields![u32,
REG0 [
    /// power down the zero cross detection function for discontinuous conductor mode
    PWD_ZCD OFFSET(0) NUMBITS(1) [],
    /// Disable automatic clock switch from internal osc to xtal clock.
    DISABLE_AUTO_CLK_SWITCH OFFSET(1) NUMBITS(1) [],
    /// select 24 MHz Crystal clock for DCDC, when dcdc_disable_auto_clk_switch is set.
    SEL_CLK OFFSET(2) NUMBITS(1) [],
    /// Power down internal osc. Only set this bit, when 24 MHz crystal osc is available
    PWD_OSC_INT OFFSET(3) NUMBITS(1) [],
    /// The power down signal of the current detector.
    PWD_CUR_SNS_CMP OFFSET(4) NUMBITS(1) [],
    /// Set the threshold of current detector, if the peak current of the inductor excee
    CUR_SNS_THRSH OFFSET(5) NUMBITS(3) [],
    /// power down overcurrent detection comparator
    PWD_OVERCUR_DET OFFSET(8) NUMBITS(1) [],
    /// The threshold of over current detection in run mode and power save mode: run mod
    OVERCUR_TRIG_ADJ OFFSET(9) NUMBITS(2) [],
    /// set to "1" to power down the low voltage detection comparator
    PWD_CMP_BATT_DET OFFSET(11) NUMBITS(1) [],
    /// adjust value to poslimit_buck register
    ADJ_POSLIMIT_BUCK OFFSET(12) NUMBITS(4) [],
    /// enable the overload detection in power save mode, if current is larger than the
    EN_LP_OVERLOAD_SNS OFFSET(16) NUMBITS(1) [],
    /// power down overvoltage detection comparator
    PWD_HIGH_VOLT_DET OFFSET(17) NUMBITS(1) [],
    /// the threshold of the counting number of charging times during the period that lp
    LP_OVERLOAD_THRSH OFFSET(18) NUMBITS(2) [],
    /// the period of counting the charging times in power save mode 0: eight 32k cycle
    LP_OVERLOAD_FREQ_SEL OFFSET(20) NUMBITS(1) [],
    /// Adjust hysteretic value in low power from 12.5mV to 25mV
    LP_HIGH_HYS OFFSET(21) NUMBITS(1) [],
    /// power down output range comparator
    PWD_CMP_OFFSET OFFSET(26) NUMBITS(1) [],
    /// 1'b1: Disable xtalok detection circuit 1'b0: Enable xtalok detection circuit
    XTALOK_DISABLE OFFSET(27) NUMBITS(1) [],
    /// reset current alert signal
    CURRENT_ALERT_RESET OFFSET(28) NUMBITS(1) [],
    /// set to 1 to switch internal ring osc to xtal 24M
    XTAL_24M_OK OFFSET(29) NUMBITS(1) [],
    /// Status register to indicate DCDC status. 1'b1: DCDC already settled 1'b0: DCDC i
    STS_DC_OK OFFSET(31) NUMBITS(1) []
],
REG1 [
    /// select the feedback point of the internal regulator
    REG_FBK_SEL OFFSET(7) NUMBITS(2) [],
    /// control the load resistor of the internal regulator of DCDC, the load resistor i
    REG_RLOAD_SW OFFSET(9) NUMBITS(1) [],
    /// set the current bias of low power comparator 0x0: 50 nA 0x1: 100 nA 0x2: 200 nA
    LP_CMP_ISRC_SEL OFFSET(12) NUMBITS(2) [],
    /// increase the threshold detection for common mode analog comparator
    LOOPCTRL_HST_THRESH OFFSET(21) NUMBITS(1) [],
    /// Enable hysteresis in switching converter common mode analog comparators
    LOOPCTRL_EN_HYST OFFSET(23) NUMBITS(1) [],
    /// trim bandgap voltage
    VBG_TRIM OFFSET(24) NUMBITS(5) []
],
REG2 [
    /// Ratio of integral control parameter to proportional control parameter in the swi
    LOOPCTRL_DC_C OFFSET(0) NUMBITS(2) [],
    /// Magnitude of proportional control parameter in the switching DC-DC converter con
    LOOPCTRL_DC_R OFFSET(2) NUMBITS(4) [],
    /// Two's complement feed forward step in duty cycle in the switching DC-DC converte
    LOOPCTRL_DC_FF OFFSET(6) NUMBITS(3) [],
    /// Enable analog circuit of DC-DC converter to respond faster under transient load
    LOOPCTRL_EN_RCSCALE OFFSET(9) NUMBITS(3) [],
    /// Increase the threshold detection for RC scale circuit.
    LOOPCTRL_RCSCALE_THRSH OFFSET(12) NUMBITS(1) [],
    /// Invert the sign of the hysteresis in DC-DC analog comparators.
    LOOPCTRL_HYST_SIGN OFFSET(13) NUMBITS(1) [],
    /// Set to "0" : stop charging if the duty cycle is lower than what set by dcdc_negl
    DISABLE_PULSE_SKIP OFFSET(27) NUMBITS(1) [],
    /// Set high to improve the transition from heavy load to light load
    DCM_SET_CTRL OFFSET(28) NUMBITS(1) []
],
REG3 [
    /// Target value of VDD_SOC, 25 mV each step 0x0: 0.8V 0xE: 1.15V 0x1F:1.575V
    TRG OFFSET(0) NUMBITS(5) [],
    /// Target value of standby (low power) mode 0x0: 0
    TARGET_LP OFFSET(8) NUMBITS(3) [],
    /// Set DCDC clock to half freqeuncy for continuous mode
    MINPWR_DC_HALFCLK OFFSET(24) NUMBITS(1) [],
    /// Ajust delay to reduce ground noise
    MISC_DELAY_TIMING OFFSET(27) NUMBITS(1) [],
    /// Reserved
    MISC_DISABLEFET_LOGIC OFFSET(28) NUMBITS(1) [],
    /// Disable stepping for the output VDD_SOC of DCDC
    DISABLE_STEP OFFSET(30) NUMBITS(1) []
]
];
const DCDC_BASE: StaticRef<DcdcRegisters> =
    unsafe { StaticRef::new(0x40080000 as *const DcdcRegisters) };

/// DCDC converter
pub struct Dcdc<'a> {
    registers: StaticRef<DcdcRegisters>,
    clock_gate: ccm::PeripheralClock<'a>,
}

impl<'a> Dcdc<'a> {
    /// Construct a new DCDC peripheral that can control its own clock
    pub const fn new(ccm: &'a ccm::Ccm) -> Self {
        Self {
            registers: DCDC_BASE,
            clock_gate: ccm::PeripheralClock::ccgr6(ccm, ccm::HCLK6::DCDC),
        }
    }
    /// Returns the interface that controls the DCDC clock
    pub fn clock(&self) -> &(impl ClockInterface + '_) {
        &self.clock_gate
    }
    /// Set the target value of `VDD_SOC`, in milliamps
    ///
    /// Values are clamped between 800mV and 1575mV, with 25mV step
    /// sizes.
    pub fn set_target_vdd_soc(&self, millivolts: u32) {
        let millivolts = millivolts.min(1575).max(800);
        let trg = (millivolts - 800) / 25;
        self.registers.reg3.modify(REG3::TRG.val(trg));
        while !self.registers.reg0.is_set(REG0::STS_DC_OK) {}
    }
}
