use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// SYSCPUSS registers
    CpussRegisters {
        /// Identity
        (0x000 => identity: ReadWrite<u32, IDENTITY::Register>),
        (0x004 => _reserved0),
        /// Product identifier and version (same as CoreSight RomTables)
        (0x010 => product_id: ReadWrite<u32, PRODUCT_ID::Register>),
        (0x014 => _reserved1),
        /// Debug port status
        (0x020 => dp_status: ReadWrite<u32, DP_STATUS::Register>),
        (0x024 => _reserved2),
        /// Buffer control
        (0x030 => buff_ctl: ReadWrite<u32, BUFF_CTL::Register>),
        (0x034 => _reserved3),
        /// Calibration support set and read
        (0x040 => cal_sup_set: ReadWrite<u32, CAL_SUP_SET::Register>),
        /// Calibration support clear and reset
        (0x044 => cal_sup_clr: ReadWrite<u32, CAL_SUP_CLR::Register>),
        (0x048 => _reserved4),
        /// Infrastructure Control
        (0x050 => infra_ctl: ReadWrite<u32, INFRA_CTL::Register>),
        (0x054 => _reserved5),
        /// Secure SysTick timer control
        (0x100 => systick_s_ctl: ReadWrite<u32, SYSTICK_S_CTL::Register>),
        (0x104 => _reserved6),
        /// Non Secure SysTick timer control
        (0x120 => systick_ns_ctl: ReadWrite<u32, SYSTICK_NS_CTL::Register>),
        (0x124 => _reserved7),
        /// Master security controller Interrupt
        (0x200 => intr_msc: ReadWrite<u32, INTR_MSC::Register>),
        (0x204 => _reserved8),
        /// Master security controller Interrupt mask
        (0x208 => intr_mask_msc: ReadWrite<u32, INTR_MASK_MSC::Register>),
        /// Master security controller Interrupt masked
        (0x20C => intr_masked_msc: ReadWrite<u32, INTR_MASKED_MSC::Register>),
        (0x210 => _reserved9),
        /// Access port control
        (0x1000 => ap_ctl: ReadWrite<u32, AP_CTL::Register>),
        (0x1004 => _reserved10),
        /// Protection status
        (0x2004 => protection: ReadWrite<u32, PROTECTION::Register>),
        (0x2008 => @END),
    }
}
register_bitfields![u32,
IDENTITY [
    /// This field specifies the privileged setting ('0': user mode; '1': privileged mode) of the transfer that reads the register.
    P OFFSET(0) NUMBITS(1) [],
    /// This field specifies the security setting ('0': secure mode; '1': non-secure mode) of the transfer that reads the register.
    NS OFFSET(1) NUMBITS(1) [],
    /// This field specifies the protection context of the transfer that reads the register.
    PC OFFSET(4) NUMBITS(4) [],
    /// This field specifies the bus master identifier of the transfer that reads the register.
    MS OFFSET(8) NUMBITS(8) []
],
PRODUCT_ID [
    /// Family ID. Common ID for a product family.
    FAMILY_ID OFFSET(0) NUMBITS(12) [],
    /// Major Revision, starts with 1, increments with all layer tape-out (implemented with metal ECO-able  tie-off)
    MAJOR_REV OFFSET(16) NUMBITS(4) [],
    /// Minor Revision, starts with 1, increments with metal layer only tape-out (implemented with metal ECO-able  tie-off)
    MINOR_REV OFFSET(20) NUMBITS(4) []
],
DP_STATUS [
    /// Specifies if the SWJ debug port is connected; i.e. debug host interface is active:
/// '0': Not connected/not active.
/// '1': Connected/active.
    SWJ_CONNECTED OFFSET(0) NUMBITS(1) [],
    /// Specifies if SWJ debug is enabled, i.e. CDBGPWRUPACK is '1' and thus debug clocks are on:
/// '0': Disabled.
/// '1': Enabled.
    SWJ_DEBUG_EN OFFSET(1) NUMBITS(1) [],
    /// Specifies if the JTAG interface is selected.
/// '0': JTAG not selected.
/// '1': JTAG selected.
    SWJ_JTAG_SEL OFFSET(2) NUMBITS(1) [],
    /// Specifies if the SWD interface is selected.
/// '0': SWD not selected.
/// '1': SWD selected.
    SWJ_SWD_SEL OFFSET(3) NUMBITS(1) []
],
BUFF_CTL [
    /// Specifies if write transfer can be buffered in the bus infrastructure bridges:
/// '0': Write transfers are not buffered, independent of the transfer's bufferable attribute.
/// '1': Write transfers can be buffered, if the transfer's bufferable attribute indicates that the transfer is a bufferable/posted write.
///
/// This bit will control only the IPs which use mxambatk AHB2AHB bridge (mxambatk_ahb2ahb) and it will NOT control the buffering that may be happening in bus infrastructure components used from ARM SIE200.
    WRITE_BUFF OFFSET(0) NUMBITS(1) []
],
CAL_SUP_SET [
    /// Read without side effect, write 1 to set
    DATA OFFSET(0) NUMBITS(32) []
],
CAL_SUP_CLR [
    /// Read side effect: when read all bits are cleared, write 1 to clear a specific bit
/// Note: no exception for the debug host, it also causes the read side effect
    DATA OFFSET(0) NUMBITS(32) []
],
INFRA_CTL [
    /// Force Infrastructure clock gating to be always ON.
/// 0: Disabled
/// 1: Enabled
    CLOCK_FORCE OFFSET(0) NUMBITS(1) []
],
SYSTICK_S_CTL [
    /// Specifies the number of clock source cycles (minus 1) that make up 10 ms. E.g., for a 32,768 Hz reference clock, TENMS is 328 - 1 = 327.
    TENMS OFFSET(0) NUMBITS(24) [],
    /// Specifies an external clock source:
/// '0': The low frequency clock 'clk_lf' is selected. The precision of this clock depends on whether the low frequency clock source is a SRSS internal RC oscillator (imprecise) or a device external crystal oscillator (precise).
/// '1': The internal main oscillator (IMO) clock 'clk_imo' is selected. The MXS40 platform uses a fixed frequency IMO clock.
/// o '2': The external crystal oscillator (ECO) clock 'clk_eco' is selected.
/// '3': The SRSS 'clk_timer' is selected ('clk_timer' is a divided/gated version of 'clk_hf' or 'clk_imo').
///
/// Note: If NOREF is '1', the CLOCK_SOURCE value is NOT used.
/// Note: It is SW's responsibility to provide the correct NOREF, SKEW and TENMS field values for the selected clock source.
    CLOCK_SOURCE OFFSET(24) NUMBITS(2) [],
    /// Specifies the precision of the clock source and if the TENMS field represents exactly 10 ms (clock source frequency is a multiple of 100 Hz). This affects the suitability of the SysTick timer as a SW real-time clock:
/// '0': Precise.
/// '1': Imprecise.
    SKEW OFFSET(30) NUMBITS(1) [],
    /// Specifies if an external clock source is provided:
/// '0': An external clock source is provided.
/// '1': An external clock source is NOT provided and only the CPU internal clock can be used as SysTick timer clock source.
    NOREF OFFSET(31) NUMBITS(1) []
],
SYSTICK_NS_CTL [
    /// Specifies the number of clock source cycles (minus 1) that make up 10 ms. E.g., for a 32,768 Hz reference clock, TENMS is 328 - 1 = 327.
    TENMS OFFSET(0) NUMBITS(24) [],
    /// Specifies an external clock source:
/// '0': The low frequency clock 'clk_lf' is selected. The precision of this clock depends on whether the low frequency clock source is a SRSS internal RC oscillator (imprecise) or a device external crystal oscillator (precise).
/// '1': The internal main oscillator (IMO) clock 'clk_imo' is selected. The MXS40 platform uses a fixed frequency IMO clock.
/// o '2': The external crystal oscillator (ECO) clock 'clk_eco' is selected.
/// '3': The SRSS 'clk_timer' is selected ('clk_timer' is a divided/gated version of 'clk_hf' or 'clk_imo').
///
/// Note: If NOREF is '1', the CLOCK_SOURCE value is NOT used.
/// Note: It is SW's responsibility to provide the correct NOREF, SKEW and TENMS field values for the selected clock source.
    CLOCK_SOURCE OFFSET(24) NUMBITS(2) [],
    /// Specifies the precision of the clock source and if the TENMS field represents exactly 10 ms (clock source frequency is a multiple of 100 Hz). This affects the suitability of the SysTick timer as a SW real-time clock:
/// '0': Precise.
/// '1': Imprecise.
    SKEW OFFSET(30) NUMBITS(1) [],
    /// Specifies if an external clock source is provided:
/// '0': An external clock source is provided.
/// '1': An external clock source is NOT provided and only the CPU internal clock can be used as SysTick timer clock source.
    NOREF OFFSET(31) NUMBITS(1) []
],
INTR_MSC [
    /// This interrupt cause field is activated (HW sets the field to '1') when there is a MSC interrupt violation.
///
/// SW writes a '1' to this field to clear the interrupt cause to '0'. The HW captures a new MSC interrupt only after clearing the interrupt cause.
    CODE_MS0_MSC OFFSET(0) NUMBITS(1) [],
    /// N/A
    SYS_MS0_MSC OFFSET(1) NUMBITS(1) [],
    /// N/A
    SYS_MS1_MSC OFFSET(2) NUMBITS(1) [],
    /// N/A
    EXP_MS_MSC OFFSET(3) NUMBITS(1) [],
    /// N/A
    DMAC0_MSC OFFSET(4) NUMBITS(1) [],
    /// N/A
    DMAC1_MSC OFFSET(5) NUMBITS(1) []
],
INTR_MASK_MSC [
    /// Mask bit for corresponding field in the INTR register.
    CODE_MS0_MSC OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding field in the INTR register.
    SYS_MS0_MSC OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding field in the INTR register.
    SYS_MS1_MSC OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding field in the INTR register.
    EXP_MS_MSC OFFSET(3) NUMBITS(1) [],
    /// Mask bit for corresponding field in the INTR register.
    DMAC0_MSC OFFSET(4) NUMBITS(1) [],
    /// Mask bit for corresponding field in the INTR register.
    DMAC1_MSC OFFSET(5) NUMBITS(1) []
],
INTR_MASKED_MSC [
    /// Logical and of corresponding INTR and INTR_MASK fields.
    CODE_MS0_MSC OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding INTR and INTR_MASK fields.
    SYS_MS0_MSC OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding INTR and INTR_MASK fields.
    SYS_MS1_MSC OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding INTR and INTR_MASK fields.
    EXP_MS_MSC OFFSET(3) NUMBITS(1) [],
    /// Logical and of corresponding INTR and INTR_MASK fields.
    DMAC0_MSC OFFSET(4) NUMBITS(1) [],
    /// Logical and of corresponding INTR and INTR_MASK fields.
    DMAC1_MSC OFFSET(5) NUMBITS(1) []
],
AP_CTL [
    /// Enables the CM33_0 AP interface:
/// '0': Disabled.
/// '1': Enabled.
    CM33_0_ENABLE OFFSET(0) NUMBITS(1) [],
    /// Enables the CM33_1 AP interface:
/// '0': Disabled.
/// '1': Enabled.
    CM33_1_ENABLE OFFSET(1) NUMBITS(1) [],
    /// Enables the system AP interface:
/// '0': Disabled.
/// '1': Enabled.
    SYS_ENABLE OFFSET(2) NUMBITS(1) [],
    /// Invasive debug enable for CM33_0.
/// '0': Disables all halt-mode and invasive debug features.
/// '1': Enables invasive debug features.
    CM33_0_DBG_ENABLE OFFSET(4) NUMBITS(1) [],
    /// Non-invasive debug enable for CM33_0.
/// '0': Disables all trace and non-invasive debug features.
/// '1': Enables all trace and non-invasive debug features.
    CM33_0_NID_ENABLE OFFSET(5) NUMBITS(1) [],
    /// Secure invasive debug enable for CM33_0.
/// '0': disables all halt mode and invasive debug features when the processor is in Secure state.
/// '1': Enables all halt mode and invasive debug features when the processor is in Secure state.
    CM33_0_SPID_ENABLE OFFSET(6) NUMBITS(1) [],
    /// Secure non-invasive debug enable for CM33_0.
/// '0': Disables non-invasive debug features when the processor is in Secure state.
/// '1': Enables non-invasive debug features when the processor is in Secure state.
    CM33_0_SPNID_ENABLE OFFSET(7) NUMBITS(1) [],
    /// Refer CM33_0_DBG_ENABLE.
    CM33_1_DBG_ENABLE OFFSET(8) NUMBITS(1) [],
    /// Refer CM33_0_NID_ENABLE.
    CM33_1_NID_ENABLE OFFSET(9) NUMBITS(1) [],
    /// Refer CM33_0_SPID_ENABLE.
    CM33_1_SPID_ENABLE OFFSET(10) NUMBITS(1) [],
    /// Refer CM33_0_SPNID_ENABLE.
    CM33_1_SPNID_ENABLE OFFSET(11) NUMBITS(1) [],
    /// Enables the CM33_0 secure AP interface:
/// '0': Disabled.
/// '1': Enabled.
    CM33_0_SECURE_ENABLE OFFSET(12) NUMBITS(1) [],
    /// Enables the CM33_1 secure AP interface:
/// '0': Disabled.
/// '1': Enabled.
    CM33_1_SECURE_ENABLE OFFSET(13) NUMBITS(1) [],
    /// Enables the system secure AP interface:
/// '0': Disabled.
/// '1': Enabled.
    SYS_SECURE_ENABLE OFFSET(14) NUMBITS(1) [],
    /// Disables the CM33_0 AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when CM0_DISABLE is '0' and CM0_ENABLE is '1'.
    CM33_0_DISABLE OFFSET(16) NUMBITS(1) [],
    /// Disables the CM33_1 AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when CM33_DISABLE is '0' and CM33_ENABLE is '1'.
    CM33_1_DISABLE OFFSET(17) NUMBITS(1) [],
    /// Disables the system AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when SYS_DISABLE is '0' and SYS_ENABLE is '1'.
    SYS_DISABLE OFFSET(18) NUMBITS(1) [],
    /// Disable Invasive debug for CM33_0.
/// '1': Disables all halt-mode and invasive debug features.
/// '0': Enables invasive debug features.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The invasive debug is only enabled when CM33_0_DBG_DISABLE is '0' and CM33_0_DBG_ENABLE is '1'.
    CM33_0_DBG_DISABLE OFFSET(20) NUMBITS(1) [],
    /// Disable Non-invasive debug for CM33_0.
/// '1': Disables all trace and non-invasive debug features.
/// '0': Enables all trace and non-invasive debug features.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The non-invasive debug is only enabled when CM33_0_NID_DISABLE is '0' and CM33_0_NID_ENABLE is '1'.
    CM33_0_NID_DISABLE OFFSET(21) NUMBITS(1) [],
    /// Secure invasive debug disable for CM33_0.
/// '1': disables all halt mode and invasive debug features when the processor is in Secure state.
/// '0': Enables all halt mode and invasive debug features when the processor is in Secure state.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The invasive debug in secure state is only enabled when CM33_0_SPID_DISABLE is '0' and CM33_0_SPID_ENABLE is '1'.
    CM33_0_SPID_DISABLE OFFSET(22) NUMBITS(1) [],
    /// Secure non-invasive debug disable for CM33_0.
/// '1': Disables non-invasive debug features when the processor is in Secure state.
/// '0': Enables non-invasive debug features when the processor is in Secure state.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The non-invasive debug in secure state is only enabled when CM33_0_SPNID_DISABLE is '0' and CM33_0_SPNID_ENABLE is '1'.
    CM33_0_SPNID_DISABLE OFFSET(23) NUMBITS(1) [],
    /// Refer CM33_0_DBG_DISABLE description.
    CM33_1_DBG_DISABLE OFFSET(24) NUMBITS(1) [],
    /// Refer CM33_0_NID_DISABLE description.
    CM33_1_NID_DISABLE OFFSET(25) NUMBITS(1) [],
    /// Refer CM33_0_SPID_DISABLE description.
    CM33_1_SPID_DISABLE OFFSET(26) NUMBITS(1) [],
    /// Refer CM33_0_SPNID_DISABLE description.
    CM33_1_SPNID_DISABLE OFFSET(27) NUMBITS(1) [],
    /// Disables the CM33_0 secure AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when CM0_DISABLE is '0' and CM0_ENABLE is '1'.
    CM33_0_SECURE_DISABLE OFFSET(28) NUMBITS(1) [],
    /// Disables the CM33_1 secure AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when CM33_DISABLE is '0' and CM33_ENABLE is '1'.
    CM33_1_SECURE_DISABLE OFFSET(29) NUMBITS(1) [],
    /// Disables the system secure AP interface:
/// '0': Enabled.
/// '1': Disabled.
///
/// Typically, this field is set by the Cypress boot code with information from eFUSE. The access port is only enabled when SYS_DISABLE is '0' and SYS_ENABLE is '1'.
    SYS_SECURE_DISABLE OFFSET(30) NUMBITS(1) []
],
PROTECTION [
    /// Protection state:
///             PROTECTION is '0x5B719A4F': UNKNOWN state.
///             PROTECTION is '0x5D48F714': VIRGIN state.
///             PROTECTION is '0xC39D5455': OPEN state
///             PROTECTION is '0x652372F7': NORMAL state.
///             PROTECTION is '0x8DF117A1': SECURE state.
///             PROTECTION is '0xFBF6D1B6': RMA state
///             PROTECTION is '0x2E94B3DD': DEAD state.
///             PROTECTION is '0x3A5BC6F1': CORRUPTED state.
///             PROTECTION is '0x3F80442F': TESTMODE state.
///
/// The following state transitions are allowed (and enforced by HW):
/// - UNKNOWN => VIRGIN/RMA/OPEN/NORMAL/SECURE/DEAD/CORRUPTED
/// - RMA/OPEN/NORMAL/SECURE => DEAD
/// - RMA/OPEN/NORMAL/SECURE => TESTMODE
/// - RMA/OPEN/NORMAL/SECURE => CORRUPTED
/// An attempt to make a NOT allowed state transition will NOT affect this register field.
    STATE OFFSET(0) NUMBITS(32) []
]
];
const CPUSS_BASE: StaticRef<CpussRegisters> =
    unsafe { StaticRef::new(0x421C0000 as *const CpussRegisters) };

pub struct Cpuss {
    registers: StaticRef<CpussRegisters>,
}

impl Cpuss {
    pub const fn new() -> Cpuss {
        Cpuss {
            registers: CPUSS_BASE,
        }
    }
}
