// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Access port protection
//!
//! <https://infocenter.nordicsemi.com/index.jsp?topic=%2Fps_nrf52840%2Fdif.html&cp=5_0_0_3_7_1&anchor=register.DISABLE>
//!
//! The logic around APPROTECT was changed in newer revisions of the nRF52
//! series chips (Oct 2021) and later which requires more careful disabling of
//! the access port (JTAG), both in the UICR register and in a software written
//! register. This module enables the kernel to disable the protection on boot.
//!
//! Example code to disable the APPROTECT protection in software:
//!
//! ```rust,ignore
//! let approtect = nrf52::approtect::Approtect::new();
//! approtect.sw_disable_approtect();
//! ```

use crate::ficr;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

const APPROTECT_BASE: StaticRef<ApprotectRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const ApprotectRegisters) };

register_structs! {
    ApprotectRegisters {
        (0x000 => _reserved0),
        (0x550 => forceprotect: ReadWrite<u32, Forceprotect::Register>),
        (0x554 => _reserved1),
        (0x558 => disable: ReadWrite<u32, Disable::Register>),
        (0x55c => @END),
    }
}

register_bitfields! [u32,
    Forceprotect [
        FORCEPROTECT OFFSET(0) NUMBITS(8) [
            FORCE = 0
        ]
    ],
    /// Access port protection
    Disable [
        DISABLE OFFSET(0) NUMBITS(8) [
            SWDISABLE = 0x5a
        ]
    ]
];

pub struct Approtect {
    registers: StaticRef<ApprotectRegisters>,
}

impl Approtect {
    pub const fn new() -> Approtect {
        Approtect {
            registers: APPROTECT_BASE,
        }
    }

    /// Software disable the Access Port Protection mechanism.
    ///
    /// On newer variants of the nRF52, to enable JTAG, APPROTECT must be
    /// disabled both in the UICR register (hardware) and in this register
    /// (software). For older variants this is just a no-op.
    ///
    /// - <https://devzone.nordicsemi.com/f/nordic-q-a/96590/how-to-disable-approtect-permanently-dfu-is-needed>
    /// - <https://devzone.nordicsemi.com/nordic/nordic-blog/b/blog/posts/working-with-the-nrf52-series-improved-approtect>
    pub fn sw_disable_approtect(&self) {
        let factory_config = ficr::Ficr::new();
        match factory_config.variant() {
            ficr::Variant::AAF0 | ficr::Variant::Unspecified => {
                // Newer revisions of the chip require setting the APPROTECT
                // software register to `SwDisable`. We assume that an unspecified
                // version means that it is new and the FICR module hasn't been
                // updated to recognize it.
                self.registers.disable.write(Disable::DISABLE::SWDISABLE);
            }

            // Exhaustively list variants here to produce compiler error on
            // adding a new variant, which would otherwise not match the above
            // condition.
            ficr::Variant::AAA0
            | ficr::Variant::AAAA
            | ficr::Variant::AAAB
            | ficr::Variant::AAB0
            | ficr::Variant::AABA
            | ficr::Variant::AABB
            | ficr::Variant::AAC0
            | ficr::Variant::AACA
            | ficr::Variant::AACB
            | ficr::Variant::AAD0
            | ficr::Variant::AAD1
            | ficr::Variant::AADA
            | ficr::Variant::AAE0
            | ficr::Variant::AAEA
            | ficr::Variant::ABBA
            | ficr::Variant::BAAA
            | ficr::Variant::CAAA => {
                // All other revisions don't need this.
            }
        }
    }
}
