// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! LEDs on the MPS2 AN385/AN386 FPGA images, via the "FPGAIO" register
//! block.
//!
//! This is *not* implemented on top of the CMSDK AHB GPIO peripheral: QEMU
//! models the four CMSDK GPIO banks as inert stubs (writes are discarded,
//! reads always return 0) on every MPS2/MPS2-TZ machine, so pin state
//! changes made through that peripheral are never observable in emulation.
//!
//! `FPGAIO`'s `LED0` register, by contrast, is fully emulated in QEMU
//! (`hw/misc/mps2-fpgaio.c`) and drives an actual `LEDState` per bit, so
//! we use this to get observable LED behavior under QEMU.

use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

pub const FPGAIO_BASE: StaticRef<FpgaioRegisters> =
    unsafe { StaticRef::new(0x4002_8000 as *const FpgaioRegisters) };

/// Number of LEDs QEMU wires up to `LED0` for the an385/an386 machines
/// (the `mps2-fpgaio` device's `num-leds` property default).
pub const NUM_LEDS: u32 = 2;

#[repr(C)]
pub struct FpgaioRegisters {
    led0: ReadWrite<u32>,
}

pub struct Fpgaio {
    registers: StaticRef<FpgaioRegisters>,
}

impl Fpgaio {
    pub const fn new(registers: StaticRef<FpgaioRegisters>) -> Self {
        Fpgaio { registers }
    }

    /// Returns a [`hil::led::Led`] handle for LED `INDEX`.
    pub fn led<const INDEX: u32>(&self) -> Led<'_> {
        const { assert!(INDEX < NUM_LEDS) };
        Led {
            fpgaio: self,
            mask: 1 << INDEX,
        }
    }
}

pub struct Led<'a> {
    fpgaio: &'a Fpgaio,
    mask: u32,
}

impl hil::led::Led for Led<'_> {
    fn init(&self) {}

    fn on(&self) {
        let v = self.fpgaio.registers.led0.get() | self.mask;
        self.fpgaio.registers.led0.set(v);
    }

    fn off(&self) {
        let v = self.fpgaio.registers.led0.get() & !self.mask;
        self.fpgaio.registers.led0.set(v);
    }

    fn toggle(&self) {
        let v = self.fpgaio.registers.led0.get() ^ self.mask;
        self.fpgaio.registers.led0.set(v);
    }

    fn read(&self) -> bool {
        self.fpgaio.registers.led0.get() & self.mask != 0
    }
}
