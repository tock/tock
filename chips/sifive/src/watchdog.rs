// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Watchdog driver.

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_bitfields, register_structs};

register_structs! {
    pub WatchdogRegisters {
        /// Watchdog Configuration Register
        (0x000 => wdogcfg: ReadWrite<u32, cfg::Register>),
        (0x004 => _reserved0),
        /// Watchdog Counter Register
        (0x008 => wdogcount: ReadWrite<u32>),
        (0x00c => _reserved1),
        /// Watchdog Scaled Counter Register
        (0x010 => wdogs: ReadWrite<u32>),
        (0x014 => _reserved2),
        /// Watchdog Feed Register
        (0x018 => wdogfeed: ReadWrite<u32, feed::Register>),
        /// Watchdog Key Register
        (0x01c => wdogkey: WriteOnly<u32, key::Register>),
        /// Watchdog Compare Register
        (0x020 => wdogcmp: ReadWrite<u32>),
        (0x024 => @END),
    }
}

register_bitfields![u32,
    cfg [
        cmpip OFFSET(28) NUMBITS(1) [],
        encoreawake OFFSET(13) NUMBITS(1) [],
        enalways OFFSET(12) NUMBITS(1) [],
        zerocmp OFFSET(9) NUMBITS(1) [],
        rsten OFFSET(8) NUMBITS(1) [],
        scale OFFSET(0) NUMBITS(4) []
    ],
    key [
        key OFFSET(0) NUMBITS(32) []
    ],
    feed [
        feed OFFSET(0) NUMBITS(32) []
    ]
];

pub struct Watchdog {
    registers: StaticRef<WatchdogRegisters>,
}

impl Watchdog {
    pub const fn new(base: StaticRef<WatchdogRegisters>) -> Watchdog {
        Watchdog { registers: base }
    }

    fn unlock(&self) {
        self.registers.wdogkey.write(key::key.val(0x51F15E));
    }

    fn feed(&self) {
        self.unlock();
        self.registers.wdogfeed.write(feed::feed.val(0xD09F00D));
    }

    pub fn disable(&self) {
        self.unlock();
        self.registers.wdogcfg.write(
            cfg::scale.val(0)
                + cfg::rsten::CLEAR
                + cfg::zerocmp::CLEAR
                + cfg::enalways::CLEAR
                + cfg::encoreawake::CLEAR,
        );
        self.feed();
    }
}
