// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Flexible Communication (Flexcomm) peripheral driver for the LPC55S6x family.
//!
//! The Flexcomm block is a multi‑protocol serial interface that can be
//! configured at runtime to operate as one of several functions:
//! - **USART** (Universal Synchronous/Asynchronous Receiver/Transmitter)
//! - **SPI** (Serial Peripheral Interface)
//! - **I²C** (Inter‑Integrated Circuit)
//! - **I²S** (Inter‑IC Sound, transmit or receive)
//!
//! Each LPC55S6x device includes up to 8 Flexcomm instances (Flexcomm0–7),
//! each with its own base address. The `PSELID` register selects the active
//! function and can be locked to prevent accidental reconfiguration.
//!
//! This module provides:
//! - Strongly‑typed register mappings for `PSELID` and `PID`
//! - Safe constructors for accessing Flexcomm instances by base address or ID
//! - Convenience methods for configuring a Flexcomm as a UART
//!
//! Reference: *LPC55S6x/LPC55S2x/LPC552x User Manual* (NXP).

use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use crate::flexcomm::PSELID::{LOCK, PERSEL};

const FLEXCOMM0_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40086000 as *const FlexcommRegisters) };
const FLEXCOMM1_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40087000 as *const FlexcommRegisters) };
const FLEXCOMM2_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40088000 as *const FlexcommRegisters) };
const FLEXCOMM3_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40089000 as *const FlexcommRegisters) };
const FLEXCOMM4_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x4008A000 as *const FlexcommRegisters) };
const FLEXCOMM5_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40096000 as *const FlexcommRegisters) };
const FLEXCOMM6_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40097000 as *const FlexcommRegisters) };
const FLEXCOMM7_BASE: StaticRef<FlexcommRegisters> =
    unsafe { StaticRef::new(0x40098000 as *const FlexcommRegisters) };

register_structs! {
    /// Flexcomm serial communication
    FlexcommRegisters {
        (0x000 => _reserved0),
        /// Peripheral Select and Flexcomm ID register.
        (0xFF8 => pselid: ReadWrite<u32, PSELID::Register>),
        /// Peripheral identification register.
        (0xFFC => pid: ReadOnly<u32, PID::Register>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
pub PSELID [
    /// Peripheral Select. This field is writable by software.
    PERSEL OFFSET(0) NUMBITS(3) [
        /// No peripheral selected.
        NoPeripheralSelected = 0,
        /// USART function selected.
        USARTFunctionSelected = 1,
        /// SPI function selected.
        SPIFunctionSelected = 2,
        /// I2C function selected.
        I2CFunctionSelected = 3,
        /// I2S transmit function selected.
        I2STransmitFunctionSelected = 4,
        /// I2S receive function selected.
        I2SReceiveFunctionSelected = 5
    ],
    /// Lock the peripheral select. This field is writable by software.
    LOCK OFFSET(3) NUMBITS(1) [
        /// Peripheral select can be changed by software.
        PeripheralSelectCanBeChangedBySoftware = 0,
        /// Peripheral select is locked and cannot be changed until this Flexcomm or the ent
        LOCKED = 1
    ],
    /// USART present indicator. This field is Read-only.
    USARTPRESENT OFFSET(4) NUMBITS(1) [
        /// This Flexcomm does not include the USART function.
        ThisFlexcommDoesNotIncludeTheUSARTFunction = 0,
        /// This Flexcomm includes the USART function.
        ThisFlexcommIncludesTheUSARTFunction = 1
    ],
    /// SPI present indicator. This field is Read-only.
    SPIPRESENT OFFSET(5) NUMBITS(1) [
        /// This Flexcomm does not include the SPI function.
        ThisFlexcommDoesNotIncludeTheSPIFunction = 0,
        /// This Flexcomm includes the SPI function.
        ThisFlexcommIncludesTheSPIFunction = 1
    ],
    /// I2C present indicator. This field is Read-only.
    I2CPRESENT OFFSET(6) NUMBITS(1) [
        /// This Flexcomm does not include the I2C function.
        ThisFlexcommDoesNotIncludeTheI2CFunction = 0,
        /// This Flexcomm includes the I2C function.
        ThisFlexcommIncludesTheI2CFunction = 1
    ],
    /// I 2S present indicator. This field is Read-only.
    I2SPRESENT OFFSET(7) NUMBITS(1) [
        /// This Flexcomm does not include the I2S function.
        ThisFlexcommDoesNotIncludeTheI2SFunction = 0,
        /// This Flexcomm includes the I2S function.
        ThisFlexcommIncludesTheI2SFunction = 1
    ],
    /// Flexcomm ID.
    ID OFFSET(12) NUMBITS(20) []
],
PID [
    /// size aperture for the register port on the bus (APB or AHB).
    APERTURE OFFSET(0) NUMBITS(8) [],
    /// Minor revision of module implementation.
    MINOR_REV OFFSET(8) NUMBITS(4) [],
    /// Major revision of module implementation.
    MAJOR_REV OFFSET(12) NUMBITS(4) [],
    /// Module identifier for the selected function.
    ID OFFSET(16) NUMBITS(16) []
]
];

/// A driver for a generic Flexcomm peripheral.
pub struct Flexcomm {
    regs: StaticRef<FlexcommRegisters>,
}

impl Flexcomm {
    pub const fn new(base_addr: usize) -> Self {
        Flexcomm {
            regs: unsafe { StaticRef::new(base_addr as *const FlexcommRegisters) },
        }
    }

    pub const fn new_id(id: u32) -> Option<Self> {
        let base_addr = match id {
            0 => FLEXCOMM0_BASE,
            1 => FLEXCOMM1_BASE,
            2 => FLEXCOMM2_BASE,
            3 => FLEXCOMM3_BASE,
            4 => FLEXCOMM4_BASE,
            5 => FLEXCOMM5_BASE,
            6 => FLEXCOMM6_BASE,
            7 => FLEXCOMM7_BASE,
            _ => return None,
        };

        Some(Flexcomm {
            regs: { base_addr },
        })
    }

    /// Configures this Flexcomm to be a UART and locks the selection.
    pub fn configure_for_uart(&self) {
        self.regs
            .pselid
            .write(PERSEL::USARTFunctionSelected + LOCK::SET);
    }
}
