use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

#[repr(C)]
struct HsiomPort {
    /// Port selection 0
    /// 8 bits for each pin, 5 bits used for selection, 3 bits reserved
    port_sel0: ReadWrite<u32>,
    /// Port selection 1
    /// 8 bits for each pin, 5 bits used for selection, 3 bits reserved
    port_sel1: ReadWrite<u32>,
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
        (0x1000 => secure_prts: [HsiomSecurePtr; 10]), // ERROR not all have 8 pins :(
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
const HSIOM_PORT_COUNT: u32 = 10;
const HSIOM_PINS_PER_PORT: u32 = 8;
const HSIOM_SEC_MASK: u32 = 0x1;

pub struct Hsiom {
    registers: StaticRef<HsiomRegisters>,
}

pub enum HsiomFunction {
    /// GPIO controls 'out'
    GPIOControlsOut = 0x00,
    /// GPIO controls 'out', DSI controls 'output enable'
    GPIOControlsOutDSIControlsOutputEnable = 0x01,
    /// DSI controls 'out' and 'output enable'
    DSIControlsOutAndOutputEnable = 0x02,
    /// DSI controls 'out', GPIO controls 'output enable'
    DSIControlsOutGPIOControlsOutputEnable = 0x03,
    /// Analog mux bus A
    AnalogMuxBusA = 0x04,
    /// Analog mux bus B
    AnalogMuxBusB = 0x05,
    /// Analog mux bus A, DSI control
    AnalogMuxBusADSIControl = 0x06,
    /// Analog mux bus B, DSI control
    AnalogMuxBusBDSIControl = 0x07,
    /// Active functionality 0
    ActiveFunctionality0 = 0x08,
    /// Active functionality 1
    ActiveFunctionality1 = 0x09,
    /// Active functionality 2
    ActiveFunctionality2 = 0x0A,
    /// Active functionality 3
    ActiveFunctionality3 = 0x0B,
    /// DeepSleep functionality 0
    DeepSleepFunctionality0 = 0x0C,
    /// DeepSleep functionality 1
    DeepSleepFunctionality1 = 0x0D,
    /// DeepSleep functionality 2
    DeepSleepFunctionality2 = 0x0E,
    /// DeepSleep functionality 3
    DeepSleepFunctionality3 = 0x0F,
    /// Active functionality 4
    ActiveFunctionality4 = 0x10,
    /// Active functionality 5
    ActiveFunctionality5 = 0x11,
    /// Active functionality 6
    ActiveFunctionality6 = 0x12,
    /// Active functionality 7
    ActiveFunctionality7 = 0x13,
    /// Active functionality 8
    ActiveFunctionality8 = 0x14,
    /// Active functionality 9
    ActiveFunctionality9 = 0x15,
    /// Active functionality 10
    ActiveFunctionality10 = 0x16,
    /// Active functionality 11
    ActiveFunctionality11 = 0x17,
    /// Active functionality 12
    ActiveFunctionality12 = 0x18,
    /// Active functionality 13
    ActiveFunctionality13 = 0x19,
    /// Active functionality 14
    ActiveFunctionality14 = 0x1A,
    /// Active functionality 15
    ActiveFunctionality15 = 0x1B,
    /// DeepSleep functionality 4
    DeepSleepFunctionality4 = 0x1C,
    /// DeepSleep functionality 5
    DeepSleepFunctionality5 = 0x1D,
    /// DeepSleep functionality 6
    DeepSleepFunctionality6 = 0x1E,
    /// DeepSleep functionality 7
    DeepSleepFunctionality7 = 0x1F,
}
const GPIO_HALF: u32 = 4;

impl Hsiom {
    pub const fn new() -> Hsiom {
        Hsiom {
            registers: HSIOM_BASE,
        }
    }

    #[no_mangle]
    pub fn set_port_sel(&self, port: u32, pin: u32, function: HsiomFunction) {
        assert!(port < HSIOM_PORT_COUNT && pin < HSIOM_PINS_PER_PORT);

        let port_addr = &self.registers.ports[port as usize];

        // Each pin occupies 8 bits with 5 bits used for the selection
        // Offset calculation: pin position within register * 8 bits per pin
        let bit_offset = pin << 3;
        let mask = 0x1F << bit_offset;

        let register = if pin < GPIO_HALF {
            &port_addr.port_sel0
        } else {
            &port_addr.port_sel1
        };
        let function_value = (function as u32) << bit_offset;

        let old_value = register.get();
        let new_value = (old_value & !mask) | (function_value as u32 & mask);
        register.set(new_value);
    }

    /// Configure the non-secure access mask for one pin in a secure GPIO port.
    ///
    /// `nonsecure = false` means secure access only.
    /// `nonsecure = true` means non-secure access only.
    pub fn set_secure_port_nonsecure_pin(&self, port: u32, pin: u32, nonsecure: bool) {
        assert!(port < HSIOM_PORT_COUNT && pin < HSIOM_PINS_PER_PORT);

        let register = &self.registers.secure_prts[port as usize].secure_prt_nonsecure_mask;
        let bit_mask = HSIOM_SEC_MASK << pin;
        let new_bit = (nonsecure as u32) << pin;

        let old_value = register.get();
        register.set((old_value & !bit_mask) | new_bit);
    }
}
