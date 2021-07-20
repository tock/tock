// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    /// Power control
    PwrRegisters {
        /// power control register
        (0x000 => cr: ReadWrite<u32, CR::Register>),
        /// power control/status register
        (0x004 => csr: ReadWrite<u32, CSR::Register>),
        (0x008 => @END),
    }
}
register_bitfields![u32,
CR [
    /// Low-power deep sleep
    LPDS OFFSET(0) NUMBITS(1) [],
    /// Power down deepsleep
    PDDS OFFSET(1) NUMBITS(1) [],
    /// Clear wakeup flag
    CWUF OFFSET(2) NUMBITS(1) [],
    /// Clear standby flag
    CSBF OFFSET(3) NUMBITS(1) [],
    /// Power voltage detector
    /// enable
    PVDE OFFSET(4) NUMBITS(1) [],
    /// PVD level selection
    PLS OFFSET(5) NUMBITS(3) [],
    /// Disable backup domain write
    /// protection
    DBP OFFSET(8) NUMBITS(1) [],
    /// Flash power down in Stop
    /// mode
    FPDS OFFSET(9) NUMBITS(1) [],
    /// Low-Power Regulator Low Voltage in
    /// deepsleep
    LPLUDS OFFSET(10) NUMBITS(1) [],
    /// Main regulator low voltage in deepsleep
    /// mode
    MRUDS OFFSET(11) NUMBITS(1) [],

    ADCDC1 OFFSET(13) NUMBITS(1) [],
    /// Regulator voltage scaling output
    /// selection
    VOS OFFSET(14) NUMBITS(2) [
        Scale3 = 0b01,
        Scale2 = 0b10,
        Scale1 = 0b11,
    ],
    /// Over-drive enable
    ODEN OFFSET(16) NUMBITS(1) [],
    /// Over-drive switching
    /// enabled
    ODSWEN OFFSET(17) NUMBITS(1) [],
    /// Under-drive enable in stop
    /// mode
    UDEN OFFSET(18) NUMBITS(2) []
],
CSR [
    /// Wakeup flag
    WUF OFFSET(0) NUMBITS(1) [],
    /// Standby flag
    SBF OFFSET(1) NUMBITS(1) [],
    /// PVD output
    PVDO OFFSET(2) NUMBITS(1) [],
    /// Backup regulator ready
    BRR OFFSET(3) NUMBITS(1) [],
    /// Enable WKUP pin
    EWUP OFFSET(8) NUMBITS(1) [],
    /// Backup regulator enable
    BRE OFFSET(9) NUMBITS(1) [],
    /// Regulator voltage scaling output
    /// selection ready bit
    VOSRDY OFFSET(14) NUMBITS(1) [],
    /// Over-drive mode ready
    ODRDY OFFSET(16) NUMBITS(1) [],
    /// Over-drive mode switching
    /// ready
    ODSWRDY OFFSET(17) NUMBITS(1) [],
    /// Under-drive ready flag
    UDRDY OFFSET(18) NUMBITS(2) []
]
];
const PWR_BASE: StaticRef<PwrRegisters> =
    unsafe { StaticRef::new(0x40007000 as *const PwrRegisters) };

#[inline(never)]
pub fn enable_backup_access() -> Result<(), ErrorCode> {
    PWR_BASE.cr.modify(CR::DBP::SET);
    Ok(())
}
