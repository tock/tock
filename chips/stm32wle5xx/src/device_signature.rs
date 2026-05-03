// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{register_bitfields, ReadOnly};
use kernel::utilities::StaticRef;

struct Uid64Registers {
    uid_high: ReadOnly<u32, UIDHIGH::Register>,
    uid_low: ReadOnly<u32, UIDLOW::Register>,
}

register_bitfields![u32,
    UIDHIGH [
        /// Device number.
        UID OFFSET(0) NUMBITS(32) []
    ],
    UIDLOW [
        /// Company ID - 0x0080E1 for STMicroelectronics.
        STID OFFSET(8) NUMBITS(24) [],
        /// Device ID.
        DEVID OFFSET(0) NUMBITS(8) []
    ]
];

const UID64_BASE: StaticRef<Uid64Registers> =
    unsafe { StaticRef::new(0x1FFF7580 as *const Uid64Registers) };

pub struct Uid64 {
    registers: StaticRef<Uid64Registers>,
}

impl Uid64 {
    pub const fn new() -> Uid64 {
        Uid64 {
            registers: UID64_BASE,
        }
    }

    pub fn get_device_uid64(&self) -> u64 {
        let uid_high = self.registers.uid_high.get();
        let uid_low = self.registers.uid_low.get();
        ((uid_high as u64) << 32) | (uid_low as u64)
    }
}
