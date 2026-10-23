// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! System protection unit (SPU) of the nRF5340 application core.
//!
//! The SPU sets the security of flash and RAM regions, peripherals and pins.
//! Only secure code can access it. Bus masters other than this core, such as
//! the network core, are non-secure, so they reach only non-secure memory.

use core::ops::Range;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

const FLASH_REGION_SIZE: usize = 0x4000;
const RAM_START: usize = 0x2000_0000;
const RAM_REGION_SIZE: usize = 0x2000;

register_structs! {
    pub SpuRegisters {
        (0x000 => _reserved0),
        /// PERM and LOCK of each GPIO port.
        (0x4C0 => gpioport: [ReadWrite<u32>; 4]),
        (0x4D0 => _reserved1),
        /// REGION and SIZE of each non-secure callable flash window.
        (0x500 => flashnsc: [ReadWrite<u32>; 4]),
        (0x510 => _reserved2),
        (0x600 => flashregion_perm: [ReadWrite<u32, Perm::Register>; 64]),
        (0x700 => ramregion_perm: [ReadWrite<u32, Perm::Register>; 64]),
        (0x800 => periphid_perm: [ReadWrite<u32, PeriphPerm::Register>; 256]),
        (0xC00 => @END),
    }
}

register_bitfields! [u32,
    Perm [
        SECATTR OFFSET(4) NUMBITS(1) [
            NonSecure = 0,
            Secure = 1
        ]
    ],
    PeriphPerm [
        SECATTR OFFSET(4) NUMBITS(1) [
            NonSecure = 0,
            Secure = 1
        ],
        DMASEC OFFSET(5) NUMBITS(1) [
            NonSecure = 0,
            Secure = 1
        ]
    ]
];

pub struct Spu {
    registers: StaticRef<SpuRegisters>,
}

impl Spu {
    pub const fn new(registers: StaticRef<SpuRegisters>) -> Self {
        Self { registers }
    }

    /// Make the 16 KB flash regions that `range` touches non-secure.
    pub fn make_flash_nonsecure(&self, range: Range<usize>) {
        let regions = range.start / FLASH_REGION_SIZE..range.end.div_ceil(FLASH_REGION_SIZE);
        for perm in &self.registers.flashregion_perm[regions] {
            perm.modify(Perm::SECATTR::NonSecure);
        }
    }

    /// Make `range` non-secure callable. It must be the last 32 to 4096 bytes,
    /// a power of two, of a secure flash region.
    pub fn make_flash_nonsecure_callable(&self, range: Range<usize>) {
        self.registers.flashnsc[0].set((range.start / FLASH_REGION_SIZE) as u32);
        // SIZE counts from 1 for 32 bytes.
        self.registers.flashnsc[1].set(range.len().trailing_zeros() - 4);
    }

    /// Make the 8 KB RAM regions that `range` touches non-secure.
    pub fn make_ram_nonsecure(&self, range: Range<usize>) {
        let regions = (range.start - RAM_START) / RAM_REGION_SIZE
            ..(range.end - RAM_START).div_ceil(RAM_REGION_SIZE);
        for perm in &self.registers.ramregion_perm[regions] {
            perm.modify(Perm::SECATTR::NonSecure);
        }
    }

    /// Make peripheral `id` and its DMA non-secure. A peripheral's ID is bits
    /// 12 to 19 of its address, and also its interrupt number.
    pub fn make_peripheral_nonsecure(&self, id: usize) {
        self.registers.periphid_perm[id]
            .modify(PeriphPerm::SECATTR::NonSecure + PeriphPerm::DMASEC::NonSecure);
    }

    /// Make every pin of GPIO `port` non-secure.
    pub fn make_gpio_nonsecure(&self, port: usize) {
        self.registers.gpioport[2 * port].set(0);
    }
}
