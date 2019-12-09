//! User information configuration registers
//!
//! Minimal implementation to support activation of the reset button on
//! nRF52-DK.

use enum_primitive::cast::FromPrimitive;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

use crate::gpio::Pin;

const UICR_BASE: StaticRef<UicrRegisters> =
    unsafe { StaticRef::new(0x10001200 as *const UicrRegisters) };

#[repr(C)]
struct UicrRegisters {
    /// Mapping of the nRESET function (see POWER chapter for details)
    /// - Address: 0x200 - 0x204
    pselreset0: ReadWrite<u32, Pselreset::Register>,
    /// Mapping of the nRESET function (see POWER chapter for details)
    /// - Address: 0x204 - 0x208
    pselreset1: ReadWrite<u32, Pselreset::Register>,
    /// Access Port protection
    /// - Address: 0x208 - 0x20c
    approtect: ReadWrite<u32, ApProtect::Register>,
    /// Setting of pins dedicated to NFC functionality: NFC antenna or GPIO
    /// - Address: 0x20c - 0x210
    nfcpins: ReadWrite<u32, NfcPins::Register>,
    _reserved1: [u32; 60],
    /// External circuitry to be supplied from VDD pin.
    /// - Address: 0x300 - 0x304
    extsupply: ReadWrite<u32, ExtSupply::Register>,
    /// GPIO reference voltage
    /// - Address: 0x304 - 0x308
    regout0: ReadWrite<u32, RegOut::Register>,
}

register_bitfields! [u32,
    /// Task register
    Pselreset [
        /// GPIO number Px.nn onto which Reset is exposed
        PIN OFFSET(0) NUMBITS(5) [],
        /// GPIO port number Pn.xx onto with Reset is exposed
        PORT OFFSET(5) NUMBITS(1) [],
        /// Connection
        CONNECTION OFFSET(31) NUMBITS(1) [
            DISCONNECTED = 1,
            CONNECTED = 0
        ]
    ],
    /// Access port protection
    ApProtect [
        /// Ready event
        PALL OFFSET(0) NUMBITS(8) [
            /// Enable
            ENABLED = 0x00,
            /// Disable
            DISABLED = 0xff
        ]
    ],
    /// Setting of pins dedicated to NFC functionality: NFC antenna or GPIO
    NfcPins [
        /// Setting pins dedicated to NFC functionality
        PROTECT OFFSET(0) NUMBITS(1) [
            /// Operation as GPIO pins. Same protection as normal GPIO pins
            DISABLED = 0,
            /// Operation as NFC antenna pins. Configures the protection for
            /// NFC operation
            NFC = 1
        ]
    ],
    /// Enable external circuitry to be supplied from VDD pin
    ExtSupply [
        /// Enable external circuitry to be supplied from VDD pin
        EXTSUPPLY OFFSET(0) NUMBITS(1) [
            /// No current can be drawn from the VDD pin
            DISABLED = 0,
            /// It is allowed to supply external circuitry from the VDD pin
            ENABLED = 1
        ]
    ],
    /// GPIO reference voltage / external output supply voltage
    RegOut [
        /// Output voltage from REG0 regulator stage
        VOUT OFFSET(0) NUMBITS(3) [
            V1_8 = 0,
            V2_1 = 1,
            V2_4 = 2,
            V2_7 = 3,
            V3_0 = 4,
            V3_3 = 5,
            DEFAULT = 7
        ]
    ]
];

pub struct Uicr {
    registers: StaticRef<UicrRegisters>,
}

#[derive(Copy, Clone, PartialEq)]
/// Output voltage from REG0 regulator stage.
/// The value is board dependent (e.g. the nRF52840dk board uses 1.8V
/// whereas the nRF52840-Dongle requires 3.0V to light its LEDs).
/// When a chip is out of the factory or fully erased, the default value (7)
/// will output 1.8V.
pub enum Regulator0Output {
    V1_8 = 0,
    V2_1 = 1,
    V2_4 = 2,
    V2_7 = 3,
    V3_0 = 4,
    V3_3 = 5,
    DEFAULT = 7,
}

impl From<u32> for Regulator0Output {
    fn from(val: u32) -> Self {
        match val & 7 {
            0 => Regulator0Output::V1_8,
            1 => Regulator0Output::V2_1,
            2 => Regulator0Output::V2_4,
            3 => Regulator0Output::V2_7,
            4 => Regulator0Output::V3_0,
            5 => Regulator0Output::V3_3,
            7 => Regulator0Output::DEFAULT,
            _ => Regulator0Output::DEFAULT, // Invalid value, fall back to DEFAULT
        }
    }
}

impl Uicr {
    pub const fn new() -> Uicr {
        Uicr {
            registers: UICR_BASE,
        }
    }

    pub fn set_psel0_reset_pin(&self, pin: Pin) {
        let regs = &*self.registers;
        regs.pselreset0.set(pin as u32);
    }

    pub fn get_psel0_reset_pin(&self) -> Option<Pin> {
        Pin::from_u32(self.registers.pselreset0.get())
    }

    pub fn set_psel1_reset_pin(&self, pin: Pin) {
        let regs = &*self.registers;
        regs.pselreset1.set(pin as u32);
    }

    pub fn get_psel1_reset_pin(&self) -> Option<Pin> {
        Pin::from_u32(self.registers.pselreset1.get())
    }

    pub fn set_vout(&self, vout: Regulator0Output) {
        let regs = &*self.registers;
        regs.regout0.modify(RegOut::VOUT.val(vout as u32));
    }

    pub fn get_vout(&self) -> Regulator0Output {
        Regulator0Output::from(self.registers.regout0.read(RegOut::VOUT))
    }

    pub fn set_nfc_pins_protection(&self, protected: bool) {
        let regs = &*self.registers;
        if protected {
            regs.nfcpins.write(NfcPins::PROTECT::NFC);
        } else {
            regs.nfcpins.write(NfcPins::PROTECT::DISABLED);
        }
    }

    pub fn is_nfc_pins_protection_enabled(&self) -> bool {
        self.registers.nfcpins.matches_all(NfcPins::PROTECT::NFC)
    }

    pub fn is_ap_protect_enabled(&self) -> bool {
        // Here we compare to DISABLED value because any other value should enable the protection.
        !self
            .registers
            .approtect
            .matches_all(ApProtect::PALL::DISABLED)
    }

    pub fn set_ap_protect(&self) {
        self.registers.approtect.write(ApProtect::PALL::ENABLED);
    }
}
