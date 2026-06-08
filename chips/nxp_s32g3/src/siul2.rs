// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! System Integration Unit Lite2 (SIUL2) for NXP S32G3.
//!
//! Register definitions and bitfields are taken from the S32G3 Reference
//! Manual, Chapter 16.
//!
//! MSCRn and IMCRn registers must only be configured during application initialization
//! and must not be modified at runtime. They support only 32-bit accesses
//! (8/16-bit writes cause a transfer error), and accessing a reserved
//! instance also generates a transfer error.

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

// Base addresses from S32G3 memory map (RM §16.1.1 / §16.3 / §16.4).
/// Base address of SIUL2_0 instance.
pub const SIUL2_0_BASE: StaticRef<Siul2Registers> =
    unsafe { StaticRef::new(0x4009_C000 as *const Siul2Registers) };

/// Base address of SIUL2_1 instance.
pub const SIUL2_1_BASE: StaticRef<Siul2Registers> =
    unsafe { StaticRef::new(0x4401_0000 as *const Siul2Registers) };

// The MSCR/IMCR/GPDO/GPDI register arrays below cover the full 0..512
// instances of each register. The S32G3 silicon implements every index in
// the linear array; the per-instance chip-specific address map has holes
// (RM §16.3.1 / §16.4.1) and reserved instances generate a transfer error
// if accessed (RM §16.4.13, §16.4.14).
register_structs! {
    pub Siul2Registers {
        /// Reserved address space covering the SIUL2 MCU ID, DMA/interrupt
        /// configuration, MSCR0..MSCR111 region, and other control registers
        /// RM §16.3.1,
        /// RM §16.4.1.
        (0x000 => _reserved0),
        /// Multiplexed Signal Configuration Registers MSCR0..MSCR511
        /// RM §16.3.12,
        /// RM §16.4.13.
        (0x240 => mscr: [ReadWrite<u32, MSCR::Register>; 512]),
        /// Reserved address space between the MSCR and IMCR register arrays
        /// RM §16.3.1.
        (0x240 + 512 * 4 => _reserved1),
        /// Input Multiplexed Signal Configuration Registers IMCR0..IMCR511
        /// RM §16.3.13,
        /// RM §16.4.14.
        (0xA40 => imcr: [ReadWrite<u32, IMCR::Register>; 512]),
        /// Reserved address space between the IMCR and GPDO register arrays
        /// RM §16.3.1.
        (0xA40 + 512 * 4 => _reserved2),
        /// GPIO Pad Data Output Registers GPDO0..GPDO511
        /// RM §16.3.14,
        /// RM §16.4.15.
        (0x1300 => gpdo: [ReadWrite<u8, GPDO::Register>; 512]),
        /// Reserved address space between the GPDO and GPDI register arrays
        /// RM §16.3.1.
        (0x1300 + 512 => _reserved3),
        /// GPIO Pad Data Input Registers GPDI0..GPDI511
        /// RM §16.3.15,
        /// RM §16.4.16.
        (0x1500 => gpdi: [ReadOnly<u8, GPDI::Register>; 512]),
        (0x1500 + 512 => @END),
    }
}

register_bitfields![u32,
    /// Multiplexed Signal Configuration Register
    /// RM §16.3.12,
    /// RM §16.4.13.
    /// Configures pad electrical properties and the alternate-function source
    /// signal select (SSS) for the corresponding chip pin.
    pub MSCR [
        /// Reserved. Write 0; reads return 0 (RM §16.3.12 field `31-22`).
        _RSV_22_31 OFFSET(22) NUMBITS(10) [],
        /// GPIO Output Buffer Enable. Applies only to digital pins; otherwise
        /// this bit is reserved (RM §16.3.12 field `21 OBE`).
        OBE  OFFSET(21) NUMBITS(1) [
            /// Output driver disabled.
            Disabled = 0,
            /// Output driver enabled.
            Enabled = 1,
        ],
        /// Open Drain Enable. Open-drain is active only when both OBE and ODE
        /// are set (RM §16.3.12 field `20 ODE`).
        ODE  OFFSET(20) NUMBITS(1) [
            /// Open-drain function disabled.
            Disabled = 0,
            /// Open-drain function enabled (when OBE is also 1).
            Enabled = 1,
        ],
        /// Input Buffer Enable. Enables the associated pin's input buffer
        /// (RM §16.3.12 field `19 IBE`).
        IBE  OFFSET(19) NUMBITS(1) [
            /// Input buffer disabled.
            Disabled = 0,
            /// Input buffer enabled.
            Enabled = 1,
        ],
        /// Reserved. Has no function (RM §16.3.12 field `18-17`).
        _RSV_17_18 OFFSET(17) NUMBITS(2) [],
        /// Slew Rate Control. Selects the maximum supported toggle frequency
        /// for the pad. The mapping depends on the pad type (3.3V/1.8V FAST,
        /// 1.8V GPIO, or 3.3V GPIO); see RM §16.3.12 field `16-14 SRE` and
        /// RM §16.4.13 field `16-14 SRE` for the per-type Fmax tables.
        SRE  OFFSET(14) NUMBITS(3) [
            /// Fmax = 208 MHz (1.8V), 166 MHz (3.3V) on FAST pads.
            Sre208M = 0b000,
            /// Reserved. Do not use.
            Reserved001 = 0b001,
            /// Reserved. Do not use.
            Reserved010 = 0b010,
            /// Reserved. Do not use.
            Reserved011 = 0b011,
            /// Fmax = 166 MHz (1.8V), 150 MHz (3.3V) on FAST pads.
            Sre166M = 0b100,
            /// Fmax = 150 MHz (1.8V), 133 MHz (3.3V) on FAST pads.
            Sre150M = 0b101,
            /// Fmax = 133 MHz (1.8V), 100 MHz (3.3V) on FAST pads.
            Sre133M = 0b110,
            /// Fmax = 100 MHz (1.8V), 83 MHz (3.3V) on FAST pads.
            Sre100M = 0b111,
        ],
        /// Pull Enable. Enables the pull function on the pad
        /// (RM §16.3.12 field `13 PUE`).
        PUE  OFFSET(13) NUMBITS(1) [
            /// Pull disabled.
            Disabled = 0,
            /// Pull enabled.
            Enabled = 1,
        ],
        /// Pull Select. Selects pullup or pulldown when PUE is set
        /// (RM §16.3.12 field `12 PUS`).
        PUS  OFFSET(12) NUMBITS(1) [
            /// Pulldown.
            Pulldown = 0,
            /// Pullup.
            Pullup = 1,
        ],
        /// Reserved. Has no function (RM §16.3.12 fields `11`, `10`, `9-6`).
        _RSV_6_11 OFFSET(6) NUMBITS(6) [],
        /// Safe Mode Control. Specifies whether the chip disables the pin's
        /// output buffer when entering FCCU Fault state (SIUL2_0) or chip Safe
        /// mode (SIUL2_1) — see RM §16.3.12 field `5 SMC` and
        /// RM §16.4.13 field `5 SMC`.
        SMC  OFFSET(5) NUMBITS(1) [
            /// Disable: output buffer returns to its previous state on exit.
            Disable = 0,
            /// Do not disable: output buffer is held in its current state.
            Keep = 1,
        ],
        /// Reserved. Has no function (RM §16.3.12 field `4-3`).
        _RSV_3_4 OFFSET(3) NUMBITS(2) [],
        /// Source Signal Select. Selects the pad function (GPIO, ALT1..ALT4).
        /// Refer to the SSS column of the IOMUX spreadsheet attached to the RM
        /// (RM §16.3.12 field `2-0 SSS`, RM §16.4.13 field `2-0 SSS`).
        SSS  OFFSET(0) NUMBITS(3) [
            /// GPIO function (pad controlled by GPDO/GPDI).
            GPIO = 0,
            /// Alternate function 1.
            ALT1 = 1,
            /// Alternate function 2.
            ALT2 = 2,
            /// Alternate function 3.
            ALT3 = 3,
            /// Alternate function 4.
            ALT4 = 4,
        ]
    ],
    /// Input Multiplexed Signal Configuration Register
    /// RM §16.3.13, RM §16.4.14.
    /// Selects which source signal is connected to the associated peripheral
    /// input destination.
    pub IMCR [
        /// Reserved. Write 0; reads return 0 (RM §16.3.13 field `31-3`).
        _RSV_3_31 OFFSET(3) NUMBITS(29) [],
        /// Input Source Select. Selects which source signal is connected to
        /// the associated destination (chip pin) — see the SSS column of the
        /// IOMUX spreadsheet attached to the RM
        /// (RM §16.3.13 field `2-0 SSS`, RM §16.4.14 field `2-0 SSS`).
        SSS  OFFSET(0) NUMBITS(3) []
    ]
];

register_bitfields![u8,
    /// GPIO Pad Data Output Register
    /// RM §16.3.14, RM §16.4.15.
    /// Stores the data to be driven out on the external GPIO pad when the pad
    /// is configured as an output. Supports 8-, 16-, and 32-bit accesses.
    pub GPDO [
        /// Reserved. Write 0; reads return 0 (RM §16.3.14 field `7-1`).
        _RSV_1_7 OFFSET(1) NUMBITS(7) [],
        /// Pad Data Out. `PDO_n` where n is the register instance
        /// (RM §16.3.14 field `0 PDO_n`).
        VAL OFFSET(0) NUMBITS(1) [
            /// Logic low value driven on the pad.
            Low = 0,
            /// Logic high value driven on the pad.
            High = 1,
        ]
    ],
    /// GPIO Pad Data Input Register
    /// RM §16.3.15, RM §16.4.16.
    /// Reads the current value of the external GPIO pad. Supports 8-, 16-, and
    /// 32-bit accesses.
    pub GPDI [
        /// Reserved. Read as 0 (RM §16.3.15 field `7-1`).
        _RSV_1_7 OFFSET(1) NUMBITS(7) [],
        /// Pad Data In. `PDI_n` where n is the register instance
        /// (RM §16.3.15 field `0 PDI_n`).
        VAL OFFSET(0) NUMBITS(1) [
            /// Logic low observed on the pad.
            Low = 0,
            /// Logic high observed on the pad.
            High = 1,
        ]
    ]
];

/// Type-safe identifier for an SIUL2 chip pin.
///
/// The discriminant is the MSCR/GPDO/GPDI instance index, matching the S32G3
/// pad naming (`PA0..PA15`, `PB0..PB15`, ...). See RM §16.1 for the
/// chip-specific SIUL2 instance layout and the IOMUX spreadsheet attached to
/// the RM for the pin-to-function mapping.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Pin {
    PA0 = 0,
    PA1 = 1,
    PA2 = 2,
    PA3 = 3,
    PA4 = 4,
    PA5 = 5,
    PA6 = 6,
    PA7 = 7,
    PA8 = 8,
    PA9 = 9,
    PA10 = 10,
    PA11 = 11,
    PA12 = 12,
    PA13 = 13,
    PA14 = 14,
    PA15 = 15,

    PB0 = 16,
    PB1 = 17,
    PB2 = 18,
    PB3 = 19,
    PB4 = 20,
    PB5 = 21,
    PB6 = 22,
    PB7 = 23,
    PB8 = 24,
    PB9 = 25,
    PB10 = 26,
    PB11 = 27,
    PB12 = 28,
    PB13 = 29,
    PB14 = 30,
    PB15 = 31,

    PC0 = 32,
    PC1 = 33,
    PC2 = 34,
    PC3 = 35,
    PC4 = 36,
    PC5 = 37,
    PC6 = 38,
    PC7 = 39,
    PC8 = 40,
    PC9 = 41,
    PC10 = 42,
    PC11 = 43,
    PC12 = 44,
    PC13 = 45,
    PC14 = 46,
    PC15 = 47,

    PD0 = 48,
    PD1 = 49,
    PD2 = 50,
    PD3 = 51,
    PD4 = 52,
    PD5 = 53,
    PD6 = 54,
    PD7 = 55,
    PD8 = 56,
    PD9 = 57,
    PD10 = 58,
    PD11 = 59,
    PD12 = 60,
    PD13 = 61,
    PD14 = 62,
    PD15 = 63,

    PE0 = 64,
    PE1 = 65,
    PE2 = 66,
    PE3 = 67,
    PE4 = 68,
    PE5 = 69,
    PE6 = 70,
    PE7 = 71,
    PE8 = 72,
    PE9 = 73,
    PE10 = 74,
    PE11 = 75,
    PE12 = 76,
    PE13 = 77,
    PE14 = 78,
    PE15 = 79,

    PF0 = 80,
    PF1 = 81,
    PF2 = 82,
    PF3 = 83,
    PF4 = 84,
    PF5 = 85,
    PF6 = 86,
    PF7 = 87,
    PF8 = 88,
    PF9 = 89,
    PF10 = 90,
    PF11 = 91,
    PF12 = 92,
    PF13 = 93,
    PF14 = 94,
    PF15 = 95,

    PG0 = 96,
    PG1 = 97,
    PG2 = 98,
    PG3 = 99,
    PG4 = 100,
    PG5 = 101,

    PH0 = 112,
    PH1 = 113,
    PH2 = 114,
    PH3 = 115,
    PH4 = 116,
    PH5 = 117,
    PH6 = 118,
    PH7 = 119,
    PH8 = 120,
    PH9 = 121,
    PH10 = 122,
    PH11 = 123,
    PH12 = 124,
    PH13 = 125,
    PH14 = 126,
    PH15 = 127,

    PI0 = 128,
    PI1 = 129,
    PI2 = 130,
    PI3 = 131,
    PI4 = 132,
    PI5 = 133,
    PI6 = 134,
    PI7 = 135,
    PI8 = 136,
    PI9 = 137,
    PI10 = 138,
    PI11 = 139,
    PI12 = 140,
    PI13 = 141,
    PI14 = 142,
    PI15 = 143,

    PJ0 = 144,
    PJ1 = 145,
    PJ2 = 146,
    PJ3 = 147,
    PJ4 = 148,
    PJ5 = 149,
    PJ6 = 150,
    PJ7 = 151,
    PJ8 = 152,
    PJ9 = 153,
    PJ10 = 154,
    PJ11 = 155,
    PJ12 = 156,
    PJ13 = 157,
    PJ14 = 158,
    PJ15 = 159,

    PK0 = 160,
    PK1 = 161,
    PK2 = 162,
    PK3 = 163,
    PK4 = 164,
    PK5 = 165,
    PK6 = 166,
    PK7 = 167,
    PK8 = 168,
    PK9 = 169,
    PK10 = 170,
    PK11 = 171,
    PK12 = 172,
    PK13 = 173,
    PK14 = 174,
    PK15 = 175,

    PL0 = 176,
    PL1 = 177,
    PL2 = 178,
    PL3 = 179,
    PL4 = 180,
    PL5 = 181,
    PL6 = 182,
    PL7 = 183,
    PL8 = 184,
    PL9 = 185,
    PL10 = 186,
    PL11 = 187,
    PL12 = 188,
    PL13 = 189,
    PL14 = 190,
}

impl Pin {
    /// Returns the MSCR/GPDO/GPDI instance index for this pin.
    pub const fn value(self) -> usize {
        self as usize
    }
}

/// Type-safe identifier for an SIUL2 IMCR instance.
///
/// Discriminants are the IMCR register indices used by peripherals to select
/// their input source. See RM §16.3.13 (SIUL2_0) and §16.4.14 (SIUL2_1) for
/// the per-instance address mapping.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Imcr {
    /// IMCR0 — LINFlexD_0 RX input source select (SIUL2_0 instance).
    LinflexD0Rx = 0,
    /// IMCR224 — LINFlexD_1 RX input source select (SIUL2_1 instance).
    LinflexD1Rx = 224,
}
impl Imcr {
    /// Returns the IMCR instance index.
    pub const fn value(self) -> usize {
        self as usize
    }
}

/// Input source signal select for an IMCR instance.
///
/// Only values 0..=7 are encodable in the 3-bit SSS field
/// (RM §16.3.13 / §16.4.14 field `2-0 SSS`). The exact source-to-pad mapping
/// is documented in the IOMUX spreadsheet attached to the RM.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ImcrSource {
    /// Source signal 0.
    Alt0 = 0,
    /// Source signal 1.
    Alt1 = 1,
    /// Source signal 2.
    Alt2 = 2,
    /// Source signal 3.
    Alt3 = 3,
    /// Source signal 4.
    Alt4 = 4,
    /// Source signal 5.
    Alt5 = 5,
    /// Source signal 6.
    Alt6 = 6,
    /// Source signal 7.
    Alt7 = 7,
}
impl ImcrSource {
    /// Returns the raw SSS field value for this source.
    pub const fn value(self) -> u32 {
        self as u32
    }
}

/// SIUL2 driver instance.
///
/// Holds a `StaticRef` to the SIUL2 register block for one of the two
/// instances on the S32G3 (`SIUL2_0_BASE` or `SIUL2_1_BASE`). All MSCRn and
/// IMCRn configuration should be done during application initialization per
/// RM §16.3.12 / §16.4.13; runtime modification is not supported by the
/// hardware.
pub struct Siul2 {
    registers: StaticRef<Siul2Registers>,
}

impl Siul2 {
    /// Creates a new SIUL2 driver instance bound to the given register block.
    pub const fn new(registers: StaticRef<Siul2Registers>) -> Self {
        Self { registers }
    }

    /// Configures the MSCR for a chip pin.
    ///
    /// - `alt`: Source Signal Select value (0..=4 — see `MSCR::SSS` variants).
    /// - `obe`: Enable the output buffer (`true` = driven, `false` = high-Z).
    /// - `ibe`: Enable the input buffer.
    /// - `sre`: Slew-rate control value (0..=7 — see `MSCR::SRE` variants).
    ///
    /// Per RM §16.3.12, MSCRn must be configured only at application
    /// initialization and supports only 32-bit accesses.
    pub fn setup_mscr(&self, pin: Pin, alt: u32, obe: bool, ibe: bool, sre: u32) {
        // Mask incoming values down to the SSS/SRE field widths to avoid
        // accidentally setting reserved bits when callers pass raw u32s.
        self.registers.mscr[pin.value()].write(
            MSCR::SSS.val(alt & MSCR::SSS.mask)
                + MSCR::OBE.val(u32::from(obe))
                + MSCR::IBE.val(u32::from(ibe))
                + MSCR::SRE.val(sre & MSCR::SRE.mask),
        );
    }

    /// Configures the input source select for an IMCR instance.
    ///
    /// Per RM §16.3.13 / §16.4.14, IMCRn must be configured only at
    /// application initialization and supports only 32-bit accesses.
    pub fn setup_imcr(&self, imcr: Imcr, sss: ImcrSource) {
        self.registers.imcr[imcr.value()].write(IMCR::SSS.val(sss.value()));
    }

    /// Configures a pad as an alternate-function output (TX-style).
    ///
    /// Sets SSS=`alt`, OBE=1, IBE=1, and SRE=`0b101` (150 MHz at 1.8V, 133 MHz
    /// at 3.3V on FAST pads — RM §16.3.12 field `16-14 SRE`).
    pub fn setup_tx_pin(&self, pin: Pin, alt: u32) {
        // For TX ALT pins: SSS=alt, OBE=1, IBE=1, SRE=0b101.
        self.setup_mscr(pin, alt, true, true, 0b101);
    }

    /// Configures a pad as a pure input (RX-style).
    ///
    /// Sets SSS=0 (GPIO), OBE=0, IBE=1, SRE=0 — input-only with no drive.
    pub fn setup_rx_pin(&self, pin: Pin) {
        // For RX input pins: SSS=0, OBE=0, IBE=1, SRE=0.
        self.setup_mscr(pin, 0, false, true, 0);
    }

    /// Configures a pad as a software-controlled GPIO.
    ///
    /// For input pins the input buffer is enabled and the output buffer is
    /// disabled; for output pins the output buffer is enabled and the input
    /// buffer is disabled. SSS is set to `GPIO` (0) and SRE to 0.
    pub fn setup_gpio_pin(&self, pin: Pin, is_output: bool) {
        if is_output {
            // For GPIO output: SSS=0, OBE=1, IBE=0, SRE=0.
            self.setup_mscr(pin, 0, true, false, 0);
        } else {
            // For GPIO input: SSS=0, OBE=0, IBE=1, SRE=0.
            self.setup_mscr(pin, 0, false, true, 0);
        }
    }

    /// Drives a GPIO output pad high or low.
    ///
    /// GPDO registers are byte-addressed, but the per-pin offset is
    /// `1300h + (n + 3 - 2 × (n mod 4))` (RM §16.3.14). Because that formula
    /// reverses the byte order within each 4-byte group relative to the
    /// linear pin index, the array index is XORed with `0b11` to map `Pin`
    /// discriminant to the actual byte address.
    pub fn set_gpio(&self, pin: Pin, high: bool) {
        let val = u8::from(high);
        self.registers.gpdo[pin.value() ^ 3].write(GPDO::VAL.val(val));
    }

    /// Reads the current logic level of a GPIO input pad.
    ///
    /// GPDI registers use the same per-pin offset formula as GPDO
    /// (RM §16.3.15), so the byte index is XORed with `0b11` for the same
    /// reason as [`set_gpio`](Self::set_gpio).
    pub fn read_gpio(&self, pin: Pin) -> bool {
        self.registers.gpdi[pin.value() ^ 3].is_set(GPDI::VAL)
    }
}
