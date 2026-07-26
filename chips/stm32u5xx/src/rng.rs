// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::entropy::{Client32, Continue, Entropy32};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

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

/// Iterator that retreives the full entropy outputs provided by the RNG peripheral
struct TrngIter<'a, 'b: 'a, const CR_CFG: u32, const HTCR_CFG: u32, const NSCR_CFG: u32>(
    &'a Trng<'b, CR_CFG, HTCR_CFG, NSCR_CFG>,
);
impl<const CR_CFG: u32, const HTCR_CFG: u32, const NSCR_CFG: u32> Iterator
    for TrngIter<'_, '_, CR_CFG, HTCR_CFG, NSCR_CFG>
{
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.registers.sr.is_set(SR::DRDY) {
            Some(self.0.registers.dr.get())
        } else {
            None
        }
    }
}

/// Separate Iterator that does not provide any entropy. Only applicable when there
/// was an error in the peripheral
struct ErrIter;
impl Iterator for ErrIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct Trng<'a, const CR: u32, const HTCR: u32, const NSCR: u32> {
    registers: StaticRef<RngRegisters>,
    client: OptionalCell<&'a dyn Client32>,
    entropy_needed: Cell<bool>,
    deferred_call: DeferredCall,
}

impl<const CR_CFG: u32, const HTCR_CFG: u32, const NSCR_CFG: u32>
    Trng<'_, CR_CFG, HTCR_CFG, NSCR_CFG>
{
    pub fn new(base: StaticRef<RngRegisters>) -> Self {
        Self {
            registers: base,
            client: OptionalCell::empty(),
            entropy_needed: Cell::new(false),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Initialises the RNG peripheral with special config values. These should specified in the documentation
    /// (NIST compliant RNG configuration table in AN4230 available from www.st.com.)
    pub fn init(&'static self) {
        self.registers.cr.set(CR_CFG);
        self.registers.htcr.modify(HTCR::HTCFG.val(HTCR_CFG));
        self.registers.nscr.modify(NSCR::NSCFG.val(NSCR_CFG));
        self.registers.cr.modify(CR::CONFIGLOCK::SET);
        self.registers
            .cr
            .modify(CR::CONDRST::CLEAR + CR::RNGEN::SET);
        self.register();
    }

    fn send_data(&self) {
        self.entropy_needed.set(false);
        let response = self
            .client
            .map(|client| client.entropy_available(&mut TrngIter(self), Ok(())));
        match response {
            Some(Continue::Done) | None => {}
            _ => {
                self.entropy_needed.set(true);
                self.deferred_call.set();
            }
        }
    }
}

impl<'a, const CR_CFG: u32, const HTCR_CFG: u32, const NSCR_CFG: u32> Entropy32<'a>
    for Trng<'a, CR_CFG, HTCR_CFG, NSCR_CFG>
{
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

impl<const CR_CFG: u32, const HTCR_CFG: u32, const NSCR_CFG: u32> DeferredCallClient
    for Trng<'_, CR_CFG, HTCR_CFG, NSCR_CFG>
{
    fn handle_deferred_call(&self) {
        if self.registers.sr.is_set(SR::SECS) {
            self.registers.sr.modify(SR::SEIS::CLEAR);
            if self.entropy_needed.get() {
                self.client.map(|client| {
                    client.entropy_available(&mut ErrIter, Err(kernel::ErrorCode::FAIL))
                });
            }
            return;
        }
        if self.registers.sr.is_set(SR::CECS) {
            self.registers.sr.modify(SR::CEIS::CLEAR);
            if self.entropy_needed.get() {
                self.client.map(|client| {
                    client.entropy_available(&mut ErrIter, Err(kernel::ErrorCode::FAIL))
                });
            }
            return;
        }
        if !self.entropy_needed.get() {
            return;
        }
        if self.registers.sr.any_matching_bits_set(SR::DRDY::SET) {
            self.send_data();
        } else {
            self.deferred_call.set();
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
