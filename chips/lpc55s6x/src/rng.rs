// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! True Random Number Generator (RNG) for the LPC55S6x family.
//!
//! Follows the initialization and normal usage procedures for design
//! revision 1B described in UM11126 sections 48.15.5 and 48.15.6. Before
//! calling [`RNG::init`], the board must power up the entropy sources
//! (`Pmc::power_up_rng`) and enable and reset the RNG block
//! (`Clock::start_rng_clock`).

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::entropy::{self, Continue};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

use crate::pmc::Pmc;

register_structs! {
    /// RNG
    RngRegisters {
        /// This register contains a random 32 bit number which is computed on demand, at each time it is read
        (0x000 => random_number: ReadOnly<u32>),
        (0x004 => _reserved0),
        (0x008 => counter_val: ReadOnly<u32, COUNTER_VAL::Register>),
        (0x00C => counter_cfg: ReadWrite<u32, COUNTER_CFG::Register>),
        (0x010 => online_test_cfg: ReadWrite<u32, ONLINE_TEST_CFG::Register>),
        (0x014 => online_test_val: ReadOnly<u32, ONLINE_TEST_VAL::Register>),
        (0x018 => _reserved1),
        /// IP identifier
        (0xFFC => moduleid: ReadOnly<u32, MODULEID::Register>),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
COUNTER_VAL [
    /// Gives the ratio between the internal clocks frequencies and the register clock frequency for evaluation and certification purposes.
    CLK_RATIO OFFSET(0) NUMBITS(8) [],
    /// Incremented (till max possible value) each time COUNTER was updated since last reading to any *_NUMBER.
    REFRESH_CNT OFFSET(8) NUMBITS(5) []
],
COUNTER_CFG [
    /// 00: disabled 01: update once.
    MODE OFFSET(0) NUMBITS(2) [],
    /// Selects the internal clock on which to compute statistics.
    CLOCK_SEL OFFSET(2) NUMBITS(3) [],
    /// To be used to add precision to clock_ratio and determine 'entropy refill'.
    SHIFT4X OFFSET(5) NUMBITS(3) []
],
ONLINE_TEST_CFG [
    /// 0: disabled 1: activated Update rythm for VAL depends on COUNTER_CFG if data_sel is set to COUNTER.
    ACTIVATE OFFSET(0) NUMBITS(1) [],
    /// Selects source on which to apply online test: 00: LSB of COUNTER: raw data from one or all sources of entropy 01: MSB of COUNTER: raw data from one or all sources of entropy 10: RANDOM_NUMBER 11: ENCRYPTED_NUMBER 'activate' should be set to 'disabled' before changing this field.
    DATA_SEL OFFSET(1) NUMBITS(2) []
],
ONLINE_TEST_VAL [
    /// This value is updated as described in field 'activate'.
    LIVE_CHI_SQUARED OFFSET(0) NUMBITS(4) [],
    /// This field is reset when 'activate'==0.
    MIN_CHI_SQUARED OFFSET(4) NUMBITS(4) [],
    /// This field is reset when 'activate'==0.
    MAX_CHI_SQUARED OFFSET(8) NUMBITS(4) []
],
MODULEID [
    /// Aperture i.
    APERTURE OFFSET(0) NUMBITS(8) [],
    /// Minor revision i.
    MIN_REV OFFSET(8) NUMBITS(4) [],
    /// Major revision i.
    MAJ_REV OFFSET(12) NUMBITS(4) [],
    /// Identifier.
    ID OFFSET(16) NUMBITS(16) []
]
];

const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x4003A000 as *const RngRegisters) };

/// Clock on which the CHI squared statistics are computed, as recommended
/// for revision 1B devices.
const CHI_CLOCK_SEL: u32 = 4;

/// Largest MAX_CHI_SQUARED value for which the entropy is considered good.
const MAX_CHI_SQUARED_LIMIT: u32 = 4;

/// REFRESH_CNT value indicating that the entropy has been fully refilled
/// since the last read of RANDOM_NUMBER.
const REFRESH_CNT_FULL: u32 = 31;

/// Maximum number of register polls while waiting for the RNG hardware, so
/// that a misbehaving entropy source cannot hang the kernel.
const MAX_POLLS: u32 = 1_000_000;

/// Maximum number of times entropy is re-accumulated when the CHI squared
/// test fails.
const MAX_QUALITY_RETRIES: u32 = 16;

pub struct RNG<'a> {
    registers: StaticRef<RngRegisters>,
    pmc: Pmc,
    client: OptionalCell<&'a dyn entropy::Client32>,
    deferred_call: DeferredCall,
    /// Whether `init` completed successfully.
    initialized: Cell<bool>,
    /// Whether a `get` request is outstanding.
    requested: Cell<bool>,
}

impl<'a> RNG<'a> {
    pub fn new() -> RNG<'a> {
        RNG {
            registers: RNG_BASE,
            pmc: Pmc::new(),
            client: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
            initialized: Cell::new(false),
            requested: Cell::new(false),
        }
    }

    /// Accumulate initial entropy and wait until the CHI squared test
    /// passes. Must be called once before entropy is requested.
    pub fn init(&self) -> Result<(), ErrorCode> {
        if !self.pmc.is_rng_powered() {
            return Err(ErrorCode::OFF);
        }

        self.accumulate_entropy()?;
        self.ensure_entropy_quality()?;
        self.initialized.set(true);
        Ok(())
    }

    fn accumulate_entropy(&self) -> Result<(), ErrorCode> {
        let regs = self.registers;

        regs.counter_cfg
            .modify(COUNTER_CFG::CLOCK_SEL.val(CHI_CLOCK_SEL));
        regs.online_test_cfg.write(ONLINE_TEST_CFG::ACTIVATE::SET);

        // At power on MIN_CHI_SQUARED is higher than MAX_CHI_SQUARED. Wait
        // until it decreases below MAX_CHI_SQUARED.
        for _ in 0..MAX_POLLS {
            let min_chi = regs.online_test_val.read(ONLINE_TEST_VAL::MIN_CHI_SQUARED);
            let max_chi = regs.online_test_val.read(ONLINE_TEST_VAL::MAX_CHI_SQUARED);
            if min_chi <= max_chi.wrapping_sub(1) {
                return Ok(());
            }
        }
        Err(ErrorCode::FAIL)
    }

    /// While MAX_CHI_SQUARED is above 4, reset the CHI computing, increase
    /// SHIFT4X and accumulate entropy again.
    fn ensure_entropy_quality(&self) -> Result<(), ErrorCode> {
        let regs = self.registers;

        for _ in 0..MAX_QUALITY_RETRIES {
            if regs.online_test_val.read(ONLINE_TEST_VAL::MAX_CHI_SQUARED) <= MAX_CHI_SQUARED_LIMIT
            {
                return Ok(());
            }

            // Deactivate CHI computing to reset the statistics.
            regs.online_test_cfg.write(ONLINE_TEST_CFG::ACTIVATE::CLEAR);

            let shift4x = regs.counter_cfg.read(COUNTER_CFG::SHIFT4X);
            if shift4x < 7 {
                regs.counter_cfg
                    .modify(COUNTER_CFG::SHIFT4X.val(shift4x + 1));
            }

            self.accumulate_entropy()?;
        }
        Err(ErrorCode::FAIL)
    }

    fn read_entropy(&self) -> Result<u32, ErrorCode> {
        let regs = self.registers;

        regs.online_test_cfg.write(ONLINE_TEST_CFG::ACTIVATE::SET);

        // Wait for the entropy to be refilled since the last read. Reading
        // RANDOM_NUMBER before that could stall the bus.
        let mut polls = 0;
        while regs.counter_val.read(COUNTER_VAL::REFRESH_CNT) < REFRESH_CNT_FULL {
            polls += 1;
            if polls == MAX_POLLS {
                return Err(ErrorCode::FAIL);
            }
        }

        // Reading RANDOM_NUMBER resets REFRESH_CNT to 0.
        let data = regs.random_number.get();

        // Ensure the entropy is still of good quality before the next read.
        self.ensure_entropy_quality()?;

        Ok(data)
    }
}

impl DeferredCallClient for RNG<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if !self.requested.get() {
            // The request was cancelled.
            return;
        }

        let result = self.read_entropy();

        let more = self.client.map_or(false, |client| match result {
            Ok(random_val) => {
                let mut iter = core::iter::once(random_val);
                client.entropy_available(&mut iter, Ok(())) == Continue::More
            }
            Err(e) => {
                client.entropy_available(&mut core::iter::empty(), Err(e));
                false
            }
        });

        if more {
            self.deferred_call.set();
        } else {
            self.requested.set(false);
        }
    }
}

impl<'a> entropy::Entropy32<'a> for RNG<'a> {
    fn get(&self) -> Result<(), ErrorCode> {
        if !self.pmc.is_rng_powered() {
            return Err(ErrorCode::OFF);
        }
        if !self.initialized.get() {
            return Err(ErrorCode::FAIL);
        }

        if !self.requested.replace(true) {
            self.deferred_call.set();
        }
        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        // A pending deferred call cannot be unscheduled, but it will see
        // that the request is gone and return without a callback.
        self.requested.set(false);
        Ok(())
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.client.set(client);
    }
}
