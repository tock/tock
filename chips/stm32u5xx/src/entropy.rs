// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use core::cell::Cell;
use kernel::debug;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::entropy::{Client32, Continue, Entropy32};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Random number generator
    pub RngRegisters {
        /// control register
        (0x000 => cr: ReadWrite<u32, CR::Register>),
        /// status register
        (0x004 => pub sr: ReadWrite<u32, SR::Register>),
        /// data register
        (0x008 => dr: ReadOnly<u32>),
        (0x00C => nscr:  ReadWrite<u32, NSCR::Register>),
        /// health test control register
        (0x010 => htcr: ReadWrite<u32, HTCR::Register>),
        (0x014 => @END),
    }
}
register_bitfields![u32,
    pub CR [
        /// RNG Config Lock
        CONFIGLOCK OFFSET(31) NUMBITS(1) [],
        /// Conditioning soft reset
        CONDRST OFFSET(30) NUMBITS(1) [],
        /// RNG configuration 1
        RNG_CONFIG1 OFFSET(20) NUMBITS(6) [],
        /// Clock divider factor
        CLKDIV OFFSET(16) NUMBITS(4) [],
        /// RNG configuration 2
        RNG_CONFIG2 OFFSET(13) NUMBITS(3) [],
        /// Non NIST compliant
        NISTC OFFSET(12) NUMBITS(1) [],
        /// RNG configuration 3
        RNG_CONFIG3 OFFSET(8) NUMBITS(4) [],
        /// Auto reset disable
        ARDIS OFFSET(7) NUMBITS(1) [],
        /// Clock error detection
        CED OFFSET(5) NUMBITS(1) [],
        /// Interrupt Enable
        IE OFFSET(3) NUMBITS(1) [],
        /// True random number generator enable
        RNGEN OFFSET(2) NUMBITS(1) []
    ],
    pub SR [
        /// Seed error interrupt status
        SEIS OFFSET(6) NUMBITS(1) [],
        /// Clock error interrupt status
        CEIS OFFSET(5) NUMBITS(1) [],
        /// Seed error current status
        SECS OFFSET(2) NUMBITS(1) [],
        /// Clock error current status
        CECS OFFSET(1) NUMBITS(1) [],
        /// Data ready
        DRDY OFFSET(0) NUMBITS(1) []
    ],
    pub DR [
        /// Random data
        RNDATA OFFSET(0) NUMBITS(32) []
    ],
    pub NSCR [
        /// noise source control register
        NSCFG OFFSET(0) NUMBITS(32) []
    ],
    pub HTCR [
        /// health test configuration
        HTCFG OFFSET(0) NUMBITS(32) []
    ]
];
pub const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x520C0800 as *const RngRegisters) };

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);
impl Iterator for TrngIter<'_, '_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.registers.sr.is_set(SR::DRDY) {
            Some(self.0.registers.dr.get())
        } else {
            None
        }
    }
}

pub struct Trng<'a> {
    registers: StaticRef<RngRegisters>,
    client: OptionalCell<&'a dyn Client32>,
    entropy_needed: Cell<bool>,
    deferred_call: DeferredCall,
}

impl<'a> Trng<'a> {
    pub const fn new(base: StaticRef<RngRegisters>, deferred_call: DeferredCall) -> Self {
        Self {
            registers: base,
            client: OptionalCell::empty(),
            entropy_needed: Cell::new(false),
            deferred_call: deferred_call,
        }
    }

    pub fn init(&self) {
        // specified in the documentation (NIST compliant RNG configuration table in AN4230 available from www.st.com.)
        // that values for the CR, HTCR and NSCR should be 0x00F11F00, 0x76B3 and 0x24C2 respectivly
        self.registers.cr.modify(
            CR::RNG_CONFIG3.val(0b1111)
                + CR::NISTC::SET
                + CR::CLKDIV.val(0x1)
                + CR::RNG_CONFIG1.val(0b1111)
                + CR::CONDRST::SET,
        );
        self.registers.htcr.modify(HTCR::HTCFG.val(0x76B3));
        self.registers.nscr.modify(NSCR::NSCFG.val(0x24C2));
        self.registers.cr.modify(CR::CONFIGLOCK::SET);
        self.registers
            .cr
            .modify(CR::RNGEN::SET + CR::IE::SET + CR::CONDRST::CLEAR);
        debug!("CR: {:02x?}", self.registers.cr.get());
    }
    fn send_data(&self) {
        let response = self
            .client
            .map(|client| client.entropy_available(&mut TrngIter(self), Ok(())));
        match response {
            Some(Continue::Done) | None => self.entropy_needed.set(false),
            _ => {
                self.deferred_call.set();
            }
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        if regs.sr.any_matching_bits_set(SR::DRDY::SET) && self.entropy_needed.get() {
            self.send_data();
        } else {
            self.deferred_call.set();
        }
    }
}

impl<'a> Entropy32<'a> for Trng<'a> {
    fn get(&self) -> Result<(), kernel::ErrorCode> {
        let regs = self.registers;
        if regs.sr.any_matching_bits_set(SR::CECS::SET + SR::SECS::SET) {
            return Err(kernel::ErrorCode::FAIL);
        }
        self.entropy_needed.set(true);
        self.deferred_call.set();

        Ok(())
    }

    fn cancel(&self) -> Result<(), kernel::ErrorCode> {
        if self.entropy_needed.get() {
            self.entropy_needed.set(false);
        }
        Ok(())
    }

    fn set_client(&'a self, client: &'a dyn kernel::hil::entropy::Client32) {
        self.client.set(client);
    }
}

impl DeferredCallClient for Trng<'_> {
    fn handle_deferred_call(&self) {
        debug!("got");
        if !self.entropy_needed.get() {
            return;
        }
        debug!(
            "CR: {:02x?} SR: {:02x?}, need: {}",
            self.registers.cr.get(),
            self.registers.sr.get(),
            self.entropy_needed.get()
        );

        if self.registers.sr.any_matching_bits_set(SR::DRDY::SET) {
            self.send_data();
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
