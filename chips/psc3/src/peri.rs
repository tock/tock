use kernel::utilities::registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    /// Peripheral interconnect
    PeriRegisters {
        (0x000 => _reserved0),
        /// Timeout control
        (0x200 => timeout_ctl: ReadWrite<u32, TIMEOUT_CTL::Register>),
        (0x204 => _reserved1),
        /// Trigger command
        (0x2000 => tr_cmd: ReadWrite<u32, TR_CMD::Register>),
        /// Infrastructure clock force enable
        (0x2004 => infra_clk_force: ReadWrite<u32, INFRA_CLK_FORCE::Register>),
        (0x2008 => _reserved2),
        /// Group 0 registers
        (0x4000 => gr_0_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x4004 => _reserved3),
        (0x4010 => gr_0_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x4014 => gr_0_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x4018 => gr_0_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x401C => _reserved4),
        (0x4020 => gr_0_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x4024 => _reserved5),
        /// Group 1 registers
        (0x4040 => gr_1_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x4044 => _reserved6),
        (0x4050 => gr_1_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x4054 => gr_1_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x4058 => gr_1_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x405C => _reserved7),
        (0x4060 => gr_1_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x4064 => _reserved8),
        /// Group 2 registers
        (0x4080 => gr_2_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x4084 => _reserved9),
        (0x4090 => gr_2_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x4094 => gr_2_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x4098 => gr_2_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x409C => _reserved10),
        (0x40A0 => gr_2_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x40A4 => _reserved11),
        /// Group 3 registers
        (0x40C0 => gr_3_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x40C4 => _reserved12),
        (0x40D0 => gr_3_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x40D4 => gr_3_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x40D8 => gr_3_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x40DC => _reserved13),
        (0x40E0 => gr_3_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x40E4 => _reserved14),
        /// Group 4 registers
        (0x4100 => gr_4_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x4104 => _reserved15),
        (0x4110 => gr_4_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x4114 => gr_4_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x4118 => gr_4_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x411C => _reserved16),
        (0x4120 => gr_4_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x4124 => _reserved17),
        /// Group 5 registers
        (0x4140 => gr_5_clock_ctl: ReadWrite<u32, GR_CLOCK_CTL::Register>),
        (0x4144 => _reserved18),
        (0x4150 => gr_5_sl_ctl: ReadWrite<u32, GR_SL_CTL::Register>),
        (0x4154 => gr_5_sl_ctl2: ReadWrite<u32, GR_SL_CTL2::Register>),
        (0x4158 => gr_5_sl_ctl3: ReadWrite<u32, GR_SL_CTL3::Register>),
        (0x415C => _reserved19),
        (0x4160 => gr_5_sl_wound: ReadWrite<u32, GR_SL_WOUND::Register>),
        (0x4164 => _reserved20),
        /// Missing trigger control register, add when needed.
        (0xCC00 => @END),
    }
}
register_bitfields![u32,
TIMEOUT_CTL [
    /// This field specifies a number of peripheral group root undivided (clk_group_root[i]) clock cycles. If an AHB-Lite bus transfer takes more than the specified number of cycles (timeout detection), the bus transfer is terminated with an AHB5 bus error and a timeout status is set. '0x0000'-'0xfffe': Number of peripheral group clock cycles. '0xffff': This value is the default/reset value and specifies that no timeout detection is performed: a bus transfer will never be terminated, and a interrupt will never be generated.
    /// Note that TIMEOUT_CTL.TIMEOUT[15:0] in clk_pclk0_root (clk_hf0) is used directly in peripheral group clock domain clk_group_root[i], even if clk_group_root[i] is async to clk_pclk0_root. This is on the assumption that this register is programmed once by SW, remain constant. Following SW programming restrictions apply to TIMEOUT_CTL.TIMEOUT[15:0]. SW should make sure that no other AHB transactions are initiated through PERI before programming this register. SW should make sure that write to TIMEOUT_CTL.TIMEOUT[15:0] is completed by doing a readback.
    /// Note that peripheral group-0 slaves are excluded from timeout (Refer Timeout section in mxsperi.1 BROS for more details).
    TIMEOUT OFFSET(0) NUMBITS(16) [],
    /// This field provides control for HW to reset the slave that is causing the timeout to occur.
    /// 1 - no HW reset during timeout.
    /// 0 - HW resets the corresponding slave during timeout.
    /// This ensures the AHB bus not to be hung after a timeout has occurred. HW asserts the reset along with fault request (peri_gp'i'_mmio_timeout_vio_req) and holds it until fault acknowledge (mmio_peri_gp'i'_timeout_vio_ack) is received from centralized fault infrastructure.
    /// Note, SW needs to take care of the implication when clearing this bit when a HW reset has occurred as clearing this bit will cause HW reset de-assert.
    /// Note that peripheral group-0 slaves are excluded from timeout (Refer Timeout section in mxsperi.1 BROS for more details).
    HWRST_DISABLE OFFSET(31) NUMBITS(1) []
],
TR_CMD [
    /// Specifies the activated trigger when ACTIVATE is '1'. If the specified trigger is not present, the trigger activation has no effect.
    TR_SEL OFFSET(0) NUMBITS(8) [],
    /// Specifies the trigger group:
    /// '0'-'15': trigger multiplexer groups.
    /// '16'-'31': trigger 1-to-1 groups.
    GROUP_SEL OFFSET(8) NUMBITS(5) [],
    /// Specifies if the activated  trigger is treated as a level sensitive or edge sensitive  trigger.
    /// '0': level sensitive. The trigger reflects TR_CMD.ACTIVATE.
    /// '1': edge sensitive trigger. The trigger is activated for two clk_peri cycles.
    TR_EDGE OFFSET(29) NUMBITS(1) [],
    /// Specifies whether trigger activation is for a specific input or output trigger of the trigger multiplexer. Activation of a specific input trigger, will result in activation of all output triggers that have the specific input trigger selected through their TR_CTL.TR_SEL  field. Activation of a specific output trigger, will result in activation of the specified TR_SEL output trigger only.
    /// '0': TR_SEL selection and trigger activation is for an input trigger to the trigger multiplexer.
    /// '1': TR_SEL selection and trigger activation is for an output trigger from the trigger multiplexer.
    ///
    /// Note: this field is not used for trigger 1-to-1 groups.
    OUT_SEL OFFSET(30) NUMBITS(1) [],
    /// SW sets this field to '1' to activate (set to '1') a trigger as identified by TR_SEL, TR_EDGE and OUT_SEL. HW sets this field to '0' for edge sensitive triggers AFTER the selected trigger is activated for two clk_peri cycles.
    ///
    /// Note: when ACTIVATE is '1', SW should not modify the other register fields.
    /// SW MUST NOT set ACTIVATE bit to '1' while updating the other register bits simultaneously. At first the SW MUST update the other register bits as needed, and then set ACTIVATE to '1' with a new register write.
    ACTIVATE OFFSET(31) NUMBITS(1) []
],
INFRA_CLK_FORCE [
    /// Infrastructure clock force enable.
    /// 0: Disabled
    /// 1: Enabled
    ENABLED OFFSET(0) NUMBITS(1) []
],
GR_CLOCK_CTL [
    /// Specifies a group clock divider (from the peripheral clock 'clk_peri' to the group clock 'clk_group[1/2/3/4/5/...15]'). Integer division by (1+INT8_DIV). Allows for integer divisions in the range [1, 256].
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    INT8_DIV OFFSET(8) NUMBITS(8) []
],
GR_SL_CTL [
    /// Slave Enable. Each bit indicates whether the respective slave is enabled. If the slave is disabled, its clock is gated off (constant '0').
    ///
    /// Note: For peripheral group 0 slave 0,1, and 2 (the peripheral interconnect MMIO registers), this field is a constant '1' (SW: R): the slave can NOT be disabled.
    /// The peripheral IP that drives the Q-Channel back to Clock Controllers need to ensure that it has clock (usually driven by Clk_hf1~N that is only available after CPU configures their roots in the SRSS) to provide back the Q-Channel handshake, if not the deadlock situation will procure. To avoid deadlock mentioned above, all IPs in all groups other than group-0 are disabled  (SL_CTL.ENABLED is set to '0') by default after POR (cold boot) (i.e. PERI HW hardcodes local parameter SL_CTL_DEFAULT to 32'hFFFFFFFF for group-0 and to 32'h00000000 for other groups (group-1 to group-15)).  Once CPU is up and running & Clk_hf1~N configured, CPU can enable them.
    /// The SL_CTL.ENABLED are retained during DEEPSLEEP to avoid enabling configuration after wakeup.
    ENABLED OFFSET(0) NUMBITS(32) []
],
GR_SL_CTL2 [
    /// Slave reset. Each bit indicates whether the respective slave is enabled. If the slave is under reset, its clock is gated off (constant '0') and its resets are activated.
    ///
    /// Note: For peripheral group 0 slave 0,1, and 2 (the peripheral interconnect MMIO registers), this field is a constant '0' (SW: R): the slave can NOT be in reset.
    RST OFFSET(0) NUMBITS(32) []
],
GR_SL_CTL3 [
    /// Slave status to represent subsystem (SS) IP current power status. Each bit represents the respective IP power state (Note that separate mxsperi peripheral group should be defined for type4 peripheral, should not be mixed with type1/2/3 and same peripheral group can have multiple type4 peripherals)
    /// 0 - indiacates IP is in OFF state.
    /// 1 - indicates IP is in ON state.
    /// This register exists only for peripheral group with type4 peripherals (has its own PPU, P/Q-Channel consolidation and clock gating).
    /// This is readonly register connecting to PERI input signal coming from the respective SS IP.
    /// Since this register is passthorugh of status signal from peripheral the default value defined is w.r.t. respective IP reset.
    SS_POWERSTATE OFFSET(0) NUMBITS(32) []
],
GR_SL_WOUND [
    /// Slave disabled. Each bit indicates whether the respective slave is disabled. Setting this bit to 1 has the same effect as setting SL_CTL.ENABLED_0 to 0.  However, once set to 1, this bit cannot be changed back to 0 anymore.
    ///
    /// Note: For peripheral group 0 slave 0,1, and 2 (the peripheral interconnect MMIO registers), this field is a constant '0' (SW: R): the slave can NOT be disabled.
    DISABLED OFFSET(0) NUMBITS(32) []
],
];
const PERI_BASE: StaticRef<PeriRegisters> =
    unsafe { StaticRef::new(0x42000000 as *const PeriRegisters) };

pub struct Peri {
    registers: StaticRef<PeriRegisters>,
}

impl Peri {
    pub const fn new() -> Self {
        Peri {
            registers: PERI_BASE,
        }
    }

    pub fn sys_init_enable_peri(&self) {
        /* Reset values for each PERI group */
        const CY_PERI_GR1_SL_CTL: u32 = 0x0F;
        const CY_PERI_GR2_SL_CTL: u32 = 0x03;
        const CY_PERI_GR3_SL_CTL: u32 = 0x3F;
        const CY_PERI_GR4_SL_CTL: u32 = 0x03;
        const CY_PERI_GR5_SL_CTL: u32 = 0x01;

        const GROUP_SL_CTL_ENABLE_ALL: u32 = 0xFFFF_FFFF;

        if self.registers.gr_1_sl_ctl.get() != CY_PERI_GR1_SL_CTL {
            self.registers.gr_1_sl_ctl2.set(0);
            self.registers.gr_1_sl_ctl.set(GROUP_SL_CTL_ENABLE_ALL);
        }

        if self.registers.gr_2_sl_ctl.get() != CY_PERI_GR2_SL_CTL {
            self.registers.gr_2_sl_ctl2.set(0);
            self.registers.gr_2_sl_ctl.set(GROUP_SL_CTL_ENABLE_ALL);
        }

        if self.registers.gr_3_sl_ctl.get() != CY_PERI_GR3_SL_CTL {
            self.registers.gr_3_sl_ctl2.set(0);
            self.registers.gr_3_sl_ctl.set(GROUP_SL_CTL_ENABLE_ALL);
        }

        if self.registers.gr_4_sl_ctl.get() != CY_PERI_GR4_SL_CTL {
            self.registers.gr_4_sl_ctl2.set(0);
            self.registers.gr_4_sl_ctl.set(GROUP_SL_CTL_ENABLE_ALL);
        }

        if self.registers.gr_5_sl_ctl.get() != CY_PERI_GR5_SL_CTL {
            self.registers.gr_5_sl_ctl2.set(0);
            self.registers.gr_5_sl_ctl.set(GROUP_SL_CTL_ENABLE_ALL);
        }
    }
}
