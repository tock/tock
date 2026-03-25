use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

#[repr(C)]
struct HsiomPort {
    /// Port selection 0
    port_sel0: ReadWrite<u32, PRT_PORT_SEL0::Register>,
    /// Port selection 1
    port_sel1: ReadWrite<u32, PRT_PORT_SEL1::Register>,
    _reserved: [u32; 2],
}

#[repr(C)]
struct HsiomSecurePtr {
    /// Non-Secure Mask
    secure_prt_nonsecure_mask: ReadWrite<u32, SECURE_PRT_NONSECURE_MASK::Register>,
    _reserved: [u32; 3],
}

#[repr(C)]
struct AmuxSplitCtl {
    /// AMUX splitter cell control
    amux_split_ctl: ReadWrite<u32, AMUX_SPLIT_CTL::Register>,
}

register_structs! {
    /// IO Matrix (IOM)
    HsiomRegisters {
        (0x000 => ports: [HsiomPort; 10]),
        (0x0A0 => _reserved0),
        (0x1000 => secure_prts: [HsiomSecurePtr; 10]),
        (0x10A0 => _reserved1),
        (0x2000 => amux_split_ctls: [AmuxSplitCtl; 64]),
        (0x2100 => _reserved2),
        /// Power/Ground Monitor cell control 0
        (0x2200 => monitor_ctl_0: ReadWrite<u32, MONITOR_CTL::Register>),
        /// Power/Ground Monitor cell control 1
        (0x2204 => monitor_ctl_1: ReadWrite<u32, MONITOR_CTL::Register>),
        /// Power/Ground Monitor cell control 2
        (0x2208 => monitor_ctl_2: ReadWrite<u32, MONITOR_CTL::Register>),
        /// Power/Ground Monitor cell control 3
        (0x220C => monitor_ctl_3: ReadWrite<u32, MONITOR_CTL::Register>),
        (0x2210 => @END),
    }
}
register_bitfields![u32,
AMUX_SPLIT_CTL [
    /// T-switch control for Left AMUXBUSA switch:
    /// '0': switch open.
    /// '1': switch closed.
    SWITCH_AA_SL OFFSET(0) NUMBITS(1) [],
    /// T-switch control for Right AMUXBUSA switch:
    /// '0': switch open.
    /// '1': switch closed.
    SWITCH_AA_SR OFFSET(1) NUMBITS(1) [],
    /// T-switch control for AMUXBUSA vssa/ground switch:
    /// '0': switch open.
    /// '1': switch closed.
    SWITCH_AA_S0 OFFSET(2) NUMBITS(1) [],
    /// T-switch control for Left AMUXBUSB switch.
    SWITCH_BB_SL OFFSET(4) NUMBITS(1) [],
    /// T-switch control for Right AMUXBUSB switch.
    SWITCH_BB_SR OFFSET(5) NUMBITS(1) [],
    /// T-switch control for AMUXBUSB vssa/ground switch.
    SWITCH_BB_S0 OFFSET(6) NUMBITS(1) []
],
MONITOR_CTL [
    /// control for switch, which connects the power/ground supply to AMUXBUS_A/B respectively when switch is closed:
    /// '0': switch open.
    /// '1': switch closed.
    MONITOR_EN OFFSET(0) NUMBITS(32) []
],
PRT_PORT_SEL0 [
    /// Selects connection for IO pin 0 route.
    IO0_SEL OFFSET(0) NUMBITS(5) [
        /// GPIO controls 'out'
        GPIOControlsOut = 0,
        /// GPIO controls 'out', DSI controls 'output enable'
        GPIOControlsOutDSIControlsOutputEnable = 1,
        /// DSI controls 'out' and 'output enable'
        DSIControlsOutAndOutputEnable = 2,
        /// DSI controls 'out', GPIO controls 'output enable'
        DSIControlsOutGPIOControlsOutputEnable = 3,
        /// Analog mux bus A
        AnalogMuxBusA = 4,
        /// Analog mux bus B
        AnalogMuxBusB = 5,
        /// Analog mux bus A, DSI control
        AnalogMuxBusADSIControl = 6,
        /// Analog mux bus B, DSI control
        AnalogMuxBusBDSIControl = 7,
        /// Active functionality 0
        ActiveFunctionality0 = 8,
        /// Active functionality 1
        ActiveFunctionality1 = 9,
        /// Active functionality 2
        ActiveFunctionality2 = 10,
        /// Active functionality 3
        ActiveFunctionality3 = 11,
        /// DeepSleep functionality 0
        DeepSleepFunctionality0 = 12,
        /// DeepSleep functionality 1
        DeepSleepFunctionality1 = 13,
        /// DeepSleep functionality 2
        DeepSleepFunctionality2 = 14,
        /// DeepSleep functionality 3
        DeepSleepFunctionality3 = 15,
        /// Active functionality 4
        ActiveFunctionality4 = 16,
        /// Active functionality 5
        ActiveFunctionality5 = 17,
        /// Active functionality 6
        ActiveFunctionality6 = 18,
        /// Active functionality 7
        ActiveFunctionality7 = 19,
        /// Active functionality 8
        ActiveFunctionality8 = 20,
        /// Active functionality 9
        ActiveFunctionality9 = 21,
        /// Active functionality 10
        ActiveFunctionality10 = 22,
        /// Active functionality 11
        ActiveFunctionality11 = 23,
        /// Active functionality 12
        ActiveFunctionality12 = 24,
        /// Active functionality 13
        ActiveFunctionality13 = 25,
        /// Active functionality 14
        ActiveFunctionality14 = 26,
        /// Active functionality 15
        ActiveFunctionality15 = 27,
        /// DeepSleep functionality 4
        DeepSleepFunctionality4 = 28,
        /// DeepSleep functionality 5
        DeepSleepFunctionality5 = 29,
        /// DeepSleep functionality 6
        DeepSleepFunctionality6 = 30,
        /// DeepSleep functionality 7
        DeepSleepFunctionality7 = 31
    ],
    /// Selects connection for IO pin 1 route.
    IO1_SEL OFFSET(8) NUMBITS(5) [],
    /// Selects connection for IO pin 2 route.
    IO2_SEL OFFSET(16) NUMBITS(5) [],
    /// Selects connection for IO pin 3 route.
    IO3_SEL OFFSET(24) NUMBITS(5) []
],
PRT_PORT_SEL1 [
    /// Selects connection for IO pin 4 route.
    /// See PORT_SEL0 for connection details.
    IO4_SEL OFFSET(0) NUMBITS(5) [],
    /// Selects connection for IO pin 5 route.
    IO5_SEL OFFSET(8) NUMBITS(5) [],
    /// Selects connection for IO pin 6 route.
    IO6_SEL OFFSET(16) NUMBITS(5) [],
    /// Selects connection for IO pin 7 route.
    IO7_SEL OFFSET(24) NUMBITS(5) []
],
SECURE_PRT_NONSECURE_MASK [
    /// Non-secure attribute for IO0.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE0 OFFSET(0) NUMBITS(1) [],
    /// Non-secure attribute for IO1.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE1 OFFSET(1) NUMBITS(1) [],
    /// Non-secure attribute for IO2.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE2 OFFSET(2) NUMBITS(1) [],
    /// Non-secure attribute for IO3.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE3 OFFSET(3) NUMBITS(1) [],
    /// Non-secure attribute for IO4.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE4 OFFSET(4) NUMBITS(1) [],
    /// Non-secure attribute for IO5.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE5 OFFSET(5) NUMBITS(1) [],
    /// Non-secure attribute for IO6.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE6 OFFSET(6) NUMBITS(1) [],
    /// Non-secure attribute for IO7.
    /// 0 - Allows Secure access only.
    /// 1 - Allows Non-secure access only.
    NONSECURE7 OFFSET(7) NUMBITS(1) []
],
];
const HSIOM_BASE: StaticRef<HsiomRegisters> =
    unsafe { StaticRef::new(0x42400000 as *const HsiomRegisters) };

pub struct Hsiom {
    registers: StaticRef<HsiomRegisters>,
}

impl Hsiom {
    pub const fn new() -> Hsiom {
        Hsiom {
            registers: HSIOM_BASE,
        }
    }

    pub fn set_port_sel(&self, port: usize, pin: usize, function: u32) {
        todo!("");
    }
}
